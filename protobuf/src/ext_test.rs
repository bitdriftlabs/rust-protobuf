#![allow(clippy::unwrap_used)]

use crate::descriptor::field_descriptor_proto::Type;
use crate::descriptor::FileOptions;
use crate::ext::ExtFieldRepeated;
use crate::Message;

/// Returns a `FileOptions` message with the given varint values stored as
/// non-packed unknowns under `field_number`. Used to simulate proto2-style
/// non-packed repeated scalar fields.
fn msg_with_varints(field_number: u32, values: &[u64]) -> FileOptions {
    let mut msg = FileOptions::new();
    for &v in values {
        msg.mut_unknown_fields().add_varint(field_number, v);
    }
    msg
}

/// Returns a `FileOptions` message with the given fixed32 values stored as
/// non-packed unknowns under `field_number`.
fn msg_with_fixed32s(field_number: u32, values: &[u32]) -> FileOptions {
    let mut msg = FileOptions::new();
    for &v in values {
        msg.mut_unknown_fields().add_fixed32(field_number, v);
    }
    msg
}

/// Returns a `FileOptions` message with the given fixed64 values stored as
/// non-packed unknowns under `field_number`.
fn msg_with_fixed64s(field_number: u32, values: &[u64]) -> FileOptions {
    let mut msg = FileOptions::new();
    for &v in values {
        msg.mut_unknown_fields().add_fixed64(field_number, v);
    }
    msg
}

/// Returns a `FileOptions` message with the given length-delimited blobs
/// stored as unknowns under `field_number`.
fn msg_with_ld(field_number: u32, blobs: &[&[u8]]) -> FileOptions {
    let mut msg = FileOptions::new();
    for &b in blobs {
        msg.mut_unknown_fields()
            .add_length_delimited(field_number, b.to_vec());
    }
    msg
}

const FIELD: u32 = 1000;

#[test]
fn empty_returns_empty_vec() {
    let msg = FileOptions::new();
    let ext: ExtFieldRepeated<FileOptions, i32> = ExtFieldRepeated::new(FIELD, Type::TYPE_INT32);
    assert_eq!(ext.get(&msg), Vec::<i32>::new());
}

// Non-packed repeated int32: each value is a separate varint unknown.
#[test]
fn non_packed_int32() {
    let msg = msg_with_varints(FIELD, &[1, 2, 3]);
    let ext: ExtFieldRepeated<FileOptions, i32> = ExtFieldRepeated::new(FIELD, Type::TYPE_INT32);
    assert_eq!(ext.get(&msg), vec![1i32, 2, 3]);
}

// Non-packed repeated sint32: varints with zigzag decoding.
#[test]
fn non_packed_sint32() {
    // zigzag(-1)=1, zigzag(-2)=3
    let msg = msg_with_varints(FIELD, &[1, 3]);
    let ext: ExtFieldRepeated<FileOptions, i32> = ExtFieldRepeated::new(FIELD, Type::TYPE_SINT32);
    assert_eq!(ext.get(&msg), vec![-1i32, -2]);
}

// Non-packed repeated bool.
#[test]
fn non_packed_bool() {
    let msg = msg_with_varints(FIELD, &[1, 0, 1]);
    let ext: ExtFieldRepeated<FileOptions, bool> = ExtFieldRepeated::new(FIELD, Type::TYPE_BOOL);
    assert_eq!(ext.get(&msg), vec![true, false, true]);
}

// Non-packed repeated float: each value is a separate fixed32 unknown.
#[test]
fn non_packed_float() {
    let msg = msg_with_fixed32s(FIELD, &[1.0f32.to_bits(), 2.0f32.to_bits()]);
    let ext: ExtFieldRepeated<FileOptions, f32> = ExtFieldRepeated::new(FIELD, Type::TYPE_FLOAT);
    assert_eq!(ext.get(&msg), vec![1.0f32, 2.0]);
}

