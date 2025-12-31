use jitos_core::Hash;
use jitos_graph::ids::NodeId;
use jitos_graph::{NodeKey, WarpEdge, WarpGraph, WarpNode};

fn h(byte: u8) -> Hash {
    Hash([byte; 32])
}

fn node_id(byte: u8) -> NodeId {
    NodeId::from_hash(h(byte))
}

fn insert_node(
    graph: &mut WarpGraph,
    id: NodeId,
    node_type: &str,
    data: serde_json::Value,
) -> NodeKey {
    graph.nodes.insert(WarpNode {
        id,
        node_type: node_type.to_string(),
        data,
        attachment: None,
    })
}

#[test]
fn graph_hash_is_invariant_under_insertion_order() {
    // Build the same logical graph in two different insertion orders.
    //
    // If hashing depends on SlotMap insertion order or iteration order, this test should fail.

    // Graph A: insert A then B
    let mut g1 = WarpGraph::new();
    let a1 = insert_node(&mut g1, node_id(1), "demo.A", serde_json::json!({"k": "A"}));
    let b1 = insert_node(&mut g1, node_id(2), "demo.B", serde_json::json!({"k": "B"}));
    g1.edges.insert(WarpEdge {
        source: a1,
        target: b1,
        edge_type: "demo.edge".to_string(),
        attachment: None,
    });

    // Graph B: insert B then A
    let mut g2 = WarpGraph::new();
    let b2 = insert_node(&mut g2, node_id(2), "demo.B", serde_json::json!({"k": "B"}));
    let a2 = insert_node(&mut g2, node_id(1), "demo.A", serde_json::json!({"k": "A"}));
    g2.edges.insert(WarpEdge {
        source: a2,
        target: b2,
        edge_type: "demo.edge".to_string(),
        attachment: None,
    });

    let h1 = g1.compute_hash();
    let h2 = g2.compute_hash();

    assert_ne!(h1, Hash([0u8; 32]), "hash must not be placeholder-zero");
    assert_eq!(h1, h2, "same content must produce identical graph hash");
}

#[test]
fn graph_hash_changes_when_node_payload_changes() {
    let mut g1 = WarpGraph::new();
    insert_node(&mut g1, node_id(1), "demo.A", serde_json::json!({"k": "A"}));

    let mut g2 = WarpGraph::new();
    insert_node(
        &mut g2,
        node_id(1),
        "demo.A",
        serde_json::json!({"k": "A2"}),
    );

    let h1 = g1.compute_hash();
    let h2 = g2.compute_hash();

    assert_ne!(h1, h2, "payload changes must change the graph hash");
}
