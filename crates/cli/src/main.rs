//! DevMan CLI - AI execution cognitive framework.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use devman_core::{Task, TaskStatus, TaskFilter};
use devman_storage::{GitJsonStorage, Storage};
use devman_execution::{ExecutionEngine, EngineConfig};

#[derive(Parser)]
#[command(name = "devman")]
#[command(about = "AI execution cognitive framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Intent description
        intent: String,
        /// Hypothesis
        #[arg(long)]
        hypothesis: String,
        /// Priority (0-255)
        #[arg(long, default_value = "128")]
        priority: u8,
    },
    /// List tasks
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },
    /// Show task details
    Show {
        /// Task ID
        id: String,
    },
    /// Run execution cycle
    Run {
        /// Number of cycles to run
        #[arg(long, default_value = "1")]
        cycles: usize,
    },
    /// Trigger reflection on reviewed tasks
    Reflect,
    /// Show status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();

    // Open storage
    let storage_path = std::path::PathBuf::from(".devman");
    let mut storage = GitJsonStorage::new(&storage_path).await?;

    match cli.command {
        Commands::Add { intent, hypothesis, priority } => {
            let mut task = Task::new(intent, hypothesis);
            task.priority = priority;
            task.status = TaskStatus::Queued;
            storage.save_task(&task).await?;
            storage.commit("Add task").await?;
            println!("Added task: {} - {}", task.id, task.intent);
        }
        Commands::List { status } => {
            let filter = TaskFilter {
                status: status.and_then(|s| parse_status(&s)).map(|s| vec![s]),
                ..Default::default()
            };
            let tasks = storage.list_tasks(&filter).await?;

            println!("Tasks ({})", tasks.len());
            for task in tasks {
                println!("  {} | {} | {} - {}",
                    task.id,
                    format_status(task.status),
                    task.priority,
                    task.intent,
                );
            }
        }
        Commands::Show { id } => {
            let task_id = id.parse().map_err(|_| anyhow::anyhow!("Invalid task ID"))?;
            let Some(task) = storage.load_task(task_id).await? else {
                println!("Task not found");
                return Ok(());
            };

            println!("Task: {}", task.id);
            println!("  Intent: {}", task.intent);
            println!("  Hypothesis: {}", task.hypothesis);
            println!("  Status: {}", format_status(task.status));
            println!("  Priority: {}", task.priority);
            println!("  Confidence: {:.2}", task.confidence);
            println!("  Created: {}", task.created_at);
        }
        Commands::Run { cycles } => {
            let mut engine = ExecutionEngine::new(storage)
                .with_config(EngineConfig {
                    auto_commit: true,
                    max_cycles: Some(cycles),
                });

            engine.run().await?;
            info!("Completed {} cycles", engine.cycles());
        }
        Commands::Reflect => {
            use devman_reflection::ReflectionEngine;

            let mut reflector = ReflectionEngine::new(storage);
            let reports = reflector.reflect_all().await?;

            for report in reports {
                println!("Reflection on {}:", report.task_id);
                println!("  Success: {}", report.success);
                println!("  Insight: {}", report.insight);
                println!("  Confidence delta: {:+.2}", report.confidence_delta);
            }
        }
        Commands::Status => {
            let all_tasks = storage.list_tasks(&TaskFilter::default()).await?;
            let by_status: std::collections::HashMap<_, _> = all_tasks
                .iter()
                .map(|t| (t.status, t))
                .fold(std::collections::HashMap::new(), |mut acc, (status, t)| {
                    acc.entry(status).or_insert(Vec::new()).push(t);
                    acc
                });

            println!("DevMan Status");
            for status in &[TaskStatus::Idea, TaskStatus::Queued, TaskStatus::Active,
                            TaskStatus::Blocked, TaskStatus::Review, TaskStatus::Done,
                            TaskStatus::Abandoned] {
                if let Some(tasks) = by_status.get(status) {
                    println!("  {}: {}", format_status(*status), tasks.len());
                }
            }
        }
    }

    Ok(())
}

fn parse_status(s: &str) -> Option<TaskStatus> {
    match s.to_lowercase().as_str() {
        "idea" => Some(TaskStatus::Idea),
        "queued" => Some(TaskStatus::Queued),
        "active" => Some(TaskStatus::Active),
        "blocked" => Some(TaskStatus::Blocked),
        "review" => Some(TaskStatus::Review),
        "done" => Some(TaskStatus::Done),
        "abandoned" => Some(TaskStatus::Abandoned),
        _ => None,
    }
}

fn format_status(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Idea => "IDEA",
        TaskStatus::Queued => "QUEUED",
        TaskStatus::Active => "ACTIVE",
        TaskStatus::Blocked => "BLOCKED",
        TaskStatus::Review => "REVIEW",
        TaskStatus::Done => "DONE",
        TaskStatus::Abandoned => "ABANDONED",
    }
}
