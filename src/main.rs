use std::fs;

#[derive(Debug)]
enum Operation {
    Add(usize),
    Sub(usize),
    Next(usize),
    Prev(usize),
    BeginLoop(i32),
    EndLoop(i32),
    Out(usize),
    In(usize),
}

fn main() {
    let content = fs::read_to_string("./test.bf").unwrap();
    let mut ops: Vec<Operation> = vec![];
    let mut jps: Vec<i32> = vec![];
    let mut counter = 0;

    let buffer_size = 30000;

    for c in content.chars() {
        let op = match c {
            '+' => Operation::Add(1),
            '-' => Operation::Sub(1),
            '>' => Operation::Next(1),
            '<' => Operation::Prev(1),
            '[' => {
                jps.push(counter);
                counter += 1;
                Operation::BeginLoop(counter - 1)
            }
            ']' => {
                let c = jps.pop().unwrap();
                Operation::EndLoop(c)
            }
            '.' => Operation::Out(1),
            ',' => Operation::In(1),
            _ => continue,
        };

        ops.push(op);
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
    asm += &format!("mov ebx, {buffer_size}\n", buffer_size = buffer_size / 2);

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
                asm += &format!("cmp byte[buf+ebx], 0\n");
                asm += &format!("je .EndLoop{x}\n");
                asm += &format!(".BeginLoop{x}:\n");
            }
            &Operation::EndLoop(x) => {
                asm += &format!("cmp byte[buf+ebx], 0\n");
                asm += &format!("jne .BeginLoop{x}\n");
                asm += &format!(".EndLoop{x}:\n");
            }
            &Operation::Out(x) => {
                asm += "lea ecx, [buf+ebx]\n";

                asm += "mov eax, SYS_write\n";
                asm += "mov edi, stdout\n";
                asm += "mov esi, ecx\n";
                asm += &format!("mov edx, {x}\n");
                asm += "syscall\n";
            }
            _ => {}
        }
    }

    asm += "mov eax, SYS_exit\n";
    asm += "mov edi, exit_success\n";
    asm += "syscall\n";

    asm += "segment readable writable\n";
    asm += &format!("buf: rb {buffer_size}\n");

    fs::write("./test.asm", asm).unwrap();

    println!("{ops:?}");
}
