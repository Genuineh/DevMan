//! Quality check registry.

use devman_core::{QualityCheck, QualityCheckId, QualityCategory};
use std::collections::HashMap;

/// Registry for quality checks.
pub struct QualityCheckRegistry {
    checks: HashMap<QualityCheckId, QualityCheck>,
    by_category: HashMap<QualityCategory, Vec<QualityCheckId>>,
}

impl QualityCheckRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
            by_category: HashMap::new(),
        }
    }

    /// Register a check.
    pub fn register(&mut self, check: QualityCheck) -> Result<(), String> {
        let id = check.id;
        let category = check.category;

        self.by_category
            .entry(category)
            .or_default()
            .push(id);
        self.checks.insert(id, check);
        Ok(())
    }

    /// Unregister a check.
    pub fn unregister(&mut self, id: QualityCheckId) -> Option<QualityCheck> {
        let check = self.checks.remove(&id)?;
        let cat_list = self.by_category.get_mut(&check.category)?;
        cat_list.retain(|&x| x != id);
        Some(check)
    }

    /// Get a check by ID.
    pub fn get(&self, id: QualityCheckId) -> Option<&QualityCheck> {
        self.checks.get(&id)
    }

    /// List all checks.
    pub fn list(&self) -> Vec<&QualityCheck> {
        self.checks.values().collect()
    }

    /// Find checks by category.
    pub fn find_by_category(&self, category: QualityCategory) -> Vec<&QualityCheck> {
        self.by_category
            .get(&category)
            .into_iter()
            .flat_map(|ids| ids.iter().filter_map(|id| self.checks.get(id)))
            .collect()
    }
}

impl Default for QualityCheckRegistry {
    fn default() -> Self {
        Self::new()
    }
}
