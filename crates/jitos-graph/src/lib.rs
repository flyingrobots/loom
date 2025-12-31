// @ts-check
use jitos_core::Hash;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

pub mod ids;

pub use ids::{DeterministicIdAllocator, NodeId};

new_key_type! { pub struct NodeKey; }
new_key_type! { pub struct EdgeKey; }

#[derive(Debug, Clone, Serialize)]
struct GraphCommitV0 {
    version: &'static str,
    nodes: Vec<NodeCommitV0>,
    edges: Vec<EdgeCommitV0>,
}

#[derive(Debug, Clone, Serialize)]
struct NodeCommitV0 {
    node_id: NodeId,
    kind: String,
    payload_bytes: Vec<u8>,
    attachment: Option<Hash>,
}

#[derive(Debug, Clone, Serialize)]
struct EdgeCommitV0 {
    edge_id: Hash,
    from: NodeId,
    to: NodeId,
    kind: String,
    payload_bytes: Option<Vec<u8>>,
    attachment: Option<Hash>,
}

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
        self.compute_hash_checked()
            .expect("canonical graph hashing must succeed")
    }

    /// Computes the canonical graph commit digest (SPEC-WARP-0001).
    ///
    /// This is a graph-wide, deterministic commitment to state:
    /// - independent of insertion order
    /// - independent of HashMap/SlotMap iteration order
    /// - stable across runs/platforms (via SPEC-0001 canonical encoding)
    pub fn compute_hash_checked(&self) -> Result<Hash, jitos_core::canonical::CanonicalError> {
        // Nodes: sort by NodeId bytes ascending.
        let mut nodes: Vec<NodeCommitV0> = Vec::with_capacity(self.nodes.len());
        for (_k, n) in self.nodes.iter() {
            let payload_bytes = jitos_core::canonical::encode(&n.data)?;
            nodes.push(NodeCommitV0 {
                node_id: n.id,
                kind: n.node_type.clone(),
                payload_bytes,
                attachment: n.attachment,
            });
        }
        nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));

        // Edges: derive a deterministic EdgeId from semantic content (endpoints + kind + attachment),
        // then sort by that ID bytes ascending.
        let mut edges: Vec<EdgeCommitV0> = Vec::with_capacity(self.edges.len());
        for (_k, e) in self.edges.iter() {
            let from = self.nodes.get(e.source).map(|n| n.id).ok_or_else(|| {
                jitos_core::canonical::CanonicalError::Decode(
                    "edge source references missing node".into(),
                )
            })?;
            let to = self.nodes.get(e.target).map(|n| n.id).ok_or_else(|| {
                jitos_core::canonical::CanonicalError::Decode(
                    "edge target references missing node".into(),
                )
            })?;

            // Domain-separated edge id input to avoid accidental ambiguity if fields evolve.
            let edge_id_input = ("warp-edge-v0", from, to, e.edge_type.as_str(), e.attachment);
            let edge_id = jitos_core::canonical::hash_canonical(&edge_id_input)?;

            edges.push(EdgeCommitV0 {
                edge_id,
                from,
                to,
                kind: e.edge_type.clone(),
                payload_bytes: None,
                attachment: e.attachment,
            });
        }
        edges.sort_by(|a, b| a.edge_id.cmp(&b.edge_id));

        let commit = GraphCommitV0 {
            version: "graph-commit-v0",
            nodes,
            edges,
        };

        jitos_core::canonical::hash_canonical(&commit)
    }
}
