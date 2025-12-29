use serde::{Deserialize, Serialize};

/// SLAPS v2: System-Level Action Protocol Specification
/// Represents a formal Intent Definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Slaps {
    #[serde(rename = "slaps_version")]
    pub version: String,
    pub intent: String,
    pub target: Target,
    pub context: Context,
    pub scope: Scope,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub priorities: Vec<String>,
    pub success_criteria: Vec<SuccessCriteria>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Target {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    pub environment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<ContextLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextLink {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scope {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuccessCriteria {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
}
