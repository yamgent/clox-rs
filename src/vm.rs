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

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
        let mut compiler = Compiler::new(source);
        compiler.compile();
        Ok(())
    }

    fn pop_stack(&mut self) -> Value {
        self.stack.pop().unwrap_or_else(|| {
            panic!("Stack exhausted");
        })
    }

    fn push_stack(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
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
                    print!("[ {} ]", value);
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
                    println!("{}", self.pop_stack());
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = read_constant(self);
                    self.stack.push(constant);
                }
                OpCode::Negate => {
                    let last = self.stack.last_mut().unwrap_or_else(|| {
                        panic!("Stack exhausted");
                    });
                    *last = -*last;
                }
                OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide => {
                    let b = self.pop_stack();
                    let a = self.pop_stack();

                    let result = match instruction {
                        OpCode::Add => a + b,
                        OpCode::Subtract => a - b,
                        OpCode::Multiply => a * b,
                        OpCode::Divide => a / b,
                        _ => unreachable!(),
                    };

                    self.push_stack(result);
                }
            }
        }
    }
}
