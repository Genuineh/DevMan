//! Custom quality checks (business-specific).

use devman_core::{QualityCheck, QualityCheckId, QualityCategory, CustomCheckSpec, CommandSpec, ValidationSpec, Severity};

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
                    expected_exit_code: Some(0),
                },
                validation: ValidationSpec {
                    output_parser: devman_core::OutputParser::LineContains {
                        text: String::new(),
                    },
                    pass_condition: "true".to_string(),
                    extract_metrics: Vec::new(),
                },
                human_review: None,
            }),
            severity: self.severity,
            category: self.category,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
