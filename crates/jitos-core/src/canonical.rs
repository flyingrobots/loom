//! Canonical encoding for deterministic serialization.
//!
//! All ledger events, receipts, snapshots, deltas, and archives MUST use canonical encoding.
//! Non-canonical encoders (including serde_json) are FORBIDDEN in jitos-provenance.
//!
//! ## Design Principle: Structural Determinism
//!
//! **Canonical encoding prioritizes structural determinism over space efficiency.**
//!
//! - **Structural determinism:** There is nothing to decide. One logical value → one byte sequence.
//! - **Procedural determinism:** If everyone runs the same algorithm, they'll agree (fragile across languages/runtimes).
//!
//! JITOS chooses structural determinism to ensure cross-language, cross-platform correctness.
//!
//! ## Canonical Rules (RFC 8949 + SPEC-0001)
//!
//! 1. Map entries sorted by lexicographic ordering of canonical CBOR encoding of keys
//! 2. Definite-length encoding only (no streaming/indefinite lengths)
//! 3. Float encoding:
//!    - Integral values (f.fract() == 0.0 and fits i128) → encode as integer
//!    - Non-integral floats → **always** encode as float64 (0xfb)
//!    - **Forbidden:** float16 (0xf9) and float32 (0xfa) in canonical encoding
//!    - Rationale: JS, WASM, Python, SQL all use f64 natively. Eliminates width-selection heuristics.
//! 4. Float canonicalization:
//!    - NaN → IEEE-754 quiet NaN with payload=0 (bit pattern: 0x7FF8_0000_0000_0000)
//!    - ±0 normalized to +0
//!    - ±∞ preserved as-is
//!    - Subnormals flushed to zero
//! 5. No duplicate map keys
//!
//! ## Safety
//!
//! Use `hash_canonical()` instead of manual hashing to prevent accidental non-canonical hashing.
//!
//! ## Attribution
//!
//! Adapted from echo-session-proto/canonical.rs (Apache-2.0)
//! © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

use ciborium::value::{Integer, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CanonicalError {
    #[error("incomplete input")]
    Incomplete,
    #[error("trailing bytes after value")]
    Trailing,
    #[error("tags not allowed")]
    Tag,
    #[error("indefinite length not allowed")]
    Indefinite,
    #[error("non-canonical integer width")]
    NonCanonicalInt,
    #[error("non-canonical float width")]
    NonCanonicalFloat,
    #[error("float encodes integral value; must be integer")]
    FloatShouldBeInt,
    #[error("map keys not strictly increasing")]
    MapKeyOrder,
    #[error("duplicate map key")]
    DuplicateKey,
    #[error("decode error: {0}")]
    Decode(String),
}

type Result<T> = std::result::Result<T, CanonicalError>;

// Public API

/// Encode a value using canonical CBOR.
///
/// This is the ONLY valid way to serialize data for the ledger.
/// All other serialization methods are forbidden for determinism-critical data.
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    // Convert to ciborium Value
    let v: Value = ciborium::value::Value::serialized(value)
        .map_err(|e| CanonicalError::Decode(e.to_string()))?;
    encode_value(&v)
}

/// Decode a value from canonical CBOR.
pub fn decode<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    let v = decode_value(bytes)?;
    ciborium::value::Value::deserialized(&v).map_err(|e| CanonicalError::Decode(e.to_string()))
}

/// Hash a value using canonical encoding.
///
/// This is the ONLY valid way to hash data for determinism.
/// Never call `blake3::hash()` directly on serialized data.
pub fn hash_canonical<T: Serialize>(value: &T) -> Result<crate::Hash> {
    let bytes = encode(value)?;
    let hash = blake3::hash(&bytes);
    Ok(crate::Hash(*hash.as_bytes()))
}

fn encode_value(val: &Value) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    enc_value(val, &mut out)?;
    Ok(out)
}

fn decode_value(bytes: &[u8]) -> Result<Value> {
    let mut idx = 0usize;
    let v = dec_value(bytes, &mut idx, true)?;
    if idx != bytes.len() {
        return Err(CanonicalError::Trailing);
    }
    Ok(v)
}

// --- Encoder --------------------------------------------------------------

