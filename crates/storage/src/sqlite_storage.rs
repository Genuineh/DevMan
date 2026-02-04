//! SQLite storage backend for DevMan.
//!
//! Provides a high-performance SQLite-based storage with full SQL query support.
//! This is the recommended storage backend for production use.

use async_trait::async_trait;
use sqlx::Row;
use devman_core::{
    Goal, GoalId, Project, ProjectId, Phase, PhaseId, Task, TaskId, TaskFilter,
    Event, EventId, Knowledge, KnowledgeId, QualityCheck, QualityCheckId,
    WorkRecord, WorkRecordId, KnowledgeEmbedding,
};
use std::path::Path;
use tracing::warn;

use super::trait_::{Storage, StorageError, Result};

/// SQLite storage implementation.
#[derive(Clone)]
pub struct SqliteStorage {
    /// Database connection pool
    pool: sqlx::SqlitePool,
}

impl SqliteStorage {
    /// Create a new SQLite storage instance.
    pub async fn new(db_path: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(db_path)
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;

        let storage = Self { pool };
        storage.init_schema().await?;

        Ok(storage)
    }

    /// Create a new SQLite storage instance from a path.
    pub async fn new_from_path(path: &Path) -> Result<Self> {
        Self::new(path.to_str().unwrap_or(":memory:")).await
    }

