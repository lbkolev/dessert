use std::collections::HashMap;
use std::fmt;

type Stack = Vec<u16>;
type Memory = Vec<u16>;
const MAX_MEMORY_SIZE: usize = 1_000_000;

#[derive(Debug)]
pub enum VMError {
    MissingArgument(String),
    InvalidPushValue(String),
    InvalidInstruction(String),
    UndefinedLabel(String),
    StackUnderflow,
    DivisionByZero,
    CallStackUnderflow,
    MemoryAccessOutOfBounds(usize),

    IoError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
}
impl std::error::Error for VMError {}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VMError::MissingArgument(op) => write!(f, "Missing argument for operation '{}'", op),
            VMError::InvalidPushValue(val) => write!(f, "Invalid push value '{}'", val),
            VMError::InvalidInstruction(instr) => write!(f, "Invalid instruction '{}'", instr),
            VMError::UndefinedLabel(label) => write!(f, "Undefined label '{}'", label),
            VMError::StackUnderflow => write!(f, "Stack underflow encountered"),
            VMError::DivisionByZero => write!(f, "Attempted division by zero"),
            VMError::CallStackUnderflow => write!(f, "Call stack underflow encountered"),
            VMError::MemoryAccessOutOfBounds(addr) => {
                write!(f, "Memory access out of bounds at address {}", addr)
            }
            VMError::IoError(err) => write!(f, "I/O error: {}", err),
            VMError::ParseIntError(err) => write!(f, "Parse integer error: {}", err),
        }
    }
}

