//! Job Manager for async task execution.
//!
//! This module provides the JobManager trait and default implementation
//! for managing asynchronous task execution with sync/async modes.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use devman_core::{GoalId, TaskId};

/// Job ID type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct JobId(pub String);

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl JobId {
    /// Create a new job ID
    pub fn new() -> Self {
        Self(format!("job_{}", ulid::Ulid::new()))
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

use std::fmt;

/// Job status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is pending to start
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job timed out
    Timeout,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Cancelled => write!(f, "cancelled"),
            JobStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// Job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job ID
    pub id: JobId,
    /// Job type
    pub job_type: JobType,
    /// Current status
    pub status: JobStatus,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Started timestamp (None if pending)
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completed timestamp (None if not completed)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// Result data (set when completed)
    pub result: Option<serde_json::Value>,
    /// Error information (set when failed)
    pub error: Option<JobError>,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Progress message
    pub progress_message: String,
}

/// Job type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobType {
    /// Goal creation job
    CreateGoal {
        title: String,
        description: String,
    },
    /// Task creation job
    CreateTask {
        title: String,
        goal_id: Option<GoalId>,
    },
    /// Quality check job
    QualityCheck {
        check_type: String,
        target: Option<String>,
    },
    /// Tool execution job
    ToolExecution {
        tool: String,
        command: String,
    },
    /// Custom job
    Custom {
        name: String,
        data: serde_json::Value,
    },
}

/// Job error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobError {
    /// Error code (-32000 ~ -32004)
    pub code: i32,
    /// Error message
    pub message: String,
    /// Hint for AI to fix
    pub hint: Option<String>,
    /// Whether the error is retryable
    pub retryable: bool,
    /// Additional error data
    pub data: Option<serde_json::Value>,
}

/// Job creation request
#[derive(Debug, Clone)]
pub struct CreateJobRequest {
    /// Job type
    pub job_type: JobType,
    /// Timeout in seconds (default: 30s sync, 300s async)
    pub timeout_seconds: Option<u64>,
}

/// Job status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    /// Job ID
    pub job_id: String,
    /// Job status
    pub status: String,
    /// Progress (0-100)
    pub progress: u8,
    /// Progress message
    pub progress_message: String,
    /// Created timestamp
    pub created_at: String,
    /// Started timestamp (if running/completed)
    pub started_at: Option<String>,
    /// Completed timestamp (if completed)
    pub completed_at: Option<String>,
    /// Result (if completed successfully)
    pub result: Option<serde_json::Value>,
    /// Error (if failed/timed-out/cancelled)
    pub error: Option<JobError>,
}

/// Filter for listing jobs
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    /// Filter by status
    pub status: Option<JobStatus>,
    /// Filter by job type
    pub job_type: Option<JobType>,
    /// Maximum results to return
    pub limit: Option<usize>,
    /// Include completed jobs
    pub include_completed: bool,
}

/// JobManager trait - manages async job execution
#[async_trait]
pub trait JobManager: Send + Sync {
    /// Create a new job
    async fn create_job(&self, request: CreateJobRequest) -> Result<JobId, JobError>;

    /// Get job status
    async fn get_job_status(&self, job_id: &JobId) -> Option<JobStatusResponse>;

    /// Cancel a job
    async fn cancel_job(&self, job_id: &JobId) -> Result<(), JobError>;

    /// List jobs with optional filter
    async fn list_jobs(&self, filter: JobFilter) -> Vec<JobStatusResponse>;

    /// Wait for job completion (with timeout)
    async fn wait_for_completion(
        &self,
        job_id: &JobId,
        timeout: Duration,
    ) -> Option<JobStatusResponse>;
}

/// In-memory job manager implementation
pub struct InMemoryJobManager {
    /// Jobs storage
    jobs: Arc<Mutex<HashMap<JobId, Job>>>,
    /// Sync mode threshold (30 seconds)
    sync_threshold: Duration,
    /// Cleanup interval
    cleanup_interval: Duration,
    /// Last cleanup timestamp
    last_cleanup: Arc<Mutex<Instant>>,
}

impl InMemoryJobManager {
    /// Create a new in-memory job manager
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            sync_threshold: Duration::from_secs(30),
            cleanup_interval: Duration::from_secs(300),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Create with custom sync threshold
    pub fn with_sync_threshold(threshold_seconds: u64) -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            sync_threshold: Duration::from_secs(threshold_seconds),
            cleanup_interval: Duration::from_secs(300),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Determine if job should run synchronously
    fn should_run_sync(&self, request: &CreateJobRequest) -> bool {
        match request.timeout_seconds {
            Some(timeout) => timeout <= 30,
            None => self.sync_threshold.as_secs() <= 30,
        }
    }
}

