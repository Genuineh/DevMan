//! Built-in tools (Cargo, Npm, Git, File system).

use super::{r#trait::*, ToolSchema};
use async_trait::async_trait;
use tokio::process::Command;

/// Cargo tool for Rust projects.
pub struct CargoTool;

#[async_trait]
impl Tool for CargoTool {
    fn name(&self) -> &str {
        "cargo"
    }

    fn description(&self) -> &str {
        "Rust package manager"
    }

    async fn execute(&self, input: &ToolInput) -> Result<ToolOutput, anyhow::Error> {
        let start = std::time::Instant::now();

        let mut cmd = Command::new("cargo");
        cmd.args(&input.args);

        if let Some(stdin) = &input.stdin {
            // TODO: Handle stdin
        }

        for (k, v) in &input.env {
            cmd.env(k, v);
        }

        let output = cmd.output().await?;

        Ok(ToolOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration: start.elapsed(),
        })
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "cargo".to_string(),
            description: "Rust package manager".to_string(),
            parameters: vec![
                Parameter {
                    name: "subcommand".to_string(),
                    description: "Cargo subcommand (build, test, run, check, etc.)".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default: None,
                },
                Parameter {
                    name: "args".to_string(),
                    description: "Additional arguments".to_string(),
                    param_type: "array".to_string(),
                    required: false,
                    default: None,
                },
            ],
            examples: vec![],
        }
    }
}

/// Npm tool for Node.js projects.
pub struct NpmTool;

#[async_trait]
impl Tool for NpmTool {
    fn name(&self) -> &str {
        "npm"
    }

    fn description(&self) -> &str {
        "Node.js package manager"
    }

    async fn execute(&self, input: &ToolInput) -> Result<ToolOutput, anyhow::Error> {
        let start = std::time::Instant::now();

        let mut cmd = Command::new("npm");
        cmd.args(&input.args);

        for (k, v) in &input.env {
            cmd.env(k, v);
        }

        let output = cmd.output().await?;

        Ok(ToolOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration: start.elapsed(),
        })
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "npm".to_string(),
            description: "Node.js package manager".to_string(),
            parameters: vec![],
            examples: vec![],
        }
    }
}

/// Git tool for version control.
pub struct GitTool;

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Version control system"
    }

    async fn execute(&self, input: &ToolInput) -> Result<ToolOutput, anyhow::Error> {
        let start = std::time::Instant::now();

        let mut cmd = Command::new("git");
        cmd.args(&input.args);

        for (k, v) in &input.env {
            cmd.env(k, v);
        }

        let output = cmd.output().await?;

        Ok(ToolOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration: start.elapsed(),
        })
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "git".to_string(),
            description: "Version control system".to_string(),
            parameters: vec![],
            examples: vec![],
        }
    }
}

/// File system tool.
pub struct FsTool;

#[async_trait]
impl Tool for FsTool {
    fn name(&self) -> &str {
        "fs"
    }

    fn description(&self) -> &str {
        "File system operations"
    }

    async fn execute(&self, input: &ToolInput) -> Result<ToolOutput, anyhow::Error> {
        // Basic file operations
        if input.args.is_empty() {
            return Ok(ToolOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: "No operation specified".to_string(),
                duration: std::time::Duration::ZERO,
            });
        }

        let operation = &input.args[0];
        let result = match operation.as_str() {
            "read" => {
                if input.args.len() < 2 {
                    return Ok(ToolOutput {
                        exit_code: 1,
                        stdout: String::new(),
                        stderr: "No file specified".to_string(),
                        duration: std::time::Duration::ZERO,
                    });
                }
                tokio::fs::read_to_string(&input.args[1]).await
                    .map_err(|e| anyhow::anyhow!(e))
            }
            "write" => {
                if input.args.len() < 3 {
                    return Ok(ToolOutput {
                        exit_code: 1,
                        stdout: String::new(),
                        stderr: "Usage: fs write <file> <content>".to_string(),
                        duration: std::time::Duration::ZERO,
                    });
                }
                tokio::fs::write(&input.args[1], &input.args[2]).await
                    .map(|_| String::new())
                    .map_err(|e| anyhow::anyhow!(e))
            }
            "exists" => {
                if input.args.len() < 2 {
                    return Ok(ToolOutput {
                        exit_code: 1,
                        stdout: String::new(),
                        stderr: "No file specified".to_string(),
                        duration: std::time::Duration::ZERO,
                    });
                }
                tokio::fs::try_exists(&input.args[1])
                    .await
                    .map(|exists| exists.to_string())
                    .map_err(|e| anyhow::anyhow!(e))
            }
            _ => Err(anyhow::anyhow!("Unknown operation: {}", operation)),
        };

        match result {
            Ok(stdout) => Ok(ToolOutput {
                exit_code: 0,
                stdout,
                stderr: String::new(),
                duration: std::time::Duration::ZERO,
            }),
            Err(e) => Ok(ToolOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: e.to_string(),
                duration: std::time::Duration::ZERO,
            }),
        }
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "fs".to_string(),
            description: "File system operations".to_string(),
            parameters: vec![],
            examples: vec![],
        }
    }
}
