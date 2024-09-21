use std::collections::HashMap;

type Stack = Vec<u8>;
type Memory = Vec<u8>;

#[derive(Clone, Debug)]
struct Context {
    pub stack: Stack,
    pub memory: Memory,
    pub pc: usize,
    pub call_stack: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    // Stack operations
    Push(u8),
    Pop,
    Print,

    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,

    // Memory operations
    Load,
    Store,

    // Control flow
    Jump(String),
    JumpZ(String), // Jump if zero
    JumpNotZ(String),
    Call(String),
    Ret,

    // Resolved control flow (after label resolution)
    JumpResolved(usize),
    JumpZResolved(usize),
    JumpNotZResolved(usize),
    CallResolved(usize),

    // Program control
    Halt,

    // For label definitions
    Label(String),
}

fn map_op(s: (&str, Option<&str>)) -> Instruction {
    use Instruction::*;

    match s.0 {
        "push" => Push(
            s.1.expect("missing push arg")
                .parse::<u8>()
                .expect("invalid push u8"),
        ),
        "pop" => Pop,
        "print" => Print,
        "add" => Add,
        "sub" => Sub,
        "mul" => Mul,
        "div" => Div,
        "load" => Load,
        "store" => Store,
        "jump" => Jump(s.1.expect("missing jump arg").to_string()),
        "jumpz" => JumpZ(s.1.expect("missing jumpz arg").to_string()),
        "jumpnotz" => JumpNotZ(s.1.expect("missing jumpnotz arg").to_string()),
        "call" => Call(s.1.expect("missing call arg").to_string()),
        "ret" => Ret,
        "halt" => Halt,
        _ => {
            if s.0.ends_with(':') {
                Label(s.0[..s.0.len() - 1].to_string())
            } else {
                panic!("Invalid instruction {:?}", s.0)
            }
        }
    }
}

fn run(instructions: Vec<Instruction>) -> Result<(), Box<dyn std::error::Error>> {
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
                context.stack.pop();
                context.pc += 1;
            }
            Print => {
                println!("{}", context.stack.last().expect("Empty Stack"));
                context.pc += 1;
            }
            Add => {
                let b = context.stack.pop().expect("Empty Stack");
                let a = context.stack.pop().expect("Empty Stack");
                context.stack.push(a.wrapping_add(b));
                context.pc += 1;
            }
            Sub => {
                let b = context.stack.pop().expect("Empty Stack");
                let a = context.stack.pop().expect("Empty Stack");
                context.stack.push(a.wrapping_sub(b));
                context.pc += 1;
            }
            Mul => {
                let b = context.stack.pop().expect("Empty Stack");
                let a = context.stack.pop().expect("Empty Stack");
                context.stack.push(a.wrapping_mul(b));
                context.pc += 1;
            }
            Div => {
                let b = context.stack.pop().expect("Empty Stack");
                let a = context.stack.pop().expect("Empty Stack");
                if b == 0 {
                    eprintln!("Division by zero");
                    std::process::exit(1);
                }
                context.stack.push(a / b);
                context.pc += 1;
            }
            Load => {
                let addr = context.stack.pop().expect("Empty Stack") as usize;
                let value = if addr < context.memory.len() {
                    context.memory[addr]
                } else {
                    0
                };
                context.stack.push(value);
                context.pc += 1;
            }
            Store => {
                let addr = context.stack.pop().expect("Empty Stack") as usize;
                let value = context.stack.pop().expect("Empty Stack");
                if addr >= context.memory.len() {
                    context.memory.resize(addr + 1, 0);
                }
                context.memory[addr] = value;
                context.pc += 1;
            }
            JumpResolved(addr) => {
                context.pc = *addr;
            }
            JumpZResolved(addr) => {
                let cond = context.stack.pop().expect("Empty Stack");
                if cond == 0 {
                    context.pc = *addr;
                } else {
                    context.pc += 1;
                }
            }
            JumpNotZResolved(addr) => {
                let cond = context.stack.pop().expect("Empty Stack");
                if cond != 0 {
                    context.pc = *addr;
                } else {
                    context.pc += 1;
                }
            }
            CallResolved(addr) => {
                context.call_stack.push(context.pc + 1);
                context.pc = *addr;
            }
            Ret => {
                context.pc = context.call_stack.pop().expect("Call stack underflow");
            }
            Halt => {
                break;
            }
            Label(_) => {
                // Labels are no-ops during execution
                context.pc += 1;
            }
            Jump(_) | JumpZ(_) | JumpNotZ(_) | Call(_) => {
                // These should have been resolved before execution
                panic!("Unresolved label at pc {}: {:?}", context.pc, ins);
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use Instruction::*;

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <instruction_file>", args[0]);
        eprintln!("Runs the specified instruction file in the stack-based VM (pancake).");
        std::process::exit(1)
    }

    // Read all lines
    let binding = std::fs::read_to_string(args[1].clone())?;
    let lines = binding
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect::<Vec<&str>>();

    let mut raw_instructions = Vec::new();

    // First pass: parse instructions and collect labels
    let mut labels = HashMap::new(); // Map<String, usize>
    let mut pc = 0;
    for line in lines {
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Some(label) = line.strip_suffix(':') {
            if labels.contains_key(&label) {
                panic!("Duplicate label: {}", label);
            }
            labels.insert(label.clone(), pc);
            raw_instructions.push(Instruction::Label(label.into()));
        } else {
            let mut parts = line.split_whitespace();
            let op = parts.next().unwrap();
            let arg = parts.next();
            let instr = map_op((op, arg));
            raw_instructions.push(instr);
            pc += 1;
        }
    }

    // Second pass: resolve labels in instructions
    let mut instructions = Vec::new();
    for instr in raw_instructions {
        match instr {
            Jump(label) => {
                let addr = *labels
                    .get(&label.as_str())
                    .unwrap_or_else(|| panic!("Undefined label: {}", label));
                instructions.push(JumpResolved(addr));
            }
            JumpZ(label) => {
                let addr = *labels
                    .get(&label.as_str())
                    .unwrap_or_else(|| panic!("Undefined label: {}", label));
                instructions.push(JumpZResolved(addr));
            }
            JumpNotZ(label) => {
                let addr = *labels
                    .get(&label.as_str())
                    .unwrap_or_else(|| panic!("Undefined label: {}", label));
                instructions.push(JumpNotZResolved(addr));
            }
            Call(label) => {
                let addr = *labels
                    .get(&label.as_str())
                    .unwrap_or_else(|| panic!("Undefined label: {}", label));
                instructions.push(CallResolved(addr));
            }
            _ => {
                instructions.push(instr);
            }
        }
    }

    run(instructions)
}
