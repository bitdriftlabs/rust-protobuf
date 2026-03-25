//! Utilities to support "extension" fields.
//!
//! This is a stopgap implementation that supports fetching singular and repeated scalar,
//! string, bytes, and message extension values. Anything similar to an extension registry
//! is not implemented yet.
//!
//! Extensions are [described in the official protobuf documentation][exts].
//!
//! [exts]: https://developers.google.com/protocol-buffers/docs/proto#extensions

use std::marker::PhantomData;

use crate::descriptor::field_descriptor_proto::Type;
use crate::reflect::runtime_types::RuntimeTypeTrait;
use crate::reflect::ProtobufValue;
use crate::Message;

#[cfg(test)]
#[path = "./ext_test.rs"]
mod tests;

/// Optional ext field
///
/// This is initialized from generated code, do not instantiate directly.
pub struct ExtFieldOptional<M, T> {
    /// Extension field number.
    field_number: u32,
    /// Extension field type.
    field_type: Type,
    /// Marker
    phantom: PhantomData<(M, T)>,
}

/// Repeated ext field
///
/// This is initialized from generated code, do not instantiate directly.
pub struct ExtFieldRepeated<M, V> {
    /// Extension field number
    field_number: u32,
    /// Field type.
    field_type: Type,
    /// Extension field number
    phantom: PhantomData<(M, V)>,
}

impl<M, V> ExtFieldOptional<M, V> {
    /// Constructor. Called from generated code.
    pub const fn new(field_number: u32, field_type: Type) -> Self {
        ExtFieldOptional {
            field_number,
            field_type,
            phantom: PhantomData,
        }
    }
}

impl<M: Message, V: ProtobufValue> ExtFieldOptional<M, V> {
    /// Get a copy of value from a message.
    ///
    /// Extension data is stored in [`UnknownFields`](crate::UnknownFields).
    pub fn get(&self, m: &M) -> Option<V> {
        m.unknown_fields()
            .get(self.field_number)
            .and_then(|u| V::RuntimeType::get_from_unknown(u, self.field_type))
    }
}

impl<M, V> ExtFieldRepeated<M, V> {
    /// Constructor. Called from generated code.
    pub const fn new(field_number: u32, field_type: Type) -> Self {
        ExtFieldRepeated {
            field_number,
            field_type,
            phantom: PhantomData,
        }
    }
}

impl<M: Message, V: ProtobufValue> ExtFieldRepeated<M, V> {
    /// Get all values of this repeated extension field from a message.
    ///
    /// Extension data is stored in [`UnknownFields`](crate::UnknownFields). Both non-packed
    /// (proto2 default) and packed (proto3 default for scalars) encodings are handled.
    pub fn get(&self, m: &M) -> Vec<V> {
        let mut result = Vec::new();
        for uv in m.unknown_fields().get_all(self.field_number) {
            V::RuntimeType::decode_repeated_from_unknown(uv, self.field_type, &mut result);
        }
        result
    }
}
