#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
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
    fn test_add() {
        let mut value_array = ValueArray::new();
        assert_eq!(value_array.add(Value::Number(7.0)), 0);
        assert_eq!(value_array.add(Value::Number(5.5)), 1);
        assert_eq!(value_array.add(Value::Number(9.0)), 2);
        assert_eq!(
            value_array.values,
            vec![Value::Number(7.0), Value::Number(5.5), Value::Number(9.0)]
        );
    }

    #[test]
    fn test_get() {
        let value_array = ValueArray {
            values: vec![Value::Number(7.0), Value::Number(5.5), Value::Number(9.0)],
        };
        assert_eq!(value_array.get(0), Value::Number(7.0));
        assert_eq!(value_array.get(1), Value::Number(5.5));
        assert_eq!(value_array.get(2), Value::Number(9.0));
    }
}
