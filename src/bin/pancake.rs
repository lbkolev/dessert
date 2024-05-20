use std::collections::HashMap;

type Stack = Vec<u8>;
type Memory = Vec<u8>;

#[derive(Clone, Debug)]
struct Context {
    pub stack: Stack,
    pub memory: Memory,
    pub pc: u64,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    // Pushes a value onto the stack.
    Push(u8),
    // Removes the top value from the stack.
    Pop,
    // Prints the top value of the stack.
    Print,

    // Arithmetic operations on the top values of the stack.
    Add,
    Sub,
    Mul,
    Div,

    // Loads a value from memory onto the stack
    Load,
    // Stores the value from the stack into memory
    Store,

    // Jumps to the instruction /used by loops/
    Jump(usize),

    // /Optionally/ Designates the end of the program execution
    Halt,
}

fn map_op(s: (&str, Option<&str>)) -> Instruction {
    use Instruction::*;

    match s.0 {
        "push" => Push(
            s.1.expect("missing push arg")
                .parse::<u8>()
                .expect("invalud push u8"),
        ),
        "pop" => Pop,
        "print" => Print,
        "add" => Add,
        "sub" => Sub,
        "mul" => Mul,
        "div" => Div,
        "load" => Load,
        "store" => Store,
        "jump" => Jump(
            s.1.expect("missing jump arg")
                .parse::<usize>()
                .expect("invalid jump usize"),
        ),
        "halt" => Halt,
        _ => {
            panic!("Invalid instruction {:?}", s.0)
        }
    }
}

fn run(instructions: Vec<Instruction>) -> Result<(), Box<dyn std::error::Error>> {
    use Instruction::*;

    let mut context = Context {
        stack: vec![],
        memory: vec![],
        pc: 0,
    };
    let mut labels: HashMap<&str, u64> = HashMap::new();

    while context.pc < instructions.len() as u64 {
        let ins = &instructions[context.pc as usize];

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
                let r = context.stack.pop().expect("Empty Stack")
                    + context.stack.pop().expect("Empty Stack");
                context.stack.push(r);
                context.pc += 1;
            }
            Sub => {
                let r = context.stack.pop().expect("Empty Stack")
                    - context.stack.pop().expect("Empty Stack");
                context.stack.push(r);
                context.pc += 1;
            }
            Mul => {
                let r = context.stack.pop().expect("Empty Stack")
                    * context.stack.pop().expect("Empty Stack");
                context.stack.push(r);
                context.pc += 1;
            }
            Div => {
                let r = context.stack.pop().expect("Empty Stack")
                    / context.stack.pop().expect("Empty Stack");
                context.stack.push(r);
                context.pc += 1;
            }
            Load => {
                let r = context.memory.pop().expect("Empty memory");
                context.stack.push(r);
                context.pc += 1;
            }
            Store => {
                let r = context.stack.pop().expect("Empty stack");
                context.memory.push(r);
                context.pc += 1;
            }
            Halt => {
                std::process::exit(0);
            }
            Jump(_) => {
                eprintln!("Jump Instruction set not implemented");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <instruction_file>", args[0]);
        eprintln!("Runs the specified instruction file in the stack-based VM (pancake).");
        std::process::exit(1)
    }

    let content = std::fs::read_to_string(args[1].clone())?
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut line = line.split_whitespace();

            map_op((line.next().unwrap(), line.next()))
        })
        .collect::<Vec<Instruction>>();

    run(content)
}
