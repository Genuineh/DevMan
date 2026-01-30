//! Task guidance system - tells AI what to do next.

use devman_core::{TaskState, TaskId, AbandonReason, QualityCheckType, GenericCheckType, TaskQualityCheckResult, TaskQualityOverallStatus, CheckDetails, Severity};

/// Generate guidance for a task in a given state.
pub struct TaskGuidanceGenerator;

impl TaskGuidanceGenerator {
    /// Generate comprehensive guidance for a task.
    pub fn generate_guidance(
        task_id: TaskId,
        current_state: &TaskState,
        context: &GuidanceContext,
    ) -> TaskGuidanceInfo {
        let next_action = Self::determine_next_action(current_state, context);
        let prerequisites = Self::check_prerequisites(current_state, context);
        let allowed_ops = Self::get_allowed_operations(current_state);
        let health = Self::assess_task_health(current_state, context);
        let message = Self::build_guidance_message(current_state, &next_action, &prerequisites);

        TaskGuidanceInfo {
            task_id,
            current_state: current_state.clone(),
            next_action,
            prerequisites_satisfied: prerequisites.is_empty(),
            missing_prerequisites: prerequisites,
            allowed_operations: allowed_ops,
            guidance_message: message,
            task_health: health,
        }
    }

    fn determine_next_action(state: &TaskState, context: &GuidanceContext) -> NextActionInfo {
        match state {
            TaskState::Created { .. } => NextActionInfo::ReadContext {
                suggested_first: true,
            },

            TaskState::ContextRead { .. } => {
                let queries = Self::suggest_knowledge_queries(context);
                NextActionInfo::ReviewKnowledge { suggested_queries: queries }
            }

            TaskState::KnowledgeReviewed { .. } => {
                let workflow = Self::suggest_workflow(context);
                NextActionInfo::StartExecution { suggested_workflow: workflow }
            }

            TaskState::InProgress { .. } => {
                let required_logs = Self::get_required_work_logs(context);
                NextActionInfo::ContinueExecution { required_logs }
            }

            TaskState::WorkRecorded { .. } => {
                let required_checks = Self::get_required_quality_checks(context);
                NextActionInfo::RunQualityCheck { required_checks }
            }

            TaskState::QualityChecking { .. } => {
                NextActionInfo::WaitForQualityCheck
            }

            TaskState::QualityCompleted { result, .. } => {
                match result.overall_status {
                    devman_core::TaskQualityOverallStatus::Passed => NextActionInfo::CompleteTask,
                    devman_core::TaskQualityOverallStatus::PassedWithWarnings => {
                        let issues = Self::extract_warnings_from_summary(&result);
                        NextActionInfo::FixQualityIssues { issues }
                    }
                    devman_core::TaskQualityOverallStatus::Failed => {
                        let issues = Self::extract_failures_from_summary(&result);
                        NextActionInfo::FixQualityIssues { issues }
                    }
                    _ => NextActionInfo::ReviewQualityResult,
                }
            }

            TaskState::Paused { reason, .. } => NextActionInfo::Paused {
                reason: reason.clone(),
            },

            TaskState::Abandoned { reason, .. } => NextActionInfo::Abandoned {
                reason: reason.clone(),
            },

            TaskState::Completed { .. } => NextActionInfo::TaskCompleted,
        }
    }

    fn suggest_knowledge_queries(context: &GuidanceContext) -> Vec<String> {
        let task_desc = context.task_description.to_lowercase();
        let mut queries = Vec::new();

        // Extract keywords from task description
        if task_desc.contains("auth") || task_desc.contains("login") {
            queries.push("authentication best practices".to_string());
            queries.push("security considerations".to_string());
        }
        if task_desc.contains("api") || task_desc.contains("endpoint") {
            queries.push("REST API design patterns".to_string());
            queries.push("API versioning".to_string());
        }
        if task_desc.contains("test") || task_desc.contains("testing") {
            queries.push("unit testing patterns".to_string());
            queries.push("test coverage".to_string());
        }

        // Add domain-specific queries
        for domain in &context.domains {
            queries.push(format!("{} best practices", domain));
        }

        if queries.is_empty() {
            queries.push(context.task_description.clone());
        }

        queries
    }

    fn suggest_workflow(context: &GuidanceContext) -> Option<String> {
        // Suggest workflow based on task type
        let task_lower = context.task_description.to_lowercase();

        if task_lower.contains("feature") || task_lower.contains("implement") {
            Some("tdd_workflow".to_string())
        } else if task_lower.contains("bug") || task_lower.contains("fix") {
            Some("debugging_workflow".to_string())
        } else if task_lower.contains("refactor") {
            Some("refactoring_workflow".to_string())
        } else {
            Some("standard_workflow".to_string())
        }
    }

