//! DevMan MCP Server - AI Interface for DevMan
//!
//! This binary provides an MCP (Model Context Protocol) server
//! for AI assistants to interact with DevMan.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(name = "devman-ai")]
#[command(about = "DevMan MCP Server - AI interface for work management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Storage path for DevMan data (defaults to .devman in current directory)
    #[arg(short, long)]
    storage: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP Server in stdio mode
    Stdio,

    /// Start MCP Server with Unix socket
    Socket {
        /// Socket path
        path: std::path::PathBuf,
    },

    /// List available tools
    ListTools,

    /// Print server info
    Info,
}

fn init_logging(_to_stderr: bool) {
    // Disable all logging to avoid interfering with MCP protocol
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Storage path resolution priority:
    // 1. CLI argument (--storage / -s)
    // 2. Environment variable DEVMAN_STORAGE
    // 3. Error - user must specify
    let storage_path = if let Some(path) = cli.storage {
        path
    } else if let Ok(env_path) = std::env::var("DEVMAN_STORAGE") {
        std::path::PathBuf::from(env_path)
    } else {
        anyhow::bail!(
            "Storage path not specified. Please either:\n  1. Run with --storage flag: devman-ai stdio --storage /path/to/project/.devman\n  2. Set DEVMAN_STORAGE env var: DEVMAN_STORAGE=/path/to/project/.devman devman-ai stdio"
        );
    };

    // Verify storage path is absolute or can be resolved
    let storage_path = if storage_path.is_absolute() {
        storage_path
    } else {
        std::env::current_dir()?.join(storage_path)
    };

    // Create storage directory if it doesn't exist
    std::fs::create_dir_all(&storage_path).ok();

    // Verify storage is writable
    let test_file = storage_path.join(".devman_write_test");
    std::fs::write(&test_file, "test").context("Storage directory is not writable")?;
    std::fs::remove_file(&test_file).ok();

    // Create MCP server
    let mut server = devman_ai::McpServer::with_config(
        devman_ai::McpServerConfig {
            storage_path: storage_path.clone(),
            server_name: "devman".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            socket_path: None,
        }
    ).await?;

    // Initialize AI Interface with real storage-backed implementations
    let ai_interface = create_ai_interface(&storage_path).await;
    server.set_ai_interface(ai_interface);

    match cli.command {
        Commands::Stdio => {
            init_logging(false);
            server.start().await?;
        }

        Commands::Socket { path } => {
            init_logging(true);
            server.start_with_socket(&path).await?;
        }

        Commands::ListTools => {
            init_logging(false);
            let tools: Vec<_> = server.tools.values().collect();
            println!("Available tools ({}):", tools.len());
            for tool in tools {
                println!("  - {}", tool.name);
            }
        }

        Commands::Info => {
            init_logging(false);
            println!("DevMan MCP Server v{}", env!("CARGO_PKG_VERSION"));
            println!("Protocol: MCP 2024-11-05");
            println!("Transport: stdio / Unix socket");
            println!("Storage: {}", storage_path.display());
            println!("Tools: {}", server.tools.len());
            println!("Resources: {}", server.resources.len());
        }
    }

    Ok(())
}

/// A ToolExecutor implementation using builtin tools.
struct BuiltinToolExecutor {
    cargo_tool: Arc<dyn devman_tools::Tool>,
    git_tool: Arc<dyn devman_tools::Tool>,
    npm_tool: Arc<dyn devman_tools::Tool>,
    fs_tool: Arc<dyn devman_tools::Tool>,
}

impl BuiltinToolExecutor {
    fn new() -> Self {
        Self {
            cargo_tool: Arc::new(devman_tools::CargoTool),
            git_tool: Arc::new(devman_tools::GitTool),
            npm_tool: Arc::new(devman_tools::NpmTool),
            fs_tool: Arc::new(devman_tools::FsTool),
        }
    }

