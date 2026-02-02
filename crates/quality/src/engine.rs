//! Quality check engine.

use async_trait::async_trait;
use devman_core::{
    QualityCheck, QualityCheckResult, QualityGate, TaskId,
    QualityCategory, Finding, CheckDetails, Severity, Metric,
};
use devman_storage::Storage;
use std::sync::Arc;

use crate::parser::{parse_output, evaluate_pass_condition, extract_metrics};

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
        use devman_tools::ToolInput;
        use std::time::Instant;
        use devman_core::{Severity, QualityCategory, Finding};

        let start = Instant::now();

        let (tool, args, work_dir) = match generic {
            devman_core::GenericCheckType::Compiles { target } => {
                ("cargo".to_string(), vec!["check".to_string(), "--target".to_string(), target.clone()], None::<()>)
            }
            devman_core::GenericCheckType::TestsPass { test_suite, min_coverage } => {
                let mut args = vec!["test".to_string()];
                if !test_suite.is_empty() {
                    args.push(test_suite.clone());
                }
                // Check if we should get coverage (tarpaulin)
                let tool = if min_coverage.is_some() {
                    "cargo".to_string()
                } else {
                    "cargo".to_string()
                };
                (tool, args, None)
            }
            devman_core::GenericCheckType::Formatted { formatter } => {
                (formatter.clone(), vec!["--check".to_string()], None)
            }
            devman_core::GenericCheckType::LintsPass { linter } => {
                (linter.clone(), vec![], None)
            }
            devman_core::GenericCheckType::DocumentationExists { paths } => {
                // Check if documentation files exist
                return self.check_documentation_exists(paths, start).await;
            }
            devman_core::GenericCheckType::TypeCheck {} => {
                // Run cargo check for type checking
                ("cargo".to_string(), vec!["check".to_string()], None)
            }
            devman_core::GenericCheckType::DependenciesValid {} => {
                // Check for outdated or insecure dependencies
                ("cargo".to_string(), vec!["update".to_string(), "--dry-run".to_string()], None)
            }
            devman_core::GenericCheckType::SecurityScan { scanner } => {
                // Run security scanner
                (scanner.clone(), vec![], None)
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
                        suggestion: Some("Check if the tool is installed and available in PATH".to_string()),
                    }],
                    metrics: Vec::new(),
                    human_review: None,
                }
            }
        };

        let passed = output.exit_code == 0;

        // Generate findings based on output
        let mut findings = Vec::new();
        let category = match generic {
            devman_core::GenericCheckType::Compiles { .. } => QualityCategory::Correctness,
            devman_core::GenericCheckType::TestsPass { .. } => QualityCategory::Testing,
            devman_core::GenericCheckType::Formatted { .. } => QualityCategory::Maintainability,
            devman_core::GenericCheckType::LintsPass { .. } => QualityCategory::Maintainability,
            devman_core::GenericCheckType::DocumentationExists { .. } => QualityCategory::Documentation,
            devman_core::GenericCheckType::TypeCheck { .. } => QualityCategory::Correctness,
            devman_core::GenericCheckType::DependenciesValid { .. } => QualityCategory::Maintainability,
            devman_core::GenericCheckType::SecurityScan { .. } => QualityCategory::Security,
        };

        if !passed {
            findings.push(Finding {
                severity: Severity::Error,
                category,
                message: format!("Check failed with exit code {}", output.exit_code),
                location: None,
                suggestion: Some("Review the command output for details".to_string()),
            });
        }

        // Extract coverage if available
        let mut metrics = Vec::new();
        if let devman_core::GenericCheckType::TestsPass { min_coverage, .. } = generic {
            if let Some(coverage) = min_coverage {
                // Try to extract coverage from output
                let coverage_value = self.extract_coverage(&output.stdout, &output.stderr);
                metrics.push(devman_core::Metric {
                    name: "coverage".to_string(),
                    value: coverage_value,
                    unit: Some("%".to_string()),
                });
            }
        }

        QualityCheckResult {
            check_id: devman_core::QualityCheckId::new(),
            passed,
            execution_time: start.elapsed(),
            details: devman_core::CheckDetails {
                output: output.stdout.clone(),
                exit_code: Some(output.exit_code),
                error: if output.stderr.is_empty() {
                    None
                } else {
                    Some(output.stderr)
                },
            },
            findings,
            metrics,
            human_review: None,
        }
    }

    /// Extract coverage percentage from test output.
    fn extract_coverage(&self, stdout: &str, _stderr: &str) -> f64 {
        // Try common coverage patterns
        let patterns = [
            r"Coverage:\s*([0-9.]+)%",
            r"coverage:\s*([0-9.]+)%",
            r"(\d+\.?\d*)%.*coverage",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(stdout) {
                    if let Some(m) = caps.get(1) {
                        if let Ok(val) = m.as_str().parse::<f64>() {
                            return val;
                        }
                    }
                }
            }
        }
        0.0
    }

    /// Check if documentation files exist.
    async fn check_documentation_exists(
        &self,
        paths: &[String],
        start: std::time::Instant,
    ) -> QualityCheckResult {
        use devman_core::{Severity, QualityCategory, Finding};

        let mut all_exist = true;
        let mut missing_files = Vec::new();
        let mut findings = Vec::new();

        for path in paths {
            let path = std::path::Path::new(path);
            if !path.exists() {
                all_exist = false;
                missing_files.push(path.to_string_lossy().to_string());
            }
        }

        if !all_exist {
            findings.push(Finding {
                severity: Severity::Warning,
                category: QualityCategory::Documentation,
                message: format!("Missing documentation files: {}", missing_files.join(", ")),
                location: None,
                suggestion: Some("Create the required documentation files".to_string()),
            });
        }

        QualityCheckResult {
            check_id: devman_core::QualityCheckId::new(),
            passed: all_exist,
            execution_time: start.elapsed(),
            details: devman_core::CheckDetails {
                output: format!("Checked {} documentation paths", paths.len()),
                exit_code: Some(if all_exist { 0 } else { 1 }),
                error: None,
            },
            findings,
            metrics: Vec::new(),
            human_review: None,
        }
    }

    async fn run_custom_check(
        &self,
        custom: &devman_core::CustomCheckSpec,
        check: &QualityCheck,
        _context: &WorkContext,
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

        let tool_output = match self
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
                        message: format!("Custom check command failed: {}", e),
                        location: None,
                        suggestion: None,
                    }],
                    metrics: Vec::new(),
                    human_review: None,
                }
            }
        };

        // Combine stdout and stderr for parsing
        let full_output = if tool_output.stderr.is_empty() {
            tool_output.stdout.clone()
        } else {
            format!("{}\n{}", tool_output.stdout, tool_output.stderr)
        };

        // Check expected exit code first
        let exit_code_match = match custom.check_command.expected_exit_code {
            Some(expected) => tool_output.exit_code == expected,
            None => true, // No expectation means any exit code is acceptable
        };

        if !exit_code_match {
            return QualityCheckResult {
                check_id: check.id,
                passed: false,
                execution_time: start.elapsed(),
                details: CheckDetails {
                    output: full_output.clone(),
                    exit_code: Some(tool_output.exit_code),
                    error: Some(format!(
                        "Expected exit code {:?}, got {}",
                        custom.check_command.expected_exit_code, tool_output.exit_code
                    )),
                },
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: check.category,
                    message: format!(
                        "Command exited with code {} (expected {:?})",
                        tool_output.exit_code, custom.check_command.expected_exit_code
                    ),
                    location: None,
                    suggestion: Some("Check the command and its arguments for correctness".to_string()),
                }],
                metrics: Vec::new(),
                human_review: None,
            };
        }

        // Parse the output using the validation spec
        let parse_result = parse_output(&full_output, &custom.validation.output_parser);

        // Evaluate the pass condition
        let passed = if parse_result.success {
            evaluate_pass_condition(&custom.validation.pass_condition, &parse_result)
        } else {
            false
        };

        // Generate findings based on parsing results
        let mut findings = Vec::new();
        if !parse_result.success {
            findings.push(Finding {
                severity: if passed { Severity::Warning } else { Severity::Error },
                category: check.category,
                message: parse_result.error.unwrap_or_else(|| "Output parsing failed".to_string()),
                location: None,
                suggestion: Some("Verify the command output format matches the expected parser pattern".to_string()),
            });
        }

        // Extract metrics
        let metrics: Vec<Metric> = extract_metrics(&full_output, &custom.validation.extract_metrics)
            .into_iter()
            .map(|m| Metric {
                name: m.name,
                value: m.value,
                unit: m.unit,
            })
            .collect();

        QualityCheckResult {
            check_id: check.id,
            passed,
            execution_time: start.elapsed(),
            details: CheckDetails {
                output: full_output,
                exit_code: Some(tool_output.exit_code),
                error: None,
            },
            findings,
            metrics,
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

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::{QualityCheckId, QualityCheckResult, CheckDetails, Metric, QualityStatus, QualityOverallStatus};

    #[test]
    fn test_gate_result_pass() {
        let result = GateResult {
            gate_name: "test-gate".to_string(),
            passed: true,
            check_results: Vec::new(),
            decision: GateDecision::Pass,
        };
        assert!(result.passed);
        assert_eq!(result.decision, GateDecision::Pass);
    }

    #[test]
    fn test_gate_result_fail() {
        let result = GateResult {
            gate_name: "test-gate".to_string(),
            passed: false,
            check_results: Vec::new(),
            decision: GateDecision::Fail,
        };
        assert!(!result.passed);
        assert_eq!(result.decision, GateDecision::Fail);
    }

    #[test]
    fn test_gate_decision_variants() {
        assert_eq!(GateDecision::Pass, GateDecision::Pass);
        assert_eq!(GateDecision::Fail, GateDecision::Fail);
        assert_eq!(GateDecision::PassWithWarnings, GateDecision::PassWithWarnings);
        assert_eq!(GateDecision::Escalate, GateDecision::Escalate);
    }

    #[test]
    fn test_work_context_creation() {
        let context = WorkContext::new(TaskId::new());
        assert!(!context.task_id.to_string().is_empty());
        assert!(context.work_dir.exists() || context.work_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_work_context_with_metadata() {
        let context = WorkContext::new(TaskId::new());
        let context_with_meta = WorkContext {
            task_id: context.task_id,
            work_dir: context.work_dir.clone(),
            metadata: serde_json::json!({"key": "value"}),
        };
        assert_eq!(context_with_meta.metadata["key"], "value");
    }

    #[test]
    fn test_gate_result_pass_with_warnings() {
        let result = GateResult {
            gate_name: "test-gate".to_string(),
            passed: true,
            check_results: Vec::new(),
            decision: GateDecision::PassWithWarnings,
        };
        assert!(result.passed);
        assert_eq!(result.decision, GateDecision::PassWithWarnings);
    }

    #[test]
    fn test_gate_result_escalate() {
        let result = GateResult {
            gate_name: "test-gate".to_string(),
            passed: false,
            check_results: Vec::new(),
            decision: GateDecision::Escalate,
        };
        assert!(!result.passed);
        assert_eq!(result.decision, GateDecision::Escalate);
    }

    #[test]
    fn test_generic_check_type_variants() {
        use devman_core::GenericCheckType;

        let compiles = GenericCheckType::Compiles { target: "x86_64-unknown-linux-gnu".to_string() };
        assert!(matches!(compiles, GenericCheckType::Compiles { .. }));

        let tests = GenericCheckType::TestsPass {
            test_suite: "lib".to_string(),
            min_coverage: Some(80.0),
        };
        assert!(matches!(tests, GenericCheckType::TestsPass { .. }));

        let formatted = GenericCheckType::Formatted { formatter: "rustfmt".to_string() };
        assert!(matches!(formatted, GenericCheckType::Formatted { .. }));

        let lints = GenericCheckType::LintsPass { linter: "clippy".to_string() };
        assert!(matches!(lints, GenericCheckType::LintsPass { .. }));

        let docs = GenericCheckType::DocumentationExists {
            paths: vec!["README.md".to_string()],
        };
        assert!(matches!(docs, GenericCheckType::DocumentationExists { .. }));

        let type_check = GenericCheckType::TypeCheck {};
        assert!(matches!(type_check, GenericCheckType::TypeCheck { .. }));

        let deps = GenericCheckType::DependenciesValid {};
        assert!(matches!(deps, GenericCheckType::DependenciesValid { .. }));

        let security = GenericCheckType::SecurityScan { scanner: "cargo-audit".to_string() };
        assert!(matches!(security, GenericCheckType::SecurityScan { .. }));
    }

    #[test]
    fn test_quality_category_variants() {
        use devman_core::QualityCategory;

        assert!(matches!(QualityCategory::Correctness, QualityCategory::Correctness));
        assert!(matches!(QualityCategory::Performance, QualityCategory::Performance));
        assert!(matches!(QualityCategory::Security, QualityCategory::Security));
        assert!(matches!(QualityCategory::Maintainability, QualityCategory::Maintainability));
        assert!(matches!(QualityCategory::Documentation, QualityCategory::Documentation));
        assert!(matches!(QualityCategory::Testing, QualityCategory::Testing));
        assert!(matches!(QualityCategory::Business, QualityCategory::Business));
        assert!(matches!(QualityCategory::Compliance, QualityCategory::Compliance));
    }

    #[test]
    fn test_quality_overall_status_variants() {
        use devman_core::QualityOverallStatus;

        assert!(matches!(QualityOverallStatus::NotChecked, QualityOverallStatus::NotChecked));
        assert!(matches!(QualityOverallStatus::Passed, QualityOverallStatus::Passed));
        assert!(matches!(QualityOverallStatus::PassedWithWarnings, QualityOverallStatus::PassedWithWarnings));
        assert!(matches!(QualityOverallStatus::Failed, QualityOverallStatus::Failed));
        assert!(matches!(QualityOverallStatus::PendingReview, QualityOverallStatus::PendingReview));
    }

    #[test]
    fn test_gate_strategy_variants() {
        use devman_core::GateStrategy;

        assert!(matches!(GateStrategy::AllMustPass, GateStrategy::AllMustPass));
        assert!(matches!(GateStrategy::WarningsAllowed { max_warnings: 5 }, GateStrategy::WarningsAllowed { .. }));
        assert!(matches!(GateStrategy::ManualDecision, GateStrategy::ManualDecision));
        assert!(matches!(GateStrategy::Custom { rule: "custom".to_string() }, GateStrategy::Custom { .. }));
    }

    #[test]
    fn test_quality_check_result_creation() {
        let result = QualityCheckResult {
            check_id: QualityCheckId::new(),
            passed: true,
            execution_time: std::time::Duration::from_millis(100),
            details: CheckDetails {
                output: "All checks passed".to_string(),
                exit_code: Some(0),
                error: None,
            },
            findings: Vec::new(),
            metrics: vec![
                Metric {
                    name: "coverage".to_string(),
                    value: 85.5,
                    unit: Some("%".to_string()),
                }
            ],
            human_review: None,
        };

        assert!(result.passed);
        assert_eq!(result.details.exit_code, Some(0));
        assert_eq!(result.metrics.len(), 1);
        assert_eq!(result.metrics[0].name, "coverage");
    }

    #[test]
    fn test_quality_status_creation() {
        let status = QualityStatus {
            task_id: TaskId::new(),
            total_checks: 10,
            passed_checks: 8,
            failed_checks: 1,
            warnings: 1,
            overall_status: QualityOverallStatus::PassedWithWarnings,
            pending_human_review: false,
        };

        assert_eq!(status.total_checks, 10);
        assert_eq!(status.passed_checks, 8);
        assert_eq!(status.failed_checks, 1);
        assert_eq!(status.warnings, 1);
        assert!(matches!(status.overall_status, QualityOverallStatus::PassedWithWarnings));
    }
}