    fn get_required_work_logs(context: &GuidanceContext) -> Vec<String> {
        let mut required = vec![];

        // Always require work logging
        required.push("记录实现的功能".to_string());
        required.push("记录运行的测试".to_string());

        if context.has_quality_requirements {
            required.push("记录质检结果".to_string());
        }

        required
    }

    fn get_required_quality_checks(context: &GuidanceContext) -> Vec<QualityCheckType> {
        let mut checks = vec![
            QualityCheckType::Generic(GenericCheckType::Compiles { target: context.tech_stack.first().cloned().unwrap_or("unknown".to_string()) }),
            QualityCheckType::Generic(GenericCheckType::TestsPass { test_suite: "unit".to_string(), min_coverage: None }),
        ];

        if context.tech_stack.contains(&"rust".to_string()) {
            checks.push(QualityCheckType::Generic(GenericCheckType::LintsPass { linter: "clippy".to_string() }));
            checks.push(QualityCheckType::Generic(GenericCheckType::Formatted { formatter: "rustfmt".to_string() }));
        }

        for check_type in &context.required_quality_checks {
            checks.push(check_type.clone());
        }

        checks
    }

    fn extract_warnings_from_summary(result: &TaskQualityCheckResult) -> Vec<String> {
        if result.warnings_count > 0 {
            vec![
                format!("质检发现 {} 个警告，请查看详细报告", result.warnings_count),
                format!("总共有 {} 个问题需要关注", result.findings_count),
            ]
        } else {
            vec!["质检通过但有警告".to_string()]
        }
    }

    fn extract_failures_from_summary(result: &TaskQualityCheckResult) -> Vec<String> {
        let mut failures = vec![];

        if result.findings_count > 0 {
            failures.push(format!("质检未通过，发现 {} 个问题", result.findings_count));
        }

        if result.warnings_count > 0 {
            failures.push(format!("另外有 {} 个警告", result.warnings_count));
        }

        if failures.is_empty() {
            failures.push("质检未通过，请查看详细报告".to_string());
        }

        failures
    }

    fn check_prerequisites(state: &TaskState, context: &GuidanceContext) -> Vec<String> {
        let mut missing = vec![];

        match state {
            TaskState::ContextRead { .. } => {
                if !context.has_read_context {
                    missing.push("读取任务上下文".to_string());
                }
            }

            TaskState::KnowledgeReviewed { .. } => {
                if context.reviewed_knowledge.is_empty() {
                    missing.push("学习相关知识".to_string());
                }
            }

            TaskState::WorkRecorded { .. } => {
                if context.work_logs.is_empty() {
                    missing.push("记录工作进展".to_string());
                }
            }

            TaskState::QualityChecking { .. } => {
                // No prerequisites, waiting for quality check
            }

            _ => {}
        }

        missing
    }

    fn get_allowed_operations(state: &TaskState) -> Vec<String> {
        state.allowed_operations().into_iter().map(|s| s.to_string()).collect()
    }

    fn assess_task_health(state: &TaskState, context: &GuidanceContext) -> TaskHealthInfo {
        let mut warnings = vec![];
        let mut issues = vec![];
        let mut blockers = vec![];

        match state {
            TaskState::Created { .. } => {
                if time_since(state, 24) {
                    blockers.push("任务创建超过24小时未开始".to_string());
                }
            }

            TaskState::ContextRead { .. } => {
                if time_since(state, 4) {
                    warnings.push("读取上下文后长时间未学习知识".to_string());
                }
            }

            TaskState::InProgress { .. } => {
                if time_since(state, 24) {
                    warnings.push("任务执行超过24小时".to_string());
                }
                if context.work_logs.is_empty() && time_since(state, 2) {
                    issues.push(TaskIssue {
                        severity: IssueSeverity::Medium,
                        description: "执行超过2小时未记录工作".to_string(),
                        suggested_action: "使用 log_work() 记录当前进展".to_string(),
                    });
                }
            }

            TaskState::QualityChecking { .. } => {
                if time_since(state, 2) {
                    warnings.push("质检运行时间较长".to_string());
                }
            }

            TaskState::Paused { .. } => {
                blockers.push("任务已暂停".to_string());
            }

            TaskState::Abandoned { .. } => {
                blockers.push("任务已放弃".to_string());
            }

            _ => {}
        }

        if !blockers.is_empty() {
            TaskHealthInfo::Critical { blockers }
        } else if !issues.is_empty() {
            TaskHealthInfo::Attention { issues }
        } else if !warnings.is_empty() {
            TaskHealthInfo::Warning { warnings }
        } else {
            TaskHealthInfo::Healthy
        }
    }

