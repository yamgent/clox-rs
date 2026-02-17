pub type Value = f64;

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
        assert_eq!(value_array.add(7.0), 0);
        assert_eq!(value_array.add(5.5), 1);
        assert_eq!(value_array.add(9.0), 2);
        assert_eq!(value_array.values, vec![7.0, 5.5, 9.0]);
    }

    #[test]
    fn test_get() {
        let value_array = ValueArray {
            values: vec![7.0, 5.5, 9.0],
        };
        assert_eq!(value_array.get(0), 7.0);
        assert_eq!(value_array.get(1), 5.5);
        assert_eq!(value_array.get(2), 9.0);
    }
}
