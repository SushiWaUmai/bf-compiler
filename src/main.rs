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

    for op in &ops {
        match op {
            &Operation::Add(x) => {
                asm += &format!("mov eax, DWORD [ebx]\n");
                asm += &format!("add al, {x}\n");
                asm += &format!("mov DWORD [ebx], eax\n");
            }
            &Operation::Sub(x) => {
                asm += &format!("mov eax, DWORD [ebx]\n");
                asm += &format!("sub al, {x}\n");
                asm += &format!("mov DWORD [ebx], eax\n");
            }
            &Operation::Next(x) => {
                asm += &format!("add ebx, {x}\n");
            }
            &Operation::Prev(x) => {
                asm += &format!("sub ebx, {x}\n");
            }
            &Operation::Out => {
                asm += "mov eax, 4\n";
                asm += "mov ebx, 1\n";
                asm += "mov ecx, DWORD [ebx]\n";
                asm += "mov edx, 1\n";
                asm += "int 0x80\n";
            }
            _ => {}
        }
    }

    fs::write("./test.asm", asm).unwrap();

    println!("{ops:?}");
}
