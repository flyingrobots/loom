//! Behavioral black-box tests for canonical encoding.
//!
//! These tests validate the behavioral requirements from SPEC-0001,
//! not implementation details.

use jitos_core::canonical::{self, CanonicalError};
use std::collections::BTreeMap;

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

#[test]
fn test_empty_array_canonical() {
    let arr: Vec<i32> = Vec::new();
    let bytes = canonical::encode(&arr).unwrap();

    // Should match CBOR empty array: 0x80
    assert_eq!(bytes, vec![0x80]);

    // Round-trip
    let decoded: Vec<i32> = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, arr);
}

#[test]
fn test_map_key_ordering() {
    let mut map = BTreeMap::new();
    map.insert("zebra".to_string(), 1);
    map.insert("apple".to_string(), 2);

    let bytes = canonical::encode(&map).unwrap();

    // Keys must be sorted: "apple" before "zebra"
    // Decode and re-encode multiple times to ensure stability
    for _ in 0..100 {
        let decoded: BTreeMap<String, i32> = canonical::decode(&bytes).unwrap();
        let re_encoded = canonical::encode(&decoded).unwrap();
        assert_eq!(
            re_encoded, bytes,
            "Re-encoding must produce identical bytes"
        );
    }
}

#[test]
fn test_map_key_ordering_with_insertion_order() {
    // Create two maps with different insertion orders
    let mut map1 = BTreeMap::new();
    map1.insert("zebra", 1);
    map1.insert("apple", 2);
    map1.insert("mango", 3);

    let mut map2 = BTreeMap::new();
    map2.insert("apple", 2);
    map2.insert("mango", 3);
    map2.insert("zebra", 1);

    let bytes1 = canonical::encode(&map1).unwrap();
    let bytes2 = canonical::encode(&map2).unwrap();

    // Must produce identical bytes regardless of insertion order
    assert_eq!(bytes1, bytes2, "Insertion order must not affect encoding");
}

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
    assert_eq!(
        float_bytes, expected_nan,
        "NaN must use canonical bit pattern"
    );

    // Verify decoder recognizes canonical NaN
    let decoded_nan: f64 = canonical::decode(&nan_bytes).unwrap();
    assert!(decoded_nan.is_nan(), "Decoded value should be NaN");

    // ±0 must normalize to +0
    let pos_zero_bytes = canonical::encode(&0.0f64).unwrap();
    let neg_zero_bytes = canonical::encode(&(-0.0f64)).unwrap();
    assert_eq!(pos_zero_bytes, neg_zero_bytes, "±0 must normalize to +0");
}

#[test]
fn test_infinity_preservation() {
    let pos_inf = f64::INFINITY;
    let neg_inf = f64::NEG_INFINITY;

    let pos_bytes = canonical::encode(&pos_inf).unwrap();
    let neg_bytes = canonical::encode(&neg_inf).unwrap();

    // ±∞ must be preserved as-is (different encodings)
    assert_ne!(pos_bytes, neg_bytes, "±∞ must be distinct");

    // Round-trip
    let decoded_pos: f64 = canonical::decode(&pos_bytes).unwrap();
    let decoded_neg: f64 = canonical::decode(&neg_bytes).unwrap();

    assert!(decoded_pos.is_infinite() && decoded_pos.is_sign_positive());
    assert!(decoded_neg.is_infinite() && decoded_neg.is_sign_negative());
}

#[test]
fn test_reject_duplicate_keys() {
    // Manually craft CBOR with duplicate keys: {a: 1, a: 2}
    // CBOR map with 2 entries: 0xA2
    // Key "a": 0x61 0x61
    // Value 1: 0x01
    // Key "a" again: 0x61 0x61
    // Value 2: 0x02
    let bad_cbor = vec![0xA2, 0x61, 0x61, 0x01, 0x61, 0x61, 0x02];

    let result: Result<BTreeMap<String, i32>, _> = canonical::decode(&bad_cbor);
    assert!(result.is_err(), "Should reject duplicate keys");

    match result {
        Err(CanonicalError::DuplicateKey) => {
            // Expected error type
        }
        Err(e) => panic!("Expected DuplicateKey error, got: {:?}", e),
        Ok(_) => panic!("Should have rejected duplicate keys"),
    }
}

#[test]
fn test_simple_types_roundtrip() {
    // u8
    let val = 42u8;
    let bytes = canonical::encode(&val).unwrap();
    let decoded: u8 = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, val);

    // u64
    let val = 1000u64;
    let bytes = canonical::encode(&val).unwrap();
    let decoded: u64 = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, val);

    // i32
    let val = -42i32;
    let bytes = canonical::encode(&val).unwrap();
    let decoded: i32 = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, val);

    // f64
    let val = 3.14f64;
    let bytes = canonical::encode(&val).unwrap();
    let decoded: f64 = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, val);

    // String
    let val = "hello world";
    let bytes = canonical::encode(&val).unwrap();
    let decoded: String = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, val);

    // bool
    let val = true;
    let bytes = canonical::encode(&val).unwrap();
    let decoded: bool = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, val);
}

#[test]
fn test_nested_structures() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Inner {
        x: i32,
        y: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Outer {
        inner: Inner,
        values: Vec<i32>,
    }

    let value = Outer {
        inner: Inner {
            x: 42,
            y: "test".to_string(),
        },
        values: vec![1, 2, 3],
    };

    let bytes = canonical::encode(&value).unwrap();
    let decoded: Outer = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, value);

    // Re-encode and verify bytes are identical
    let re_encoded = canonical::encode(&decoded).unwrap();
    assert_eq!(re_encoded, bytes);
}

#[test]
fn test_large_integers() {
    // Test boundary values for unsigned integers
    let test_values = vec![
        u64::MIN,
        u64::MAX,
        0u64,
        1u64,
        255u64,
        256u64,
        65535u64,
        65536u64,
    ];

    for val in test_values {
        let bytes = canonical::encode(&val).unwrap();
        let decoded: u64 = canonical::decode(&bytes).unwrap();
        assert_eq!(decoded, val);

        // Verify re-encoding produces identical bytes
        let re_encoded = canonical::encode(&decoded).unwrap();
        assert_eq!(re_encoded, bytes);
    }
}

#[test]
fn test_byte_strings() {
    let byte_data = vec![0u8, 1, 2, 255, 128, 64];
    let bytes = canonical::encode(&byte_data).unwrap();
    let decoded: Vec<u8> = canonical::decode(&bytes).unwrap();
    assert_eq!(decoded, byte_data);
}

#[test]
fn test_encoding_determinism() {
    // This test validates that encoding is deterministic within the current runtime
    // by encoding the same value multiple times.
    // Note: Cross-platform verification requires CI matrix jobs across different
    // architectures and Rust versions
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestStruct {
        field1: String,
        field2: i64,
        field3: f64,
        field4: Vec<u8>,
    }

    let value = TestStruct {
        field1: "deterministic".to_string(),
        field2: 123456789,
        field3: 3.141592653589793,
        field4: vec![1, 2, 3, 4, 5],
    };

    // Encode 10 times and verify all bytes are identical
    let first_encoding = canonical::encode(&value).unwrap();
    for _ in 0..10 {
        let encoding = canonical::encode(&value).unwrap();
        assert_eq!(
            encoding, first_encoding,
            "Encoding must be deterministic"
        );
    }
}
