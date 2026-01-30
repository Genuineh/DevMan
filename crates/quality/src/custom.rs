//! Custom quality checks (business-specific).

use devman_core::{
    QualityCheck, QualityCheckId, QualityCategory, CustomCheckSpec, CommandSpec,
    ValidationSpec, OutputParser, MetricExtractor, Severity, HumanReviewSpec,
};

/// Registry for custom business quality checks.
pub struct CustomCheckRegistry {
    checks: Vec<QualityCheck>,
}

impl CustomCheckRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    /// Register a custom check.
    pub fn register(&mut self, check: QualityCheck) -> Result<(), String> {
        // Validate check
        if check.name.is_empty() {
            return Err("Check name cannot be empty".to_string());
        }

        self.checks.push(check);
        Ok(())
    }

    /// Unregister a check.
    pub fn unregister(&mut self, id: QualityCheckId) -> Option<QualityCheck> {
        let pos = self.checks.iter().position(|c| c.id == id)?;
        Some(self.checks.remove(pos))
    }

    /// Get a check by ID.
    pub fn get(&self, id: QualityCheckId) -> Option<&QualityCheck> {
        self.checks.iter().find(|c| c.id == id)
    }

    /// List all checks.
    pub fn list(&self) -> &[QualityCheck] {
        &self.checks
    }

    /// Find checks by category.
    pub fn find_by_category(&self, category: QualityCategory) -> Vec<&QualityCheck> {
        self.checks
            .iter()
            .filter(|c| c.category == category)
            .collect()
    }
}

impl Default for CustomCheckRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating custom quality checks.
pub struct CustomCheckBuilder {
    name: String,
    description: String,
    severity: Severity,
    category: QualityCategory,
    command: String,
    args: Vec<String>,
    timeout: std::time::Duration,
    expected_exit_code: Option<i32>,
    output_parser: OutputParser,
    pass_condition: String,
    extract_metrics: Vec<MetricExtractor>,
    human_review: Option<HumanReviewSpec>,
}

