use std::fs;

#[derive(Debug)]
enum Operation {
    Add(i32),
    Sub(i32),
    Next(i32),
    Prev(i32),
    BeginLoop(i32),
    EndLoop(i32),
    In,
    Out,
}

fn main() {
    let content = fs::read_to_string("./test.bf").unwrap();
    let mut ops: Vec<Operation> = vec![];
    let mut counter = 0;

    for c in content.chars() {
        match c {
            '+' => ops.push(Operation::Add(1)),
            '-' => ops.push(Operation::Sub(1)),
            '>' => ops.push(Operation::Next(1)),
            '<' => ops.push(Operation::Prev(1)),
            '[' => {
                ops.push(Operation::BeginLoop(counter));
                counter += 1;
            }
            ']' => {
                ops.push(Operation::EndLoop(counter));
                counter -= 1;
            }
            '.' => ops.push(Operation::Out),
            ',' => ops.push(Operation::In),
            _ => {}
        }
    }

    let mut asm = String::new();

    asm += "format ELF64 executable\n";
    asm += "segment readable executable\n";
    asm += "entry main\n";
    asm += "define SYS_exit     60\n";
    asm += "define SYS_write    1\n";
    asm += "define stdout       1\n";
    asm += "define exit_success 0\n";

    asm += "main:\n";

    for op in &ops {
        match op {
            &Operation::Add(x) => {
                asm += &format!("add byte[buf+ebx], {x}\n");
            }
            &Operation::Sub(x) => {
                asm += &format!("sub byte [buf+ebx], {x}\n");
            }
            &Operation::Next(x) => {
                asm += &format!("add ebx, {x}\n");
            }
            &Operation::Prev(x) => {
                asm += &format!("sub ebx, {x}\n");
            }
            &Operation::BeginLoop(x) => {
                asm += &format!(".BeginLoop{x}:\n");
                asm += &format!("jz .EndLoop{x}\n");
            }
            &Operation::EndLoop(x) => {
                asm += &format!(".EndLoop{x}:\n");
                asm += &format!("jnz .BeginLoop{x}\n");
            }
            &Operation::Out => {
                asm += "mov eax, SYS_write\n";
                asm += "mov edi, stdout\n";
                asm += "mov esi, DWORD [buf+ebx]\n";
                asm += "mov edx, 1\n";
                asm += "syscall\n";
            }
            _ => {}
        }
    }

    asm += "mov eax, SYS_exit\n";
    asm += "mov edi, exit_success\n";
    asm += "syscall\n";

    asm += "segment readable writable\n";
    asm += "buf rb 30000\n";

    fs::write("./test.asm", asm).unwrap();

    println!("{ops:?}");
}
