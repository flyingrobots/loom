// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Deterministic ID Allocation
//!
//! SPEC-0005: Node IDs MUST be deterministic based on content, not insertion order.
//! This ensures antichain swaps (reordering independent operations) produce identical IDs.

use jitos_core::{canonical, Hash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deterministic node ID (content-addressed, not insertion-order-dependent)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Hash);

impl NodeId {
    /// Create NodeId from hash
    pub fn from_hash(hash: Hash) -> Self {
        Self(hash)
    }

    /// Get the underlying hash
    pub fn hash(&self) -> Hash {
        self.0
    }
}

/// Deterministic ID allocator for a single tick/batch
///
/// IDs are computed from:
/// - tick_hash: H(normalized operation set) - same for any ordering
/// - operation_hash: H(specific operation content)
/// - counter: per-operation sequence number
///
/// This ensures:
/// - Same operations → same tick_hash
/// - Different orderings → same IDs (antichain swap safety)
/// - Reproducible on replay
/// - Allocation order independence (counter is per-operation, not global)
#[derive(Debug, Clone)]
pub struct DeterministicIdAllocator {
    tick_hash: Hash,
    counters: HashMap<Hash, u64>,
}

impl DeterministicIdAllocator {
    /// Create allocator for a tick with normalized operations
    ///
    /// Operations MUST be sorted deterministically before hashing.
    /// Typically: sort by hash of operation content.
    pub fn new_for_tick(operations: &[Hash]) -> Self {
        // Sort operations to ensure deterministic tick hash
        let mut sorted = operations.to_vec();
        sorted.sort_by_key(|h| h.0);

        // Compute tick hash from normalized (sorted) operations
        let tick_hash = canonical::encode(&sorted)
            .map(|bytes| Hash(*blake3::hash(&bytes).as_bytes()))
            .unwrap_or(Hash([0u8; 32]));

        Self {
            tick_hash,
            counters: HashMap::new(),
        }
    }

    /// Allocate next node ID for an operation
    ///
    /// ID = H(tick_hash || operation_hash || counter)
    ///
    /// - tick_hash: ensures operations in same tick share prefix
    /// - operation_hash: distinguishes different operations
    /// - counter: per-operation counter (allocation order independent)
    pub fn alloc_node_id(&mut self, operation_hash: Hash) -> NodeId {
        // Get or initialize counter for this specific operation
        let counter = self.counters.entry(operation_hash).or_insert(0);

        let id_input = (&self.tick_hash, &operation_hash, *counter);

        let id_hash = canonical::encode(&id_input)
            .map(|bytes| Hash(*blake3::hash(&bytes).as_bytes()))
            .unwrap_or(Hash([0u8; 32]));

        *counter += 1;

        NodeId(id_hash)
    }

