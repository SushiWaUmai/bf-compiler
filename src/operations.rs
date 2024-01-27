#![allow(dead_code)]

use std::{str::FromStr, vec};

#[derive(Debug, Eq, PartialEq)]
pub enum Operation {
    Add(usize),
    Sub(usize),
    Next(usize),
    Prev(usize),
    BeginLoop(i32),
    EndLoop(i32),
    Out,
    In,
    Nop,
}

impl Operation {
    pub fn as_assembly(&self) -> String {
        match self {
            Operation::Add(x) => format!("add byte[buf+rbx], {x}\n"),
            Operation::Sub(x) => format!("sub byte [buf+rbx], {x}\n"),
            Operation::Next(x) => format!("add rbx, {x}\n"),
            Operation::Prev(x) => format!("sub rbx, {x}\n"),
            Operation::BeginLoop(x) => {
                String::from("cmp byte[buf+rbx], 0\n")
                    + &format!("je .EndLoop{x}\n")
                    + &format!(".BeginLoop{x}:\n")
            }
            Operation::EndLoop(x) => {
                String::from("cmp byte[buf+rbx], 0\n")
                    + &format!("jne .BeginLoop{x}\n")
                    + &format!(".EndLoop{x}:\n")
            }
            Operation::Out => {
                String::from("lea rcx, [buf+rbx]\n")
                    + "mov rax, SYS_write\n"
                    + "mov rdi, stdout\n"
                    + "mov rsi, rcx\n"
                    + "mov rdx, 1\n"
                    + "syscall\n"
            }
            Operation::In => {
                String::from("lea rcx, [buf+rbx]\n")
                    + "mov rax, SYS_read\n"
                    + "mov rdi, stdin\n"
                    + "mov rsi, rcx\n"
                    + "mov rdx, 1\n"
                    + "syscall\n"
            }
            Operation::Nop => String::from("nop\n"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Operations(Vec<Operation>);
impl Operations {
    pub fn new(op: Vec<Operation>) -> Self {
        Self(op)
    }
    pub fn as_assembly(&self) -> String {
        self.0
            .iter()
            .map(|op| op.as_assembly())
            .fold(String::new(), |acc, s| acc + &s)
    }

    pub fn push_op(&mut self, operation: Operation) {
        match operation {
            Operation::Add(x) => {
                if let Operation::Add(y) = self.0.last_mut().unwrap() {
                    *y += x;
                } else {
                    self.0.push(operation);
                }
            }
            Operation::Sub(x) => {
                if let Operation::Sub(y) = self.0.last_mut().unwrap() {
                    *y += x;
                } else {
                    self.0.push(operation);
                }
            }
            Operation::Next(x) => {
                if let Operation::Next(y) = self.0.last_mut().unwrap() {
                    *y += x;
                } else {
                    self.0.push(operation);
                }
            }
            Operation::Prev(x) => {
                if let Operation::Prev(y) = self.0.last_mut().unwrap() {
                    *y += x;
                } else {
                    self.0.push(operation);
                }
            }
            _ => self.0.push(operation),
        }
    }
}

impl FromStr for Operations {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut operations: Operations = Self(vec![Operation::Nop]);

        let mut jp_counter = 0;
        let mut jumps: Vec<i32> = vec![];

        s.chars().fold(&mut operations, move |acc, c| {
            match c {
                '+' => acc.push_op(Operation::Add(1)),
                '-' => acc.push_op(Operation::Sub(1)),
                '>' => acc.push_op(Operation::Next(1)),
                '<' => acc.push_op(Operation::Prev(1)),
                '.' => acc.push_op(Operation::Out),
                ',' => acc.push_op(Operation::In),
                '[' => {
                    jumps.push(jp_counter);
                    jp_counter += 1;
                    acc.push_op(Operation::BeginLoop(jp_counter - 1))
                }
                ']' => {
                    let c = jumps.pop().unwrap();
                    acc.push_op(Operation::EndLoop(c))
                }
                _ => (),
            };
            acc
        });
        operations.0.remove(0);
        Ok(operations)
    }
}

pub struct Program {
    operations: Operations,
}

impl Program {
    pub fn new(operations: Operations) -> Result<Self, ()> {
        // TODO Semantic checking, for example unclosed loops etc.
        Ok(Self { operations })
    }

    pub fn transpile(&self, prelude: String, postlude: String) -> String {
        prelude + &self.operations.as_assembly() + &postlude
    }
}

pub fn compile_program(program: Program, buffer_size: usize) -> String {
    let mut prelude = String::from(concat!(
        "format ELF64 executable\n",
        "segment readable executable\n",
        "entry main\n",
        "define SYS_exit     60\n",
        "define SYS_write    1\n",
        "define SYS_read     0\n",
        "define stdout       1\n",
        "define stdin        0\n",
        "define exit_success 0\n",
        "main:\n",
    ));
    prelude += &format!("mov rbx, {buffer_size}\n", buffer_size = buffer_size / 2);

    let mut postlude = String::from(concat!(
        "mov rax, SYS_exit\n",
        "mov rdi, exit_success\n",
        "syscall\n",
        "segment readable writable\n",
    ));
    postlude += &format!("buf: rb {buffer_size}\n");
    program.transpile(prelude, postlude)
}

#[cfg(test)]
mod collect_operations_tests {
    use super::*;

    #[test]
    fn test_collect_operations_simple_program() {
        let input = "++-->";
        let expected = Operations(vec![
            Operation::Add(2),
            Operation::Sub(2),
            Operation::Next(1),
        ]);
        assert_eq!(Operations::from_str(input).unwrap(), expected);
    }

    #[test]
    fn test_collect_operations_program_with_loops() {
        let input = "++[>++<-]";
        let expected = Operations(vec![
            Operation::Add(2),
            Operation::BeginLoop(0),
            Operation::Next(1),
            Operation::Add(2),
            Operation::Prev(1),
            Operation::Sub(1),
            Operation::EndLoop(0),
        ]);
        assert_eq!(Operations::from_str(input).unwrap(), expected);
    }

    #[test]
    fn test_add_operations() {
        let result = Operations::from_str("+++").unwrap();
        let expected = Operations(vec![Operation::Add(3)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sub_operations() {
        let result = Operations::from_str("---").unwrap();
        let expected = Operations(vec![Operation::Sub(3)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_next_prev_operations() {
        let result = Operations::from_str(">>><<<").unwrap();
        let expected = Operations(vec![Operation::Next(3), Operation::Prev(3)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_out_in_operations() {
        let result = Operations::from_str(".,").unwrap();
        let expected = Operations(vec![Operation::Out, Operation::In]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_loop_operations() {
        let result = Operations::from_str("[->+<]").unwrap();
        let expected = Operations(vec![
            Operation::BeginLoop(0),
            Operation::Sub(1),
            Operation::Next(1),
            Operation::Add(1),
            Operation::Prev(1),
            Operation::EndLoop(0),
        ]);
        assert_eq!(result, expected);
    }
    #[test]
    fn test_combined_operations() {
        let result = Operations::from_str("++>[-<+>]<.").unwrap();
        let expected = Operations(vec![
            Operation::Add(2),
            Operation::Next(1),
            Operation::BeginLoop(0),
            Operation::Sub(1),
            Operation::Prev(1),
            Operation::Add(1),
            Operation::Next(1),
            Operation::EndLoop(0),
            Operation::Prev(1),
            Operation::Out,
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_input() {
        let result = Operations::from_str("").unwrap();
        let expected = Operations(vec![]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_operations() {
        let result = Operations::from_str("+-><,.").unwrap();
        let expected = Operations(vec![
            Operation::Add(1),
            Operation::Sub(1),
            Operation::Next(1),
            Operation::Prev(1),
            Operation::In,
            Operation::Out,
        ]);
        assert_eq!(result, expected);
    }
}
