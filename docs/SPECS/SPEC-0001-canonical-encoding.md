# SPEC-0001: Canonical Encoding Standard

**Status:** Approved - Implementation Complete
**Related:** NEXT-MOVES.md Phase 0.5.1
**Estimated Effort:** 2-3 hours

---

## Problem Statement

BLAKE3 hashing over non-canonical serialization causes hash divergence. If two runtimes serialize the "same" logical structure into different bytes (field order, map order, varint encoding, NaN representation), replay hashes will differ and determinism breaks.

**Without this:** Cross-browser/cross-platform replay fails golden tests.
**With this:** Same logical state → identical bytes → identical hashes everywhere.

---

## User Story

**As a** JITOS kernel developer
**I want** a canonical encoding standard
**So that** replay produces identical hashes across all platforms/runtimes

---

## Requirements

### Functional Requirements

#### Core Principle: Structural Determinism

**Canonical encoding prioritizes structural determinism over space efficiency.**

- **Structural determinism:** There is nothing to decide. One logical value → one byte sequence.
- **Procedural determinism:** If everyone runs the same algorithm, they'll agree (fragile across languages/runtimes).

JITOS targets structural determinism. Float width selection is not permitted in canonical contexts.

#### Encoding Rules

1. **Choose Encoding Format:** Canonical CBOR (RFC 8949) for all ledger/events/archives
2. **Non-Optional Rule:** All ledger events, receipts, snapshots, deltas, and archives **MUST** use canonical encoding. Non-canonical encoders (including `serde_json`) are **FORBIDDEN** in `jitos-provenance`. No "debug mode" bypass. No "temporary JSON". If something isn't canonical, it doesn't get a hash.
3. **Strict Ordering:** Map entries sorted by the lexicographic ordering of the canonical CBOR encoding of their keys, per RFC 8949. This ensures correct ordering for non-string keys (byte strings, integers).
4. **Definite Lengths:** No streaming/indefinite-length encoding
5. **Float Encoding:**
   - **Integral values:** If `f.fract() == 0.0` and fits in i128, encode as integer
   - **Non-integral floats:** Always encode as float64 (`0xfb` major type 7, ai=27)
   - **Forbidden:** float16 (`0xf9`) and float32 (`0xfa`) in canonical encoding
   - **Rationale:** Cross-language compatibility (JS, WASM, Python, SQL all use f64 natively). Eliminates width-selection heuristics.
6. **Float Canonicalization:**
   - NaN → IEEE-754 quiet NaN with payload=0, bit pattern: `0x7FF8_0000_0000_0000`
   - ±0 normalized to +0
   - Subnormals → 0.0
   - ±∞ preserved as-is
7. **No Duplicates:** Reject duplicate map keys
8. **Test Vectors:** 100+ edge cases covering all data types

#### Optional Optimization Layer

Echo's "smallest width that round-trips" logic (f16/f32/f64 selection) may be used in:
- Non-canonical storage layers
- Transport compression
- Snapshot packing
- Explicitly marked optimization paths

But **never** in canonical event hashing, receipts, or replay bytes.

### Non-Functional Requirements

1. **Performance:** Encoding/decoding overhead <10% vs non-canonical
2. **Portability:** Works in WASM, Node, Deno, browsers
3. **Auditability:** Clear documentation of what breaks determinism

---

## Acceptance Criteria

### AC1: Canonical Module Exists
- [ ] File `jitos-core/src/canonical.rs` exists
- [ ] Exports `encode()` and `decode()` functions
- [ ] Uses `ciborium` or equivalent CBOR library with canonical mode

### AC2: Test Vectors Pass
- [ ] Test suite with 100+ edge cases
- [ ] Tests cover: empty maps/arrays, nested structures, floats (NaN, ±0, ±∞), large integers, byte strings
- [ ] All platforms produce identical bytes for each test case

### AC3: Round-Trip Compliance
- [ ] For all test vectors: `serialize(deserialize(bytes)) == bytes`
- [ ] Rejected: duplicate keys, non-canonical length encoding, unsorted maps

### AC4: Documentation
- [ ] "What Breaks Determinism" guide in docs/
- [ ] Explains why `HashMap.iter()` is forbidden
- [ ] Explains why `f64::NAN != f64::NAN` matters

---

## Test Plan (Behavioral Black-Box)

### Test Suite: `jitos-core/tests/canonical_encoding_tests.rs`

#### Test 1: Empty Structures
```rust
#[test]
fn test_empty_map_canonical() {
    let map: BTreeMap<String, i32> = BTreeMap::new();
    let bytes = canonical::encode(&map).unwrap();

    // Should match CBOR empty map: 0xA0
    assert_eq!(bytes, vec![0xA0]);

    // Round-trip
    let decoded: BTreeMap<String, i32> = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, map);
}
```

