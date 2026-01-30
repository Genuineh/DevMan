//! Knowledge template system.

use devman_core::{Knowledge, KnowledgeId, TemplateParameter as CoreTemplateParameter};
use std::collections::HashMap;

/// A parameterized knowledge template.
pub struct KnowledgeTemplate {
    /// The base knowledge template
    template: Knowledge,

    /// Parameters that can be substituted
    parameters: Vec<TemplateParameter>,
}

/// A template parameter (local wrapper for core TemplateParameter).
#[derive(Debug, Clone)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,

    /// Description
    pub description: String,

    /// Default value
    pub default_value: Option<String>,

    /// Whether this parameter is required
    pub required: bool,
}

impl From<TemplateParameter> for CoreTemplateParameter {
    fn from(p: TemplateParameter) -> Self {
        Self {
            name: p.name,
            description: p.description,
            default_value: p.default_value,
            required: p.required,
        }
    }
}

/// Template validation result.
#[derive(Debug, Clone)]
pub struct TemplateValidation {
    /// Whether validation passed
    pub valid: bool,

    /// Missing required parameters
    pub missing_required: Vec<String>,

    /// Errors encountered
    pub errors: Vec<String>,
}

impl TemplateValidation {
    /// Create a successful validation.
    pub fn success() -> Self {
        Self {
            valid: true,
            missing_required: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create a failed validation.
    pub fn failure(missing_required: Vec<String>, errors: Vec<String>) -> Self {
        Self {
            valid: false,
            missing_required,
            errors,
        }
    }
}

/// Registry for managing knowledge templates.
pub struct TemplateRegistry {
    templates: Vec<KnowledgeTemplate>,
}

impl TemplateRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    /// Register a template.
    pub fn register(&mut self, template: KnowledgeTemplate) {
        self.templates.push(template);
    }

    /// Get a template by name.
    pub fn get_by_name(&self, name: &str) -> Option<&KnowledgeTemplate> {
        self.templates.iter().find(|t| t.template.title.contains(name))
    }

    /// List all templates.
    pub fn list(&self) -> &[KnowledgeTemplate] {
        &self.templates
    }

    /// Find templates by tag.
    pub fn find_by_tag(&self, tag: &str) -> Vec<&KnowledgeTemplate> {
        self.templates
            .iter()
            .filter(|t| t.template.tags.iter().any(|t| t == tag))
            .collect()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating knowledge templates.
pub struct TemplateBuilder {
    name: String,
    description: String,
    parameters: Vec<TemplateParameter>,
    tags: Vec<String>,
    domains: Vec<String>,
}

impl TemplateBuilder {
    /// Create a new template builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            parameters: Vec::new(),
            tags: Vec::new(),
            domains: Vec::new(),
        }
    }

    /// Set description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a parameter.
    pub fn parameter(mut self, param: TemplateParameter) -> Self {
        self.parameters.push(param);
        self
    }

    /// Add a required parameter.
    pub fn required_parameter(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.parameters.push(TemplateParameter {
            name: name.into(),
            description: description.into(),
            default_value: None,
            required: true,
        });
        self
    }

    /// Add an optional parameter.
    pub fn optional_parameter(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        default: impl Into<String>,
    ) -> Self {
        self.parameters.push(TemplateParameter {
            name: name.into(),
            description: description.into(),
            default_value: Some(default.into()),
            required: false,
        });
        self
    }

    /// Add a tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add a domain.
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domains.push(domain.into());
        self
    }

    /// Build the template with content.
    pub fn build(self, summary: impl Into<String>, detail: impl Into<String>) -> KnowledgeTemplate {
        let detail_str = detail.into();
        let summary_str = summary.into();

        KnowledgeTemplate {
            template: Knowledge {
                id: KnowledgeId::new(),
                title: self.name.clone(),
                knowledge_type: devman_core::KnowledgeType::Template {
                    template: devman_core::TemplateContent {
                        template: detail_str.clone(),
                        parameters: self.parameters.iter().map(|p| CoreTemplateParameter::from(p.clone())).collect(),
                    },
                    适用场景: vec![],
                },
                content: devman_core::KnowledgeContent {
                    summary: summary_str,
                    detail: detail_str,
                    examples: vec![],
                    references: vec![],
                },
                metadata: devman_core::KnowledgeMetadata {
                    domain: self.domains,
                    tech_stack: vec![],
                    scenarios: vec![],
                    quality_score: 0.0,
                    verified: false,
                },
                tags: self.tags,
                related_to: vec![],
                derived_from: vec![],
                usage_stats: devman_core::UsageStats {
                    times_used: 0,
                    last_used: None,
                    success_rate: 0.0,
                    feedback: vec![],
                },
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            parameters: self.parameters,
        }
    }
}

impl Default for TemplateBuilder {
    fn default() -> Self {
        Self::new("")
    }
}

impl KnowledgeTemplate {
    /// Create a new template from knowledge.
    pub fn new(template: Knowledge, parameters: Vec<TemplateParameter>) -> Self {
        Self {
            template,
            parameters,
        }
    }

