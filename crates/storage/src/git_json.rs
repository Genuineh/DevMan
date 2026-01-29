//! Git+JSON storage implementation.
//!
//! Stores data as JSON files in a directory, with Git tracking all changes.

use std::path::Path;
use std::sync::Arc;
use devman_core::{Task, Event, KnowledgeNode, TaskFilter, TaskId, EventId, NodeId};
use super::{Storage, StorageError, Result};
use tokio::fs;
use tracing::{debug, info};
use tokio::sync::Mutex;

/// Git+JSON storage backend.
///
/// Data is stored as JSON files:
/// - tasks/{id}.json
/// - events/{id}.json
/// - knowledge/{id}.json
///
/// Each write creates a Git commit for full history tracking.
pub struct GitJsonStorage {
    /// Root directory for storage
    root: std::path::PathBuf,
    /// Pending changes (not yet committed)
    pending: Arc<Mutex<bool>>,
}

impl GitJsonStorage {
    /// Create a new Git+JSON storage at the given path.
    ///
    /// Initializes the directory and Git repository if needed.
    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();

        // Create directories
        fs::create_dir_all(root.join("tasks")).await?;
        fs::create_dir_all(root.join("events")).await?;
        fs::create_dir_all(root.join("knowledge")).await?;

        // Initialize Git repo if needed
        if !root.join(".git").exists() {
            info!("Initializing new Git repository at {}", root.display());
            git2::Repository::init(&root)?;
        }

        Ok(Self {
            root,
            pending: Arc::new(Mutex::new(false)),
        })
    }

    /// Path to a task file
    fn task_path(&self, id: TaskId) -> std::path::PathBuf {
        self.root.join("tasks").join(format!("{}.json", id))
    }

    /// Path to an event file
    fn event_path(&self, id: EventId) -> std::path::PathBuf {
        self.root.join("events").join(format!("{}.json", id))
    }

    /// Path to a knowledge file
    fn knowledge_path(&self, id: NodeId) -> std::path::PathBuf {
        self.root.join("knowledge").join(format!("{}.json", id))
    }

    /// Open the Git repository.
    fn open_repo(&self) -> std::result::Result<git2::Repository, git2::Error> {
        git2::Repository::open(&self.root)
    }

    /// Stage and commit changes
    fn do_commit_sync(&self, message: &str) -> Result<()> {
        let repo = self.open_repo()?;

        let mut index = repo.index()?;
        index.add_all(["*"], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let sig = repo.signature()?;
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

        if let Some(parent) = &parent {
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                message,
                &tree,
                &[parent],
            )?;
        } else {
            // Initial commit
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                message,
                &tree,
                &[],
            )?;
        }

        debug!("Committed: {}", message);
        Ok(())
    }

    /// Set pending flag
    async fn set_pending(&self) {
        *self.pending.lock().await = true;
    }

    /// Check if pending
    async fn is_pending(&self) -> bool {
        *self.pending.lock().await
    }
}

#[async_trait::async_trait]
impl Storage for GitJsonStorage {
    async fn save_task(&mut self, task: &Task) -> Result<()> {
        let path = self.task_path(task.id);
        let json = serde_json::to_string_pretty(task)?;
        fs::write(&path, json).await?;
        self.set_pending().await;
        debug!("Saved task {}", task.id);
        Ok(())
    }

    async fn load_task(&self, id: TaskId) -> Result<Option<Task>> {
        let path = self.task_path(id);
        match fs::read_to_string(&path).await {
            Ok(json) => {
                let task = serde_json::from_str(&json)?;
                Ok(Some(task))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        let mut dir = fs::read_dir(self.root.join("tasks")).await?;

        while let Some(entry) = dir.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let json = fs::read_to_string(entry.path()).await?;
            let task: Task = serde_json::from_str(&json)?;

            // Apply filter
            if let Some(statuses) = &filter.status {
                if !statuses.contains(&task.status) {
                    continue;
                }
            }
            if let Some(min_prio) = filter.min_priority {
                if task.priority < min_prio {
                    continue;
                }
            }
            if let Some(min_conf) = filter.min_confidence {
                if task.confidence < min_conf {
                    continue;
                }
            }

            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn delete_task(&mut self, id: TaskId) -> Result<()> {
        let path = self.task_path(id);
        fs::remove_file(path).await.or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })?;
        self.set_pending().await;
        Ok(())
    }

    async fn save_event(&mut self, event: &Event) -> Result<()> {
        let path = self.event_path(event.id);
        let json = serde_json::to_string_pretty(event)?;
        fs::write(&path, json).await?;
        self.set_pending().await;
        Ok(())
    }

    async fn load_event(&self, id: EventId) -> Result<Option<Event>> {
        let path = self.event_path(id);
        match fs::read_to_string(&path).await {
            Ok(json) => {
                let event = serde_json::from_str(&json)?;
                Ok(Some(event))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_events(&self) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        let mut dir = fs::read_dir(self.root.join("events")).await?;

        while let Some(entry) = dir.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let json = fs::read_to_string(entry.path()).await?;
            let event: Event = serde_json::from_str(&json)?;
            events.push(event);
        }

        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(events)
    }

    async fn save_knowledge(&mut self, node: &KnowledgeNode) -> Result<()> {
        let path = self.knowledge_path(node.id);
        let json = serde_json::to_string_pretty(node)?;
        fs::write(&path, json).await?;
        self.set_pending().await;
        Ok(())
    }

    async fn load_knowledge(&self, id: NodeId) -> Result<Option<KnowledgeNode>> {
        let path = self.knowledge_path(id);
        match fs::read_to_string(&path).await {
            Ok(json) => {
                let node = serde_json::from_str(&json)?;
                Ok(Some(node))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_knowledge(&self) -> Result<Vec<KnowledgeNode>> {
        let mut nodes = Vec::new();
        let mut dir = fs::read_dir(self.root.join("knowledge")).await?;

        while let Some(entry) = dir.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let json = fs::read_to_string(entry.path()).await?;
            let node: KnowledgeNode = serde_json::from_str(&json)?;
            nodes.push(node);
        }

        Ok(nodes)
    }

    async fn commit(&mut self, message: &str) -> Result<()> {
        if self.is_pending().await {
            let msg = message.to_string();
            self.do_commit_sync(&msg)?;
            *self.pending.lock().await = false;
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<()> {
        // Git rollback: reset to last commit
        let _ = self.open_repo()
            .and_then(|repo| {
                repo.head()
                    .and_then(|h| h.peel_to_commit())
                    .and_then(|commit| {
                        repo.reset(
                            commit.as_object(),
                            git2::ResetType::Hard,
                            None,
                        )
                    })
            });
        *self.pending.lock().await = false;
        Ok(())
    }
}
