//! JSON file storage implementation.
//!
//! Stores data as JSON files in a `.devman` directory and keeps small per-object
//! meta markers (version + updated_at). Full repository snapshot/versioning is
//! expected to be handled by the project's Git repository if desired.

use std::path::Path;
use std::sync::Arc;
use devman_core::{
    Goal, GoalId, Project, ProjectId, Phase, PhaseId, Task, TaskId, TaskFilter,
    Event, EventId, Knowledge, KnowledgeId, QualityCheck, QualityCheckId,
    WorkRecord, WorkRecordId, KnowledgeEmbedding,
};
use super::{Storage, StorageError, Result};
use tokio::fs;
use tokio::sync::Mutex;

/// File-based JSON storage backend.
pub struct JsonStorage {
    root: std::path::PathBuf,
    pending: Arc<Mutex<bool>>,
}

impl JsonStorage {
    /// Create storage. This will create the `.devman/` subdirectories needed for
    /// data and meta markers. It does NOT initialize or manage a Git repository.
    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();

        // Ensure primary directories
        fs::create_dir_all(root.join("goals")).await?;
        fs::create_dir_all(root.join("projects")).await?;
        fs::create_dir_all(root.join("phases")).await?;
        fs::create_dir_all(root.join("tasks")).await?;
        fs::create_dir_all(root.join("events")).await?;
        fs::create_dir_all(root.join("knowledge")).await?;
        fs::create_dir_all(root.join("embeddings")).await?;
        fs::create_dir_all(root.join("quality")).await?;
        fs::create_dir_all(root.join("work_records")).await?;

        // Directories for meta/versioning (only meta markers are stored)
        fs::create_dir_all(root.join("meta").join("goals")).await?;
        fs::create_dir_all(root.join("meta").join("projects")).await?;
        fs::create_dir_all(root.join("meta").join("phases")).await?;
        fs::create_dir_all(root.join("meta").join("tasks")).await?;
        fs::create_dir_all(root.join("meta").join("events")).await?;
        fs::create_dir_all(root.join("meta").join("knowledge")).await?;
        fs::create_dir_all(root.join("meta").join("quality")).await?;
        fs::create_dir_all(root.join("meta").join("work_records")).await?;

        Ok(Self {
            root,
            pending: Arc::new(Mutex::new(false)),
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
    fn embedding_path(&self, knowledge_id: &str) -> std::path::PathBuf {
        self.root.join("embeddings").join(format!("{}.json", knowledge_id))
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
        let _ = fs::write(&path, serde_json::to_string_pretty(&meta)?.as_bytes()).await?;
        Ok(version)
    }

}

#[async_trait::async_trait]
impl Storage for JsonStorage {
    async fn save_goal(&mut self, goal: &Goal) -> Result<()> {
        let path = self.goal_path(goal.id);
        let json = serde_json::to_string_pretty(goal)?;
        fs::write(&path, json.as_bytes()).await?;

        // Versioning (meta only)
        let id_str = format!("{}", goal.id);
        let _ver = self.bump_version("goals", &id_str).await?;

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
        let _ver = self.bump_version("projects", &id_str).await?;

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
        let _ver = self.bump_version("phases", &id_str).await?;

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
        let _ver = self.bump_version("tasks", &id_str).await?;

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
        let _ver = self.bump_version("events", &id_str).await?;

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
        let _ver = self.bump_version("knowledge", &id_str).await?;

        self.set_pending().await;
        Ok(())
    }

    async fn load_knowledge(&self, id: KnowledgeId) -> Result<Option<Knowledge>> {
        read_json(&self.knowledge_path(id)).await
    }

    async fn list_knowledge(&self) -> Result<Vec<Knowledge>> {
        list_dir(&self.root.join("knowledge")).await
    }

    // === Vector Embedding operations ===

    async fn save_vector_embedding(&mut self, embedding: &KnowledgeEmbedding) -> Result<()> {
        let path = self.embedding_path(&embedding.knowledge_id.to_string());
        let json = serde_json::to_string_pretty(embedding)?;
        fs::write(&path, json.as_bytes()).await?;
        self.set_pending().await;
        Ok(())
    }

    async fn load_vector_embedding(&self, knowledge_id: &str) -> Result<Option<KnowledgeEmbedding>> {
        read_json(&self.embedding_path(knowledge_id)).await
    }

    async fn list_vector_embeddings(&self) -> Result<Vec<KnowledgeEmbedding>> {
        list_dir(&self.root.join("embeddings")).await
    }

    async fn save_quality_check(&mut self, check: &QualityCheck) -> Result<()> {
        let path = self.quality_check_path(check.id);
        let json = serde_json::to_string_pretty(check)?;
        fs::write(&path, json.as_bytes()).await?;

        let id_str = format!("{}", check.id);
        let _ver = self.bump_version("quality", &id_str).await?;

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
        let _ver = self.bump_version("work_records", &id_str).await?;

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

    async fn commit(&mut self, _message: &str) -> Result<()> {
        // No Git management by default; commit is a no-op that clears pending state.
        *self.pending.lock().await = false;
        Ok(())
    }

    async fn rollback(&mut self) -> Result<()> {
        // No Git integration; rollback simply clears pending state.
        *self.pending.lock().await = false;
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