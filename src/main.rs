use getopts::Options;
use std::{env, fs, process};

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

fn usage(progname: &str, opts: getopts::Options) {
    let brief = format!("Usage: {progname} [options] [file]");
    let usage = opts.usage(&brief);
    eprint!("{usage}");
}

fn collect_operations(content: String) -> Vec<Operation> {
    let mut ops: Vec<Operation> = vec![];
    let mut jps: Vec<i32> = vec![];
    let mut jp_counter = 0;

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

    ops
}

fn compile_operations(ops: Vec<Operation>, buffer_size: usize) -> String {
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

    asm
}

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();
    let progname = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print help and exit");
    opts.optopt("o", "output", "Output file name", "filename");
    opts.optopt("b", "buffersize", "Length of the brainf**k buffer", "30000");

    let matches = match opts.parse(&args[1..]) {
        Ok(x) => x,
        Err(x) => {
            eprintln!("{x}");
            usage(&progname, opts);
            return 1;
        }
    };

    if matches.opt_present("h") {
        usage(&progname, opts);
        return 0;
    }

    let output_path = match matches.opt_str("o") {
        Some(x) => x,
        None => String::from("a.asm"),
    };

    let buffer_size: usize = match matches.opt_str("b") {
        Some(x) => match x.parse() {
            Ok(x) => x,
            Err(x) => {
                eprintln!("{x}");
                return 1;
            }
        },
        None => 30000,
    };

    let file_path = match matches.free.get(0) {
        Some(x) => x,
        None => {
            eprintln!("No input file specified!");
            return 1;
        }
    };

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(x) => {
            eprintln!("Could not read file {file_path}:");
            eprintln!("{x}");
            return 1;
        }
    };

    let ops = collect_operations(content);
    let asm = compile_operations(ops, buffer_size);

    match fs::write(&output_path, asm) {
        Err(x) => {
            eprintln!("Failed to write assembly to {output_path}");
            eprintln!("{x}");
            return 1;
        }
        _ => {}
    }

    0
}

fn main() {
    process::exit(run());
}
