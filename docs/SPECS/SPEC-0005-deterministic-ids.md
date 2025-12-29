# SPEC-0005: Deterministic ID Allocation

**Status:** Approved (v1.0)
**Related:** NEXT-MOVES.md Phase 0.5.6, SPEC-0001 Canonical Encoding
**Estimated Effort:** 2 hours

---

## Problem Statement

Using insertion-order-dependent IDs (like SlotMap keys) breaks determinism. If the same set of independent operations executes in different orders, they get different IDs, causing graph hashes to diverge even though the semantic content is identical.

**Without this:** Antichain swaps (reordering independent operations) produce different node IDs → different graph hashes → replay instability.
**With this:** Node IDs are content-addressed - same operation always gets same ID, regardless of execution order.

---

## User Story

**As a** graph/scheduler developer
**I want** node IDs determined by operation content, not insertion order
**So that** antichain swaps produce identical graph states and hashes

---

## Requirements

### Functional Requirements

#### Core Principle: IDs from Content, Not Order

**Node IDs in JITOS are deterministic functions of operation content.**

- **Content-addressed:** ID = H(tick_hash, operation_hash, counter)
- **Order-independent:** Same operations → same tick_hash (normalized)
- **Antichain-safe:** Swapping independent ops → identical IDs
- **Replay-safe:** Replaying event log → identical IDs at every step

#### NodeId Type

Deterministic, content-addressed node identifier.

```rust
pub struct NodeId(pub Hash);
```

- Wraps a Hash (BLAKE3 digest)
- Not insertion-order-dependent
- Can be compared, hashed, serialized

#### DeterministicIdAllocator

Allocates IDs for operations within a single tick/batch.

**Fields:**
- `tick_hash: Hash` - H(sorted operations) - same for all orderings
- `counter: u64` - sequence number within this allocator

**Invariants:**
1. tick_hash is computed from **sorted** operation hashes (order-independent)
2. Same set of operations → same tick_hash (commutative)
3. Each allocation increments counter → distinct IDs even for same operation

#### ID Computation Formula

```
tick_hash = H(sort(operations))
node_id = H(tick_hash || operation_hash || counter)
```

**Key properties:**
- **Tick normalization**: Sorting ensures same tick_hash regardless of input order
- **Operation distinction**: operation_hash separates different operations
- **Multiplicity**: counter handles multiple nodes from same operation

---

## API

### NodeId

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Hash);

impl NodeId {
    pub fn from_hash(hash: Hash) -> Self;
    pub fn hash(&self) -> Hash;
}
```

### DeterministicIdAllocator

```rust
pub struct DeterministicIdAllocator {
    tick_hash: Hash,
    counter: u64,
}

impl DeterministicIdAllocator {
    /// Create allocator for a tick with normalized operations
    /// Operations MUST be sorted before hashing to ensure determinism
    pub fn new_for_tick(operations: &[Hash]) -> Self;

    /// Allocate next node ID for an operation
    /// ID = H(tick_hash || operation_hash || counter)
    pub fn alloc_node_id(&mut self, operation_hash: Hash) -> NodeId;

    /// Reset counter (typically at start of new tick)
    pub fn reset_counter(&mut self);
}
```

### Usage Example

```rust
use jitos_graph::{DeterministicIdAllocator, NodeId};
use jitos_core::Hash;

// Operations in a tick (from scheduler)
let op1 = Hash([1u8; 32]);
let op2 = Hash([2u8; 32]);
let op3 = Hash([3u8; 32]);

// Create allocator (operations get sorted internally)
let mut alloc = DeterministicIdAllocator::new_for_tick(&[op1, op2, op3]);

// Allocate IDs for each operation
let node1 = alloc.alloc_node_id(op1);
let node2 = alloc.alloc_node_id(op2);
let node3 = alloc.alloc_node_id(op3);