    /// Validate that all required parameters are provided.
    pub fn validate(&self, params: &HashMap<String, String>) -> TemplateValidation {
        let mut missing_required: Vec<String> = Vec::new();

        for param in &self.parameters {
            if param.required && !params.contains_key(&param.name) {
                missing_required.push(param.name.clone());
            }
        }

        if !missing_required.is_empty() {
            return TemplateValidation::failure(
                missing_required,
                vec!["Missing required parameters".to_string()],
            );
        }

        TemplateValidation::success()
    }

    /// Instantiate the template with given parameters.
    pub fn instantiate(&self, params: &HashMap<String, String>) -> Result<Knowledge, String> {
        // Validate parameters first
        let validation = self.validate(params);
        if !validation.valid {
            return Err(format!(
                "Template validation failed: {:?}",
                validation.missing_required
            ));
        }

        let mut knowledge = self.template.clone();
        knowledge.id = KnowledgeId::new();

        // Build a complete parameter map including defaults
        let mut full_params = HashMap::new();
        for param in &self.parameters {
            if let Some(value) = params.get(&param.name) {
                full_params.insert(param.name.clone(), value.clone());
            } else if let Some(default) = &param.default_value {
                full_params.insert(param.name.clone(), default.clone());
            }
        }

        // Substitute parameters in content
        for (key, value) in &full_params {
            let placeholder = format!("{{{{{}}}}}", key);
            knowledge.content.summary = knowledge.content.summary.replace(&placeholder, value);
            knowledge.content.detail = knowledge.content.detail.replace(&placeholder, value);

            // Also substitute in examples if they contain the placeholder
            for example in &mut knowledge.content.examples {
                example.code = example.code.replace(&placeholder, value);
                example.description = example.description.replace(&placeholder, value);
            }
        }

        // Handle conditional sections: {{#if param}}...{{/if param}}
        knowledge = self.process_conditionals(knowledge, &full_params);

        // Handle list iterations: {{#each items}}...{{/each items}}
        knowledge = self.process_iterations(knowledge, &full_params);

        Ok(knowledge)
    }

    /// Process conditional sections in the template.
    fn process_conditionals(&self, knowledge: Knowledge, _params: &HashMap<String, String>) -> Knowledge {
        // Conditional sections require backreferences which Rust regex doesn't support.
        // For now, we keep the content as-is. Users can use simple parameter substitution.
        knowledge
    }

    /// Process list iterations in the template.
    fn process_iterations(&self, knowledge: Knowledge, _params: &HashMap<String, String>) -> Knowledge {
        // Iterations require backreferences which Rust regex doesn't support.
        // For now, we keep the content as-is. Users can use simple parameter substitution.
        knowledge
    }

    /// Get the parameters for this template.
    pub fn parameters(&self) -> &[TemplateParameter] {
        &self.parameters
    }

    /// Get the template name.
    pub fn name(&self) -> &str {
        &self.template.title
    }

    /// Get required parameters.
    pub fn required_parameters(&self) -> Vec<&TemplateParameter> {
        self.parameters
            .iter()
            .filter(|p| p.required)
            .collect()
    }