    /// Initialize the database schema.
    async fn init_schema(&self) -> Result<()> {
        // Create entities table (stores all entities as JSON)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        // Create embeddings table for vector storage
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS embeddings (
                knowledge_id TEXT PRIMARY KEY,
                embedding BLOB NOT NULL,
                model TEXT NOT NULL,
                dimension INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    /// Create an in-memory SQLite storage for testing.
    pub async fn in_memory() -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;

        let storage = Self { pool };
        storage.init_schema().await?;

        Ok(storage)
    }

    /// Helper to extract string from row.
    fn get_string(row: &sqlx::sqlite::SqliteRow, column: &str) -> String {
        row.try_get(column).unwrap_or_default()
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    // === Goal operations ===

    async fn save_goal(&mut self, goal: &Goal) -> Result<()> {
        let data = serde_json::to_string(goal).map_err(|e| StorageError::Json(e.into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR REPLACE INTO entities (id, entity_type, data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(goal.id.to_string())
        .bind("goal")
        .bind(data)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_goal(&self, id: GoalId) -> Result<Option<Goal>> {
        let row = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE id = ? AND entity_type = 'goal'",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let data = Self::get_string(&row, "data");
                let goal: Goal = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))?;
                Ok(Some(goal))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    async fn list_goals(&self) -> Result<Vec<Goal>> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'goal' ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let goals: Vec<Goal> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(goals)
    }

    // === Project operations ===

    async fn save_project(&mut self, project: &Project) -> Result<()> {
        let data = serde_json::to_string(project).map_err(|e| StorageError::Json(e.into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR REPLACE INTO entities (id, entity_type, data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(project.id.to_string())
        .bind("project")
        .bind(data)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_project(&self, id: ProjectId) -> Result<Option<Project>> {
        let row = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE id = ? AND entity_type = 'project'",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let data = Self::get_string(&row, "data");
                let project: Project = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))?;
                Ok(Some(project))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    // === Phase operations ===

    async fn save_phase(&mut self, phase: &Phase) -> Result<()> {
        let data = serde_json::to_string(phase).map_err(|e| StorageError::Json(e.into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR REPLACE INTO entities (id, entity_type, data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(phase.id.to_string())
        .bind("phase")
        .bind(data)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_phase(&self, id: PhaseId) -> Result<Option<Phase>> {
        let row = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE id = ? AND entity_type = 'phase'",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let data = Self::get_string(&row, "data");
                let phase: Phase = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))?;
                Ok(Some(phase))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    // === Task operations ===

    async fn save_task(&mut self, task: &Task) -> Result<()> {
        let data = serde_json::to_string(task).map_err(|e| StorageError::Json(e.into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR REPLACE INTO entities (id, entity_type, data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(task.id.to_string())
        .bind("task")
        .bind(data)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_task(&self, id: TaskId) -> Result<Option<Task>> {
        let row = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE id = ? AND entity_type = 'task'",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let data = Self::get_string(&row, "data");
                let task: Task = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))?;
                Ok(Some(task))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'task' ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let mut tasks: Vec<Task> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        // Apply filters
        if let Some(statuses) = &filter.status {
            let status_set: std::collections::HashSet<_> = statuses.iter().collect();
            tasks.retain(|t| status_set.contains(&t.status));
        }

        Ok(tasks)
    }

    async fn delete_task(&mut self, id: TaskId) -> Result<()> {
        sqlx::query("DELETE FROM entities WHERE id = ? AND entity_type = 'task'")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    // === Event operations ===

    async fn save_event(&mut self, event: &Event) -> Result<()> {
        let data = serde_json::to_string(event).map_err(|e| StorageError::Json(e.into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR REPLACE INTO entities (id, entity_type, data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(event.id.to_string())
        .bind("event")
        .bind(data)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_event(&self, id: EventId) -> Result<Option<Event>> {
        let row = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE id = ? AND entity_type = 'event'",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let data = Self::get_string(&row, "data");
                let event: Event = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))?;
                Ok(Some(event))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    async fn list_events(&self) -> Result<Vec<Event>> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'event' ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let events: Vec<Event> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(events)
    }

    // === Knowledge operations ===

    async fn save_knowledge(&mut self, knowledge: &Knowledge) -> Result<()> {
        let data = serde_json::to_string(knowledge).map_err(|e| StorageError::Json(e.into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR REPLACE INTO entities (id, entity_type, data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(knowledge.id.to_string())
        .bind("knowledge")
        .bind(data)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_knowledge(&self, id: KnowledgeId) -> Result<Option<Knowledge>> {
        let row = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE id = ? AND entity_type = 'knowledge'",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let data = Self::get_string(&row, "data");
                let knowledge: Knowledge = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))?;
                Ok(Some(knowledge))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    async fn list_knowledge(&self) -> Result<Vec<Knowledge>> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'knowledge' ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let knowledge_items: Vec<Knowledge> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(knowledge_items)
    }

    // === Vector Embedding operations ===

    async fn save_vector_embedding(&mut self, embedding: &KnowledgeEmbedding) -> Result<()> {
        let embedding_bytes = embedding
            .embedding
            .iter()
            .flat_map(|f| f.to_bits().to_le_bytes())
            .collect::<Vec<u8>>();

        sqlx::query(
            "INSERT OR REPLACE INTO embeddings (knowledge_id, embedding, model, dimension, created_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(embedding.knowledge_id.to_string())
        .bind(embedding_bytes)
        .bind(format!("{:?}", embedding.model))
        .bind(embedding.embedding.len() as i32)
        .bind(embedding.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_vector_embedding(&self, knowledge_id: &str) -> Result<Option<KnowledgeEmbedding>> {
        let row = sqlx::query(
            "SELECT * FROM embeddings WHERE knowledge_id = ?",
        )
        .bind(knowledge_id)
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let embedding_bytes: Vec<u8> = row.try_get("embedding").unwrap_or_default();
                let embedding_vec: Vec<f32> = embedding_bytes
                    .chunks_exact(4)
                    .map(|bytes| f32::from_le_bytes(*<&[u8; 4]>::try_from(bytes).unwrap()))
                    .collect();
                let model_str = Self::get_string(&row, "model");
                let model = match model_str.as_str() {
                    "Qwen3Embedding0_6B" => devman_core::EmbeddingModel::Qwen3Embedding0_6B,
                    "OpenAIAda002" => devman_core::EmbeddingModel::OpenAIAda002,
                    _ => devman_core::EmbeddingModel::Qwen3Embedding0_6B,
                };
                Ok(Some(KnowledgeEmbedding {
                    knowledge_id: knowledge_id.parse().unwrap_or_default(),
                    embedding: embedding_vec,
                    model,
                    created_at: Self::get_string(&row, "created_at").parse().unwrap_or(chrono::Utc::now()),
                }))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    async fn list_vector_embeddings(&self) -> Result<Vec<KnowledgeEmbedding>> {
        let rows = sqlx::query("SELECT * FROM embeddings")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let embedding_bytes: Vec<u8> = row.try_get("embedding").unwrap_or_default();
                let embedding_vec: Vec<f32> = embedding_bytes
                    .chunks_exact(4)
                    .map(|bytes| f32::from_le_bytes(*<&[u8; 4]>::try_from(bytes).unwrap()))
                    .collect();
                let model_str = Self::get_string(&row, "model");
                let model = match model_str.as_str() {
                    "Qwen3Embedding0_6B" => devman_core::EmbeddingModel::Qwen3Embedding0_6B,
                    "OpenAIAda002" => devman_core::EmbeddingModel::OpenAIAda002,
                    _ => devman_core::EmbeddingModel::Qwen3Embedding0_6B,
                };
                KnowledgeEmbedding {
                    knowledge_id: Self::get_string(&row, "knowledge_id").parse().unwrap_or_default(),
                    embedding: embedding_vec,
                    model,
                    created_at: Self::get_string(&row, "created_at").parse().unwrap_or(chrono::Utc::now()),
                }
            })
            .collect())
    }

    // === Quality Check operations ===

    async fn save_quality_check(&mut self, check: &QualityCheck) -> Result<()> {
        let data = serde_json::to_string(check).map_err(|e| StorageError::Json(e.into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR REPLACE INTO entities (id, entity_type, data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(check.id.to_string())
        .bind("quality_check")
        .bind(data)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_quality_check(&self, id: QualityCheckId) -> Result<Option<QualityCheck>> {
        let row = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE id = ? AND entity_type = 'quality_check'",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let data = Self::get_string(&row, "data");
                let check: QualityCheck = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))?;
                Ok(Some(check))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    async fn list_quality_checks(&self) -> Result<Vec<QualityCheck>> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'quality_check' ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let checks: Vec<QualityCheck> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(checks)
    }

    // === Work Record operations ===

    async fn save_work_record(&mut self, record: &WorkRecord) -> Result<()> {
        let data = serde_json::to_string(record).map_err(|e| StorageError::Json(e.into()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR REPLACE INTO entities (id, entity_type, data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(record.id.to_string())
        .bind("work_record")
        .bind(data)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        Ok(())
    }

    async fn load_work_record(&self, id: WorkRecordId) -> Result<Option<WorkRecord>> {
        let row = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE id = ? AND entity_type = 'work_record'",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => {
                let data = Self::get_string(&row, "data");
                let record: WorkRecord = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))?;
                Ok(Some(record))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(StorageError::Other(e.to_string())),
        }
    }

    async fn list_work_records(&self, task_id: TaskId) -> Result<Vec<WorkRecord>> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'work_record' ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let records: Vec<WorkRecord> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(records.into_iter().filter(|r| r.task_id == task_id).collect())
    }

    // === Transaction support ===

    async fn commit(&mut self, _message: &str) -> Result<()> {
        Ok(())
    }

    async fn rollback(&mut self) -> Result<()> {
        warn!("Rollback called on SqliteStorage");
        Ok(())
    }
}

// === Extended query methods ===

impl SqliteStorage {
    /// Find all blocked tasks.
    pub async fn find_blocked_tasks(&self) -> Result<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'task' ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let tasks: Vec<Task> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(tasks.into_iter().filter(|t| t.status == devman_core::TaskStatus::Blocked).collect())
    }

    /// Find recent active tasks by work record activity.
    pub async fn find_recent_active_tasks(&self, _days: i32, limit: i32) -> Result<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'task' ORDER BY updated_at DESC LIMIT 100"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let tasks: Vec<Task> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(tasks.into_iter().take(limit as usize).collect())
    }

    /// Get task statistics.
    pub async fn get_task_stats(&self) -> Result<TaskStats> {
        let rows = sqlx::query(
            "SELECT id, data, created_at, updated_at FROM entities WHERE entity_type = 'task'"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Other(e.to_string()))?;

        let tasks: Vec<Task> = rows
            .into_iter()
            .map(|row| {
                let data = Self::get_string(&row, "data");
                serde_json::from_str(&data)
                    .map_err(|e| StorageError::Json(e.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(TaskStats {
            total: tasks.len(),
            completed: tasks.iter().filter(|t| t.status == devman_core::TaskStatus::Done).count(),
            blocked: tasks.iter().filter(|t| t.status == devman_core::TaskStatus::Blocked).count(),
            in_progress: tasks.iter().filter(|t| t.status == devman_core::TaskStatus::Active).count(),
        })
    }

    /// Check if the database is healthy.
    pub async fn health_check(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await.is_ok()
    }
}

/// Task statistics.
pub struct TaskStats {
    /// Total number of tasks.
    pub total: usize,
    /// Number of completed tasks.
    pub completed: usize,
    /// Number of blocked tasks.
    pub blocked: usize,
    /// Number of in-progress tasks.
    pub in_progress: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::{GoalStatus, TaskStatus, TaskIntent, TaskContext, TaskProgress};

    fn create_test_task() -> Task {
        Task {
            id: TaskId::new(),
            phase_id: PhaseId::new(),
            title: "Test Task".to_string(),
            description: "Description".to_string(),
            intent: TaskIntent {
                natural_language: "Test intent".to_string(),
                context: TaskContext {
                    relevant_knowledge: vec![],
                    similar_tasks: vec![],
                    affected_files: vec![],
                },
                success_criteria: vec![],
            },
            steps: vec![],
            inputs: vec![],
            expected_outputs: vec![],
            quality_gates: vec![],
            status: TaskStatus::Idea,
            progress: TaskProgress::default(),
            depends_on: vec![],
            blocks: vec![],
            work_records: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_in_memory_storage() {
        let mut storage = SqliteStorage::in_memory().await.unwrap();

        let goal = Goal {
            id: GoalId::new(),
            title: "Test Goal".to_string(),
            description: "Test description".to_string(),
            success_criteria: vec![],
            progress: devman_core::GoalProgress::default(),
            project_id: ProjectId::new(),
            current_phase: PhaseId::new(),
            status: GoalStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        storage.save_goal(&goal).await.unwrap();
        let loaded = storage.load_goal(goal.id.clone()).await.unwrap().unwrap();

        assert_eq!(loaded.title, goal.title);
        assert_eq!(loaded.id, goal.id);
    }

    #[tokio::test]
    async fn test_task_operations() {
        let mut storage = SqliteStorage::in_memory().await.unwrap();

        let task = create_test_task();

        storage.save_task(&task).await.unwrap();
        let loaded = storage.load_task(task.id.clone()).await.unwrap().unwrap();
        assert_eq!(loaded.title, task.title);

        let filter = TaskFilter::default();
        let tasks = storage.list_tasks(&filter).await.unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_blocked_tasks() {
        let mut storage = SqliteStorage::in_memory().await.unwrap();

        let mut blocked_task = create_test_task();
        blocked_task.title = "Blocked Task".to_string();
        blocked_task.status = TaskStatus::Blocked;

        storage.save_task(&blocked_task).await.unwrap();

        let blocked = storage.find_blocked_tasks().await.unwrap();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].title, "Blocked Task");
    }

    #[tokio::test]
    async fn test_task_stats() {
        let mut storage = SqliteStorage::in_memory().await.unwrap();

        for i in 0..5 {
            let mut task = create_test_task();
            task.title = format!("Task {}", i);
            task.status = match i {
                0 => TaskStatus::Done,
                1 => TaskStatus::Blocked,
                _ => TaskStatus::Queued,
            };
            storage.save_task(&task).await.unwrap();
        }

        let stats = storage.get_task_stats().await.unwrap();
        assert_eq!(stats.total, 5);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.blocked, 1);
    }

    #[tokio::test]
    async fn test_health_check() {
        let storage = SqliteStorage::in_memory().await.unwrap();
        assert!(storage.health_check().await);
    }
}
