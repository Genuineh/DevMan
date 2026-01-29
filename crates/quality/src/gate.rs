//! Quality gates and profiles.

use devman_core::{QualityCheckId, QualityGate, PhaseId, PassCondition, GateStrategy, QualityProfile};

/// Extension trait for QualityGate providing builder methods.
pub trait QualityGateBuilder: Sized {
    /// Create a new quality gate.
    fn new(name: impl Into<String>) -> Self;

    /// Set description.
    fn with_description(self, desc: impl Into<String>) -> Self;

    /// Add a check.
    fn add_check(self, check: QualityCheckId) -> Self;

    /// Set pass condition.
    fn with_pass_condition(self, condition: PassCondition) -> Self;

    /// Set failure action.
    fn with_on_failure(self, action: devman_core::FailureAction) -> Self;
}

/// Extension trait for QualityProfile providing builder methods.
pub trait QualityProfileBuilder: Sized {
    /// Create a new quality profile.
    fn new(name: impl Into<String>) -> Self;

    /// Set description.
    fn with_description(self, desc: impl Into<String>) -> Self;

    /// Set default strategy.
    fn with_default_strategy(self, strategy: GateStrategy) -> Self;

    /// Add a check to this profile.
    fn add_check(self, check: QualityCheckId) -> Self;

    /// Add a phase gate.
    fn add_phase_gate(self, gate: devman_core::PhaseGate) -> Self;
}
