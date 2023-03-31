use brainfuck::Interpreter;
use clap::Parser as ClapParser;
use std::io::{Error, ErrorKind, Result};

#[derive(ClapParser, Debug)]
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

    #[arg(short, long, value_name = "COUNT")]
    /// How many iterations of optimizing to run on the parsed code. Entering
    /// zero means that the optimizer will run until the code is fully
    /// optimized.
    optimize: Option<u32>,

    #[arg(default_value_t = false, long)]
    /// Outputs the machine binary data. Exclusive with "--hex". TODO
    binary: bool,

    #[arg(default_value_t = false, long)]
    /// Outputs the machine hex data. Exclusive with "--binary". TODO
    hex: bool,

    #[arg(long, value_name = "FILE")]
    /// Outputs the machine data to a given FILE. Use "--hex" and "--binary" to
    /// switch from ASCII encoding to other formats. TODO
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match &args.file {
        Some(file_name) => {
            let size = args.size.unwrap_or(30000);
            let mut interpreter = Interpreter::from_file(&file_name, size)?;
            if let Some(value) = args.optimize {
                interpreter.run_with_optimization(value)?;
            } else {
                interpreter.run()?;
            }
            Ok(())
        }
        None => Err(Error::new(
            ErrorKind::NotFound,
            "No file has been specified",
        )),
    }
}
