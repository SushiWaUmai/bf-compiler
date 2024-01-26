use std::fs;

#[derive(Debug, PartialEq)]
enum Operation {
    Add(usize),
    Sub(usize),
    Next(usize),
    Prev(usize),
    BeginLoop(i32),
    EndLoop(i32),
    Out,
    In,
}

fn main() {
    let content = fs::read_to_string("./test.bf").unwrap();
    let mut ops: Vec<Operation> = vec![];
    let mut jps: Vec<i32> = vec![];
    let mut jp_counter = 0;

    let buffer_size = 30000;

    let mut current_char = ' ';
    let mut repetition_count = 1;

    for c in content.chars() {
        if c == current_char {
            match c {
                '+' | '-' | '>' | '<' => {
                    repetition_count += 1;
                    continue;
                }
                _ => {}
            }
        }

        let op = match current_char {
            '+' => Some(Operation::Add(repetition_count)),
            '-' => Some(Operation::Sub(repetition_count)),
            '>' => Some(Operation::Next(repetition_count)),
            '<' => Some(Operation::Prev(repetition_count)),
            _ => None,
        };
        if let Some(op) = op {
            ops.push(op);
        }

        let op = match c {
            '.' => Some(Operation::Out),
            ',' => Some(Operation::In),
            '[' => {
                jps.push(jp_counter);
                jp_counter += 1;
                Some(Operation::BeginLoop(jp_counter - 1))
            }
            ']' => {
                let c = jps.pop().unwrap();
                Some(Operation::EndLoop(c))
            }
            _ => None,
        };
        if let Some(op) = op {
            ops.push(op);
        }

        repetition_count = 1;
        current_char = c;
    }

    let op = match current_char {
        '+' => Some(Operation::Add(repetition_count)),
        '-' => Some(Operation::Sub(repetition_count)),
        '>' => Some(Operation::Next(repetition_count)),
        '<' => Some(Operation::Prev(repetition_count)),
        _ => None,
    };
    if let Some(op) = op {
        ops.push(op);
    }

    let mut asm = String::new();

    asm += "format ELF64 executable\n";
    asm += "segment readable executable\n";
    asm += "entry main\n";
    asm += "define SYS_exit     60\n";
    asm += "define SYS_write    1\n";
    asm += "define SYS_read     0\n";
    asm += "define stdout       1\n";
    asm += "define stdin        0\n";
    asm += "define exit_success 0\n";

    asm += "main:\n";
    asm += &format!("mov rbx, {buffer_size}\n", buffer_size = buffer_size / 2);

    for op in &ops {
        match op {
            &Operation::Add(x) => {
                asm += &format!("add byte[buf+rbx], {x}\n");
            }
            &Operation::Sub(x) => {
                asm += &format!("sub byte [buf+rbx], {x}\n");
            }
            &Operation::Next(x) => {
                asm += &format!("add rbx, {x}\n");
            }
            &Operation::Prev(x) => {
                asm += &format!("sub rbx, {x}\n");
            }
            &Operation::BeginLoop(x) => {
                asm += &format!("cmp byte[buf+rbx], 0\n");
                asm += &format!("je .EndLoop{x}\n");
                asm += &format!(".BeginLoop{x}:\n");
            }
            &Operation::EndLoop(x) => {
                asm += &format!("cmp byte[buf+rbx], 0\n");
                asm += &format!("jne .BeginLoop{x}\n");
                asm += &format!(".EndLoop{x}:\n");
            }
            &Operation::Out => {
                asm += "lea rcx, [buf+rbx]\n";

                asm += "mov rax, SYS_write\n";
                asm += "mov rdi, stdout\n";
                asm += "mov rsi, rcx\n";
                asm += "mov rdx, 1\n";
                asm += "syscall\n";
            }
            &Operation::In => {
                asm += "lea rcx, [buf+rbx]\n";

                asm += "mov rax, SYS_read\n";
                asm += "mov rdi, stdin\n";
                asm += "mov rsi, rcx\n";
                asm += "mov rdx, 1\n";
                asm += "syscall\n";
            }
        }
    }

    asm += "mov rax, SYS_exit\n";
    asm += "mov rdi, exit_success\n";
    asm += "syscall\n";

    asm += "segment readable writable\n";
    asm += &format!("buf: rb {buffer_size}\n");

    fs::write("./test.asm", asm).unwrap();

    println!("{ops:?}");
}
