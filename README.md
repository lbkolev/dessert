# Dessert
16bit stack-based VM ([Pancake](#pancake)) & a compiler targeting it ([Syrup](#syrup))

To run raw instructions through the Virtual Machine:
```
cargo r --pancake -- <path to example file>
```

Few examples can be found at [./examples](./examples/) directory.
```
cargo r --pancake -- examples/fibonacci.pancake
cargo r --pancake -- examples/factorial.pancake
cargo r --pancake -- examples/loop_through.pancake
```

## Pancake
Supported instructions
| **Instruction**      | **Arguments**    | **Description**                                                           |
|----------------------|------------------|---------------------------------------------------------------------------|
| **Swap**             | None             | Swap the top two elements of the stack.                                   |
| **Push**             | `u16` value      | Push a 16-bit unsigned integer onto the stack.                            |
| **Pop**              | None             | Remove the top element from the stack.                                    |
| **Print**            | None             | Print the top element of the stack.                                       |
| **Add**              | None             | Pop the top two elements, add them, and push the result.                  |
| **Sub**              | None             | Pop the top two elements, subtract the second from the first, and push the result. |
| **Mul**              | None             | Pop the top two elements, multiply them, and push the result.             |
| **Div**              | None             | Pop the top two elements, divide the first by the second, and push the result. |
| **Load**             | None             | Load a value from memory onto the stack.                                  |
| **Store**            | None             | Store the top value from the stack into memory.                           |
| **Jump**             | `String` label   | Jump to the specified label.                                              |
| **JumpZ**            | `String` label   | Jump to the specified label if the top of the stack is zero.              |
| **JumpNotZ**         | `String` label   | Jump to the specified label if the top of the stack is not zero.          |
| **Call**             | `String` label   | Call the subroutine at the specified label.                               |
| **Ret**              | None             | Return from a subroutine.                                                 |
| **JumpResolved**     | `usize` address  | Jump to the specified memory address.                                     |
| **JumpZResolved**    | `usize` address  | Jump to the memory address if the top of the stack is zero.               |
| **JumpNotZResolved** | `usize` address  | Jump to the memory address if the top of the stack is not zero.           |
| **CallResolved**     | `usize` address  | Call the subroutine at the specified memory address.                      |
| **Halt**             | None             | Stop program execution.                                                   |
| **Label**            | `String` name    | Define a label with the specified name.                                   |

## Syrup