impl CustomCheckBuilder {
    /// Create a new builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            severity: Severity::Warning,
            category: QualityCategory::Correctness,
            command: String::new(),
            args: Vec::new(),
            timeout: std::time::Duration::from_secs(60),
            expected_exit_code: Some(0),
            output_parser: OutputParser::LineContains {
                text: String::new(),
            },
            pass_condition: "true".to_string(),
            extract_metrics: Vec::new(),
            human_review: None,
        }
    }

    /// Set description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set severity.
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set category.
    pub fn category(mut self, category: QualityCategory) -> Self {
        self.category = category;
        self
    }

    /// Set command.
    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.command = cmd.into();
        self
    }

    /// Add argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set timeout.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the check.
    pub fn build(self) -> QualityCheck {
        use devman_core::{CustomCheckSpec, CommandSpec, ValidationSpec};

        let name = self.name.clone();
        QualityCheck {
            id: QualityCheckId::new(),
            name: self.name,
            description: self.description,
            check_type: devman_core::QualityCheckType::Custom(CustomCheckSpec {
                name,
                check_command: CommandSpec {
                    command: self.command,
                    args: self.args,
                    timeout: self.timeout,
                    expected_exit_code: self.expected_exit_code,
                },
                validation: ValidationSpec {
                    output_parser: self.output_parser,
                    pass_condition: self.pass_condition,
                    extract_metrics: self.extract_metrics,
                },
                human_review: self.human_review,
            }),
            severity: self.severity,
            category: self.category,
        }
    }

    /// Set the expected exit code.
    pub fn expected_exit_code(mut self, code: i32) -> Self {
        self.expected_exit_code = Some(code);
        self
    }

    /// Set the output parser.
    pub fn output_parser(mut self, parser: OutputParser) -> Self {
        self.output_parser = parser;
        self
    }

    /// Set the pass condition.
    pub fn pass_condition(mut self, condition: impl Into<String>) -> Self {
        self.pass_condition = condition.into();
        self
    }

    /// Add a metric extractor.
    pub fn extract_metric(mut self, extractor: MetricExtractor) -> Self {
        self.extract_metrics.push(extractor);
        self
    }

    /// Set human review spec.
    pub fn human_review(mut self, review: HumanReviewSpec) -> Self {
        self.human_review = Some(review);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::{AnswerType, ReviewQuestion};

    #[test]
    fn test_custom_check_builder() {
        let check = CustomCheckBuilder::new("test-check")
            .description("A test check")
            .severity(Severity::Error)
            .category(QualityCategory::Performance)
            .command("echo")
            .arg("hello")
            .build();

        assert_eq!(check.name, "test-check");
        assert_eq!(check.description, "A test check");
    }

    #[test]
    fn test_custom_check_with_regex_parser() {
        let check = CustomCheckBuilder::new("coverage-check")
            .description("Check test coverage")
            .command("cargo")
            .arg("test")
            .output_parser(devman_core::OutputParser::Regex {
                pattern: r"Coverage: (?P<coverage>[0-9.]+)%".to_string(),
            })
            .pass_condition("coverage >= 80")
            .build();

        assert_eq!(check.name, "coverage-check");
    }

    #[test]
    fn test_custom_check_with_jsonpath_parser() {
        let check = CustomCheckBuilder::new("json-check")
            .description("Check JSON output")
            .command("node")
            .arg("check.js")
            .output_parser(devman_core::OutputParser::JsonPath {
                path: "status".to_string(),
            })
            .pass_condition("value == passed")
            .build();

        assert_eq!(check.name, "json-check");
    }

    #[test]
    fn test_custom_check_with_metric_extraction() {
        let check = CustomCheckBuilder::new("coverage-metrics")
            .description("Extract coverage metrics")
            .command("cargo")
            .arg("test")
            .output_parser(devman_core::OutputParser::Regex {
                pattern: r"Coverage: (?P<value>[0-9.]+)%".to_string(),
            })
            .pass_condition("value >= 80")
            .extract_metric(MetricExtractor {
                name: "coverage".to_string(),
                extractor: devman_core::OutputParser::Regex {
                    pattern: r"(?P<value>[0-9.]+)%".to_string(),
                },
                unit: Some("%".to_string()),
            })
            .build();

        assert_eq!(check.name, "coverage-metrics");
    }

    #[test]
    fn test_custom_check_with_human_review() {
        let check = CustomCheckBuilder::new("business-rule-check")
            .description("Check business rules")
            .command("python")
            .arg("check_rules.py")
            .output_parser(devman_core::OutputParser::LineContains {
                text: "All rules passed".to_string(),
            })
            .human_review(HumanReviewSpec {
                reviewers: vec!["business-team@example.com".to_string()],
                review_guide: "Review the business rule validation results".to_string(),
                review_form: vec![
                    ReviewQuestion {
                        question: "Are the business rules correctly implemented?".to_string(),
                        answer_type: AnswerType::YesNo,
                        required: true,
                    },
                ],
                timeout: std::time::Duration::from_secs(24 * 60 * 60),
                auto_pass_threshold: None,
            })
            .build();

        assert_eq!(check.name, "business-rule-check");
    }

    #[test]
    fn test_custom_check_registry() {
        let mut registry = CustomCheckRegistry::new();

        let check1 = CustomCheckBuilder::new("check1")
            .description("First check")
            .command("echo")
            .arg("test")
            .build();

        let check2 = CustomCheckBuilder::new("check2")
            .description("Second check")
            .category(QualityCategory::Security)
            .command("echo")
            .arg("security")
            .build();

        // Register checks
        assert!(registry.register(check1.clone()).is_ok());
        assert!(registry.register(check2).is_ok());

        // List all checks
        assert_eq!(registry.list().len(), 2);

        // Find by category
        let security_checks = registry.find_by_category(QualityCategory::Security);
        assert_eq!(security_checks.len(), 1);

        // Get by ID
        let retrieved = registry.get(check1.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "check1");

        // Unregister
        let removed = registry.unregister(check1.id);
        assert!(removed.is_some());
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_custom_check_registry_empty_name() {
        let mut registry = CustomCheckRegistry::new();
        let check = CustomCheckBuilder::new("")
            .description("Invalid check")
            .command("echo")
            .arg("test")
            .build();

        let result = registry.register(check);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Check name cannot be empty");
    }

    #[test]
    fn test_custom_check_expected_exit_code() {
        let check = CustomCheckBuilder::new("exit-code-check")
            .description("Check with custom exit code")
            .command("custom-tool")
            .arg("validate")
            .expected_exit_code(1)  // Tool uses 1 for success
            .build();

        assert_eq!(check.name, "exit-code-check");
    }
}
