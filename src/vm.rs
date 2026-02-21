use std::io;

use crate::{
    chunk::{Chunk, OpCode},
    compiler::Compiler,
    debug,
    value::Value,
};

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: vec![],
        }
    }

    pub fn interpret(&mut self, source: String) -> Result<Option<Value>, InterpretError> {
        let mut compiler = Compiler::new(source);
        let chunk = compiler
            .compile()
            .map_err(|_| InterpretError::CompileError)?;

        self.chunk = chunk;
        self.ip = 0;

        self.run()
    }

    fn pop_stack(&mut self) -> Value {
        self.stack.pop().unwrap_or_else(|| {
            panic!("Stack exhausted");
        })
    }

    fn push_stack(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn run(&mut self) -> Result<Option<Value>, InterpretError> {
        fn read_byte(vm: &mut VM) -> u8 {
            let instruction = vm.chunk.get_code(vm.ip);
            vm.ip += 1;
            instruction
        }

        fn read_constant(vm: &mut VM) -> Value {
            let byte = read_byte(vm);
            vm.chunk.constants().get(byte as usize)
        }

        loop {
            if debug::is_debug_trace_execution_enabled() {
                print!("          ");
                self.stack.iter().for_each(|value| {
                    print!("[ {:?} ]", value);
                });
                println!();
                debug::disassemble_instruction(&mut io::stdout(), &self.chunk, self.ip);
            }

            let instruction = read_byte(self);

            let instruction: OpCode = instruction.try_into().unwrap_or_else(|_| {
                panic!("Invalid opcode {}", instruction);
            });

            match instruction {
                OpCode::Return => {
                    let value = self.pop_stack();
                    println!("{:?}", value);
                    return Ok(Some(value));
                }
                OpCode::Constant => {
                    let constant = read_constant(self);
                    self.stack.push(constant);
                }
                OpCode::Negate => {
                    let last = self.stack.last_mut().unwrap_or_else(|| {
                        panic!("Stack exhausted");
                    });
                    match last {
                        Value::Number(num) => {
                            *num = -*num;
                        }
                        _ => {
                            self.runtime_error("Operand must be a number.");
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                }
                OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide => {
                    let b = self.pop_stack();
                    let a = self.pop_stack();

                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            let result = match instruction {
                                OpCode::Add => a + b,
                                OpCode::Subtract => a - b,
                                OpCode::Multiply => a * b,
                                OpCode::Divide => a / b,
                                _ => unreachable!(),
                            };

                            self.push_stack(Value::Number(result));
                        }
                        _ => {
                            self.runtime_error("Operands must be numbers.");
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                }
            }
        }
    }

    fn runtime_error<S: AsRef<str>>(&mut self, message: S) {
        eprintln!("{}", message.as_ref());

        let line = self.chunk.get_line(self.ip - 1);
        eprintln!("[line {}] in script", line);

        self.reset_stack();
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_interpret() {
        // this whole test is just black-box testing

        // test error
        {
            let mut vm = VM::new();
            assert_eq!(
                vm.interpret("1 +".to_string()),
                Err(InterpretError::CompileError)
            );
        }

        // test unary ops
        {
            let mut vm = VM::new();
            assert_eq!(
                vm.interpret("-3".to_string()),
                Ok(Some(Value::Number(-3.0)))
            );
        }

        // test binary ops
        {
            let mut vm = VM::new();
            assert_eq!(
                vm.interpret("1 + 2".to_string()),
                Ok(Some(Value::Number(3.0)))
            );
        }

        {
            let mut vm = VM::new();
            assert_eq!(
                vm.interpret("8 - 3".to_string()),
                Ok(Some(Value::Number(5.0)))
            );
        }

        {
            let mut vm = VM::new();
            assert_eq!(
                vm.interpret("5 * 6".to_string()),
                Ok(Some(Value::Number(30.0)))
            );
        }

        {
            let mut vm = VM::new();
            assert_eq!(
                vm.interpret("28 / 4".to_string()),
                Ok(Some(Value::Number(7.0)))
            );
        }

        // test complex expressions
        {
            let mut vm = VM::new();
            assert_eq!(
                vm.interpret("(-1 + 2) * 3 - -4".to_string()),
                Ok(Some(Value::Number(7.0)))
            );
        }
    }
}
