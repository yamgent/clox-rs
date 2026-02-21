#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        matches!(self, Value::Nil | Value::Bool(false))
    }
}

#[derive(Debug, PartialEq)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn add(&mut self, value: Value) -> usize {
        self.values.push(value);
        self.values.len() - 1
    }

    pub fn get(&self, i: usize) -> Value {
        self.values[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_is_falsey() {
        // in Lox, only nil and false is falsey, everything else is true (even 0!)
        assert!(Value::Nil.is_falsey());
        assert!(Value::Bool(false).is_falsey());

        assert!(!Value::Bool(true).is_falsey());
        assert!(!Value::Number(0.0).is_falsey());
        assert!(!Value::Number(1.0).is_falsey());
        assert!(!Value::Number(-1.0).is_falsey());
        assert!(!Value::Number(0.5).is_falsey());
    }

    #[test]
    fn test_value_array_add() {
        let mut value_array = ValueArray::new();
        assert_eq!(value_array.add(Value::Number(7.0)), 0);
        assert_eq!(value_array.add(Value::Number(5.5)), 1);
        assert_eq!(value_array.add(Value::Number(9.0)), 2);
        assert_eq!(value_array.add(Value::Nil), 3);
        assert_eq!(value_array.add(Value::Bool(true)), 4);
        assert_eq!(value_array.add(Value::Bool(false)), 5);
        assert_eq!(
            value_array.values,
            vec![
                Value::Number(7.0),
                Value::Number(5.5),
                Value::Number(9.0),
                Value::Nil,
                Value::Bool(true),
                Value::Bool(false)
            ]
        );
    }

    #[test]
    fn test_value_array_get() {
        let value_array = ValueArray {
            values: vec![
                Value::Number(7.0),
                Value::Number(5.5),
                Value::Number(9.0),
                Value::Nil,
                Value::Bool(true),
                Value::Bool(false),
            ],
        };
        assert_eq!(value_array.get(0), Value::Number(7.0));
        assert_eq!(value_array.get(1), Value::Number(5.5));
        assert_eq!(value_array.get(2), Value::Number(9.0));
        assert_eq!(value_array.get(3), Value::Nil);
        assert_eq!(value_array.get(4), Value::Bool(true));
        assert_eq!(value_array.get(5), Value::Bool(false));
    }
}
