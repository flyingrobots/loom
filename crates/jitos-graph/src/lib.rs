// @ts-check
use jitos_core::Hash;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

pub mod ids;

pub use ids::{DeterministicIdAllocator, NodeId};

new_key_type! { pub struct NodeKey; }
new_key_type! { pub struct EdgeKey; }

/// A node in the WARP graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpNode {
    pub id: NodeId,
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
        // TODO: Implement deterministic sorting/hashing logic
        // For now, a placeholder
        Hash([0u8; 32])
    }
}