    /// Reset all counters (typically at start of new tick)
    pub fn reset_counter(&mut self) {
        self.counters.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_operations_same_tick_hash() {
        // Same operations in different orders should produce same tick hash
        let op1 = Hash([1u8; 32]);
        let op2 = Hash([2u8; 32]);
        let op3 = Hash([3u8; 32]);

        let alloc1 = DeterministicIdAllocator::new_for_tick(&[op1, op2, op3]);
        let alloc2 = DeterministicIdAllocator::new_for_tick(&[op3, op1, op2]);
        let alloc3 = DeterministicIdAllocator::new_for_tick(&[op2, op3, op1]);

        assert_eq!(
            alloc1.tick_hash, alloc2.tick_hash,
            "different orderings must produce same tick_hash"
        );
        assert_eq!(
            alloc2.tick_hash, alloc3.tick_hash,
            "different orderings must produce same tick_hash"
        );
    }

    #[test]
    fn test_antichain_swap_produces_identical_ids() {
        // THE KEY PROPERTY: Swapping independent operations produces identical node IDs
        let op1 = Hash([1u8; 32]);
        let op2 = Hash([2u8; 32]);
        let op3 = Hash([3u8; 32]);

        // Original order
        let mut alloc1 = DeterministicIdAllocator::new_for_tick(&[op1, op2, op3]);
        let id1_orig = alloc1.alloc_node_id(op1);
        let id2_orig = alloc1.alloc_node_id(op2);
        let id3_orig = alloc1.alloc_node_id(op3);

        // Swapped order
        let mut alloc2 = DeterministicIdAllocator::new_for_tick(&[op3, op1, op2]);
        let id1_swap = alloc2.alloc_node_id(op1);
        let id2_swap = alloc2.alloc_node_id(op2);
        let id3_swap = alloc2.alloc_node_id(op3);

        // IDs MUST be identical (same operation → same ID)
        assert_eq!(
            id1_orig, id1_swap,
            "op1 must get same ID regardless of tick ordering"
        );
        assert_eq!(
            id2_orig, id2_swap,
            "op2 must get same ID regardless of tick ordering"
        );
        assert_eq!(
            id3_orig, id3_swap,
            "op3 must get same ID regardless of tick ordering"
        );
    }

    #[test]
    fn test_allocation_order_independence() {
        // CRITICAL: Allocation call order must not affect IDs
        // This test would FAIL with global counter (Codex P1 bug)
        let op1 = Hash([1u8; 32]);
        let op2 = Hash([2u8; 32]);

        // Same tick, allocation order A then B
        let mut alloc1 = DeterministicIdAllocator::new_for_tick(&[op1, op2]);
        let id1_first = alloc1.alloc_node_id(op1); // op1 with counter=0
        let id2_first = alloc1.alloc_node_id(op2); // op2 with counter=0

        // Same tick, allocation order B then A
        let mut alloc2 = DeterministicIdAllocator::new_for_tick(&[op1, op2]);
        let id2_second = alloc2.alloc_node_id(op2); // op2 with counter=0
        let id1_second = alloc2.alloc_node_id(op1); // op1 with counter=0

        // IDs MUST be identical regardless of allocation call order
        assert_eq!(
            id1_first, id1_second,
            "op1 must get same ID regardless of allocation call order"
        );
        assert_eq!(
            id2_first, id2_second,
            "op2 must get same ID regardless of allocation call order"
        );
    }

    #[test]
    fn test_different_operations_different_ids() {
        // Different operations should get different IDs
        let op1 = Hash([1u8; 32]);
        let op2 = Hash([2u8; 32]);

        let mut alloc = DeterministicIdAllocator::new_for_tick(&[op1, op2]);

        let id1 = alloc.alloc_node_id(op1);
        let id2 = alloc.alloc_node_id(op2);

        assert_ne!(id1, id2, "different operations must get different IDs");
    }

    #[test]
    fn test_counter_produces_different_ids() {
        // Multiple allocations for same operation should get different IDs
        let op = Hash([1u8; 32]);
        let mut alloc = DeterministicIdAllocator::new_for_tick(&[op]);

        let id1 = alloc.alloc_node_id(op);
        let id2 = alloc.alloc_node_id(op);
        let id3 = alloc.alloc_node_id(op);

        assert_ne!(id1, id2, "counter must produce different IDs");
        assert_ne!(id2, id3, "counter must produce different IDs");
        assert_ne!(id1, id3, "counter must produce different IDs");
    }

    #[test]
    fn test_replay_reproducibility() {
        // Replaying same operations should produce identical IDs
        let ops = vec![Hash([1u8; 32]), Hash([2u8; 32]), Hash([3u8; 32])];

        let results1 = {
            let mut alloc = DeterministicIdAllocator::new_for_tick(&ops);
            vec![
                alloc.alloc_node_id(ops[0]),
                alloc.alloc_node_id(ops[1]),
                alloc.alloc_node_id(ops[2]),
            ]
        };

        let results2 = {
            let mut alloc = DeterministicIdAllocator::new_for_tick(&ops);
            vec![
                alloc.alloc_node_id(ops[0]),
                alloc.alloc_node_id(ops[1]),
                alloc.alloc_node_id(ops[2]),
            ]
        };

        assert_eq!(results1, results2, "replay must produce identical IDs");
    }

    #[test]
    fn test_antichain_swap_stress_1000_permutations() {
        // NEXT-MOVES.md requirement: "swap independent rewrites 1000 times → same graph hash every time"
        //
        // Scenario: 5 independent operations, swapped 1000 times in random orders
        // Expected: All permutations produce identical ID sets
        use std::collections::HashMap;

        let ops = vec![
            Hash([1u8; 32]),
            Hash([2u8; 32]),
            Hash([3u8; 32]),
            Hash([4u8; 32]),
            Hash([5u8; 32]),
        ];

        // Collect IDs from 1000 random permutations
        let mut all_results = Vec::new();

        for i in 0..1000 {
            // Create a deterministic permutation based on iteration number
            let mut permuted = ops.clone();
            // Simple shuffle based on iteration (deterministic for reproducibility)
            permuted.rotate_left(i % 5);
            if i % 2 == 0 {
                permuted.swap(0, 2);
            }
            if i % 3 == 0 {
                permuted.swap(1, 3);
            }

            // Allocate IDs for each operation
            let mut alloc = DeterministicIdAllocator::new_for_tick(&permuted);

            // Collect IDs in a map keyed by operation hash
            let mut id_map = HashMap::new();
            for op in &ops {
                let id = alloc.alloc_node_id(*op);
                id_map.insert(*op, id);
            }

            all_results.push(id_map);
        }

        // Verify ALL permutations produced IDENTICAL ID mappings
        let first_result = &all_results[0];
        for (i, result) in all_results.iter().enumerate().skip(1) {
            for op in &ops {
                assert_eq!(
                    first_result.get(op),
                    result.get(op),
                    "permutation {} produced different ID for operation {:?}",
                    i,
                    op
                );
            }
        }

        println!("✅ 1000 permutations → identical IDs (antichain swap property verified)");
    }
}
