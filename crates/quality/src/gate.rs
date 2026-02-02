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

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::{PassCondition, FailureAction, PhaseGate};

    #[test]
    fn test_quality_gate_creation() {
        let gate = QualityGate {
            name: "test-gate".to_string(),
            description: "Test quality gate".to_string(),
            checks: vec![],
            pass_condition: PassCondition::AllPassed,
            on_failure: FailureAction::Block,
        };

        assert_eq!(gate.name, "test-gate");
        assert!(matches!(gate.pass_condition, PassCondition::AllPassed));
        assert!(matches!(gate.on_failure, FailureAction::Block));
    }

    #[test]
    fn test_quality_profile_creation() {
        let profile = QualityProfile {
            name: "test-profile".to_string(),
            description: "Test profile".to_string(),
            checks: vec![],
            phase_gates: vec![],
            default_strategy: GateStrategy::AllMustPass,
        };

        assert_eq!(profile.name, "test-profile");
        assert!(matches!(profile.default_strategy, GateStrategy::AllMustPass));
    }

    #[test]
    fn test_phase_gate_creation() {
        let gate = PhaseGate {
            phase: PhaseId::new(),
            checks: vec![],
            strategy: GateStrategy::AllMustPass,
        };

        assert!(!gate.phase.to_string().is_empty());
    }

    #[test]
    fn test_gate_strategy_warnings_allowed() {
        let strategy = GateStrategy::WarningsAllowed { max_warnings: 5 };
        if let GateStrategy::WarningsAllowed { max_warnings } = strategy {
            assert_eq!(max_warnings, 5);
        } else {
            panic!("Expected WarningsAllowed variant");
        }
    }

    #[test]
    fn test_gate_strategy_custom() {
        let strategy = GateStrategy::Custom { rule: "custom-rule".to_string() };
        if let GateStrategy::Custom { rule } = strategy {
            assert_eq!(rule, "custom-rule");
        } else {
            panic!("Expected Custom variant");
        }
    }
}
