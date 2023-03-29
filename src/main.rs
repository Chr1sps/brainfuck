use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{env, io};
extern crate termios;
use std::os::unix::io::RawFd;
use termios::{tcsetattr, Termios, ICANON, TCSANOW};

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

fn main() {
    // let machine: BrainfuckMachine = BrainfuckMachine::new(1000, true);
    let args: Vec<String> = env::args().collect();
    // dbg!(&args);
    if args.len() == 2 {
        let filename = &args[1];
        let path = Path::new(filename);
        let display = path.display();
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(why) => panic!("Couldn't open {}: {}", display, why),
        };
        let mut data = String::new();
        if let Err(why) = file.read_to_string(&mut data) {
            panic!("Couldn't open {}: {}", display, why)
        };
        println!("Data: {}", data);
    }
    dbg!(&args);
    println!("{}", env::consts::OS);

    // this sets the console to only read a single char at once
    let termios = Termios::from_fd(0).unwrap();
    let mut new_termios = termios.clone();
    new_termios.c_lflag &= !(ICANON);
    tcsetattr(0, TCSANOW, &mut new_termios).unwrap();

    let stdout = io::stdout();
    let mut buffer = [0; 1];
    let mut reader = io::stdin();
    stdout.lock().flush().unwrap();
    reader.read_exact(&mut buffer).unwrap();
    println!("{:?}", buffer[0] as char);

    // this resets the console back to its normal state
    tcsetattr(0, TCSANOW, &termios).unwrap();
}
