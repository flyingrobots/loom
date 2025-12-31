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
    payload_bytes: Vec<u8>,
) -> NodeKey {
    graph.nodes.insert(WarpNode {
        id,
        node_type: node_type.to_string(),
        payload_bytes,
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
    let a1 = insert_node(&mut g1, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());
    let b1 = insert_node(&mut g1, node_id(2), "demo.B", br#"{"k":"B"}"#.to_vec());
    g1.edges.insert(WarpEdge {
        source: a1,
        target: b1,
        edge_type: "demo.edge".to_string(),
        payload_bytes: None,
        attachment: None,
    });

    // Graph B: insert B then A
    let mut g2 = WarpGraph::new();
    let b2 = insert_node(&mut g2, node_id(2), "demo.B", br#"{"k":"B"}"#.to_vec());
    let a2 = insert_node(&mut g2, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());
    g2.edges.insert(WarpEdge {
        source: a2,
        target: b2,
        edge_type: "demo.edge".to_string(),
        payload_bytes: None,
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
    insert_node(&mut g1, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());

    let mut g2 = WarpGraph::new();
    insert_node(&mut g2, node_id(1), "demo.A", br#"{"k":"A2"}"#.to_vec());

    let h1 = g1.compute_hash();
    let h2 = g2.compute_hash();

    assert_ne!(h1, h2, "payload changes must change the graph hash");
}

#[test]
fn graph_hash_depends_on_payload_bytes_not_json_semantics() {
    // The WARP graph commit digest treats payload as opaque bytes (SPEC-WARP-0001).
    // If two payloads are "JSON-equivalent" but differ at the byte level, the digest MUST differ.
    //
    // This test protects against accidental re-introduction of JSON canonicalization in hashing.
    let mut g1 = WarpGraph::new();
    insert_node(&mut g1, node_id(1), "demo.A", br#"{"a":1,"b":2}"#.to_vec());

    let mut g2 = WarpGraph::new();
    insert_node(&mut g2, node_id(1), "demo.A", br#"{"b":2,"a":1}"#.to_vec());

    let h1 = g1.compute_hash();
    let h2 = g2.compute_hash();

    assert_ne!(
        h1, h2,
        "hash must treat payload as bytes (not canonicalized JSON semantics)"
    );
}

#[test]
fn edge_payload_affects_graph_hash() {
    let mut g1 = WarpGraph::new();
    let a1 = insert_node(&mut g1, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());
    let b1 = insert_node(&mut g1, node_id(2), "demo.B", br#"{"k":"B"}"#.to_vec());
    g1.edges.insert(WarpEdge {
        source: a1,
        target: b1,
        edge_type: "demo.edge".to_string(),
        payload_bytes: Some(vec![1, 2, 3]),
        attachment: None,
    });

    let mut g2 = WarpGraph::new();
    let a2 = insert_node(&mut g2, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());
    let b2 = insert_node(&mut g2, node_id(2), "demo.B", br#"{"k":"B"}"#.to_vec());
    g2.edges.insert(WarpEdge {
        source: a2,
        target: b2,
        edge_type: "demo.edge".to_string(),
        payload_bytes: None,
        attachment: None,
    });

    assert_ne!(
        g1.compute_hash(),
        g2.compute_hash(),
        "edge payload must affect graph commit digest"
    );
}

#[test]
fn edge_payload_none_vs_empty_are_distinct() {
    let mut g1 = WarpGraph::new();
    let a1 = insert_node(&mut g1, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());
    let b1 = insert_node(&mut g1, node_id(2), "demo.B", br#"{"k":"B"}"#.to_vec());
    g1.edges.insert(WarpEdge {
        source: a1,
        target: b1,
        edge_type: "demo.edge".to_string(),
        payload_bytes: None,
        attachment: None,
    });

    let mut g2 = WarpGraph::new();
    let a2 = insert_node(&mut g2, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());
    let b2 = insert_node(&mut g2, node_id(2), "demo.B", br#"{"k":"B"}"#.to_vec());
    g2.edges.insert(WarpEdge {
        source: a2,
        target: b2,
        edge_type: "demo.edge".to_string(),
        payload_bytes: Some(vec![]),
        attachment: None,
    });

    assert_ne!(
        g1.compute_hash(),
        g2.compute_hash(),
        "None vs Some(empty) edge payload must be identity-distinct"
    );
}

#[test]
fn edge_payload_byte_level_sensitivity() {
    let mut g1 = WarpGraph::new();
    let a1 = insert_node(&mut g1, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());
    let b1 = insert_node(&mut g1, node_id(2), "demo.B", br#"{"k":"B"}"#.to_vec());
    g1.edges.insert(WarpEdge {
        source: a1,
        target: b1,
        edge_type: "demo.edge".to_string(),
        payload_bytes: Some(br#"{"a":1,"b":2}"#.to_vec()),
        attachment: None,
    });

    let mut g2 = WarpGraph::new();
    let a2 = insert_node(&mut g2, node_id(1), "demo.A", br#"{"k":"A"}"#.to_vec());
    let b2 = insert_node(&mut g2, node_id(2), "demo.B", br#"{"k":"B"}"#.to_vec());
    g2.edges.insert(WarpEdge {
        source: a2,
        target: b2,
        edge_type: "demo.edge".to_string(),
        payload_bytes: Some(br#"{"b":2,"a":1}"#.to_vec()),
        attachment: None,
    });

    assert_ne!(
        g1.compute_hash(),
        g2.compute_hash(),
        "edge payloads must be treated as opaque bytes"
    );
}