// Non-packed repeated double: each value is a separate fixed64 unknown.
#[test]
fn non_packed_double() {
    let msg = msg_with_fixed64s(FIELD, &[1.0f64.to_bits(), 2.0f64.to_bits()]);
    let ext: ExtFieldRepeated<FileOptions, f64> = ExtFieldRepeated::new(FIELD, Type::TYPE_DOUBLE);
    assert_eq!(ext.get(&msg), vec![1.0f64, 2.0]);
}

// Non-packed repeated sfixed32.
#[test]
fn non_packed_sfixed32() {
    let msg = msg_with_fixed32s(FIELD, &[(-7i32) as u32, 8u32]);
    let ext: ExtFieldRepeated<FileOptions, i32> = ExtFieldRepeated::new(FIELD, Type::TYPE_SFIXED32);
    assert_eq!(ext.get(&msg), vec![-7i32, 8]);
}

// Non-packed repeated string: each value is a separate length-delimited unknown.
#[test]
fn non_packed_string() {
    let msg = msg_with_ld(FIELD, &[b"hello", b"world"]);
    let ext: ExtFieldRepeated<FileOptions, String> =
        ExtFieldRepeated::new(FIELD, Type::TYPE_STRING);
    assert_eq!(ext.get(&msg), vec!["hello", "world"]);
}

// Non-packed repeated bytes.
#[test]
fn non_packed_bytes() {
    let msg = msg_with_ld(FIELD, &[b"\x01\x02", b"\x03\x04"]);
    let ext: ExtFieldRepeated<FileOptions, Vec<u8>> =
        ExtFieldRepeated::new(FIELD, Type::TYPE_BYTES);
    assert_eq!(ext.get(&msg), vec![vec![1u8, 2], vec![3u8, 4]]);
}

// Packed repeated int32 (proto3 default): a single length-delimited entry
// containing three concatenated varints [1, 2, 3].
#[test]
fn packed_int32() {
    let packed_bytes: &[u8] = &[1, 2, 3];
    let msg = msg_with_ld(FIELD, &[packed_bytes]);
    let ext: ExtFieldRepeated<FileOptions, i32> = ExtFieldRepeated::new(FIELD, Type::TYPE_INT32);
    assert_eq!(ext.get(&msg), vec![1i32, 2, 3]);
}

// Packed repeated sint32: single LengthDelimited with zigzag-encoded varints.
// zigzag(-1)=1, zigzag(-2)=3 → packed bytes = [0x01, 0x03]
#[test]
fn packed_sint32() {
    let packed_bytes: &[u8] = &[0x01, 0x03];
    let msg = msg_with_ld(FIELD, &[packed_bytes]);
    let ext: ExtFieldRepeated<FileOptions, i32> = ExtFieldRepeated::new(FIELD, Type::TYPE_SINT32);
    assert_eq!(ext.get(&msg), vec![-1i32, -2]);
}

// Packed repeated float: single LengthDelimited containing two little-endian
// 4-byte floats.
#[test]
fn packed_float() {
    // 1.0f32 = 0x3F800000, 2.0f32 = 0x40000000 in little-endian
    let packed_bytes: &[u8] = &[0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x40];
    let msg = msg_with_ld(FIELD, &[packed_bytes]);
    let ext: ExtFieldRepeated<FileOptions, f32> = ExtFieldRepeated::new(FIELD, Type::TYPE_FLOAT);
    assert_eq!(ext.get(&msg), vec![1.0f32, 2.0]);
}

// Mixed non-packed and packed entries in the same field (valid per spec):
// a non-packed varint followed by a packed group.
#[test]
fn mixed_packed_and_non_packed() {
    let mut msg = FileOptions::new();
    // Non-packed entry: value 10
    msg.mut_unknown_fields().add_varint(FIELD, 10);
    // Packed entry: values [20, 30]
    msg.mut_unknown_fields()
        .add_length_delimited(FIELD, vec![20, 30]);
    let ext: ExtFieldRepeated<FileOptions, i32> = ExtFieldRepeated::new(FIELD, Type::TYPE_INT32);
    assert_eq!(ext.get(&msg), vec![10i32, 20, 30]);
}
