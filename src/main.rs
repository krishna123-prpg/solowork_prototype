pub mod compiler;
pub mod interpret;
pub mod tokenize;

use crate::compiler::{Compiler, FileData};
use crate::interpret::Interpret;
use crate::tokenize::Lexer;
use std::env;
use std::process::exit;

fn usage(program: &str) -> ! {
    eprintln!("Usage: {program} <file>");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help     Show this help message");
    exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = "soloc"; 

    // Parse arguments
    let mut source_file: Option<String> = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => usage(program),
            flag if flag.starts_with('-') => {
                eprintln!("{program}: unknown flag '{flag}'");
                eprintln!("Try '{program} --help' for usage.");
                exit(1);
            }
            file => {
                if source_file.is_some() {
                    eprintln!("{program}: too many input files");
                    eprintln!("Try '{program} --help' for usage.");
                    exit(1);
                }
                source_file = Some(file.to_string());
            }
        }
    }

    let path = match source_file {
        Some(p) => p,
        None => {
            eprintln!("{program}: no input file");
            eprintln!("Try '{program} --help' for usage.");
            exit(1);
        }
    };

    // Register the file with the compiler (file_id = 1)
    let mut compiler = Compiler::new();
    compiler.add_file(FileData::new(1, &path));

    // Lex from file
    let lexer = match Lexer::from_file(&compiler, 1) {
        Some(l) => l,
        None => {
            eprintln!("{program}: cannot open '{path}': no such file or directory");
            exit(1);
        }
    };
    // Run
    let mut interpreter = Interpret::new(&compiler, lexer.collect());
    interpreter.run();
}
