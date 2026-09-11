use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Null => "null",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    // Array operations
    pub fn push(&mut self, value: Value) -> Result<(), String> {
        match self {
            Value::Array(arr) => {
                arr.push(value);
                Ok(())
            }
            _ => Err("push can only be called on arrays".to_string()),
        }
    }

    pub fn pop(&mut self) -> Result<Value, String> {
        match self {
            Value::Array(arr) => arr.pop().ok_or_else(|| "Cannot pop from empty array".to_string()),
            _ => Err("pop can only be called on arrays".to_string()),
        }
    }

    pub fn get_length(&self) -> Result<f64, String> {
        match self {
            Value::Array(arr) => Ok(arr.len() as f64),
            Value::String(s) => Ok(s.len() as f64),
            _ => Err("length can only be called on arrays or strings".to_string()),
        }
    }

    pub fn contains(&self, value: &Value) -> Result<bool, String> {
        match self {
            Value::Array(arr) => Ok(arr.contains(value)),
            Value::String(s) => {
                if let Some(search) = value.as_string() {
                    Ok(s.contains(search))
                } else {
                    Err("String contains requires a string argument".to_string())
                }
            }
            _ => Err("contains can only be called on arrays or strings".to_string()),
        }
    }

    // String operations
    pub fn to_uppercase(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err("uppercase can only be called on strings".to_string()),
        }
    }

    pub fn to_lowercase(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Err("lowercase can only be called on strings".to_string()),
        }
    }

    pub fn trim(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Err("trim can only be called on strings".to_string()),
        }
    }

    pub fn split_string(&self, delimiter: &str) -> Result<Value, String> {
        match self {
            Value::String(s) => {
                let parts: Vec<Value> = s
                    .split(delimiter)
                    .map(|part| Value::String(part.to_string()))
                    .collect();
                Ok(Value::Array(parts))
            }
            _ => Err("split can only be called on strings".to_string()),
        }
    }

    pub fn replace_string(&self, from: &str, to: &str) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(s.replace(from, to))),
            _ => Err("replace can only be called on strings".to_string()),
        }
    }

    // Array-specific operations
    pub fn join(&self, separator: &str) -> Result<Value, String> {
        match self {
            Value::Array(arr) => {
                let strings: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                Ok(Value::String(strings.join(separator)))
            }
            _ => Err("join can only be called on arrays".to_string()),
        }
    }

    pub fn reverse(&mut self) -> Result<(), String> {
        match self {
            Value::Array(arr) => {
                arr.reverse();
                Ok(())
            }
            _ => Err("reverse can only be called on arrays".to_string()),
        }
    }

    pub fn sort_array(&mut self) -> Result<(), String> {
        match self {
            Value::Array(arr) => {
                arr.sort_by(|a, b| {
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Value::String(x), Value::String(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    }
                });
                Ok(())
            }
            _ => Err("sort can only be called on arrays".to_string()),
        }
    }

    pub fn first(&self) -> Result<Value, String> {
        match self {
            Value::Array(arr) => arr.first().cloned().ok_or_else(|| "Array is empty".to_string()),
            _ => Err("first can only be called on arrays".to_string()),
        }
    }

    pub fn last(&self) -> Result<Value, String> {
        match self {
            Value::Array(arr) => arr.last().cloned().ok_or_else(|| "Array is empty".to_string()),
            _ => Err("last can only be called on arrays".to_string()),
        }
    }

    // More string methods
    pub fn substring(&self, start: usize, end: Option<usize>) -> Result<Value, String> {
        match self {
            Value::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                let end_pos = end.unwrap_or(chars.len());

                if start > chars.len() || end_pos > chars.len() || start > end_pos {
                    return Err("Invalid substring range".to_string());
                }

                let substring: String = chars[start..end_pos].iter().collect();
                Ok(Value::String(substring))
            }
            _ => Err("substring can only be called on strings".to_string()),
        }
    }

    pub fn index_of(&self, search: &str) -> Result<Value, String> {
        match self {
            Value::String(s) => {
                match s.find(search) {
                    Some(pos) => Ok(Value::Number(pos as f64)),
                    None => Ok(Value::Number(-1.0)),
                }
            }
            _ => Err("indexOf can only be called on strings".to_string()),
        }
    }

    pub fn starts_with(&self, prefix: &str) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::Boolean(s.starts_with(prefix))),
            _ => Err("startsWith can only be called on strings".to_string()),
        }
    }

    pub fn ends_with(&self, suffix: &str) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::Boolean(s.ends_with(suffix))),
            _ => Err("endsWith can only be called on strings".to_string()),
        }
    }

    pub fn char_at(&self, index: usize) -> Result<Value, String> {
        match self {
            Value::String(s) => {
                s.chars().nth(index)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| format!("Index {} out of bounds", index))
            }
            _ => Err("charAt can only be called on strings".to_string()),
        }
    }

    pub fn concat_string(&self, other: &str) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::String(format!("{}{}", s, other))),
            _ => Err("concat can only be called on strings".to_string()),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{:.0}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, val)?;
                }
                write!(f, "}}")
            }
        }
    }
}

// Conversion traits
impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<Vec<Value>> for Value {
    fn from(arr: Vec<Value>) -> Self {
        Value::Array(arr)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(obj: HashMap<String, Value>) -> Self {
        Value::Object(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_types() {
        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::Number(42.0).type_name(), "number");
        assert_eq!(Value::String("hello".to_string()).type_name(), "string");
        assert_eq!(Value::Boolean(true).type_name(), "boolean");
        assert_eq!(Value::Array(vec![]).type_name(), "array");
        assert_eq!(Value::Object(HashMap::new()).type_name(), "object");
    }

    #[test]
    fn test_is_truthy() {
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
        assert!(Value::Boolean(true).is_truthy());
        assert!(!Value::Number(0.0).is_truthy());
        assert!(Value::Number(42.0).is_truthy());
        assert!(!Value::String("".to_string()).is_truthy());
        assert!(Value::String("hello".to_string()).is_truthy());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Value::Null), "null");
        assert_eq!(format!("{}", Value::Number(42.0)), "42");
        assert_eq!(format!("{}", Value::Number(3.14)), "3.14");
        assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
        assert_eq!(format!("{}", Value::Boolean(true)), "true");
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
    fn test_conversions() {
        let v: Value = 42.into();
        assert_eq!(v, Value::Number(42.0));

        let v: Value = "hello".into();
        assert_eq!(v, Value::String("hello".to_string()));

        let v: Value = true.into();
        assert_eq!(v, Value::Boolean(true));
    }
}