// Different order → SAME IDs
let mut alloc2 = DeterministicIdAllocator::new_for_tick(&[op3, op1, op2]);
assert_eq!(alloc2.alloc_node_id(op1), node1); // ✅ Order-independent
assert_eq!(alloc2.alloc_node_id(op2), node2);
assert_eq!(alloc2.alloc_node_id(op3), node3);
```

---

## Test Plan

### Unit Tests

1. **Same Operations → Same Tick Hash**
   - Given: Same operations in different orders
   - When: Creating allocators
   - Then: tick_hash is identical

2. **Antichain Swap → Identical IDs**
   - Given: Operations in order [A, B, C]
   - When: Swapped to [C, A, B]
   - Then: Each operation gets same ID

3. **Different Operations → Different IDs**
   - Given: Two distinct operations
   - When: Allocating IDs
   - Then: IDs are different

4. **Counter → Distinct IDs**
   - Given: Same operation allocated multiple times
   - When: Incrementing counter
   - Then: Each allocation gets unique ID

5. **Replay Reproducibility**
   - Given: Same operation sequence
   - When: Replaying twice
   - Then: Identical IDs both times

### Stress Test

6. **1000 Permutations → Same IDs**
   - Given: 5 operations
   - When: Swapped 1000 times in different orders
   - Then: ALL permutations produce identical ID sets
   - **This is the NEXT-MOVES.md acceptance criterion**

---

## Acceptance Criteria

- [x] NodeId type is content-addressed (Hash-based)
- [x] DeterministicIdAllocator normalizes operations (sorts by hash)
- [x] Same operations → same tick_hash (commutative)
- [x] Antichain swap → identical IDs (order-independent)
- [x] Replay → identical IDs (reproducible)
- [x] **1000 permutations stress test passes**

---

## Design Decisions

### 1. Why sort operations by hash instead of original content?

Hashes are already normalized, fixed-size, and comparable. Sorting by hash is O(n log n) and avoids needing to define ordering for arbitrary operation types.

### 2. Why include counter in ID formula?

Some operations create multiple nodes (e.g., graph rewrite inserts 3 nodes). Counter ensures each gets a unique ID while maintaining determinism (same operation + same counter → same ID).

### 3. Why normalize at tick level, not globally?

Tick-level normalization allows parallelism within a tick while maintaining cross-tick determinism. Global normalization would require total ordering across all time.

### 4. Why not just use operation_hash directly as NodeId?

Multiple nodes from same operation would collide. Counter prevents this while maintaining determinism.

---

## Implementation Notes

### Files

- `crates/jitos-graph/src/ids.rs` - DeterministicIdAllocator implementation
- `crates/jitos-graph/src/lib.rs` - Public exports

### Dependencies

- `jitos-core` - Hash, canonical encoding
- `blake3` - Cryptographic hashing
- `serde` - Serialization

### Test Results

```
running 6 tests
test ids::tests::test_same_operations_same_tick_hash ... ok
test ids::tests::test_antichain_swap_produces_identical_ids ... ok
test ids::tests::test_different_operations_different_ids ... ok
test ids::tests::test_counter_produces_different_ids ... ok
test ids::tests::test_replay_reproducibility ... ok
test ids::tests::test_antichain_swap_stress_1000_permutations ... ok

test result: ok. 6 passed; 0 failed
```

---

## Future Work

### Phase 1 Integration

1. **WarpGraph Integration**
   - Replace SlotMap NodeKey with NodeId
   - Update WarpNode to use deterministic IDs
   - Implement deterministic graph hashing

2. **Scheduler Integration**
   - Create DeterministicIdAllocator at tick start
   - Pass operation hashes from SLAPs
   - Use allocated NodeIds in graph operations

3. **Graph Hash Stability**
   - Implement `compute_hash()` using sorted NodeIds
   - Test: same graph content → same hash
   - Golden test: 1000 replays → identical hash

### Phase 2 Enhancements

4. **Optimizations**
   - Cache tick_hash when operations don't change
   - Pre-allocate ID pools for known tick sizes
   - Benchmark: <1μs per ID allocation

---

## References

- SPEC-0001: Canonical Encoding (used for tick_hash computation)
- NEXT-MOVES.md: Phase 0.5.6 requirements
- THEORY.md: Paper II (Deterministic Execution Semantics)