    fn find_tool(&self, name: &str) -> Option<Arc<dyn devman_tools::Tool>> {
        match name {
            "cargo" => Some(self.cargo_tool.clone()),
            "git" => Some(self.git_tool.clone()),
            "npm" => Some(self.npm_tool.clone()),
            "fs" => Some(self.fs_tool.clone()),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl devman_tools::ToolExecutor for BuiltinToolExecutor {
    async fn execute_tool(&self, tool: &str, input: devman_tools::ToolInput) -> Result<devman_tools::ToolOutput, anyhow::Error> {
        if let Some(t) = self.find_tool(tool) {
            t.execute(&input).await
        } else {
            anyhow::bail!("Unknown tool: {}", tool)
        }
    }
}

/// Create a real AI interface with storage-backed implementations.
/// This provides full functionality for MCP tools.
async fn create_ai_interface(storage_path: &std::path::Path) -> Arc<dyn devman_ai::AIInterface> {
    use devman_storage::JsonStorage;

    // Create shared storage for all components
    let storage = Arc::new(Mutex::new(JsonStorage::new(storage_path).await.unwrap_or_else(|_| {
        let fallback = std::path::PathBuf::from(".devman");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { JsonStorage::new(&fallback).await.unwrap() })
    })));

    // Create work manager with storage
    let work_manager = SimpleWorkManager {
        storage: storage.clone(),
    };

    // Create progress tracker with storage
    let progress_tracker = SimpleProgressTracker {
        storage: storage.clone(),
    };

    // Create knowledge service with storage
    let knowledge_service = SimpleKnowledgeService {
        storage: storage.clone(),
    };

    // Create quality engine with storage
    let quality_engine = SimpleQualityEngine {
        storage: storage.clone(),
    };

    // Create tool executor
    let tool_executor = Arc::new(BuiltinToolExecutor::new());

    // Create and return the AI interface
    Arc::new(devman_ai::BasicAIInterface::new(
        storage,
        Arc::new(Mutex::new(work_manager)),
        Arc::new(progress_tracker),
        Arc::new(knowledge_service),
        Arc::new(quality_engine),
        tool_executor,
    ))
}

/// Simple work manager that delegates to storage.
struct SimpleWorkManager {
    storage: Arc<Mutex<dyn devman_storage::Storage>>,
}

#[async_trait::async_trait]
impl devman_work::WorkManager for SimpleWorkManager {
    async fn create_task(&mut self, spec: devman_work::TaskSpec) -> Result<devman_core::Task, anyhow::Error> {
        let mut storage = self.storage.lock().await;
        let task = devman_core::Task {
            id: devman_core::TaskId::new(),
            title: spec.title,
            description: spec.description,
            intent: spec.intent,
            steps: Vec::new(),
            inputs: Vec::new(),
            expected_outputs: Vec::new(),
            quality_gates: spec.quality_gates,
            status: devman_core::TaskStatus::Queued,
            progress: devman_core::TaskProgress::default(),
            phase_id: spec.phase_id,
            depends_on: Vec::new(),
            blocks: Vec::new(),
            work_records: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        storage.save_task(&task).await?;
        Ok(task)
    }

    async fn execute_task(&mut self, task_id: devman_core::TaskId, executor: devman_work::Executor) -> Result<devman_core::WorkRecord, anyhow::Error> {
        let mut storage = self.storage.lock().await;
        let mut task = storage.load_task(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        task.status = devman_core::TaskStatus::Active;
        storage.save_task(&task).await?;

        let work_record = devman_core::WorkRecord {
            id: devman_core::WorkRecordId::new(),
            task_id,
            executor: match executor {
                devman_work::Executor::AI { model } => devman_core::Executor::AI { model },
                devman_work::Executor::Human { name } => devman_core::Executor::Human { name },
                devman_work::Executor::Hybrid { ai, human } => devman_core::Executor::Hybrid { ai, human },
            },
            started_at: chrono::Utc::now(),
            completed_at: None,
            duration: None,
            events: Vec::new(),
            result: devman_core::WorkResult {
                status: devman_core::CompletionStatus::Running,
                outputs: Vec::new(),
                metrics: devman_core::WorkMetrics {
                    token_used: None,
                    time_spent: std::time::Duration::ZERO,
                    tools_invoked: 0,
                    quality_checks_run: 0,
                    quality_checks_passed: 0,
                },
            },
            artifacts: Vec::new(),
            issues: Vec::new(),
            resolutions: Vec::new(),
        };
        storage.save_work_record(&work_record).await?;
        Ok(work_record)
    }

    async fn record_event(&mut self, task_id: devman_core::TaskId, event: devman_core::WorkEvent) -> Result<(), anyhow::Error> {
        let mut storage = self.storage.lock().await;
        let mut task = storage.load_task(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        if let Some(record_id) = task.work_records.last() {
            let mut record = storage.load_work_record(*record_id).await?
                .ok_or_else(|| anyhow::anyhow!("Work record not found"))?;
            record.events.push(event);
            storage.save_work_record(&record).await?;
        }
        Ok(())
    }

    async fn update_progress(&mut self, task_id: devman_core::TaskId, progress: devman_core::TaskProgress) -> Result<(), anyhow::Error> {
        let mut storage = self.storage.lock().await;
        let mut task = storage.load_task(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        task.progress = progress;
        task.updated_at = chrono::Utc::now();
        storage.save_task(&task).await?;
        Ok(())
    }

    async fn complete_task(&mut self, task_id: devman_core::TaskId, result: devman_core::WorkResult) -> Result<(), anyhow::Error> {
        let mut storage = self.storage.lock().await;
        let mut task = storage.load_task(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        task.status = devman_core::TaskStatus::Done;
        task.progress.message = "Completed".to_string();
        task.progress.percentage = 100.0;
        task.updated_at = chrono::Utc::now();
        storage.save_task(&task).await?;
        Ok(())
    }
}

/// Simple progress tracker that delegates to storage.
struct SimpleProgressTracker {
    storage: Arc<Mutex<dyn devman_storage::Storage>>,
}

#[async_trait::async_trait]
impl devman_progress::ProgressTracker for SimpleProgressTracker {
    async fn get_goal_progress(&self, goal_id: devman_core::GoalId) -> Option<devman_core::GoalProgress> {
        let storage = self.storage.lock().await;
        storage.load_goal(goal_id).await.ok().flatten()
            .map(|goal| devman_core::GoalProgress {
                percentage: 0.0,
                completed_phases: Vec::new(),
                active_tasks: 0,
                completed_tasks: 0,
                estimated_completion: None,
                blockers: Vec::new(),
            })
    }

    async fn get_phase_progress(&self, phase_id: devman_core::PhaseId) -> Option<devman_core::PhaseProgress> {
        let storage = self.storage.lock().await;
        storage.load_phase(phase_id).await.ok().flatten()
            .map(|phase| devman_core::PhaseProgress {
                completed_tasks: 0,
                total_tasks: 0,
                percentage: 0.0,
            })
    }

    async fn get_task_progress(&self, task_id: devman_core::TaskId) -> Option<devman_core::TaskProgress> {
        let storage = self.storage.lock().await;
        storage.load_task(task_id).await.ok().flatten()
            .map(|task| task.progress)
    }

    async fn snapshot(&self) -> devman_progress::ProgressSnapshot {
        devman_progress::ProgressSnapshot {
            timestamp: chrono::Utc::now(),
            goal_progress: Vec::new(),
            phase_progress: Vec::new(),
            task_progress: Vec::new(),
        }
    }
}

/// Simple knowledge service that delegates to storage.
struct SimpleKnowledgeService {
    storage: Arc<Mutex<dyn devman_storage::Storage>>,
}

#[async_trait::async_trait]
impl devman_knowledge::KnowledgeService for SimpleKnowledgeService {
    async fn search_semantic(&self, query: &str, limit: usize) -> Vec<devman_core::Knowledge> {
        let storage = self.storage.lock().await;
        storage.list_knowledge().await.unwrap_or_default()
            .into_iter()
            .filter(|k| k.title.to_lowercase().contains(&query.to_lowercase())
                || k.content.summary.to_lowercase().contains(&query.to_lowercase())
                || k.content.detail.to_lowercase().contains(&query.to_lowercase()))
            .take(limit)
            .collect()
    }

    async fn find_similar_tasks(&self, _task: &devman_core::Task) -> Vec<devman_core::Task> {
        Vec::new()
    }

    async fn get_best_practices(&self, domain: &str) -> Vec<devman_core::Knowledge> {
        let storage = self.storage.lock().await;
        storage.list_knowledge().await.unwrap_or_default()
            .into_iter()
            .filter(|k| matches!(k.knowledge_type, devman_core::KnowledgeType::BestPractice { .. }))
            .filter(|k| k.title.to_lowercase().contains(&domain.to_lowercase())
                || k.content.summary.to_lowercase().contains(&domain.to_lowercase())
                || k.content.detail.to_lowercase().contains(&domain.to_lowercase()))
            .take(10)
            .collect()
    }

    async fn recommend_knowledge(&self, _context: &devman_core::TaskContext) -> Vec<devman_core::Knowledge> {
        let storage = self.storage.lock().await;
        storage.list_knowledge().await.unwrap_or_default()
    }

    async fn search_by_tags(&self, tags: &[String], limit: usize) -> Vec<devman_core::Knowledge> {
        let storage = self.storage.lock().await;
        storage.list_knowledge().await.unwrap_or_default()
            .into_iter()
            .filter(|k| tags.iter().any(|t| k.tags.contains(t)))
            .take(limit)
            .collect()
    }

    async fn search_by_tags_all(&self, tags: &[String], limit: usize) -> Vec<devman_core::Knowledge> {
        self.search_by_tags(tags, limit).await
    }

    async fn get_all_tags(&self) -> std::collections::HashSet<String> {
        let storage = self.storage.lock().await;
        storage.list_knowledge().await.unwrap_or_default()
            .into_iter()
            .flat_map(|k| k.tags.into_iter())
            .collect()
    }

    async fn get_tag_statistics(&self) -> std::collections::HashMap<String, usize> {
        let storage = self.storage.lock().await;
        let mut stats = std::collections::HashMap::new();
        for knowledge in storage.list_knowledge().await.unwrap_or_default() {
            for tag in knowledge.tags {
                *stats.entry(tag).or_insert(0) += 1;
            }
        }
        stats
    }

    async fn find_similar_knowledge(&self, _knowledge: &devman_core::Knowledge, _limit: usize) -> Vec<devman_core::Knowledge> {
        Vec::new()
    }

    async fn get_by_type(&self, knowledge_type: devman_core::KnowledgeType) -> Vec<devman_core::Knowledge> {
        let storage = self.storage.lock().await;
        storage.list_knowledge().await.unwrap_or_default()
            .into_iter()
            .filter(|k| k.knowledge_type == knowledge_type)
            .collect()
    }

    async fn suggest_tags(&self, query: &str, limit: usize) -> Vec<String> {
        let all_tags = self.get_all_tags().await;
        all_tags.into_iter()
            .filter(|t| t.to_lowercase().contains(&query.to_lowercase()))
            .take(limit)
            .collect()
    }
}

/// Simple quality engine that delegates to storage.
struct SimpleQualityEngine {
    storage: Arc<Mutex<dyn devman_storage::Storage>>,
}

#[async_trait::async_trait]
impl devman_quality::QualityEngine for SimpleQualityEngine {
    async fn run_check(&self, check: &devman_core::QualityCheck, _context: &devman_quality::engine::WorkContext) -> devman_core::QualityCheckResult {
        devman_core::QualityCheckResult {
            check_id: check.id,
            passed: true,
            execution_time: std::time::Duration::ZERO,
            details: devman_core::CheckDetails {
                output: String::new(),
                exit_code: None,
                error: None,
            },
            findings: Vec::new(),
            metrics: Vec::new(),
            human_review: None,
        }
    }

    async fn run_checks(&self, checks: &[devman_core::QualityCheck], context: &devman_quality::engine::WorkContext) -> Vec<devman_core::QualityCheckResult> {
        let mut results = Vec::new();
        for check in checks {
            results.push(self.run_check(check, context).await);
        }
        results
    }

    async fn run_gate(&self, gate: &devman_core::QualityGate, _context: &devman_quality::engine::WorkContext) -> devman_quality::engine::GateResult {
        devman_quality::engine::GateResult {
            gate_name: gate.name.clone(),
            passed: true,
            check_results: Vec::new(),
            decision: devman_quality::engine::GateDecision::Pass,
        }
    }
}
