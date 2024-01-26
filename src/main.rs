use getopts::Options;
mod operations;
use operations::*;
use std::{env, fs, process};

fn usage(progname: &str, opts: getopts::Options) {
    let brief = format!("Usage: {progname} [options] [file]");
    let usage = opts.usage(&brief);
    eprint!("{usage}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_operations() {
        let result = collect_operations("+++".to_string());
        let expected = vec![Operation::Add(3)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sub_operations() {
        let result = collect_operations("---".to_string());
        let expected = vec![Operation::Sub(3)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_next_prev_operations() {
        let result = collect_operations(">>><<<".to_string());
        let expected = vec![Operation::Next(3), Operation::Prev(3)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_out_in_operations() {
        let result = collect_operations(".,".to_string());
        let expected = vec![Operation::Out, Operation::In];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_loop_operations() {
        let result = collect_operations("[->+<]".to_string());
        let expected = vec![
            Operation::BeginLoop(0),
            Operation::Sub(1),
            Operation::Next(1),
            Operation::Add(1),
            Operation::Prev(1),
            Operation::EndLoop(0),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn test_combined_operations() {
        let result = collect_operations("++>[-<+>]<.".to_string());
        let expected = vec![
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
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_input() {
        let result = collect_operations("".to_string());
        let expected: Vec<Operation> = vec![];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_operations() {
        let result = collect_operations("+-><,.".to_string());
        let expected = vec![
            Operation::Add(1),
            Operation::Sub(1),
            Operation::Next(1),
            Operation::Prev(1),
            Operation::In,
            Operation::Out,
        ];
        assert_eq!(result, expected);
    }
}
