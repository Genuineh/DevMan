//! Workflow orchestration for multi-step tool execution.

use super::{Tool, ToolInput, ToolOutput};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Error type for workflow execution.
#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Step {0} failed: {1}")]
    StepFailed(usize, String),

    #[error("Workflow execution cancelled")]
    Cancelled,

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid workflow definition: {0}")]
    InvalidDefinition(String),
}

/// Result of a workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// Whether the workflow completed successfully
    pub success: bool,

    /// Results from each step
    pub step_results: Vec<StepResult>,

    /// Total execution duration
    pub duration: std::time::Duration,

    /// Any error message
    pub error: Option<String>,
}

/// Result of a single step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step name
    pub name: String,

    /// Whether the step succeeded
    pub success: bool,

    /// Tool output (if any)
    pub output: Option<ToolOutput>,

    /// Execution duration
    pub duration: std::time::Duration,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Whether this step was skipped
    pub skipped: bool,
}

/// A workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,

    /// Description
    pub description: String,

    /// Tool to execute
    pub tool: String,

    /// Tool input
    pub input: ToolInput,

    /// Error handling strategy
    #[serde(default)]
    pub on_failure: FailureStrategy,

    /// Conditional execution (only run if this evaluates to true)
    pub condition: Option<StepCondition>,

    /// Whether to continue on failure
    #[serde(default)]
    pub continue_on_failure: bool,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

fn default_max_retries() -> usize {
    0
}

fn default_retry_delay() -> u64 {
    1000
}

/// Strategy for handling step failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureStrategy {
    /// Stop the workflow immediately
    Stop,

    /// Skip this step and continue
    Skip,

    /// Rollback previous steps
    Rollback,

    /// Continue execution (mark as failed)
    Continue,
}

impl Default for FailureStrategy {
    fn default() -> Self {
        FailureStrategy::Stop
    }
}

/// Condition for step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepCondition {
    /// Only run if previous step succeeded
    PreviousSuccess(String),

    /// Only run if previous step failed
    PreviousFailed(String),

    /// Only run if a variable has a specific value
    VariableEquals { name: String, value: String },

    /// Only run if a variable exists
    VariableExists(String),

    /// Custom condition (evaluated at runtime)
    Custom(String),
}

/// A workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow name
    pub name: String,

    /// Description
    pub description: String,

    /// Workflow steps
    pub steps: Vec<WorkflowStep>,

    /// Overall error handling strategy
    #[serde(default)]
    pub on_failure: FailureStrategy,

    /// Variables for this workflow
    #[serde(default)]
    pub variables: HashMap<String, String>,

    /// Whether to enable rollback
    #[serde(default)]
    pub enable_rollback: bool,
}

impl Workflow {
    /// Create a new workflow.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            steps: Vec::new(),
            on_failure: FailureStrategy::Stop,
            variables: HashMap::new(),
            enable_rollback: false,
        }
    }

    /// Set description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a step.
    pub fn step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Set a variable.
    pub fn variable(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// Enable rollback.
    pub fn with_rollback(mut self) -> Self {
        self.enable_rollback = true;
        self
    }

    /// Set failure strategy.
    pub fn on_failure(mut self, strategy: FailureStrategy) -> Self {
        self.on_failure = strategy;
        self
    }
}

impl Default for Workflow {
    fn default() -> Self {
        Self::new("")
    }
}

/// Workflow executor.
#[async_trait]
pub trait WorkflowExecutor: Send + Sync {
    /// Execute a workflow.
    async fn execute(&self, workflow: &Workflow) -> Result<WorkflowResult, WorkflowError>;

    /// Execute a workflow with custom variables.
    async fn execute_with_vars(
        &self,
        workflow: &Workflow,
        variables: &HashMap<String, String>,
    ) -> Result<WorkflowResult, WorkflowError>;
}

/// Basic workflow executor implementation.
pub struct BasicWorkflowExecutor {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl BasicWorkflowExecutor {
    /// Create a new executor with registered tools.
    pub fn new(tools: Vec<Arc<dyn Tool>>) -> Self {
        let tool_map: HashMap<String, Arc<dyn Tool>> = tools
            .into_iter()
            .map(|t| (t.name().to_string(), t))
            .collect();

        Self { tools: tool_map }
    }

