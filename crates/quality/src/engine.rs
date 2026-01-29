//! Quality check engine.

use async_trait::async_trait;
use devman_core::{
    QualityCheck, QualityCheckId, QualityCheckResult, QualityGate, TaskId,
    QualityCategory, Finding, CheckDetails, Severity,
};
use devman_storage::Storage;
use std::sync::Arc;

/// Context for running quality checks.
#[derive(Debug, Clone)]
pub struct WorkContext {
    /// Associated task
    pub task_id: TaskId,

    /// Working directory
    pub work_dir: std::path::PathBuf,

    /// Additional context data
    pub metadata: serde_json::Value,
}

impl WorkContext {
    /// Create a new work context.
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            work_dir: std::env::current_dir().unwrap_or_default(),
            metadata: serde_json::Value::Null,
        }
    }
}

/// Quality check engine.
#[async_trait]
pub trait QualityEngine: Send + Sync {
    /// Run a single quality check.
    async fn run_check(
        &self,
        check: &QualityCheck,
        context: &WorkContext,
    ) -> QualityCheckResult;

    /// Run multiple checks.
    async fn run_checks(
        &self,
        checks: &[QualityCheck],
        context: &WorkContext,
    ) -> Vec<QualityCheckResult>;

    /// Run a quality gate.
    async fn run_gate(
        &self,
        gate: &QualityGate,
        context: &WorkContext,
    ) -> GateResult;
}

/// Result of running a quality gate.
#[derive(Debug, Clone)]
pub struct GateResult {
    /// Gate name
    pub gate_name: String,

    /// Whether the gate passed
    pub passed: bool,

    /// Individual check results
    pub check_results: Vec<QualityCheckResult>,

    /// Final decision
    pub decision: GateDecision,
}

/// Gate decision after running checks.
#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    /// All checks passed
    Pass,
    /// All checks failed
    Fail,
    /// Passed with warnings
    PassWithWarnings,
    /// Needs human review
    Escalate,
}

/// Basic quality engine implementation.
pub struct BasicQualityEngine<S: Storage> {
    storage: Arc<S>,
    tool_executor: Arc<dyn devman_tools::ToolExecutor>,
}

impl<S: Storage> BasicQualityEngine<S> {
    /// Create a new quality engine.
    pub fn new(storage: S, tool_executor: Arc<dyn devman_tools::ToolExecutor>) -> Self {
        Self {
            storage: Arc::new(storage),
            tool_executor,
        }
    }
}

#[async_trait]
impl<S: Storage + 'static> QualityEngine for BasicQualityEngine<S> {
    async fn run_check(
        &self,
        check: &QualityCheck,
        context: &WorkContext,
    ) -> QualityCheckResult {
        tracing::debug!("Running quality check: {}", check.name);

        match &check.check_type {
            devman_core::QualityCheckType::Generic(generic) => {
                self.run_generic_check(generic, context).await
            }
            devman_core::QualityCheckType::Custom(custom) => {
                self.run_custom_check(custom, check, context).await
            }
        }
    }

    async fn run_checks(
        &self,
        checks: &[QualityCheck],
        context: &WorkContext,
    ) -> Vec<QualityCheckResult> {
        let mut results = Vec::new();
        for check in checks {
            results.push(self.run_check(check, context).await);
        }
        results
    }

    async fn run_gate(
        &self,
        gate: &QualityGate,
        context: &WorkContext,
    ) -> GateResult {
        let mut check_results = Vec::new();

        for check_id in &gate.checks {
            if let Ok(Some(check)) = self.storage.load_quality_check(*check_id).await {
                let result = self.run_check(&check, context).await;
                check_results.push(result);
            }
        }

        let decision = self.evaluate_gate(&gate, &check_results);

        GateResult {
            gate_name: gate.name.clone(),
            passed: matches!(decision, GateDecision::Pass | GateDecision::PassWithWarnings),
            check_results,
            decision,
        }
    }
}

