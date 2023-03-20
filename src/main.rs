use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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
    let over: u8 = 128;
    println!("Over: {}", over as i8);
}