    /// Register a tool.
    pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Evaluate a step condition.
    fn evaluate_condition(
        &self,
        condition: &StepCondition,
        variables: &HashMap<String, String>,
        previous_results: &[StepResult],
    ) -> bool {
        match condition {
            StepCondition::PreviousSuccess(step_name) => previous_results
                .iter()
                .find(|r| r.name == *step_name)
                .map(|r| r.success)
                .unwrap_or(false),

            StepCondition::PreviousFailed(step_name) => previous_results
                .iter()
                .find(|r| r.name == *step_name)
                .map(|r| !r.success && !r.skipped)
                .unwrap_or(false),

            StepCondition::VariableEquals { name, value } => {
                variables.get(name).map(|v| v == value).unwrap_or(false)
            }

            StepCondition::VariableExists(name) => variables.contains_key(name),

            StepCondition::Custom(_expr) => {
                // TODO: Implement custom expression evaluation
                true
            }
        }
    }

    /// Execute a single step with retry logic.
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        variables: &HashMap<String, String>,
    ) -> Result<StepResult, WorkflowError> {
        let tool = self
            .tools
            .get(&step.tool)
            .ok_or_else(|| WorkflowError::ToolNotFound(step.tool.clone()))?;

        let start = std::time::Instant::now();
        let mut last_error = None;

        // Retry loop
        for attempt in 0..=step.max_retries {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(step.retry_delay)).await;
            }

            // Substitute variables in input
            let input = self.substitute_variables(&step.input, variables);

            match tool.execute(&input).await {
                Ok(output) => {
                    return Ok(StepResult {
                        name: step.name.clone(),
                        success: output.exit_code == 0,
                        output: Some(output.clone()),
                        duration: start.elapsed(),
                        error: if output.exit_code != 0 {
                            Some(format!("Exit code: {}", output.exit_code))
                        } else {
                            None
                        },
                        skipped: false,
                    });
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    if attempt < step.max_retries {
                        continue;
                    }
                }
            }
        }

        Ok(StepResult {
            name: step.name.clone(),
            success: false,
            output: None,
            duration: start.elapsed(),
            error: last_error,
            skipped: false,
        })
    }

    /// Substitute variables in tool input.
    fn substitute_variables(
        &self,
        input: &ToolInput,
        variables: &HashMap<String, String>,
    ) -> ToolInput {
        let mut result = input.clone();

        // Substitute in arguments
        result.args = result
            .args
            .into_iter()
            .map(|arg| {
                let mut substituted = arg;
                for (key, value) in variables {
                    substituted = substituted.replace(&format!("{{{}}}", key), value);
                }
                substituted
            })
            .collect();

        // Substitute in environment variables
        result.env = result
            .env
            .into_iter()
            .map(|(k, v)| {
                let mut substituted = v;
                for (key, value) in variables {
                    substituted = substituted.replace(&format!("{{{}}}", key), value);
                }
                (k, substituted)
            })
            .collect();

        // Substitute in stdin
        if let Some(stdin) = result.stdin {
            let mut substituted = stdin;
            for (key, value) in variables {
                substituted = substituted.replace(&format!("{{{}}}", key), value);
            }
            result.stdin = Some(substituted);
        }

        result
    }

    /// Rollback completed steps (in reverse order).
    async fn rollback_steps(
        &self,
        workflow: &Workflow,
        completed_steps: &[StepResult],
        _variables: &HashMap<String, String>,
    ) -> Result<(), WorkflowError> {
        // Reverse iteration through completed steps
        for step_result in completed_steps.iter().rev() {
            if !step_result.success || step_result.skipped {
                continue;
            }

            // Find the step definition
            let step = workflow
                .steps
                .iter()
                .find(|s| s.name == step_result.name)
                .ok_or_else(|| {
                    WorkflowError::InvalidDefinition(format!("Step not found: {}", step_result.name))
                })?;

            // Execute rollback command if defined
            // TODO: Implement explicit rollback commands
            tracing::info!("Rolling back step: {}", step.name);
        }

        Ok(())
    }
}

#[async_trait]
impl WorkflowExecutor for BasicWorkflowExecutor {
    async fn execute(&self, workflow: &Workflow) -> Result<WorkflowResult, WorkflowError> {
        self.execute_with_vars(workflow, &workflow.variables).await
    }

