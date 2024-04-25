use std::{collections::HashMap, fs::read_to_string, num::NonZeroU64};

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
    Push(u8),
    Pop,
    Print,

    // LOAD
    // STORE

    // Function definition
    Func(String),

    // Function call
    Call(String),
}

fn map_op(s: (&str, Option<&str>)) -> Instruction {
    use Instruction::*;

    match s.0 {
        s if s.starts_with('\'') => Func(s.replace('\'', "")),

        "push" => Push(
            s.1.map(|v| v.parse::<u8>().unwrap())
                .expect("Invalid push value"),
        ),
        "pop" => Pop,
        "print" => Print,
        "call" => Call(s.1.expect("Invalid function").to_string()),
        _ => {
            panic!("Invalid opcode {:?}", s.0)
        }
    }
}

fn run(instructions: Vec<Instruction>) {
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
            // Push to the top of the stack
            Push(v) => {
                context.stack.push(*v);
                context.pc += 1;
            }
            // Pop the last element of the stack
            Pop => {
                context.stack.pop();
                context.pc += 1;
            }
            // Output the last element of the stack
            Print => {
                println!("{}", context.stack.last().expect("Empty Stack"));
                context.pc += 1;
            }
            Func(name) => {
                labels.insert(name, context.pc);
                context.pc += 1;
            }
            Call(func) => {
                let pc = labels
                    .get(func.as_str())
                    .unwrap_or_else(|| panic!("No function {}", func));

                context.pc = *pc + 1;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // read out the file
    // go through each line
    // verify that each line has at most two strings separated by space
    // \n is operation separator
    // check if the first string is a valid operation
    // check the validity of the operation value

    let content = read_to_string("./examples/hello_world.pancake")?
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut line = line.split_whitespace();

            map_op((line.next().unwrap(), line.next()))
        })
        .collect::<Vec<Instruction>>();

    //println!("{:#?}", content);

    run(content);
    Ok(())
}
