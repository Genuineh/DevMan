//! DevMan CLI - AI认知工作管理系统命令行工具

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use devman_core::{Goal, GoalId};
use devman_storage::{JsonStorage, Storage};

#[derive(Parser)]
#[command(name = "devman")]
#[command(about = "AI认知工作管理系统", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新目标
    CreateGoal {
        title: String,
        description: String,
    },
    /// 列出所有目标
    ListGoals,
    /// 显示目标详情
    ShowGoal { id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();
    let storage_path = std::path::PathBuf::from(".devman");
    let mut storage = JsonStorage::new(&storage_path).await?;

    match cli.command {
        Commands::CreateGoal { title, description } => {
            let goal = Goal {
                id: GoalId::new(),
                title,
                description,
                success_criteria: Vec::new(),
                progress: devman_core::GoalProgress::default(),
                project_id: devman_core::ProjectId::new(),
                current_phase: devman_core::PhaseId::new(),
                status: devman_core::GoalStatus::Active,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            storage.save_goal(&goal).await?;
            storage.commit("Create goal").await?;
            println!("✓ 创建目标: {} - {}", goal.id, goal.title);
        }

        Commands::ListGoals => {
            let goals = storage.list_goals().await?;
            println!("目标 ({}):", goals.len());
            for goal in &goals {
                println!("  {} | {} | {} - {}",
                    goal.id,
                    format!("{:?}", goal.status),
                    goal.title,
                    goal.description);
            }
        }

        Commands::ShowGoal { id } => {
            let goal_id = id.parse()?;
            if let Some(goal) = storage.load_goal(goal_id).await? {
                println!("目标: {} - {}", goal.title, goal.description);
                println!("  状态: {:?}", goal.status);
                println!("  进度: {:.1}%", goal.progress.percentage);
                println!("  项目: {}", goal.project_id);
                println!("  当前阶段: {}", goal.current_phase);
            } else {
                println!("目标不存在");
            }
        }
    }

    Ok(())
}