fn enc_value(v: &Value, out: &mut Vec<u8>) -> Result<()> {
    match v {
        Value::Bool(b) => {
            out.push(if *b { 0xf5 } else { 0xf4 });
        }
        Value::Null => out.push(0xf6),
        Value::Integer(n) => enc_int(i128::from(*n), out),
        Value::Float(f) => enc_float(*f, out),
        Value::Text(s) => enc_text(s, out)?,
        Value::Bytes(b) => enc_bytes(b, out)?,
        Value::Array(items) => {
            enc_len(4, items.len() as u64, out);
            for it in items {
                enc_value(it, out)?;
            }
        }
        Value::Map(entries) => {
            // Encode keys and pair with values (avoid cloning keys, only clone values)
            let mut buf: Vec<(Vec<u8>, Value)> = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                let mut kb = Vec::new();
                enc_value(k, &mut kb)?;
                buf.push((kb, v.clone()));
            }

            // Sort by encoded key bytes
            buf.sort_by(|a, b| a.0.cmp(&b.0));

            // Check for duplicate keys
            for win in buf.windows(2) {
                if win[0].0 == win[1].0 {
                    return Err(CanonicalError::DuplicateKey);
                }
            }

            // Write map with sorted entries
            enc_len(5, buf.len() as u64, out);
            for (kb, v) in buf {
                out.extend_from_slice(&kb);
                enc_value(&v, out)?;
            }
        }
        Value::Tag(_, _) => return Err(CanonicalError::Tag),
        _ => return Err(CanonicalError::Decode("unsupported simple value".into())),
    }
    Ok(())
}

fn enc_len(major: u8, len: u64, out: &mut Vec<u8>) {
    write_major(major, len as u128, out);
}

fn enc_int(n: i128, out: &mut Vec<u8>) {
    if n >= 0 {
        write_major(0, n as u128, out);
    } else {
        // CBOR negative: value = -1 - n => major 1 with (-(n+1))
        let m = (-1 - n) as u128;
        write_major(1, m, out);
    }
}

/// Canonicalize a float64 value according to SPEC-0001 rules.
///
/// - NaN → IEEE-754 quiet NaN with bit pattern 0x7FF8_0000_0000_0000
/// - ±0 → +0
/// - ±∞ preserved as-is
/// - Subnormals flushed to zero
fn canonicalize_f64(val: f64) -> f64 {
    if val.is_nan() {
        // Canonical NaN: quiet NaN with payload=0
        // Per SPEC-0001: 0x7FF8_0000_0000_0000
        f64::from_bits(0x7FF8_0000_0000_0000)
    } else if val.is_subnormal() {
        // Flush subnormals to +0 (includes sign normalization)
        // Note: Negative subnormals (e.g., -5e-324) become +0.0 here,
        // so they don't reach the ±0 normalization check below
        0.0
    } else if val == 0.0 {
        // Normalize ±0 to +0 (handles regular zeros, subnormals handled above)
        0.0
    } else {
        val
    }
}

/// Encode a float as canonical CBOR.
///
/// Per SPEC-0001:
/// - Integral values (f.fract() == 0.0 and fits i128) → encode as integer
/// - Non-integral floats → always encode as float64 (0xfb) for structural determinism
///
/// Unlike Echo's "smallest width that round-trips" (f16/f32/f64), JITOS uses
/// always-f64 to ensure cross-language compatibility (JS, WASM, Python, SQL).
///
/// This eliminates width-selection heuristics, ensuring one logical value maps
/// to exactly one byte sequence with no procedural dependencies.
fn enc_float(f: f64, out: &mut Vec<u8>) {
    let canonical_f = canonicalize_f64(f);

    // If integral and fits i128, encode as integer per SPEC-0001
    if canonical_f.fract() == 0.0 && canonical_f.is_finite() {
        let i = canonical_f as i128;
        if i as f64 == canonical_f {
            enc_int(i, out);
            return;
        }
    }

    // Non-integral: encode as float64 for structural determinism
    // CBOR float64: major type 7, additional info 27 (0xFB)
    out.push(0xfb);
    out.extend_from_slice(&canonical_f.to_bits().to_be_bytes());
}

fn enc_bytes(b: &[u8], out: &mut Vec<u8>) -> Result<()> {
    enc_len(2, b.len() as u64, out);
    out.extend_from_slice(b);
    Ok(())
}

