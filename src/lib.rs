// #[cfg(test)]
// mod tests;

use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Error, ErrorKind, Read, Result, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

use termios::Termios;

#[cfg(test)]
mod tests;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Token {
    // post-lexing, pre-optimization tokens
    Increment,
    Decrement,
    ShiftLeft,
    ShiftRight,
    StartLoop,
    EndLoop,
    // io tokens
    PutChar,
    ReadChar,
}

impl Token {
    fn is_countable(&self) -> bool {
        use self::Token::*;
        matches!(self, &(Increment | Decrement | ShiftLeft | ShiftRight))
    }
    fn is_loop(&self) -> bool {
        use self::Token::*;
        matches!(self, &(StartLoop | EndLoop))
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Statement {
    MoveLeft(usize),
    MoveRight(usize),

    Add(u8),
    Substract(u8),

    JumpIf(usize),
    PutChar,
    ReadChar,
}

impl Statement {
    fn is_equal_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
    fn changes_value(&self) -> bool {
        matches!(self, &(Statement::Add(_) | Statement::Substract(_)))
    }
    fn moves_header(&self) -> bool {
        matches!(self, &(Statement::MoveLeft(_) | Statement::MoveRight(_)))
    }
}
pub struct BrainfuckMachine {
    /// Size of the tape vector.
    size: usize,
    /// Current cell index.
    index: usize,
    /// Tape vector.
    tape: Vec<u8>,
}

impl BrainfuckMachine {
    /// Creates a `BrainfuckMachine` instance of given tape size.
    pub fn new(size: usize) -> Self {
        let mut result = Self {
            size,
            index: 0,
            tape: Vec::new(),
        };
        result.tape.resize(size, 0);
        result
    }

    /// Moves the header left by a given amount. Panics when the index is out
    /// of bounds.
    pub fn move_left(&mut self, shift: usize) {
        match shift.cmp(&(self.index)) {
            Ordering::Greater => panic!(
                "Index out of bounds.
Index before move: {}.
Left shift value: {}.
",
                self.index, shift,
            ),
            _ => self.index -= shift,
        }
    }
    /// Moves the header right by a given amount. Panics when the index is out
    /// of bounds.
    pub fn move_right(&mut self, shift: usize) {
        match shift.cmp(&(self.size - self.index)) {
            Ordering::Greater => panic!(
                "Index out of bounds.
Index before move: {}.
Right shift value: {}.
Max possible index: {}.
",
                self.index,
                shift,
                self.size - 1
            ),
            _ => self.index += shift,
        }
    }

    /// Adds a given value to the current cell, with wrapping.
    pub fn add(&mut self, value: u8) {
        let current = self.tape[self.index];
        self.tape[self.index] = current.wrapping_add(value);
    }

    /// Substracts a given value to the current cell, with wrapping.
    pub fn substract(&mut self, value: u8) {
        let current = self.tape[self.index];
        self.tape[self.index] = current.wrapping_sub(value);
    }

    /// Inserts a given char's ASCII value into the current cell.
    pub fn read_char(&mut self, input: char) {
        self.tape[self.index] = input as u8
    }

    /// Returns the current cell's value ASCII encoded into a char.
    pub fn put_char(&self) -> char {
        self.tape[self.index] as char
    }

    /// Returns `true` if the current cell's value is non-zero.
    pub fn check_loop(&self) -> bool {
        self.tape[self.index] != 0
    }
}

// Brainfuck grammar:
// code := (stmt_block)*
//
// stmt_block := stmt | loop
//
// loop := '[' stmt_block+ ']'
//
// stmt := '+' | '-' | '<' | '>' | ',' | '.'
struct Lexer<T: BufRead> {
    reader: T,
}

impl<T: BufRead> Lexer<T> {
    pub fn from_reader(reader: T) -> Self {
        Self { reader }
    }
    fn next_token(&mut self) -> Option<Token> {
        let mut buf: [u8; 1] = [0];
        match self.reader.read(&mut buf) {
            Err(msg) => {
                panic!("Error when reading a token: {}", msg);
            }
            Ok(0) => None,
            Ok(_) => {
                let ascii = buf[0];
                let to_token = ascii as char;
                Self::tokenize(&to_token)
            }
        }
    }
    fn eof(&mut self) -> bool {
        match self.reader.fill_buf() {
            Ok(buf) => buf.is_empty(),
            Err(msg) => {
                panic!("EOF check failed: {}", msg);
            }
        }
    }
    fn tokenize(input: &char) -> Option<Token> {
        use crate::Token::*;

        match input {
            '+' => Some(Increment),
            '-' => Some(Decrement),
            '<' => Some(ShiftLeft),
            '>' => Some(ShiftRight),
            ',' => Some(ReadChar),
            '.' => Some(PutChar),
            '[' => Some(StartLoop),
            ']' => Some(EndLoop),
            _ => None,
        }
    }
    fn iter(self) -> LexerIter<T> {
        LexerIter { lexer: self }
    }
    fn ref_iter(&mut self) -> LexerRefIter<T> {
        LexerRefIter { lexer: self }
    }
}

struct LexerIter<T: BufRead> {
    lexer: Lexer<T>,
}

impl<T: BufRead> Iterator for LexerIter<T> {
    type Item = Option<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.eof() {
            true => None,
            false => Some(self.lexer.next_token()),
        }
    }
}

impl<T: BufRead> IntoIterator for Lexer<T> {
    type Item = Option<Token>;
    type IntoIter = LexerIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

struct LexerRefIter<'a, T: BufRead> {
    lexer: &'a mut Lexer<T>,
}

impl<'a, T: BufRead> Iterator for LexerRefIter<'a, T> {
    type Item = Option<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.eof() {
            true => None,
            false => Some(self.lexer.next_token()),
        }
    }
}