    fn build_guidance_message(state: &TaskState, next_action: &NextActionInfo, missing: &[String]) -> String {
        let base_msg = state.get_guidance();

        if !missing.is_empty() {
            format!("{}\n\n缺少前置条件:\n- {}", base_msg, missing.join("\n- "))
        } else {
            base_msg.to_string()
        }
    }
}

/// Information returned to AI about what to do next.
#[derive(Debug, Clone)]
pub struct TaskGuidanceInfo {
    pub task_id: TaskId,
    pub current_state: TaskState,
    pub next_action: NextActionInfo,
    pub prerequisites_satisfied: bool,
    pub missing_prerequisites: Vec<String>,
    pub allowed_operations: Vec<String>,
    pub guidance_message: String,
    pub task_health: TaskHealthInfo,
}

/// Next action information.
#[derive(Debug, Clone)]
pub enum NextActionInfo {
    ReadContext { suggested_first: bool },
    ReviewKnowledge { suggested_queries: Vec<String> },
    StartExecution { suggested_workflow: Option<String> },
    ContinueExecution { required_logs: Vec<String> },
    SubmitWork,
    RunQualityCheck { required_checks: Vec<QualityCheckType> },
    FixQualityIssues { issues: Vec<String> },
    CompleteTask,
    WaitForQualityCheck,
    ReviewQualityResult,
    Paused { reason: String },
    Abandoned { reason: AbandonReason },
    TaskCompleted,
}

/// Task health information.
#[derive(Debug, Clone)]
pub enum TaskHealthInfo {
    Healthy,
    Warning { warnings: Vec<String> },
    Attention { issues: Vec<TaskIssue> },
    Critical { blockers: Vec<String> },
}

/// Task issue with severity and suggested action.
#[derive(Debug, Clone)]
pub struct TaskIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub suggested_action: String,
}

/// Issue severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Context for generating guidance.
#[derive(Debug, Clone, Default)]
pub struct GuidanceContext {
    pub task_description: String,
    pub domains: Vec<String>,
    pub tech_stack: Vec<String>,
    pub has_read_context: bool,
    pub reviewed_knowledge: Vec<devman_core::KnowledgeId>,
    pub work_logs: Vec<String>,
    pub has_quality_requirements: bool,
    pub required_quality_checks: Vec<QualityCheckType>,
}

/// Helper function to calculate time since a state was entered.
fn time_since(state: &TaskState, hours: i64) -> bool {
    let now = chrono::Utc::now();
    let state_time = match state {
        TaskState::Created { created_at, .. } => *created_at,
        TaskState::ContextRead { read_at, .. } => *read_at,
        TaskState::KnowledgeReviewed { reviewed_at, .. } => *reviewed_at,
        TaskState::InProgress { started_at, .. } => *started_at,
        TaskState::WorkRecorded { recorded_at, .. } => *recorded_at,
        TaskState::QualityChecking { started_at, .. } => *started_at,
        TaskState::QualityCompleted { completed_at, .. } => *completed_at,
        TaskState::Paused { paused_at, .. } => *paused_at,
        TaskState::Abandoned { abandoned_at, .. } => *abandoned_at,
        TaskState::Completed { completed_at, .. } => *completed_at,
    };

    now.signed_duration_since(state_time)
        .num_hours()
        .abs()
        > hours
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_context() -> GuidanceContext {
        GuidanceContext {
            task_description: "Implement user authentication".to_string(),
            domains: vec!["security".to_string()],
            tech_stack: vec!["rust".to_string()],
            has_read_context: false,
            reviewed_knowledge: vec![],
            work_logs: vec![],
            has_quality_requirements: true,
            required_quality_checks: vec![],
        }
    }

    #[test]
    fn test_guidance_for_created_task() {
        let task_id = TaskId::new();
        let state = TaskState::Created {
            created_at: Utc::now(),
            created_by: "test".to_string(),
        };
        let context = make_context();

        let guidance = TaskGuidanceGenerator::generate_guidance(task_id, &state, &context);

        assert!(guidance.missing_prerequisites.is_empty());
        assert!(!guidance.allowed_operations.is_empty());
    }

    #[test]
    fn test_guidance_for_context_read_task() {
        let task_id = TaskId::new();
        let state = TaskState::ContextRead {
            read_at: Utc::now(),
        };
        let mut context = make_context();
        context.has_read_context = true;

        let guidance = TaskGuidanceGenerator::generate_guidance(task_id, &state, &context);

        assert!(matches!(guidance.next_action, NextActionInfo::ReviewKnowledge { .. }));
    }

    #[test]
    fn test_suggest_knowledge_queries() {
        let context = GuidanceContext {
            task_description: "Implement user authentication with JWT".to_string(),
            ..Default::default()
        };

        let queries = TaskGuidanceGenerator::suggest_knowledge_queries(&context);
        assert!(!queries.is_empty());
        assert!(queries.iter().any(|q| q.contains("authentication")));
    }
}
