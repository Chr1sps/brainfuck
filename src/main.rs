use std::io::{Error, ErrorKind, Result};
extern crate termios;
use brainfuck::Interpreter;
use clap::Parser as ClapParser;

// pub mod brainfuck;
// use crate::brainfuck::BrainfuckMachine;

// struct CLIOption<T> {
//     name: String,
//     value: T,
// }
//
// impl CLIOption<String> {
//     fn make_option(name: String, value: String) -> Self <String> {
//         Self {
//             name,
//             value: value.strip_prefix("--").unwrap().to_string(),
//         }
//     }
// }
//
// impl<T> CLIOption<T> where T: num::Integer {
//     fn make_option(name: String, value: String) -> Self <T> {
//         Self {
//             name,
//             value: value.trim().parse(),
//         }
//     }
// }
//

// CLI args grammar:
// args := args_with_filename | options
//
// args_with_filename := (options ' ')? filename (' ' options)?
//
// options := option (' ' option)*
//
// option := '--' name (' ' value)?
//
// filename := $ dozwolona nazwa pliku w danym systemie $
// name := $ dowolna pasująca nazwa $
// value := $ dowolna pasująca wartość $

#[derive(ClapParser)]
#[command(name = "Brainfuck interpreter")]
#[command(author = "Chr1sps")]
#[command(version = "1.0")]
#[command(about = "Brainfuck interpreter for unix based systems.", long_about = None)]
struct Cli {
    /// Number of cells that the tape has.
    #[arg(short, long, value_name = "SIZE")]
    size: Option<usize>,

    /// Name of the file to open.
    file: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match &args.file {
        Some(file_name) => {
            let size = args.size.unwrap_or(30000);
            let mut interpreter = Interpreter::from_file(&file_name, size)?;
            interpreter.parse_and_run()
        }
        None => Err(Error::new(
            ErrorKind::NotFound,
            "No file has been specified",
        )),
    }
}