impl<'a, T: BufRead> IntoIterator for &'a mut Lexer<T> {
    type Item = Option<Token>;
    type IntoIter = LexerRefIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.ref_iter()
    }
}
pub struct Parser<T: BufRead> {
    lexer: Lexer<T>,
}

impl<T: BufRead> Parser<T> {
    fn from_lexer(lexer: Lexer<T>) -> Self {
        Self { lexer }
    }
    fn from_reader(reader: T) -> Self {
        Self::from_lexer(Lexer { reader })
    }
    fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut result = Vec::new();
        let mut loop_stack: Vec<usize> = Vec::new();
        for opt_token in &mut self.lexer {
            match opt_token {
                Some(token) => match token {
                    Token::Increment => result.push(Statement::Add(1)),
                    Token::Decrement => result.push(Statement::Substract(1)),
                    Token::ShiftLeft => result.push(Statement::MoveLeft(1)),
                    Token::ShiftRight => result.push(Statement::MoveRight(1)),
                    Token::PutChar => result.push(Statement::PutChar),
                    Token::ReadChar => result.push(Statement::ReadChar),
                    Token::StartLoop => loop_stack.push(result.len()),
                    Token::EndLoop => {
                        let address_opt = loop_stack.pop();
                        match address_opt {
                            Some(address) if address == result.len() => {}
                            Some(address) => {
                                result.push(Statement::JumpIf(address));
                            }
                            None => {
                                // return Error("Error: ']' found with no matching '['.".to_string())
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "Error: ']' found with no matching '['.".to_string(),
                                ));
                            }
                        }
                    }
                },
                None => {}
            }
        }
        if !loop_stack.is_empty() {
            Err(Error::new(
                ErrorKind::InvalidData,
                "Error: '[' found with no matching ']'.".to_string(),
            ))
        } else {
            Ok(result)
        }
    }
}

struct Optimizer {
    statements: Vec<Statement>,
    wrap_tape: bool,
    wrap_cells: bool,
}

impl Optimizer {
    fn new(statements: Vec<Statement>, wrap_tape: bool, wrap_cells: bool) -> Self {
        Self {
            statements,
            wrap_tape,
            wrap_cells,
        }
    }

    fn get_return_addresses(&mut self) -> Vec<usize> {
        let mut return_addresses: Vec<usize> = Vec::new();
        for statement in &self.statements {
            match statement {
                Statement::JumpIf(address) => return_addresses.push(*address),
                _ => {}
            }
        }
        return_addresses
    }

    fn generate_optimized_stmt(stmt_type: Statement, value: &mut usize) -> Option<Statement> {
        let result = match value {
            0 => None,
            _ => match stmt_type {
                Statement::Add(_) => Some(Statement::Add(*value as u8)),
                Statement::Substract(_) => Some(Statement::Substract(*value as u8)),
                Statement::MoveLeft(_) => Some(Statement::MoveLeft(*value)),
                Statement::MoveRight(_) => Some(Statement::MoveRight(*value)),
                _ => None,
            },
        };
        *value = 0;
        result
    }

