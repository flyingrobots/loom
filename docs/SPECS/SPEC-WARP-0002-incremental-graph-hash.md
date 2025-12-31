# SPEC-WARP-0002: Incremental Graph Hash (Acceleration Structure)

**Status:** Draft (Design-Ready; not required for Milestone 1 gate)  
**Owner:** James Ross  
**Applies to:** fast tick updates, slice proofs, viewer caching/diffing  
**Related:** `docs/SPECS/SPEC-WARP-0001-graph-commit-digest.md`

---

## 0. Purpose

This spec defines an **incremental** hash structure for graph state.

Key design rule:

- **This is an optimization, not semantics.**
- It MUST NOT replace the canonical commitment defined in SPEC-WARP-0001.

The goal is to support:
- fast updates when a small part of the graph changes
- efficient inclusion proofs / slicing
- viewer-friendly diff/caching surfaces

---

## 1. Non-goals

- Replacing GraphCommitDigest.
- Defining persistence/WAL formats.
- Defining collapse semantics.

---

## 2. Terms and types

- **Hash:** 32-byte BLAKE3 digest.
- **NodeId / EdgeId:** stable 32-byte identifiers (see SPEC-WARP-0001).

---

## 3. Leaf hashes (what gets committed)

### 3.1 Node leaf hash

Define a node leaf hash as:

```
NodeLeafHashV0(node) = blake3( canonical_encode({
  version: "node-leaf-v0",
  node_id: NodeId,
  kind: String,
  payload_bytes: Bytes,
  attachment: Optional<Hash>,
}) )
```

### 3.2 Edge leaf hash

Define an edge leaf hash as:

```
EdgeLeafHashV0(edge) = blake3( canonical_encode({
  version: "edge-leaf-v0",
  edge_id: EdgeId,
  from: NodeId,
  to: NodeId,
  kind: String,
  payload_bytes: Optional<Bytes>,
  attachment: Optional<Hash>,
}) )
```

These leaf hashes are the unit that:
- the incremental structure commits to, and
- inclusion proofs are built around.

---

## 4. Sparse Merkle Tree (SMT) construction

This spec uses a **Sparse Merkle Tree** keyed by 256-bit keys.

Why SMT:
- deterministic (fixed height)
- updates are O(256) (bounded, predictable)
- supports inclusion proofs naturally
- does not require shifting indices when items are inserted/removed

### 4.1 Key space separation

To avoid NodeId / EdgeId collisions in a single tree, define distinct key derivations:

```
NodeKeyV0(node_id) = blake3( "JITOS:SMT:node:v0" || node_id ).as_bytes()
EdgeKeyV0(edge_id) = blake3( "JITOS:SMT:edge:v0" || edge_id ).as_bytes()
```

Each key is 32 bytes.

### 4.2 Leaf value

- For nodes: `value = NodeLeafHashV0(node)`
- For edges: `value = EdgeLeafHashV0(edge)`

### 4.3 Node hashing

Define SMT hashing with domain separation:

Leaf hash:
```
H_leaf(depth, key, value) = blake3( canonical_encode({
  version: "smt-leaf-v0",
  depth: u16,        // always 256 for the bottom leaf position
  key: Bytes32,
  value: Hash,
}) )
```

Inner hash:
```
H_inner(depth, left, right) = blake3( canonical_encode({
  version: "smt-inner-v0",
  depth: u16,        // 0..=255
  left: Hash,
  right: Hash,
}) )
```

Empty hash:

An SMT requires a deterministic “empty hash” for each depth. Define:

```
EMPTY[256] = blake3( canonical_encode({ version: "smt-empty-v0", depth: 256 }) )
EMPTY[d]   = H_inner(d, EMPTY[d+1], EMPTY[d+1])    for d = 255..0
```

### 4.4 Root hash

Given a set of (key → value) leaves, the SMT root is computed by:
- setting leaves at depth 256 to `H_leaf(256, key, value)` for present keys
- using `EMPTY[depth]` for all absent subtrees
- computing parent hashes with `H_inner` up to depth 0

This defines a unique, deterministic root for a given set of leaves.

---

## 5. GraphMerkleRoot definition

Maintain two SMTs:
- `NodeSMT` keyed by NodeKeyV0(NodeId), values NodeLeafHashV0
- `EdgeSMT` keyed by EdgeKeyV0(EdgeId), values EdgeLeafHashV0

Then define:

```
GraphMerkleRootV0(graph) = blake3( canonical_encode({
  version: "graph-merkle-root-v0",
  node_root: NodeSMT.root,
  edge_root: EdgeSMT.root,
}) )
```

---

## 6. Update complexity (incremental property)

When a single node leaf changes:
- update its leaf in `NodeSMT`
- recompute hashes along its path to the root (O(256))

Likewise for edges in `EdgeSMT`.

GraphMerkleRootV0 updates by recomputing the combined root structure (constant work once the two SMT roots are known).

---

## 7. Relationship to GraphCommitDigest (refinement rule)

GraphMerkleRoot MUST be treated as an acceleration structure.

The canonical commitment is still:
- GraphCommitDigestV0 (SPEC-WARP-0001)

The required refinement property is:

- For any graph state `G`, both the GraphCommitDigest and GraphMerkleRoot are computed from the same logical set of node and edge records.
- A verifier MUST be able to recompute GraphCommitDigest from the underlying node/edge records, independent of the SMT.
- The SMT is allowed to accelerate proofs (“this leaf is part of the committed set”) but MUST NOT change the state semantics.

---

## 8. Test requirements (future milestone)

Implementations MUST include:
- insertion-order invariance for GraphMerkleRoot (set semantics)
- stable empty-root definitions across platforms
- inclusion proof verification for a node leaf and an edge leaf
- update locality test: changing one leaf changes only O(256) internal nodes (structural test)