fn enc_text(s: &str, out: &mut Vec<u8>) -> Result<()> {
    enc_len(3, s.len() as u64, out);
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

fn write_major(major: u8, n: u128, out: &mut Vec<u8>) {
    debug_assert!(major <= 7);
    match n {
        0..=23 => out.push((major << 5) | n as u8),
        24..=0xff => {
            out.push((major << 5) | 24);
            out.push(n as u8);
        }
        0x100..=0xffff => {
            out.push((major << 5) | 25);
            out.extend_from_slice(&(n as u16).to_be_bytes());
        }
        0x1_0000..=0xffff_ffff => {
            out.push((major << 5) | 26);
            out.extend_from_slice(&(n as u32).to_be_bytes());
        }
        _ => {
            out.push((major << 5) | 27);
            out.extend_from_slice(&(n as u64).to_be_bytes());
        }
    }
}

// --- Decoder --------------------------------------------------------------

fn dec_value(bytes: &[u8], idx: &mut usize, strict: bool) -> Result<Value> {
    if *idx >= bytes.len() {
        return Err(CanonicalError::Incomplete);
    }
    let b0 = bytes[*idx];
    *idx += 1;
    let major = b0 >> 5;
    let ai = b0 & 0x1f;

    // forbid tags
    if major == 6 {
        return Err(CanonicalError::Tag);
    }

    // forbid indefinite
    if ai == 31 {
        return Err(CanonicalError::Indefinite);
    }

    // For major type 7 (floats/simples), handle ai directly without parsing length
    if major == 7 {
        return match ai {
            20 => Ok(Value::Bool(false)),
            21 => Ok(Value::Bool(true)),
            22 | 23 => Ok(Value::Null),
            24 => Err(CanonicalError::Decode("simple value not supported".into())),
            25 | 26 => {
                // Per SPEC-0001: Reject float16/float32, require float64
                Err(CanonicalError::NonCanonicalFloat)
            }
            27 => {
                // Check for truncated input before reading
                if *idx + 8 > bytes.len() {
                    return Err(CanonicalError::Incomplete);
                }

                let bits = take_u(bytes, idx, 8);
                let f = f64::from_bits(bits);

                // Per SPEC-0001: Integral floats MUST be encoded as integers
                if strict && float_should_be_int(f) {
                    return Err(CanonicalError::FloatShouldBeInt);
                }

                // Verify canonicalization (NaN/±0/subnormal)
                if strict {
                    let canonical_f = canonicalize_f64(f);
                    if canonical_f.to_bits() != f.to_bits() {
                        return Err(CanonicalError::NonCanonicalFloat);
                    }
                }

                Ok(Value::Float(f))
            }
            _ => Err(CanonicalError::Decode("unknown simple/float".into())),
        };
    }

    let n = match ai {
        0..=23 => ai as u64,
        24 => take_u(bytes, idx, 1),
        25 => take_u(bytes, idx, 2),
        26 => take_u(bytes, idx, 4),
        27 => take_u(bytes, idx, 8),
        _ => return Err(CanonicalError::Decode("invalid additional info".into())),
    };

    match major {
        0 => {
            // unsigned int
            check_min_int(ai, n, false, strict)?;
            Ok(int_to_value(n as u128, false))
        }
        1 => {
            // negative
            check_min_int(ai, n, true, strict)?;
            Ok(int_to_value(n as u128, true))
        }
        2 => {
            let len = n as usize;
            let end = *idx + len;
            if end > bytes.len() {
                return Err(CanonicalError::Incomplete);
            }
            let v = Value::Bytes(bytes[*idx..end].to_vec());
            *idx = end;
            Ok(v)
        }
        3 => {
            let len = n as usize;
            let end = *idx + len;
            if end > bytes.len() {
                return Err(CanonicalError::Incomplete);
            }
            let s = std::str::from_utf8(&bytes[*idx..end])
                .map_err(|e| CanonicalError::Decode(e.to_string()))?
                .to_string();
            *idx = end;
            Ok(Value::Text(s))
        }
        4 => {
            let len = n as usize;
            let mut items = Vec::with_capacity(len);
            for _ in 0..len {
                items.push(dec_value(bytes, idx, strict)?);
            }
            Ok(Value::Array(items))
        }
        5 => {
            let len = n as usize;
            let mut entries = Vec::with_capacity(len);
            let mut prev_bytes: Option<Vec<u8>> = None;
            for _ in 0..len {
                let key_start = *idx;
                let key = dec_value(bytes, idx, strict)?;
                let key_end = *idx;
                let key_bytes = &bytes[key_start..key_end];
                let curr_bytes = key_bytes.to_vec();
                if let Some(pb) = &prev_bytes {
                    match pb.cmp(&curr_bytes) {
                        std::cmp::Ordering::Less => {}
                        std::cmp::Ordering::Equal => return Err(CanonicalError::DuplicateKey),
                        std::cmp::Ordering::Greater => return Err(CanonicalError::MapKeyOrder),
                    }
                }
                prev_bytes = Some(curr_bytes);
                let val = dec_value(bytes, idx, strict)?;
                entries.push((key, val));
            }
            Ok(Value::Map(entries))
        }
        6 => unreachable!(),
        7 => unreachable!(), // handled above
        _ => Err(CanonicalError::Decode("unknown major".into())),
    }
}

fn take_u(bytes: &[u8], idx: &mut usize, len: usize) -> u64 {
    let mut buf = [0u8; 8];
    let end = *idx + len;
    if end > bytes.len() {
        return 0; // will be caught as incomplete later
    }
    buf[8 - len..].copy_from_slice(&bytes[*idx..end]);
    *idx = end;
    u64::from_be_bytes(buf)
}

fn check_min_int(ai: u8, n: u64, _negative: bool, strict: bool) -> Result<()> {
    if !strict {
        return Ok(());
    }
    let min_ok = match ai {
        0..=23 => true,
        24 => n >= 24,
        25 => n > 0xff,
        26 => n > 0xffff,
        27 => n > 0xffff_ffff,
        _ => false,
    };
    if min_ok {
        Ok(())
    } else {
        Err(CanonicalError::NonCanonicalInt)
    }
}

/// Check if a float value should have been encoded as an integer.
///
/// Per SPEC-0001: Integral floats (f.fract() == 0.0 and fits i128) MUST be encoded as integers.
fn float_should_be_int(f: f64) -> bool {
    f.is_finite() && f.fract() == 0.0 && fits_i128(f)
}

/// Check if a float value fits in an i128.
fn fits_i128(f: f64) -> bool {
    const MAX: f64 = i128::MAX as f64;
    const MIN: f64 = i128::MIN as f64;
    (MIN..=MAX).contains(&f)
}

fn int_to_value(n: u128, negative: bool) -> Value {
    if negative {
        // value = -1 - n
        let v = -1i128 - (n as i128);
        Value::Integer(Integer::try_from(v).expect("integer out of range"))
    } else {
        Value::Integer(Integer::try_from(n as i128).expect("integer out of range"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Encoding tests

    #[test]
    fn ec01_encode_decode_roundtrip() {
        let value = 42u64;
        let bytes = encode(&value).unwrap();
        let decoded: u64 = decode(&bytes).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn ec02_minimal_int_widths() {
        // Ported from echo-session-proto canonical.rs
        assert_eq!(encode_value(&Value::Integer(23.into())).unwrap()[0], 0x17);
        assert_eq!(
            encode_value(&Value::Integer(24.into())).unwrap(),
            vec![0x18, 0x18]
        );
        assert_eq!(
            encode_value(&Value::Integer(255.into())).unwrap(),
            vec![0x18, 0xff]
        );
        assert_eq!(
            encode_value(&Value::Integer(256.into())).unwrap(),
            vec![0x19, 0x01, 0x00]
        );
    }

    #[test]
    fn ec03_float64_always_used() {
        // Per SPEC-0001: Integral floats encode as integers, non-integral as float64
        // (Adapted from echo ec04_ints_not_floats_and_smallest_float_width)

        // 1.0 is integral, should encode as integer
        let one = encode_value(&Value::Float(1.0)).unwrap();
        assert_eq!(one[0], 0x01); // integer 1, not float

        // 0.5 is non-integral, should encode as float64 (not f16 like echo)
        let half = encode_value(&Value::Float(0.5)).unwrap();
        assert_eq!(half[0], 0xfb); // float64 tag (JITOS always uses f64)
        assert_eq!(half.len(), 9); // 1 byte tag + 8 bytes data
    }

    #[test]
    fn ec04_canonical_nan() {
        // Per SPEC-0001: NaN must be 0x7FF8_0000_0000_0000
        let nan_bytes = encode_value(&Value::Float(f64::NAN)).unwrap();
        assert_eq!(nan_bytes.len(), 9);
        assert_eq!(nan_bytes[0], 0xfb);
        let expected_nan: [u8; 8] = 0x7FF8_0000_0000_0000u64.to_be_bytes();
        assert_eq!(&nan_bytes[1..9], &expected_nan);
    }

    #[test]
    fn ec05_zero_normalization() {
        // Per SPEC-0001: ±0 must normalize to +0
        let pos_zero = encode_value(&Value::Float(0.0)).unwrap();
        let neg_zero = encode_value(&Value::Float(-0.0)).unwrap();
        assert_eq!(pos_zero, neg_zero);
    }

    // Decoding rejection tests (ported from echo-session-proto)

    #[test]
    fn dc01_reject_indefinite() {
        // Ported from echo dc02_reject_indefinite
        let bytes = vec![0x9f, 0x01, 0x02, 0xff];
        let res = decode_value(&bytes);
        assert!(matches!(res, Err(CanonicalError::Indefinite)));
    }

    #[test]
    fn dc02_reject_non_canonical_int() {
        // Ported from echo dc03_reject_non_canonical_int
        let bytes = vec![0x19, 0x00, 0x01];
        let res = decode_value(&bytes);
        assert!(matches!(res, Err(CanonicalError::NonCanonicalInt)));
    }

    #[test]
    fn dc03_reject_tag() {
        // Ported from echo dc04_reject_tag
        let bytes = vec![0xc0, 0x00];
        let res = decode_value(&bytes);
        assert!(matches!(res, Err(CanonicalError::Tag)));
    }

    #[test]
    fn dc04_reject_duplicate_keys() {
        // Ported from echo dc05_reject_duplicate_keys
        let bytes = vec![0xa2, 0x61, 0x61, 0x01, 0x61, 0x61, 0x02];
        let res = decode_value(&bytes);
        assert!(matches!(res, Err(CanonicalError::DuplicateKey)));
    }

    #[test]
    fn dc05_reject_wrong_order() {
        // Ported from echo dc06_reject_wrong_order
        let bytes = vec![0xa2, 0x61, 0x7a, 0x01, 0x61, 0x61, 0x01];
        let res = decode_value(&bytes);
        assert!(matches!(res, Err(CanonicalError::MapKeyOrder)));
    }

    #[test]
    fn dc06_reject_float16() {
        // Per SPEC-0001: Only float64 allowed
        let bytes = vec![0xf9, 0x38, 0x00]; // half-float 0.5
        let res = decode_value(&bytes);
        assert!(matches!(res, Err(CanonicalError::NonCanonicalFloat)));
    }

    #[test]
    fn dc07_reject_float32() {
        // Per SPEC-0001: Only float64 allowed
        let bytes = vec![0xfa, 0x3f, 0x00, 0x00, 0x00]; // float32 0.5
        let res = decode_value(&bytes);
        assert!(matches!(res, Err(CanonicalError::NonCanonicalFloat)));
    }

    #[test]
    fn dc08_reject_non_canonical_nan() {
        // Reject NaN with non-canonical bit pattern
        let non_canonical_nan = vec![0xfb, 0x7f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
        let res = decode_value(&non_canonical_nan);
        assert!(matches!(res, Err(CanonicalError::NonCanonicalFloat)));
    }

    #[test]
    fn dc09_reject_float64_encoding_of_integral() {
        // Per SPEC-0001: Integral floats MUST be encoded as integers
        // float64(1.0) should be rejected, should be integer 1
        let float_one = vec![0xfb, 0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let res = decode_value(&float_one);
        assert!(
            matches!(res, Err(CanonicalError::FloatShouldBeInt)),
            "Expected FloatShouldBeInt, got: {:?}",
            res
        );
    }

    #[test]
    fn dc10_reject_truncated_float64() {
        // Truncated float64 should return Incomplete, not silently decode as 0.0
        let truncated = vec![0xfb]; // float64 marker without 8-byte payload
        let res = decode_value(&truncated);
        assert!(
            matches!(res, Err(CanonicalError::Incomplete)),
            "Expected Incomplete, got: {:?}",
            res
        );
    }
}
