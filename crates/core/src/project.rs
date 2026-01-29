//! Project model - engineering context and configuration.

use serde::{Deserialize, Serialize};
use crate::id::{ProjectId, PhaseId, QualityProfileId};
use crate::Time;

/// A project represents the engineering context for a goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier
    pub id: ProjectId,

    /// Project name
    pub name: String,

    /// Description
    pub description: String,

    /// Project configuration
    pub config: ProjectConfig,

    /// Project phases
    pub phases: Vec<PhaseId>,

    /// Current active phase
    pub current_phase: PhaseId,

    /// When created
    pub created_at: Time,
}

/// Project configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Technology stack
    pub tech_stack: Vec<String>,

    /// Directory structure
    pub structure: DirStructure,

    /// Quality profile
    pub quality_profile: QualityProfileId,

    /// Tool configuration
    pub tools: ToolConfig,
}

/// Directory structure conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirStructure {
    /// Required directories
    pub dirs: Vec<String>,

    /// Naming and organization conventions
    pub conventions: Vec<String>,
}

/// Tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Build tool
    pub build: BuildTool,

    /// Test framework
    pub test_framework: TestFramework,

    /// Linters
    pub linters: Vec<String>,

    /// Formatters
    pub formatters: Vec<String>,
}

/// Build tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildTool {
    Cargo,
    Npm,
    Yarn,
    Make,
    Gradle,
    Maven,
}

/// Test frameworks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestFramework {
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "jest")]
    Jest,
    #[serde(rename = "pytest")]
    Pytest,
    #[serde(rename = "gotest")]
    GoTest,
}