    async fn execute_with_vars(
        &self,
        workflow: &Workflow,
        variables: &HashMap<String, String>,
    ) -> Result<WorkflowResult, WorkflowError> {
        let start = std::time::Instant::now();
        let mut step_results = Vec::new();
        let mut completed_steps = Vec::new();
        let mut workflow_error = None;

        for (index, step) in workflow.steps.iter().enumerate() {
            // Check condition
            if let Some(condition) = &step.condition {
                if !self.evaluate_condition(condition, variables, &step_results) {
                    step_results.push(StepResult {
                        name: step.name.clone(),
                        success: true,
                        output: None,
                        duration: std::time::Duration::ZERO,
                        error: None,
                        skipped: true,
                    });
                    continue;
                }
            }

            // Execute step
            let result = self.execute_step(step, variables).await?;

            if !result.success {
                match &step.on_failure {
                    FailureStrategy::Stop => {
                        workflow_error = Some(WorkflowError::StepFailed(
                            index,
                            result.error.clone().unwrap_or_else(|| "Unknown error".to_string()),
                        ));
                        step_results.push(result);
                        break;
                    }
                    FailureStrategy::Skip => {
                        step_results.push(StepResult {
                            name: step.name.clone(),
                            success: false,
                            output: None,
                            duration: result.duration,
                            error: result.error.clone(),
                            skipped: true,
                        });
                    }
                    FailureStrategy::Rollback => {
                        let error_msg = result.error.clone().unwrap_or_else(|| "Unknown error".to_string());
                        step_results.push(result);
                        if workflow.enable_rollback {
                            if let Err(e) = self.rollback_steps(workflow, &completed_steps, variables).await {
                                return Ok(WorkflowResult {
                                    success: false,
                                    step_results,
                                    duration: start.elapsed(),
                                    error: Some(format!("Rollback failed: {}", e)),
                                });
                            }
                        }
                        workflow_error = Some(WorkflowError::StepFailed(index, error_msg));
                        break;
                    }
                    FailureStrategy::Continue => {
                        step_results.push(result);
                    }
                }
            } else {
                completed_steps.push(result.clone());
                step_results.push(result);
            }
        }

        let success = workflow_error.is_none()
            && step_results.iter().all(|r| r.success || r.skipped);

        Ok(WorkflowResult {
            success,
            step_results,
            duration: start.elapsed(),
            error: workflow_error.map(|e| e.to_string()),
        })
    }
}

/// Builder for creating workflow steps.
pub struct WorkflowStepBuilder {
    name: String,
    tool: String,
    description: String,
    input: ToolInput,
    on_failure: FailureStrategy,
    max_retries: usize,
    retry_delay: u64,
    condition: Option<StepCondition>,
    continue_on_failure: bool,
}