impl From<std::io::Error> for VMError {
    fn from(error: std::io::Error) -> Self {
        VMError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for VMError {
    fn from(error: std::num::ParseIntError) -> Self {
        VMError::ParseIntError(error)
    }
}

#[derive(Clone, Debug)]
struct Context {
    pub stack: Stack,
    pub memory: Memory,
    pub pc: usize,
    pub call_stack: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    // stack operations
    Push(u16),
    Pop,
    Print,

    // arithmetic operations
    Add,
    Sub,
    Mul,
    Div,

    // memory operations
    Load,
    Store,

    // control flow
    Jump(String),
    JumpZ(String), // jump if zero
    JumpNotZ(String),
    Call(String),
    Ret,

    // resolved control flow (after label resolution)
    JumpResolved(usize),
    JumpZResolved(usize),
    JumpNotZResolved(usize),
    CallResolved(usize),

    // program control
    Halt,

    // for label definitions
    Label(String),
}

fn map_op(s: (&str, Option<&str>)) -> Result<Instruction, VMError> {
    use Instruction::*;

    match s.0 {
        "push" => {
            let arg = s.1.ok_or_else(|| VMError::MissingArgument("push".into()))?;
            let value = arg
                .parse::<u16>()
                .map_err(|_| VMError::InvalidPushValue(arg.into()))?;
            Ok(Push(value))
        }
        "pop" => Ok(Pop),
        "print" => Ok(Print),
        "add" => Ok(Add),
        "sub" => Ok(Sub),
        "mul" => Ok(Mul),
        "div" => Ok(Div),
        "load" => Ok(Load),
        "store" => Ok(Store),
        "jump" => {
            let label =
                s.1.ok_or_else(|| VMError::MissingArgument("jump".into()))?
                    .to_string();
            Ok(Jump(label))
        }
        "jumpz" => {
            let label =
                s.1.ok_or_else(|| VMError::MissingArgument("jumpz".into()))?
                    .to_string();
            Ok(JumpZ(label))
        }
        "jumpnotz" => {
            let label =
                s.1.ok_or_else(|| VMError::MissingArgument("jumpnotz".into()))?
                    .to_string();
            Ok(JumpNotZ(label))
        }
        "call" => {
            let label =
                s.1.ok_or_else(|| VMError::MissingArgument("call".into()))?
                    .to_string();
            Ok(Call(label))
        }
        "ret" => Ok(Ret),
        "halt" => Ok(Halt),
        _ => Err(VMError::InvalidInstruction(s.0.into())),
    }
}

fn run(instructions: Vec<Instruction>) -> Result<(), VMError> {
    use Instruction::*;

    let mut context = Context {
        stack: vec![],
        memory: vec![],
        pc: 0,
        call_stack: vec![],
    };

    while context.pc < instructions.len() {
        let ins = &instructions[context.pc];

        match ins {
            Push(v) => {
                context.stack.push(*v);
                context.pc += 1;
            }
            Pop => {
                context.stack.pop().ok_or(VMError::StackUnderflow)?;
                context.pc += 1;
            }
            Print => {
                if let Some(&value) = context.stack.last() {
                    println!("{}", value);
                } else {
                    return Err(VMError::StackUnderflow);
                }
                context.pc += 1;
            }
            Add => {
                let b = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                context.stack.push(a.wrapping_add(b));
                context.pc += 1;
            }
            Sub => {
                let b = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                context.stack.push(a.wrapping_sub(b));
                context.pc += 1;
            }
            Mul => {
                let b = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                context.stack.push(a.wrapping_mul(b));
                context.pc += 1;
            }
            Div => {
                let b = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                let a = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                context.stack.push(a / b);
                context.pc += 1;
            }
            Load => {
                let addr = context.stack.pop().ok_or(VMError::StackUnderflow)? as usize;
                let value = if addr < context.memory.len() {
                    *context
                        .memory
                        .get(addr)
                        .ok_or(VMError::MemoryAccessOutOfBounds(addr))?
                } else {
                    0
                };
                context.stack.push(value);
                context.pc += 1;
            }
            Store => {
                let addr = context.stack.pop().ok_or(VMError::StackUnderflow)? as usize;
                let value = context.stack.pop().ok_or(VMError::StackUnderflow)?;

                if addr >= context.memory.len() {
                    if addr + 1 > MAX_MEMORY_SIZE {
                        return Err(VMError::MemoryAccessOutOfBounds(addr));
                    }
                    context.memory.resize(addr + 1, 0);
                }

                context.memory[addr] = value;
                context.pc += 1;
            }
            JumpResolved(addr) => {
                if *addr >= instructions.len() {
                    return Err(VMError::UndefinedLabel(format!(
                        "Jump address {} is out of bounds",
                        addr
                    )));
                }
                context.pc = *addr;
            }
            JumpZResolved(addr) => {
                let cond = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                if cond == 0 {
                    if *addr >= instructions.len() {
                        return Err(VMError::UndefinedLabel(format!(
                            "Jump address {} is out of bounds",
                            addr
                        )));
                    }
                    context.pc = *addr;
                } else {
                    context.pc += 1;
                }
            }
            JumpNotZResolved(addr) => {
                let cond = context.stack.pop().ok_or(VMError::StackUnderflow)?;
                if cond != 0 {
                    if *addr >= instructions.len() {
                        return Err(VMError::UndefinedLabel(format!(
                            "Jump address {} is out of bounds",
                            addr
                        )));
                    }
                    context.pc = *addr;
                } else {
                    context.pc += 1;
                }
            }
            CallResolved(addr) => {
                if *addr >= instructions.len() {
                    return Err(VMError::UndefinedLabel(format!(
                        "Call address {} is out of bounds",
                        addr
                    )));
                }
                context.call_stack.push(context.pc + 1);
                context.pc = *addr;
            }
            Ret => {
                context.pc = context
                    .call_stack
                    .pop()
                    .ok_or(VMError::CallStackUnderflow)?;
            }
            Halt => {
                break;
            }
            Label(_) => {
                context.pc += 1;
            }
            Jump(_) | JumpZ(_) | JumpNotZ(_) | Call(_) => {
                return Err(VMError::InvalidInstruction(format!(
                    "Unresolved label at pc {}: {:?}",
                    context.pc, ins
                )));
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), VMError> {
    use Instruction::*;

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <instruction_file>", args[0]);
        eprintln!("Runs the specified instruction file in the stack-based VM (pancake).");
        std::process::exit(1);
    }

    let binding = std::fs::read_to_string(&args[1])?;
    let lines = binding
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect::<Vec<&str>>();

    let mut raw_instructions = Vec::new();

    let mut labels = HashMap::new();
    let mut pc = 0;
    for line in &lines {
        if let Some(label) = line.strip_suffix(':') {
            labels.insert(label.to_string(), pc);
            continue;
        } else {
            let mut parts = line.split_whitespace();
            let op = parts.next().unwrap();
            let arg = parts.next();
            let instr = map_op((op, arg))?;
            raw_instructions.push(instr);
            pc += 1;
        }
    }

    // resolve labels in instructions
    let mut instructions = Vec::new();
    for instr in raw_instructions {
        match instr {
            Jump(label) => {
                let addr = *labels
                    .get(&label)
                    .ok_or_else(|| VMError::UndefinedLabel(label.clone()))?;
                instructions.push(JumpResolved(addr));
            }
            JumpZ(label) => {
                let addr = *labels
                    .get(&label)
                    .ok_or_else(|| VMError::UndefinedLabel(label.clone()))?;
                instructions.push(JumpZResolved(addr));
            }
            JumpNotZ(label) => {
                let addr = *labels
                    .get(&label)
                    .ok_or_else(|| VMError::UndefinedLabel(label.clone()))?;
                instructions.push(JumpNotZResolved(addr));
            }
            Call(label) => {
                let addr = *labels
                    .get(&label)
                    .ok_or_else(|| VMError::UndefinedLabel(label.clone()))?;
                instructions.push(CallResolved(addr));
            }
            other => instructions.push(other),
        }
    }

    run(instructions)?;

    Ok(())
}
