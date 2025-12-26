// @ts-check
use jitos_core::Hash;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

new_key_type! { pub struct NodeKey; }
new_key_type! { pub struct EdgeKey; }

/// A node in the WARP graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpNode {
    pub id: String,
    pub node_type: String,
    pub data: serde_json::Value,
    pub attachment: Option<Hash>, // Reference to another WARP graph
}

/// A directed edge in the WARP graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpEdge {
    pub source: NodeKey,
    pub target: NodeKey,
    pub edge_type: String,
    pub attachment: Option<Hash>,
}

/// The WARP Graph structure (Paper I).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WarpGraph {
    pub nodes: SlotMap<NodeKey, WarpNode>,
    pub edges: SlotMap<EdgeKey, WarpEdge>,
}

impl WarpGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes the BLAKE3 root hash of the graph state.
    pub fn compute_hash(&self) -> Hash {
        let mut hasher = blake3::Hasher::new();
        // Deterministic sorting/hashing logic would go here
        // For now, a placeholder
        Hash([0u8; 32])
    }
}