impl WorkflowStepBuilder {
    /// Create a new step builder.
    pub fn new(name: impl Into<String>, tool: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tool: tool.into(),
            description: String::new(),
            input: ToolInput {
                args: Vec::new(),
                env: HashMap::new(),
                stdin: None,
                timeout: None,
            },
            on_failure: FailureStrategy::default(),
            max_retries: 0,
            retry_delay: 1000,
            condition: None,
            continue_on_failure: false,
        }
    }

    /// Set description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add arguments.
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.input.args = args;
        self
    }

    /// Add environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.input.env.insert(key.into(), value.into());
        self
    }

    /// Set stdin.
    pub fn stdin(mut self, stdin: impl Into<String>) -> Self {
        self.input.stdin = Some(stdin.into());
        self
    }

    /// Set timeout.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.input.timeout = Some(timeout);
        self
    }

    /// Set failure strategy.
    pub fn on_failure(mut self, strategy: FailureStrategy) -> Self {
        self.on_failure = strategy;
        self
    }

    /// Set max retries.
    pub fn max_retries(mut self, max: usize) -> Self {
        self.max_retries = max;
        self
    }

    /// Build the step.
    pub fn build(self) -> WorkflowStep {
        WorkflowStep {
            name: self.name,
            description: self.description,
            tool: self.tool,
            input: self.input,
            on_failure: self.on_failure,
            condition: self.condition,
            continue_on_failure: self.continue_on_failure,
            max_retries: self.max_retries,
            retry_delay: self.retry_delay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_step(name: &str, tool: &str) -> WorkflowStep {
        WorkflowStep {
            name: name.to_string(),
            description: String::new(),
            tool: tool.to_string(),
            input: ToolInput {
                args: vec!["--version".to_string()],
                env: HashMap::new(),
                stdin: None,
                timeout: None,
            },
            on_failure: FailureStrategy::Stop,
            condition: None,
            continue_on_failure: false,
            max_retries: 0,
            retry_delay: 1000,
        }
    }

    #[test]
    fn test_workflow_new() {
        let workflow = Workflow::new("test-workflow");
        assert_eq!(workflow.name, "test-workflow");
        assert!(workflow.steps.is_empty());
    }

    #[test]
    fn test_workflow_builder() {
        let workflow = Workflow::new("test")
            .description("Test workflow")
            .variable("key", "value")
            .with_rollback()
            .on_failure(FailureStrategy::Continue);

        assert_eq!(workflow.name, "test");
        assert_eq!(workflow.description, "Test workflow");
        assert_eq!(workflow.variables.get("key"), Some(&"value".to_string()));
        assert!(workflow.enable_rollback);
        assert!(matches!(workflow.on_failure, FailureStrategy::Continue));
    }

    #[test]
    fn test_workflow_step_builder() {
        let step = WorkflowStepBuilder::new("test-step", "cargo")
            .args(vec!["build".to_string(), "--release".to_string()])
            .env("RUSTFLAGS", "-D warnings")
            .build();

        assert_eq!(step.name, "test-step");
        assert_eq!(step.tool, "cargo");
        assert_eq!(step.input.args, vec!["build", "--release"]);
        assert_eq!(step.input.env.get("RUSTFLAGS"), Some(&"-D warnings".to_string()));
    }

    #[test]
    fn test_condition_variable_exists() {
        let executor = BasicWorkflowExecutor::new(vec![]);
        let mut vars = HashMap::new();
        vars.insert("test_var".to_string(), "value".to_string());

        let condition = StepCondition::VariableExists("test_var".to_string());
        assert!(executor.evaluate_condition(&condition, &vars, &[]));

        let condition = StepCondition::VariableExists("nonexistent".to_string());
        assert!(!executor.evaluate_condition(&condition, &vars, &[]));
    }

    #[test]
    fn test_condition_variable_equals() {
        let executor = BasicWorkflowExecutor::new(vec![]);
        let mut vars = HashMap::new();
        vars.insert("test_var".to_string(), "expected".to_string());

        let condition = StepCondition::VariableEquals {
            name: "test_var".to_string(),
            value: "expected".to_string(),
        };
        assert!(executor.evaluate_condition(&condition, &vars, &[]));

        let condition = StepCondition::VariableEquals {
            name: "test_var".to_string(),
            value: "unexpected".to_string(),
        };
        assert!(!executor.evaluate_condition(&condition, &vars, &[]));
    }

    #[test]
    fn test_condition_previous_success() {
        let executor = BasicWorkflowExecutor::new(vec![]);
        let previous_results = vec![
            StepResult {
                name: "step1".to_string(),
                success: true,
                output: None,
                duration: std::time::Duration::ZERO,
                error: None,
                skipped: false,
            },
            StepResult {
                name: "step2".to_string(),
                success: false,
                output: None,
                duration: std::time::Duration::ZERO,
                error: Some("error".to_string()),
                skipped: false,
            },
        ];

        let vars = HashMap::new();
        let condition = StepCondition::PreviousSuccess("step1".to_string());
        assert!(executor.evaluate_condition(&condition, &vars, &previous_results));

        let condition = StepCondition::PreviousSuccess("step2".to_string());
        assert!(!executor.evaluate_condition(&condition, &vars, &previous_results));
    }

    #[test]
    fn test_failure_strategy_default() {
        let strategy = FailureStrategy::default();
        assert!(matches!(strategy, FailureStrategy::Stop));
    }

    #[test]
    fn test_substitute_variables() {
        let executor = BasicWorkflowExecutor::new(vec![]);
        let mut vars = HashMap::new();
        vars.insert("project".to_string(), "myproject".to_string());
        vars.insert("version".to_string(), "1.0.0".to_string());

        let input = ToolInput {
            args: vec!["build".to_string(), "{project}".to_string()],
            env: {
                let mut map = HashMap::new();
                map.insert("VERSION".to_string(), "{version}".to_string());
                map
            },
            stdin: Some("{project} data".to_string()),
            timeout: None,
        };

        let result = executor.substitute_variables(&input, &vars);

        assert_eq!(result.args, vec!["build", "myproject"]);
        assert_eq!(result.env.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(result.stdin, Some("myproject data".to_string()));
    }

    #[test]
    fn test_workflow_result_default() {
        let workflow = Workflow::default();
        assert_eq!(workflow.name, "");
    }
}