impl Default for InMemoryJobManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobManager for InMemoryJobManager {
    async fn create_job(&self, request: CreateJobRequest) -> Result<JobId, JobError> {
        let job_id = JobId::new();
        let timeout = request.timeout_seconds.unwrap_or(300);

        let job = Job {
            id: job_id.clone(),
            job_type: request.job_type.clone(),
            status: JobStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            timeout_seconds: timeout,
            result: None,
            error: None,
            progress: 0,
            progress_message: "Job created".to_string(),
        };

        let mut jobs = self.jobs.lock().await;
        jobs.insert(job_id.clone(), job);

        debug!("Created job {} with timeout {}s", job_id, timeout);

        // If sync mode, mark as running immediately
        if self.should_run_sync(&request) {
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = JobStatus::Running;
                job.started_at = Some(chrono::Utc::now());
                job.progress_message = "Running synchronously".to_string();
            }
        }

        Ok(job_id)
    }

    async fn get_job_status(&self, job_id: &JobId) -> Option<JobStatusResponse> {
        let jobs = self.jobs.lock().await;
        jobs.get(job_id).map(|job| JobStatusResponse {
            job_id: job.id.to_string(),
            status: format!("{:?}", job.status),
            progress: job.progress,
            progress_message: job.progress_message.clone(),
            created_at: job.created_at.to_rfc3339(),
            started_at: job.started_at.map(|t| t.to_rfc3339()),
            completed_at: job.completed_at.map(|t| t.to_rfc3339()),
            result: job.result.clone(),
            error: job.error.clone(),
        })
    }

    async fn cancel_job(&self, job_id: &JobId) -> Result<(), JobError> {
        let mut jobs = self.jobs.lock().await;

        match jobs.get_mut(job_id) {
            Some(job) => {
                if job.status == JobStatus::Running || job.status == JobStatus::Pending {
                    job.status = JobStatus::Cancelled;
                    job.completed_at = Some(chrono::Utc::now());
                    job.error = Some(JobError {
                        code: -32004,
                        message: "Job cancelled by user".to_string(),
                        hint: Some("The job was cancelled. You can retry or create a new job.".to_string()),
                        retryable: true,
                        data: None,
                    });
                    job.progress_message = "Job cancelled".to_string();
                    info!("Job {} cancelled", job_id);
                    Ok(())
                } else {
                    Err(JobError {
                        code: -32001,
                        message: format!("Cannot cancel job in {} state", job.status),
                        hint: Some("Only pending or running jobs can be cancelled.".to_string()),
                        retryable: false,
                        data: None,
                    })
                }
            }
            None => Err(JobError {
                code: -32002,
                message: format!("Job {} not found", job_id),
                hint: None,
                retryable: false,
                data: None,
            }),
        }
    }

    async fn list_jobs(&self, filter: JobFilter) -> Vec<JobStatusResponse> {
        let jobs = self.jobs.lock().await;
        let mut results: Vec<_> = jobs
            .values()
            .filter(|job| {
                // Filter by status
                if let Some(status) = &filter.status {
                    if job.status != *status {
                        return false;
                    }
                }

                // Filter completed jobs
                if !filter.include_completed
                    && (job.status == JobStatus::Completed
                        || job.status == JobStatus::Failed
                        || job.status == JobStatus::Cancelled
                        || job.status == JobStatus::Timeout)
                {
                    return false;
                }

                true
            })
            .map(|job| JobStatusResponse {
                job_id: job.id.to_string(),
                status: format!("{:?}", job.status),
                progress: job.progress,
                progress_message: job.progress_message.clone(),
                created_at: job.created_at.to_rfc3339(),
                started_at: job.started_at.map(|t| t.to_rfc3339()),
                completed_at: job.completed_at.map(|t| t.to_rfc3339()),
                result: job.result.clone(),
                error: job.error.clone(),
            })
            .collect();

        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        results
    }

    async fn wait_for_completion(
        &self,
        job_id: &JobId,
        timeout: Duration,
    ) -> Option<JobStatusResponse> {
        let start = Instant::now();
        let sleep_duration = Duration::from_millis(100);

        while start.elapsed() < timeout {
            if let Some(status) = self.get_job_status(job_id).await {
                match status.status.as_str() {
                    "Completed" | "Failed" | "Cancelled" | "Timeout" => return Some(status),
                    _ => {}
                }
            }
            tokio::time::sleep(sleep_duration).await;
        }

        None
    }
}

/// Persisted job snapshot (for jobs.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSnapshot {
    /// Snapshot version
    pub version: u32,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Jobs snapshot
    pub jobs: Vec<Job>,
}

impl Default for JobSnapshot {
    fn default() -> Self {
        Self {
            version: 1,
            updated_at: chrono::Utc::now(),
            jobs: Vec::new(),
        }
    }
}

/// Error codes for DevMan MCP Server
pub mod error_codes {
    /// Generic business error
    pub const BUSINESS_ERROR: i32 = -32000;
    /// State conflict (task already completed, goal finished)
    pub const STATE_CONFLICT: i32 = -32001;
    /// Resource not found
    pub const RESOURCE_NOT_FOUND: i32 = -32002;
    /// Async job timeout
    pub const JOB_TIMEOUT: i32 = -32003;
    /// Async job cancelled
    pub const JOB_CANCELLED: i32 = -32004;
}

use std::collections::HashMap;
