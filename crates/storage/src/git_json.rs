//! Git+JSON storage implementation.
//!
//! Stores data as JSON files in a directory, with Git tracking all changes.

use std::path::Path;
use std::sync::Arc;
use devman_core::{
    Goal, GoalId, Project, ProjectId, Phase, PhaseId, Task, TaskId, TaskFilter,
    Event, EventId, Knowledge, KnowledgeId, QualityCheck, QualityCheckId,
    WorkRecord, WorkRecordId,
};
use super::{Storage, StorageError, Result};
use tokio::fs;
use tracing::{debug, info};
use tokio::sync::Mutex;

/// Git+JSON storage backend.
///
/// By default this storage does not initialize or manage a Git repository; projects
/// that wish to use Git should manage it themselves. Versioning is supported via
/// a small per-object meta file and an `archives/` snapshot directory.
pub struct GitJsonStorage {
    root: std::path::PathBuf,
    pending: Arc<Mutex<bool>>,
    enable_git: bool,
}

impl GitJsonStorage {
    /// Create storage without managing Git (recommended: project manages Git itself).
    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        Self::new_with_git(root, false).await
    }

    /// Create storage and optionally enable Git management.
    pub async fn new_with_git(root: impl AsRef<Path>, enable_git: bool) -> Result<Self> {
        let root = root.as_ref().to_path_buf();

        // Ensure primary directories
        fs::create_dir_all(root.join("goals")).await?;
        fs::create_dir_all(root.join("projects")).await?;
        fs::create_dir_all(root.join("phases")).await?;
        fs::create_dir_all(root.join("tasks")).await?;
        fs::create_dir_all(root.join("events")).await?;
        fs::create_dir_all(root.join("knowledge")).await?;
        fs::create_dir_all(root.join("quality")).await?;
        fs::create_dir_all(root.join("work_records")).await?;

        // Directories for meta/versioning
        fs::create_dir_all(root.join("meta").join("goals")).await?;
        fs::create_dir_all(root.join("archives").join("goals")).await?;
        fs::create_dir_all(root.join("meta").join("projects")).await?;
        fs::create_dir_all(root.join("archives").join("projects")).await?;
        fs::create_dir_all(root.join("meta").join("phases")).await?;
        fs::create_dir_all(root.join("archives").join("phases")).await?;
        fs::create_dir_all(root.join("meta").join("tasks")).await?;
        fs::create_dir_all(root.join("archives").join("tasks")).await?;
        fs::create_dir_all(root.join("meta").join("events")).await?;
        fs::create_dir_all(root.join("archives").join("events")).await?;
        fs::create_dir_all(root.join("meta").join("knowledge")).await?;
        fs::create_dir_all(root.join("archives").join("knowledge")).await?;
        fs::create_dir_all(root.join("meta").join("quality")).await?;
        fs::create_dir_all(root.join("archives").join("quality")).await?;
        fs::create_dir_all(root.join("meta").join("work_records")).await?;
        fs::create_dir_all(root.join("archives").join("work_records")).await?;

        // Only initialize Git repository if requested
        if enable_git {
            if !root.join(".git").exists() {
                info!("Initializing Git repository at {}", root.display());
                git2::Repository::init(&root)?;
            }
        }

        Ok(Self {
            root,
            pending: Arc::new(Mutex::new(false)),
            enable_git,
        })
    }

    fn goal_path(&self, id: GoalId) -> std::path::PathBuf {
        self.root.join("goals").join(format!("{}.json", id))
    }
    fn project_path(&self, id: ProjectId) -> std::path::PathBuf {
        self.root.join("projects").join(format!("{}.json", id))
    }
    fn phase_path(&self, id: PhaseId) -> std::path::PathBuf {
        self.root.join("phases").join(format!("{}.json", id))
    }
    fn task_path(&self, id: TaskId) -> std::path::PathBuf {
        self.root.join("tasks").join(format!("{}.json", id))
    }
    fn event_path(&self, id: EventId) -> std::path::PathBuf {
        self.root.join("events").join(format!("{}.json", id))
    }
    fn knowledge_path(&self, id: KnowledgeId) -> std::path::PathBuf {
        self.root.join("knowledge").join(format!("{}.json", id))
    }
    fn quality_check_path(&self, id: QualityCheckId) -> std::path::PathBuf {
        self.root.join("quality").join(format!("{}.json", id))
    }
    fn work_record_path(&self, id: WorkRecordId) -> std::path::PathBuf {
        self.root.join("work_records").join(format!("{}.json", id))
    }

    fn meta_path(&self, kind: &str, id: &str) -> std::path::PathBuf {
        self.root.join("meta").join(kind).join(format!("{}.meta.json", id))
    }

    fn archive_path(&self, kind: &str, id: &str, version: u64) -> std::path::PathBuf {
        self.root.join("archives").join(kind).join(format!("{}.{}.json", id, version))
    }

    fn open_repo(&self) -> std::result::Result<git2::Repository, git2::Error> {
        git2::Repository::open(&self.root)
    }

    async fn set_pending(&self) {
        *self.pending.lock().await = true;
    }
    async fn is_pending(&self) -> bool {
        *self.pending.lock().await
    }

    /// Read and increment per-object version, return new version.
    async fn bump_version(&self, kind: &str, id: &str) -> Result<u64> {
        let path = self.meta_path(kind, id);
        // Read existing
        let mut version = 0u64;
        match fs::read_to_string(&path).await {
            Ok(s) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&s) {
                    if let Some(v) = json.get("version").and_then(|v| v.as_u64()) {
                        version = v;
                    }
                }
            }
            Err(_) => {
                // ignore missing
            }
        }
        version += 1;
        let meta = serde_json::json!({"version": version, "updated_at": chrono::Utc::now()});
        let _ = fs::write(&path, serde_json::to_string_pretty(&meta)?).await?;
        Ok(version)
    }

    /// Archive a snapshot of the object under its versioned archive path.
    async fn archive_snapshot(&self, kind: &str, id: &str, version: u64, json: &str) -> Result<()> {
        let path = self.archive_path(kind, id, version);
        fs::write(&path, json).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Storage for GitJsonStorage {
    async fn save_goal(&mut self, goal: &Goal) -> Result<()> {
        let path = self.goal_path(goal.id);
        let json = serde_json::to_string_pretty(goal)?;
        fs::write(&path, json.as_bytes()).await?;

        // Versioning & archiving
        let id_str = format!("{}", goal.id);
        let ver = self.bump_version("goals", &id_str).await?;
        self.archive_snapshot("goals", &id_str, ver, &json).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_goal(&self, id: GoalId) -> Result<Option<Goal>> {
        read_json(&self.goal_path(id)).await
    }

    async fn list_goals(&self) -> Result<Vec<Goal>> {
        list_dir(&self.root.join("goals")).await
    }

    async fn save_project(&mut self, project: &Project) -> Result<()> {
        let path = self.project_path(project.id);
        let json = serde_json::to_string_pretty(project)?;
        fs::write(&path, json.as_bytes()).await?;

        let id_str = format!("{}", project.id);
        let ver = self.bump_version("projects", &id_str).await?;
        self.archive_snapshot("projects", &id_str, ver, &json).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_project(&self, id: ProjectId) -> Result<Option<Project>> {
        read_json(&self.project_path(id)).await
    }

    async fn save_phase(&mut self, phase: &Phase) -> Result<()> {
        let path = self.phase_path(phase.id);
        let json = serde_json::to_string_pretty(phase)?;
        fs::write(&path, json.as_bytes()).await?;

        let id_str = format!("{}", phase.id);
        let ver = self.bump_version("phases", &id_str).await?;
        self.archive_snapshot("phases", &id_str, ver, &json).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_phase(&self, id: PhaseId) -> Result<Option<Phase>> {
        read_json(&self.phase_path(id)).await
    }

    async fn save_task(&mut self, task: &Task) -> Result<()> {
        let path = self.task_path(task.id);
        let json = serde_json::to_string_pretty(task)?;
        fs::write(&path, json.as_bytes()).await?;

        let id_str = format!("{}", task.id);
        let ver = self.bump_version("tasks", &id_str).await?;
        self.archive_snapshot("tasks", &id_str, ver, &json).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_task(&self, id: TaskId) -> Result<Option<Task>> {
        read_json(&self.task_path(id)).await
    }

    async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<Task>> {
        let all = list_dir(&self.root.join("tasks")).await?;
        Ok(all.into_iter()
            .filter(|t: &Task| {
                if let Some(statuses) = &filter.status {
                    statuses.contains(&t.status)
                } else {
                    true
                }
            })
            .collect())
    }

    async fn delete_task(&mut self, id: TaskId) -> Result<()> {
        fs::remove_file(self.task_path(id)).await.or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) }
        })?;
        self.set_pending().await;
        Ok(())
    }

    async fn save_event(&mut self, event: &Event) -> Result<()> {
        let path = self.event_path(event.id);
        let json = serde_json::to_string_pretty(event)?;
        fs::write(&path, json.as_bytes()).await?;

        let id_str = format!("{}", event.id);
        let ver = self.bump_version("events", &id_str).await?;
        self.archive_snapshot("events", &id_str, ver, &json).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_event(&self, id: EventId) -> Result<Option<Event>> {
        read_json(&self.event_path(id)).await
    }

    async fn list_events(&self) -> Result<Vec<Event>> {
        let mut events = list_dir(&self.root.join("events")).await?;
        events.sort_by(|a: &Event, b| a.timestamp.cmp(&b.timestamp));
        Ok(events)
    }

    async fn save_knowledge(&mut self, knowledge: &Knowledge) -> Result<()> {
        let path = self.knowledge_path(knowledge.id);
        let json = serde_json::to_string_pretty(knowledge)?;
        fs::write(&path, json.as_bytes()).await?;

        let id_str = format!("{}", knowledge.id);
        let ver = self.bump_version("knowledge", &id_str).await?;
        self.archive_snapshot("knowledge", &id_str, ver, &json).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_knowledge(&self, id: KnowledgeId) -> Result<Option<Knowledge>> {
        read_json(&self.knowledge_path(id)).await
    }

    async fn list_knowledge(&self) -> Result<Vec<Knowledge>> {
        list_dir(&self.root.join("knowledge")).await
    }

    async fn save_quality_check(&mut self, check: &QualityCheck) -> Result<()> {
        let path = self.quality_check_path(check.id);
        let json = serde_json::to_string_pretty(check)?;
        fs::write(&path, json.as_bytes()).await?;

        let id_str = format!("{}", check.id);
        let ver = self.bump_version("quality", &id_str).await?;
        self.archive_snapshot("quality", &id_str, ver, &json).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_quality_check(&self, id: QualityCheckId) -> Result<Option<QualityCheck>> {
        read_json(&self.quality_check_path(id)).await
    }

    async fn list_quality_checks(&self) -> Result<Vec<QualityCheck>> {
        list_dir(&self.root.join("quality")).await
    }

    async fn save_work_record(&mut self, record: &WorkRecord) -> Result<()> {
        let path = self.work_record_path(record.id);
        let json = serde_json::to_string_pretty(record)?;
        fs::write(&path, json.as_bytes()).await?;

        let id_str = format!("{}", record.id);
        let ver = self.bump_version("work_records", &id_str).await?;
        self.archive_snapshot("work_records", &id_str, ver, &json).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_work_record(&self, id: WorkRecordId) -> Result<Option<WorkRecord>> {
        read_json(&self.work_record_path(id)).await
    }

    async fn list_work_records(&self, task_id: TaskId) -> Result<Vec<WorkRecord>> {
        let all = list_dir(&self.root.join("work_records")).await?;
        Ok(all.into_iter()
            .filter(|r: &WorkRecord| r.task_id == task_id)
            .collect())
    }

    async fn commit(&mut self, message: &str) -> Result<()> {
        if !self.enable_git {
            // When Git integration is disabled, just clear pending state (no repo ops).
            *self.pending.lock().await = false;
            return Ok(());
        }

        if self.is_pending().await {
            self.do_commit_sync(message)?;
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<()> {
        if !self.enable_git {
            *self.pending.lock().await = false;
            return Ok(());
        }

        let _ = self.open_repo()
            .and_then(|repo| {
                repo.head()
                    .and_then(|h| h.peel_to_commit())
                    .and_then(|commit| {
                        repo.reset(commit.as_object(), git2::ResetType::Hard, None)
                    })
            });
        *self.pending.lock().await = false;
        Ok(())
    }
}

impl GitJsonStorage {
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
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[parent])?;
        } else {
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?;
        }

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                *self.pending.lock().await = false;
            });
        });
        debug!("Committed: {}", message);
        Ok(())
    }
}

async fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<Option<T>> {
    match fs::read_to_string(path).await {
        Ok(json) => {
            let value = serde_json::from_str(&json)?;
            Ok(Some(value))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn list_dir<T: serde::de::DeserializeOwned>(dir: &std::path::Path) -> Result<Vec<T>> {
    let mut items = Vec::new();
    let mut rd = fs::read_dir(dir).await?;
    while let Some(entry) = rd.next_entry().await? {
        if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Ok(Some(item)) = read_json(&entry.path()).await {
            items.push(item);
        }
    }
    Ok(items)
}
