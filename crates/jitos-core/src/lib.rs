// @ts-check
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod canonical;

/// A 256-bit BLAKE3 hash.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

/// System-Level Action Protocol (SLAP) v2.
/// Defines the set of valid intentional mutations to the JITOS universe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "payload")]
pub enum Slap {
    /// Create a new node in the graph.
    CreateNode { node_type: String, data: serde_json::Value },
    /// Delete an existing node.
    DeleteNode { id: String },
    /// Connect two nodes.
    Connect { source: String, target: String, edge_type: String },
    /// Invoke a sandboxed Rhai script.
    InvokeScript { script_id: Hash, args: Vec<serde_json::Value> },
    /// Set the logical time.
    SetTime { tick: u64, dt: f64 },
    /// Collapse a Shadow Working Set (SWS).
    Collapse { sws_id: String },
}

/// A deterministic record of a single tick's execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub tick: u64,
    pub state_hash: Hash,
    pub applied_slaps: Vec<Hash>,
    pub timestamp: u64,
    pub signature: Option<String>,
}

/// Standard Error types for the JITOS universe.
#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum JitosError {
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),
    #[error("Conflict detected: {0}")]
    Conflict(String),
    #[error("Access denied: {0}")]
    AccessDenied(String),
    #[error("Not found: {0}")]
    NotFound(String),
}