    /// Get optional parameters.
    pub fn optional_parameters(&self) -> Vec<&TemplateParameter> {
        self.parameters
            .iter()
            .filter(|p| !p.required)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_template() -> KnowledgeTemplate {
        let knowledge = Knowledge {
            id: KnowledgeId::new(),
            title: "Test Template".to_string(),
            knowledge_type: devman_core::KnowledgeType::Template {
                template: devman_core::TemplateContent {
                    template: "Summary: {{name}}, Detail: This is {{description}} with value {{value}}".to_string(),
                    parameters: vec![],
                },
                适用场景: vec![],
            },
            content: devman_core::KnowledgeContent {
                summary: "Summary: {{name}}".to_string(),
                detail: "This is {{description}} with value {{value}}".to_string(),
                examples: vec![],
                references: vec![],
            },
            metadata: devman_core::KnowledgeMetadata {
                domain: vec!["test".to_string()],
                tech_stack: vec![],
                scenarios: vec![],
                quality_score: 1.0,
                verified: true,
            },
            tags: vec!["template".to_string()],
            related_to: vec![],
            derived_from: vec![],
            usage_stats: devman_core::UsageStats {
                times_used: 0,
                last_used: None,
                success_rate: 1.0,
                feedback: vec![],
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let parameters = vec![
            TemplateParameter {
                name: "name".to_string(),
                description: "Name parameter".to_string(),
                default_value: None,
                required: true,
            },
            TemplateParameter {
                name: "description".to_string(),
                description: "Description parameter".to_string(),
                default_value: Some("default description".to_string()),
                required: false,
            },
            TemplateParameter {
                name: "value".to_string(),
                description: "Value parameter".to_string(),
                default_value: None,
                required: true,
            },
        ];

        KnowledgeTemplate::new(knowledge, parameters)
    }

    #[test]
    fn test_template_validation_success() {
        let template = create_test_template();
        let mut params = HashMap::new();
        params.insert("name".to_string(), "TestName".to_string());
        params.insert("value".to_string(), "42".to_string());

        let result = template.validate(&params);
        assert!(result.valid);
        assert!(result.missing_required.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_template_validation_missing_required() {
        let template = create_test_template();
        let mut params = HashMap::new();
        params.insert("name".to_string(), "TestName".to_string());
        // Missing "value" which is required

        let result = template.validate(&params);
        assert!(!result.valid);
        assert_eq!(result.missing_required, vec!["value"]);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_template_validation_all_missing() {
        let template = create_test_template();
        let params = HashMap::new();

        let result = template.validate(&params);
        assert!(!result.valid);
        assert_eq!(result.missing_required.len(), 2);
        assert!(result.missing_required.contains(&"name".to_string()));
        assert!(result.missing_required.contains(&"value".to_string()));
    }

    #[test]
    fn test_template_instantiate_full_params() {
        let template = create_test_template();
        let mut params = HashMap::new();
        params.insert("name".to_string(), "MyName".to_string());
        params.insert("description".to_string(), "MyDescription".to_string());
        params.insert("value".to_string(), "100".to_string());

        let result = template.instantiate(&params);
        assert!(result.is_ok());

        let knowledge = result.unwrap();
        assert_eq!(knowledge.content.summary, "Summary: MyName");
        assert_eq!(knowledge.content.detail, "This is MyDescription with value 100");
        assert_ne!(knowledge.id, template.template.id); // New ID generated
    }

    #[test]
    fn test_template_instantiate_with_defaults() {
        let template = create_test_template();
        let mut params = HashMap::new();
        params.insert("name".to_string(), "MyName".to_string());
        params.insert("value".to_string(), "100".to_string());
        // description uses default

        let result = template.instantiate(&params);
        assert!(result.is_ok());

        let knowledge = result.unwrap();
        assert_eq!(knowledge.content.summary, "Summary: MyName");
        assert_eq!(knowledge.content.detail, "This is default description with value 100");
    }

    #[test]
    fn test_template_instantiate_missing_required_fails() {
        let template = create_test_template();
        let mut params = HashMap::new();
        params.insert("name".to_string(), "MyName".to_string());
        // Missing required "value"

        let result = template.instantiate(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("validation failed"));
    }

    #[test]
    fn test_template_parameters() {
        let template = create_test_template();
        assert_eq!(template.parameters().len(), 3);
        assert_eq!(template.name(), "Test Template");
    }

    #[test]
    fn test_required_parameters() {
        let template = create_test_template();
        let required = template.required_parameters();
        assert_eq!(required.len(), 2);

        let required_names: Vec<_> = required.iter().map(|p| &p.name).collect();
        assert!(required_names.contains(&&"name".to_string()));
        assert!(required_names.contains(&&"value".to_string()));
    }

    #[test]
    fn test_optional_parameters() {
        let template = create_test_template();
        let optional = template.optional_parameters();
        assert_eq!(optional.len(), 1);
        assert_eq!(optional[0].name, "description");
    }

    #[test]
    fn test_template_registry() {
        let mut registry = TemplateRegistry::new();
        let template = create_test_template();
        registry.register(template);

        assert_eq!(registry.list().len(), 1);

        let found = registry.get_by_name("Test Template");
        assert!(found.is_some());

        let by_tag = registry.find_by_tag("template");
        assert_eq!(by_tag.len(), 1);
    }

    #[test]
    fn test_template_builder_basic() {
        let template = TemplateBuilder::new("Builder Test")
            .description("A test template")
            .tag("test-tag")
            .domain("test-domain")
            .build("Summary content", "Detail content");

        assert_eq!(template.name(), "Builder Test");
        assert_eq!(template.template.content.summary, "Summary content");
        assert_eq!(template.template.content.detail, "Detail content");
        assert!(template.template.tags.contains(&"test-tag".to_string()));
        assert!(template.template.metadata.domain.contains(&"test-domain".to_string()));
    }

    #[test]
    fn test_template_builder_with_parameters() {
        let template = TemplateBuilder::new("Builder With Params")
            .required_parameter("param1", "First parameter")
            .optional_parameter("param2", "Second parameter", "default_value")
            .build("Summary {{param1}}", "Detail: {{param2}}");

        assert_eq!(template.parameters().len(), 2);
        assert_eq!(template.parameters()[0].name, "param1");
        assert!(template.parameters()[0].required);
        assert_eq!(template.parameters()[1].name, "param2");
        assert!(!template.parameters()[1].required);
        assert_eq!(template.parameters()[1].default_value, Some("default_value".to_string()));
    }

    #[test]
    fn test_template_instantiate_with_examples() {
        // Create a template with examples to test parameter substitution in examples
        let knowledge = Knowledge {
            id: KnowledgeId::new(),
            title: "Template With Examples".to_string(),
            knowledge_type: devman_core::KnowledgeType::Template {
                template: devman_core::TemplateContent {
                    template: "Example template".to_string(),
                    parameters: vec![],
                },
                适用场景: vec![],
            },
            content: devman_core::KnowledgeContent {
                summary: "Summary {{name}}".to_string(),
                detail: "Detail {{name}}".to_string(),
                examples: vec![
                    devman_core::CodeSnippet {
                        language: "rust".to_string(),
                        description: "Example for {{name}}".to_string(),
                        code: "let x = {{name}};".to_string(),
                    },
                ],
                references: vec![],
            },
            metadata: devman_core::KnowledgeMetadata {
                domain: vec![],
                tech_stack: vec![],
                scenarios: vec![],
                quality_score: 1.0,
                verified: true,
            },
            tags: vec![],
            related_to: vec![],
            derived_from: vec![],
            usage_stats: devman_core::UsageStats {
                times_used: 0,
                last_used: None,
                success_rate: 1.0,
                feedback: vec![],
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let parameters = vec![
            TemplateParameter {
                name: "name".to_string(),
                description: "Name".to_string(),
                default_value: None,
                required: true,
            },
        ];

        let template = KnowledgeTemplate::new(knowledge, parameters);

        let mut params = HashMap::new();
        params.insert("name".to_string(), "test_value".to_string());

        let result = template.instantiate(&params);
        assert!(result.is_ok());

        let knowledge = result.unwrap();
        assert_eq!(knowledge.content.summary, "Summary test_value");
        assert_eq!(knowledge.content.detail, "Detail test_value");
        assert_eq!(knowledge.content.examples[0].description, "Example for test_value");
        assert_eq!(knowledge.content.examples[0].code, "let x = test_value;");
    }

    #[test]
    fn test_template_validation_success_result() {
        let validation = TemplateValidation::success();
        assert!(validation.valid);
        assert!(validation.missing_required.is_empty());
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_template_validation_failure_result() {
        let missing = vec!["param1".to_string(), "param2".to_string()];
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let validation = TemplateValidation::failure(missing.clone(), errors.clone());

        assert!(!validation.valid);
        assert_eq!(validation.missing_required, missing);
        assert_eq!(validation.errors, errors);
    }

    #[test]
    fn test_template_default_registry() {
        let registry = TemplateRegistry::default();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_template_default_builder() {
        let builder = TemplateBuilder::default();
        assert_eq!(builder.name, "");
    }

    #[test]
    fn test_template_parameter_from_conversion() {
        let local_param = TemplateParameter {
            name: "test".to_string(),
            description: "Test param".to_string(),
            default_value: Some("default".to_string()),
            required: false,
        };

        let core_param: CoreTemplateParameter = local_param.clone().into();
        assert_eq!(core_param.name, "test");
        assert_eq!(core_param.description, "Test param");
        assert_eq!(core_param.default_value, Some("default".to_string()));
        assert!(!core_param.required);
    }
}

// Note: Conditional sections and list iterations require more complex parsing
// or a different approach (e.g., two-pass parsing or no backreferences).
// For now, basic parameter substitution is supported.
