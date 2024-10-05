use std::collections::HashMap;

use dessert::{map_op, run_vm, Instruction, VMError};

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

    run_vm(instructions)?;

    Ok(())
}