    fn optimize_once(&mut self) {
        let mut result: Vec<Statement> = Vec::new();
        let mut stmt_count: usize = 0;
        let mut add_count: u8 = 0;
        let mut last_statement = Statement::ReadChar;

        // return_addresses.sort();
        // return_addresses.dedup();

        for (index, statement) in (&mut self.statements).into_iter().enumerate() {
            if !statement.is_equal_type(&last_statement) {
                match Self::generate_optimized_stmt(last_statement, &mut stmt_count) {
                    Some(statement) => result.push(statement),
                    None => {}
                }
            }
            last_statement = *statement;
            match statement {
                Statement::MoveLeft(value) => result.push(Statement::MoveLeft(*value)),
                Statement::MoveRight(value) => result.push(Statement::MoveRight(*value)),
                Statement::Add(value) => {
                    stmt_count += *value as usize;
                }
                Statement::Substract(value) => result.push(Statement::Substract(*value)),
                stmt @ (Statement::PutChar | Statement::ReadChar) => result.push(*stmt),
                Statement::JumpIf(addr) => result.push(Statement::JumpIf(*addr)),
            }
        }

        match Self::generate_optimized_stmt(last_statement, &mut stmt_count) {
            Some(statement) => result.push(statement),
            None => {}
        }
        // fn parse(&mut self) -> Result<Vec<Statement>, String> {
        //     let mut result = Vec::new();
        //     let mut token_count: usize = 0;
        //     let mut last_token = Token::ShiftLeft;
        //     let mut loop_stack: Vec<usize> = Vec::new();

        //     for opt_token in &mut self.lexer {
        //         match opt_token {
        //             Some(token) => {
        //                 if token != last_token {
        //                     if token_count != 0 {
        //                         // case when countable last_token
        //                         let stmt_to_push =
        //                             Self::generate_optimized_stmt(last_token, token_count);
        //                         result.push(stmt_to_push);
        //                     }
        //                     token_count = 0;
        //                 }
        //                 match token {
        //                     Token::StartLoop => {
        //                         loop_stack.push(result.len());
        //                     }
        //                     Token::EndLoop => {
        //                         let address_opt = loop_stack.pop();
        //                         match address_opt {
        //                             Some(address) if address == result.len() => {}
        //                             Some(address) => {
        //                                 result.push(Statement::JumpIf(address));
        //                             }
        //                             None => {
        //                                 return Err("Error: ']' found with no matching '['.".to_string())
        //                             }
        //                         }
        //                     }
        //                     Token::PutChar => {
        //                         result.push(Statement::PutChar);
        //                     }
        //                     Token::ReadChar => {
        //                         result.push(Statement::ReadChar);
        //                     }
        //                     _ => {
        //                         token_count += 1;
        //                     }
        //                 }
        //                 last_token = token;
        //             }
        //             None => {}
        //         }
        //     }
        //     if !loop_stack.is_empty() {
        //         Err("Error: '[' found with no matching ']'.".to_string())
        //     } else {
        //         if token_count != 0 {
        //             let stmt_to_push = Self::generate_optimized_stmt(last_token, token_count);
        //             result.push(stmt_to_push);
        //         }
        //         Ok(result)
        //     }
        // }
        self.statements = result;
    }

    fn yield_back(self) -> Vec<Statement> {
        self.statements
    }
}

pub struct Interpreter<T: BufRead> {
    parser: Parser<T>,
    machine: BrainfuckMachine,
    console: termios::Termios,
}

impl Interpreter<BufReader<File>> {
    pub fn from_file(file_name: &str, machine_size: usize) -> Result<Self> {
        let path = Path::new(file_name);
        let file = File::open(path)?;
        let reader: BufReader<File> = BufReader::new(file);
        Ok(Self {
            parser: Parser::<BufReader<File>>::from_reader(reader),
            machine: BrainfuckMachine::new(machine_size),
            console: termios::Termios::from_fd(0).unwrap(),
        })
    }
}

impl<T: BufRead> Interpreter<T> {
    pub fn from_reader(reader: T, machine_size: usize) -> Self {
        Self {
            parser: Parser::from_reader(reader),
            machine: BrainfuckMachine::new(machine_size),
            console: termios::Termios::from_fd(0).unwrap(),
        }
    }

    fn get_char(&mut self) -> char {
        let stdout = io::stdout();
        let mut buffer = [0; 1];
        let mut reader = io::stdin();
        stdout.lock().flush().unwrap();
        reader.read_exact(&mut buffer).unwrap();
        buffer[0] as char
    }

    fn enable_get_char_mode(&mut self) {
        let mut new_termios = self.console.clone();
        new_termios.c_lflag &= !(termios::ICANON);
        termios::tcsetattr(
            std::io::Stdin::as_raw_fd(&std::io::stdin()),
            termios::TCSANOW,
            &mut new_termios,
        )
        .unwrap();
    }

    fn disable_get_char_mode(&mut self) {
        termios::tcsetattr(
            std::io::Stdin::as_raw_fd(&std::io::stdin()),
            termios::TCSANOW,
            &self.console,
        )
        .unwrap();
    }

    pub fn parse_and_run(&mut self) -> Result<()> {
        let mut statements = self.parser.parse()?;
        self.run(statements);
        Ok(())
    }

    fn run(&mut self, statements: Vec<Statement>) {
        self.enable_get_char_mode();
        let mut index: usize = 0;
        while (index < statements.len()) {
            dbg!(&index);
            let statement = statements[index];
            dbg!(&statement);
            match statement {
                Statement::MoveLeft(value) => self.machine.move_left(value),
                Statement::MoveRight(value) => self.machine.move_right(value),
                Statement::Add(value) => self.machine.add(value),
                Statement::Substract(value) => self.machine.add(value),
                Statement::ReadChar => {
                    let chr = self.get_char();
                    self.machine.read_char(chr);
                }
                Statement::PutChar => {
                    let chr = self.machine.put_char();
                    print!("{}", chr);
                }
                Statement::JumpIf(address) => {
                    if self.machine.check_loop() {
                        index = address;
                        continue;
                    }
                }
            }
            index += 1;
        }
        self.disable_get_char_mode();
    }
}