#### Test 2: Key Ordering
```rust
#[test]
fn test_map_key_ordering() {
    let mut map = BTreeMap::new();
    map.insert("zebra", 1);
    map.insert("apple", 2);

    let bytes = canonical::encode(&map).unwrap();

    // Keys must be sorted: "apple" before "zebra"
    // Decode and re-encode multiple times
    for _ in 0..100 {
        let decoded: BTreeMap<String, i32> = canonical::decode(&bytes).unwrap();
        let re_encoded = canonical::encode(&decoded).unwrap();
        assert_eq!(re_encoded, bytes);
    }
}
```

#### Test 3: Float Canonicalization
```rust
#[test]
fn test_nan_canonicalization() {
    let values = vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -0.0];

    for val in values {
        let bytes1 = canonical::encode(&val).unwrap();
        let bytes2 = canonical::encode(&val).unwrap();
        assert_eq!(bytes1, bytes2, "Same float should encode identically");
    }

    // NaN MUST encode to IEEE-754 quiet NaN: 0x7FF8_0000_0000_0000
    let nan_bytes = canonical::encode(&f64::NAN).unwrap();
    assert_eq!(nan_bytes.len(), 9); // CBOR float64 tag (0xFB) + 8 bytes

    // Extract the float64 payload (skip CBOR tag byte)
    let float_bytes = &nan_bytes[1..9];
    let expected_nan: [u8; 8] = 0x7FF8_0000_0000_0000u64.to_be_bytes();
    assert_eq!(float_bytes, expected_nan, "NaN must use canonical bit pattern");

    // ±0 must normalize to +0
    let pos_zero_bytes = canonical::encode(&0.0f64).unwrap();
    let neg_zero_bytes = canonical::encode(&(-0.0f64)).unwrap();
    assert_eq!(pos_zero_bytes, neg_zero_bytes, "±0 must normalize to +0");
}
```

#### Test 4: Rejection Tests
```rust
#[test]
fn test_reject_duplicate_keys() {
    // Manually craft CBOR with duplicate keys
    let bad_cbor = vec![0xA2, 0x61, 0x61, 0x01, 0x61, 0x61, 0x02]; // {a: 1, a: 2}

    let result: Result<BTreeMap<String, i32>, _> = canonical::decode(&bad_cbor);
    assert!(result.is_err(), "Should reject duplicate keys");
}
```

#### Test 5: Cross-Platform Consistency
```rust
#[test]
fn test_cross_platform_vectors() {
    // Load test vectors from JSON file
    let vectors: Vec<TestVector> = load_test_vectors();

    for vector in vectors {
        let encoded = canonical::encode(&vector.value).unwrap();
        assert_eq!(encoded, vector.expected_bytes,
            "Platform divergence detected for: {:?}", vector.name);
    }
}
```

---

## Implementation Tasks (<3 hour chunks)

### Task 1: Setup (30 min)
- [ ] Add `ciborium` dependency to `jitos-core/Cargo.toml`
- [ ] Create `jitos-core/src/canonical.rs`
- [ ] Create `jitos-core/tests/canonical_encoding_tests.rs`

### Task 2: Core Implementation (1 hour)
- [ ] Implement `encode<T: Serialize>(value: &T) -> Result<Vec<u8>>`
- [ ] Implement `decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>`
- [ ] Add canonical mode enforcement (sorted keys, definite lengths)

### Task 3: Float Handling (30 min)
- [ ] Add `CanonicalFloat` wrapper type
- [ ] Implement `Serialize` for `CanonicalFloat` with NaN normalization
- [ ] Test ±0, NaN, ±∞ cases

### Task 4: Test Vectors (1 hour)
- [ ] Generate 100+ test vectors covering edge cases
- [ ] Save vectors to `jitos-core/tests/vectors/canonical.json`
- [ ] Implement cross-platform consistency tests

### Task 5: Safety Helper (15 min)
- [ ] Add `hash_canonical<T: Serialize>(value: &T) -> Result<Hash>` helper
- [ ] Implementation: `blake3::hash(&canonical::encode(value)?)`
- [ ] Document: "Use this instead of manual hashing to prevent accidental non-canonical hashing"
- [ ] Add to exports in `jitos-core/src/lib.rs`

---

## Risks & Mitigation

### Risk 1: CBOR Library Doesn't Support Canonical Mode
**Mitigation:** Use `ciborium` which supports deterministic encoding, or implement canonical wrapper.

### Risk 2: Platform-Specific Float Behavior
**Mitigation:** Explicit bit-level control over NaN representation via wrapper type.

### Risk 3: Performance Overhead
**Mitigation:** Benchmark early. If >10% overhead, consider custom binary format with documented rules.

---

## Open Questions

1. **Should we support non-CBOR formats for debugging?**
   - Recommendation: Provide optional JSON exporter for human inspection, but CBOR is canonical.

2. **How do we handle backward compatibility if canonical rules change?**
   - Recommendation: Version the canonical format in `jitos-core` version. Breaking changes require major version bump.

---

## Success Metrics

- [ ] All 100+ test vectors pass on: Chrome WASM, Firefox WASM, Safari WASM, Node.js, Deno
- [ ] Round-trip compliance: `encode(decode(bytes)) == bytes` for all vectors
- [ ] Documentation guide prevents common determinism pitfalls

---

**AWAITING HUMAN APPROVAL BEFORE PROCEEDING**
