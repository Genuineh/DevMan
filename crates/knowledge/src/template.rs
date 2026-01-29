//! Knowledge template system.

use devman_core::{Knowledge, KnowledgeId};
use std::collections::HashMap;

/// A parameterized knowledge template.
pub struct KnowledgeTemplate {
    /// The base knowledge template
    template: Knowledge,

    /// Parameters that can be substituted
    parameters: Vec<TemplateParameter>,
}

/// A template parameter.
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

impl KnowledgeTemplate {
    /// Create a new template from knowledge.
    pub fn new(template: Knowledge, parameters: Vec<TemplateParameter>) -> Self {
        Self {
            template,
            parameters,
        }
    }

    /// Instantiate the template with given parameters.
    pub fn instantiate(&self, params: &HashMap<String, String>) -> Knowledge {
        let mut knowledge = self.template.clone();
        knowledge.id = KnowledgeId::new();

        // Substitute parameters in content
        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            knowledge.content.summary = knowledge.content.summary.replace(&placeholder, value);
            knowledge.content.detail = knowledge.content.detail.replace(&placeholder, value);
        }

        knowledge
    }

    /// Get the parameters for this template.
    pub fn parameters(&self) -> &[TemplateParameter] {
        &self.parameters
    }
}
