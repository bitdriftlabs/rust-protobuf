use protobuf::reflect::ReflectValueRef;
use protobuf::{MessageDyn, MessageFull};
use protobuf_test_common::{
    dynamic_descriptor_for_descriptor, test_serialize_deserialize_no_hex_with_dynamic,
    test_serialize_deserialize_with_dynamic,
};

use super::test_optional_pb::*;

#[test]
fn serialize_deserialize() {
    let mut message = TestOptionalProto3::new();
    message.iii = Some(0x1a);
    test_serialize_deserialize_with_dynamic("f8 01 1a", &message);
}

#[test]
fn dynamic_serialization_preserves_selected_default_values() {
    let mut message = TestOptionalProto3::new();
    let descriptor = TestOptionalProto3::descriptor();

    descriptor
        .field_by_name("one_field_2")
        .unwrap()
        .set_singular_field(&mut message, 0i32.into());
    descriptor
        .field_by_name("iii")
        .unwrap()
        .set_singular_field(&mut message, 0i32.into());
    descriptor
        .field_by_name("sss")
        .unwrap()
        .set_singular_field(&mut message, String::new().into());

    test_serialize_deserialize_no_hex_with_dynamic(&message);
}

#[test]
fn dynamic_serialization_elides_implicit_default_values() {
    let descriptor = dynamic_descriptor_for_descriptor::<TestOptionalProto3>();
    let mut message = descriptor.new_instance();
    descriptor
        .field_by_name("non_optional")
        .unwrap()
        .set_singular_field(&mut *message, 0i32.into());

    assert!(message.write_to_bytes_dyn().unwrap().is_empty());
}

#[test]
fn field_types() {
    let message = TestOptionalProto3::new();
    let _iii: &Option<i32> = &message.iii;
    let _sss: &Option<String> = &message.sss;
}

#[test]
fn reflect_all_oneofs() {
    let descriptor = TestOptionalProto3::descriptor();
    let oneofs = descriptor.all_oneofs().collect::<Vec<_>>();
    assert!(oneofs.len() > 1);
    assert!(!oneofs[0].is_synthetic());
    for oneof in &oneofs[1..] {
        assert!(oneof.is_synthetic());
        let mut fields = oneof.fields().collect::<Vec<_>>();
        assert_eq!(1, fields.len());
        let field = fields.swap_remove(0);
        assert_eq!(None, field.containing_oneof());
        assert_eq!(
            Some(oneof),
            field.containing_oneof_including_synthetic().as_ref()
        );
    }
}

#[test]
fn reflect_oneofs() {
    let descriptor = TestOptionalProto3::descriptor();
    let oneofs = descriptor.oneofs().collect::<Vec<_>>();
    assert_eq!(1, oneofs.len());
    assert!(!oneofs[0].is_synthetic());

    let mut message = TestOptionalProto3::new();

    let iii = descriptor.field_by_name("iii").unwrap();

    assert_eq!(None, iii.get_singular(&mut message));

    iii.set_singular_field(&mut message, 17.into());
    assert_eq!(Some(17), message.iii);
    assert_eq!(
        Some(ReflectValueRef::I32(17)),
        iii.get_singular(&mut message)
    );
}
