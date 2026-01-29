//! Evolution optimizer - adjusts strategies based on metrics.

use crate::SystemMetrics;

/// Adjustment to a strategy parameter.
#[derive(Debug, Clone)]
pub struct StrategyAdjustment {
    /// Parameter name
    pub parameter: String,
    /// New value
    pub value: f32,
    /// Reason for adjustment
    pub reason: String,
}

/// Optimizes system strategies based on performance.
pub struct EvolutionOptimizer;

impl EvolutionOptimizer {
    /// Create a new optimizer.
    pub fn new() -> Self {
        Self
    }

    /// Analyze metrics and suggest adjustments.
    pub fn suggest_adjustments(&self, metrics: &SystemMetrics) -> Vec<StrategyAdjustment> {
        let mut adjustments = Vec::new();

        // If failure rate is high, suggest increasing caution
        if metrics.tasks.failure_rate > 0.5 {
            adjustments.push(StrategyAdjustment {
                parameter: "exploration_ratio".to_string(),
                value: 0.2,
                reason: format!("High failure rate ({:.0}%), reducing exploration", metrics.tasks.failure_rate * 100.0),
            });
        }

        // If reflection success is low, suggest adjusting thresholds
        if metrics.reflection_success_rate < 0.7 {
            adjustments.push(StrategyAdjustment {
                parameter: "min_insight_confidence".to_string(),
                value: 0.5,
                reason: "Low reflection success, lowering confidence threshold".to_string(),
            });
        }

        adjustments
    }

    /// Apply an adjustment to strategy parameters.
    pub fn apply_adjustment(
        &self,
        params: &mut std::collections::HashMap<String, f32>,
        adjustment: &StrategyAdjustment,
    ) {
        params.insert(adjustment.parameter.clone(), adjustment.value);
    }
}

impl Default for EvolutionOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