impl<S: Storage> BasicQualityEngine<S> {
    async fn run_generic_check(
        &self,
        generic: &devman_core::GenericCheckType,
        context: &WorkContext,
    ) -> QualityCheckResult {
        use devman_tools::{Tool, ToolInput};
        use std::time::Instant;

        let start = Instant::now();

        let (tool, args) = match generic {
            devman_core::GenericCheckType::Compiles { .. } => {
                ("cargo".to_string(), vec!["check".to_string()])
            }
            devman_core::GenericCheckType::TestsPass { test_suite, .. } => {
                ("cargo".to_string(), vec!["test".to_string(), test_suite.clone()])
            }
            devman_core::GenericCheckType::Formatted { formatter } => {
                (formatter.clone(), vec!["--check".to_string()])
            }
            devman_core::GenericCheckType::LintsPass { linter } => {
                (linter.clone(), vec![])
            }
            _ => {
                return QualityCheckResult {
                    check_id: devman_core::QualityCheckId::new(),
                    passed: true,
                    execution_time: start.elapsed(),
                    details: devman_core::CheckDetails {
                        output: "Check not implemented".to_string(),
                        exit_code: None,
                        error: None,
                    },
                    findings: Vec::new(),
                    metrics: Vec::new(),
                    human_review: None,
                }
            }
        };

        let input = ToolInput {
            args,
            env: Default::default(),
            stdin: None,
            timeout: Some(std::time::Duration::from_secs(300)),
        };

        let output = match self.tool_executor.execute_tool(&tool, input).await {
            Ok(o) => o,
            Err(e) => {
                return QualityCheckResult {
                    check_id: devman_core::QualityCheckId::new(),
                    passed: false,
                    execution_time: start.elapsed(),
                    details: devman_core::CheckDetails {
                        output: String::new(),
                        exit_code: None,
                        error: Some(e.to_string()),
                    },
                    findings: vec![Finding {
                        severity: Severity::Error,
                        category: QualityCategory::Correctness,
                        message: format!("Tool execution failed: {}", e),
                        location: None,
                        suggestion: None,
                    }],
                    metrics: Vec::new(),
                    human_review: None,
                }
            }
        };

        let passed = output.exit_code == 0;

        QualityCheckResult {
            check_id: devman_core::QualityCheckId::new(),
            passed,
            execution_time: start.elapsed(),
            details: devman_core::CheckDetails {
                output: output.stdout,
                exit_code: Some(output.exit_code),
                error: if output.stderr.is_empty() {
                    None
                } else {
                    Some(output.stderr)
                },
            },
            findings: Vec::new(),
            metrics: Vec::new(),
            human_review: None,
        }
    }

    async fn run_custom_check(
        &self,
        custom: &devman_core::CustomCheckSpec,
        check: &QualityCheck,
        context: &WorkContext,
    ) -> QualityCheckResult {
        tracing::debug!("Running custom check: {}", custom.name);

        let start = std::time::Instant::now();

        // Execute custom command
        use devman_tools::ToolInput;

        let input = ToolInput {
            args: custom.check_command.args.clone(),
            env: Default::default(),
            stdin: None,
            timeout: Some(custom.check_command.timeout),
        };

        let output = match self
            .tool_executor
            .execute_tool(&custom.check_command.command, input)
            .await
        {
            Ok(o) => o,
            Err(e) => {
                return QualityCheckResult {
                    check_id: check.id,
                    passed: false,
                    execution_time: start.elapsed(),
                    details: CheckDetails {
                        output: String::new(),
                        exit_code: None,
                        error: Some(e.to_string()),
                    },
                    findings: vec![Finding {
                        severity: Severity::Error,
                        category: QualityCategory::Correctness,
                        message: format!("Custom check failed: {}", e),
                        location: None,
                        suggestion: None,
                    }],
                    metrics: Vec::new(),
                    human_review: None,
                }
            }
        };

        let passed = output.exit_code == 0;

        QualityCheckResult {
            check_id: check.id,
            passed,
            execution_time: start.elapsed(),
            details: devman_core::CheckDetails {
                output: output.stdout,
                exit_code: Some(output.exit_code),
                error: if output.stderr.is_empty() {
                    None
                } else {
                    Some(output.stderr)
                },
            },
            findings: Vec::new(),
            metrics: Vec::new(),
            human_review: None,
        }
    }

    fn evaluate_gate(
        &self,
        gate: &QualityGate,
        results: &[QualityCheckResult],
    ) -> GateDecision {
        match gate.pass_condition {
            devman_core::PassCondition::AllPassed => {
                if results.iter().all(|r| r.passed) {
                    GateDecision::Pass
                } else {
                    GateDecision::Fail
                }
            }
            devman_core::PassCondition::AtLeast { count } => {
                let passed = results.iter().filter(|r| r.passed).count();
                if passed >= count {
                    GateDecision::Pass
                } else {
                    GateDecision::Fail
                }
            }
            devman_core::PassCondition::Custom { .. } => {
                // TODO: Implement custom expression evaluation
                GateDecision::Pass
            }
        }
    }
}
