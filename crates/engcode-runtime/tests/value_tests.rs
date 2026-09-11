use engcode_runtime::Value;
use std::collections::HashMap;

#[test]
fn test_value_types() {
    assert_eq!(Value::Null.type_name(), "null");
    assert_eq!(Value::Number(42.0).type_name(), "number");
    assert_eq!(Value::String("test".to_string()).type_name(), "string");
    assert_eq!(Value::Boolean(true).type_name(), "boolean");
    assert_eq!(Value::Array(vec![]).type_name(), "array");
    assert_eq!(Value::Object(HashMap::new()).type_name(), "object");
}

#[test]
fn test_truthy_values() {
    assert!(!Value::Null.is_truthy());
    assert!(!Value::Boolean(false).is_truthy());
    assert!(Value::Boolean(true).is_truthy());
    assert!(!Value::Number(0.0).is_truthy());
    assert!(Value::Number(1.0).is_truthy());
    assert!(!Value::String("".to_string()).is_truthy());
    assert!(Value::String("hello".to_string()).is_truthy());
}

#[test]
fn test_value_display() {
    assert_eq!(format!("{}", Value::Null), "null");
    assert_eq!(format!("{}", Value::Number(42.0)), "42");
    assert_eq!(format!("{}", Value::Number(3.14)), "3.14");
    assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
    assert_eq!(format!("{}", Value::Boolean(true)), "true");
    assert_eq!(format!("{}", Value::Boolean(false)), "false");
}

#[test]
fn test_array_display() {
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    assert_eq!(format!("{}", arr), "[1, 2, 3]");
}

#[test]
fn test_object_creation() {
    let mut obj = HashMap::new();
    obj.insert("name".to_string(), Value::String("Alice".to_string()));
    obj.insert("age".to_string(), Value::Number(30.0));

    let value = Value::Object(obj);
    assert_eq!(value.type_name(), "object");
}

#[test]
fn test_value_conversions() {
    let v: Value = 42.into();
    assert_eq!(v, Value::Number(42.0));

    let v: Value = "hello".into();
    assert_eq!(v, Value::String("hello".to_string()));

    let v: Value = true.into();
    assert_eq!(v, Value::Boolean(true));
}

#[test]
fn test_nested_arrays() {
    let inner = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    let outer = Value::Array(vec![inner, Value::Number(3.0)]);

    assert_eq!(outer.type_name(), "array");
    if let Value::Array(arr) = outer {
        assert_eq!(arr.len(), 2);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_value_accessors() {
    let num = Value::Number(42.0);
    assert_eq!(num.as_number(), Some(42.0));
    assert_eq!(num.as_string(), None);

    let s = Value::String("test".to_string());
    assert_eq!(s.as_string(), Some("test"));
    assert_eq!(s.as_number(), None);

    let b = Value::Boolean(true);
    assert_eq!(b.as_boolean(), Some(true));
    assert_eq!(b.as_number(), None);
}
