use brainfuck::Interpreter;
use clap::Parser as ClapParser;
use std::{
    fmt::Debug,
    fs::File,
    io::{Error, ErrorKind, Result, Write},
};

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

    #[arg(short = 'O', long, value_name = "COUNT")]
    /// How many iterations of optimizing to run on the parsed code. Entering
    /// zero means that the optimizer will run until the code is fully
    /// optimized.
    optimize: Option<u32>,

    #[arg(default_value_t = false, short, long)]
    /// If set alongside the "--output" flag, outputs the data in binary
    /// format. Exclusive with "--hex".
    binary: bool,

    #[arg(default_value_t = false, short = 'H', long)]
    /// If set alongside the "--output" flag, outputs the data in hex format.
    /// Exclusive with "--binary".
    hex: bool,

    #[arg(short, long, value_name = "FILE")]
    /// Outputs the machine data to a given FILE. Use "--hex" and "--binary" to
    /// switch from ASCII encoding to other formats.
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
            if let Some(path) = args.output {
                let mut out_file = File::create(path)?;
                let tape = interpreter.get_tape();
                let tape_data = tape.as_slice();
                if args.binary && args.hex {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Binary and hex flags can't be set simultaneously.",
                    ));
                } else if args.binary {
                    out_file.write_all(tape_data)?;
                } else if args.hex {
                    for value in tape {
                        out_file.write_all(format!("0x{value:x}").as_bytes())?;
                        out_file.write_all(",".as_bytes())?;
                    }
                } else {
                    for value in tape {
                        out_file.write_all(value.to_string().as_bytes())?;
                        out_file.write_all(",".as_bytes())?;
                    }
                }
            }
            Ok(())
        }
        None => Err(Error::new(
            ErrorKind::Other,
            "Interactive mode not yet implemented.",
        )),
    }
}
