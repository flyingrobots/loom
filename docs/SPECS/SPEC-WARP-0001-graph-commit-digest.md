# SPEC-WARP-0001: Graph Commit Digest (Canonical)

**Status:** Draft (Approval-Ready for Milestone 1)  
**Owner:** James Ross  
**Applies to:** kernel graph commitment / audit / replay verification  
**Related:** `docs/SPECS/SPEC-0001-canonical-encoding.md`, `docs/SPECS/SPEC-0005-deterministic-ids.md`, `docs/ROADMAP/M1-Kernel-Alpha/README.md`

---

## 0. Purpose

This spec defines the **canonical, from-first-principles** digest of a graph state:

- **GraphCommitDigest** is the “under oath” commitment to state.
- It is **structural** (one logical state → one byte sequence).
- It is **portable** (cross-machine, cross-runtime).
- It is allowed to be **slow** (O(N) over nodes/edges).

This digest is *not* an optimization structure. It is the ground-truth fingerprint used for:
- determinism proofs / replay verification
- audit checkpoints
- sealing / signing later (WAL / epochs)

---

## 1. Non-goals

- Defining a *fast incremental* structure (see SPEC-WARP-0002).
- Defining how NodeId / EdgeId are derived (only that they are stable and deterministic).
- Defining collapse semantics, persistence formats, or scheduling.

---

## 2. Terms and types

### 2.1 Hash primitive

- **Hash:** 32-byte BLAKE3 digest.
- **Encoding for APIs / text:** lowercase hex, length 64, no `0x` prefix.

### 2.2 Identity requirements

The digest depends on stable identifiers:

- **NodeId:** 32 bytes, stable and deterministic.
- **EdgeId:** 32 bytes, stable and deterministic.

This spec does **not** mandate whether IDs are content-addressed or allocator-derived, but it does require:

- IDs MUST NOT depend on non-deterministic data (UUID, wall clock, pointer addresses).
- Any ordering in this spec uses **lexicographic byte ordering** of these IDs.

### 2.3 Canonical byte encoding

All structures hashed by this spec MUST be encoded using the canonical encoding described in:

- `docs/SPECS/SPEC-0001-canonical-encoding.md`

In Rust terms, the reference encoding is `jitos_core::canonical::encode` and the reference hash helper is `jitos_core::canonical::hash_canonical`.

---

## 3. Canonical commitment shape

GraphCommitDigest is defined as hashing the canonical encoding of the following logical value:

```
GraphCommitV0 = {
  version: "graph-commit-v0",
  nodes: [NodeCommitV0...],  // sorted by node_id asc (bytes)
  edges: [EdgeCommitV0...],  // sorted by edge_id asc (bytes)
}

NodeCommitV0 = {
  node_id: NodeId,
  kind: String,
  payload_bytes: Bytes,
  attachment: Optional<Hash>, // optional, content-addressed reference (if used)
}

EdgeCommitV0 = {
  edge_id: EdgeId,
  from: NodeId,
  to: NodeId,
  kind: String,
  payload_bytes: Optional<Bytes>,
  attachment: Optional<Hash>, // optional, content-addressed reference (if used)
}
```

**Notes:**
- `payload_bytes` are opaque bytes. The kernel MUST NOT interpret them for hashing.
- If attachments are present as references (e.g., `Hash` of another WARP graph), that reference MUST be included.
- If attachments are not implemented yet, the field MUST be `null` (absent).

---

## 4. Ordering law (determinism-critical)

Implementations MUST obey:

- `GraphCommitV0.nodes` are sorted by `node_id` ascending (lexicographic bytes).
- `GraphCommitV0.edges` are sorted by `edge_id` ascending (lexicographic bytes).

`HashMap` / `SlotMap` iteration order MUST NOT affect the commitment.

---

## 5. Digest definition

Let `canonical_encode(x)` be the canonical byte encoding from SPEC-0001.

Then:

```
GraphCommitDigestV0(graph) = blake3( canonical_encode(GraphCommitV0(graph)) )
```

### 5.1 Streaming / fold guidance (recommended, not required)

For performance, implementations MAY compute the digest using a streaming hasher without constructing a single monolithic byte vector, as long as the bytes fed to the hasher are **exactly** the canonical encoding of `GraphCommitV0`.

This is an implementation detail; the semantic definition above is authoritative.

---

## 6. Empty graph

For an empty graph:

- `nodes = []`
- `edges = []`

The digest is the hash of the canonical encoding of `GraphCommitV0` with empty lists.

---

## 7. Test requirements (golden vectors)

Implementations MUST include behavioral tests that prove:

1. **Insertion order invariance:** inserting identical sets of nodes/edges in different orders produces identical GraphCommitDigest.
2. **Cross-runtime stability:** canonical encoding of the same logical `GraphCommitV0` produces identical bytes and therefore identical digest.

Golden vectors SHOULD include:
- a small fixed graph (2–3 nodes, 1–2 edges) with known digest hex
- at least one node/edge with non-empty `payload_bytes`

---

## 8. Relationship to fast incremental hashing

GraphCommitDigest is the **authoritative** commitment to state.

Any incremental structure (Merkle roots, sparse trees, tries, indexes) MUST be treated as:
- an acceleration structure, and
- a proof/caching aid,
not as semantics.

See: `docs/SPECS/SPEC-WARP-0002-incremental-graph-hash.md`.

