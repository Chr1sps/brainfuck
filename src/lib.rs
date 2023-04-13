//! This module exports brainfuck machine and interpreter implementations.
#![warn(missing_docs)]
use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Error, ErrorKind, Read, Result, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

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

#[derive(Clone, PartialEq, Debug)]
enum Statement {
    MoveLeft(usize),
    MoveRight(usize),

    Add(u8),

    Loop(Box<Vec<Statement>>),
    PutChar,
    ReadChar,
}

impl Statement {
    fn is_equal_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
    fn is_move(&self) -> bool {
        matches!(self, &(Statement::MoveLeft(_) | Statement::MoveRight(_)))
    }
    fn new_loop(statements: Vec<Statement>) -> Self {
        Self::Loop(Box::new(statements))
    }
}

/// This struct is used as an implementation of a brainfuck-compatible
/// Turing-like machine that supports basic operations needed for such
/// compilations. This machine works under an assumption that chars can be
/// converted into [`u8`] freely through ASCII decoding and encoding.
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

    /// Returns a copy of the vector representing the tape.
    fn get_tape(&self) -> Vec<u8> {
        self.tape.clone()
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
    fn iter(&mut self) -> LexerRefIter<'_, T> {
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
        LexerIter { lexer: self }
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
        LexerRefIter { lexer: self }
    }
}
struct Parser<T: BufRead> {
    lexer: Lexer<T>,
}

impl<T: BufRead> Parser<T> {
    fn from_lexer(lexer: Lexer<T>) -> Self {
        Self { lexer }
    }
    fn from_reader(reader: T) -> Self {
        Self::from_lexer(Lexer { reader })
    }
    fn parse_rec(
        lexer_iter: &mut LexerRefIter<T>,
        is_loop: bool,
    ) -> Result<Option<Vec<Statement>>> {
        let mut result: Vec<Statement> = Vec::new();
        while let Some(opt_token) = lexer_iter.next() {
            match opt_token {
                Some(token) => match token {
                    Token::Increment => result.push(Statement::Add(1)),
                    Token::Decrement => result.push(Statement::Add(u8::MAX)),
                    Token::ShiftLeft => result.push(Statement::MoveLeft(1)),
                    Token::ShiftRight => result.push(Statement::MoveRight(1)),
                    Token::PutChar => result.push(Statement::PutChar),
                    Token::ReadChar => result.push(Statement::ReadChar),
                    Token::StartLoop => {
                        let opt_loop = Self::parse_rec(lexer_iter, true)?;
                        if let Some(stmt_loop) = opt_loop {
                            result.push(Statement::new_loop(stmt_loop));
                        }
                    }
                    Token::EndLoop => {
                        if is_loop {
                            if result.is_empty() {
                                return Ok(None);
                            } else {
                                return Ok(Some(result));
                            }
                        } else {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "Error: ']' found with no matching '['.".to_string(),
                            ));
                        }
                    }
                },
                None => {}
            }
        }
        if is_loop {
            Err(Error::new(
                ErrorKind::InvalidData,
                "Error: '[' found with no matching ']'.".to_string(),
            ))
        } else {
            Ok(Some(result))
        }
    }

    fn parse(&mut self) -> Result<Vec<Statement>> {
        let lexer_iter: &mut LexerRefIter<T> = &mut self.lexer.iter();
        let parsed_opt = Self::parse_rec(lexer_iter, false)?;
        Ok(parsed_opt.unwrap_or_default())
    }
}

struct Optimizer {
    statements: Vec<Statement>,
}

impl Optimizer {
    fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    fn generate_optimized_stmt(stmt_type: &Statement, value: &mut usize) -> Option<Statement> {
        let result = match value {
            0 => None,
            _ => match stmt_type {
                Statement::Add(_) => Some(Statement::Add(*value as u8)),
                Statement::MoveLeft(_) => Some(Statement::MoveLeft(*value)),
                Statement::MoveRight(_) => Some(Statement::MoveRight(*value)),
                _ => None,
            },
        };
        *value = 0;
        result
    }

    fn optimize_rec(statements: &Vec<Statement>) -> Option<Vec<Statement>> {
        let mut result: Vec<Statement> = Vec::new();
        let mut stmt_count: usize = 0;
        let mut last_statement = Statement::ReadChar;

        for statement in statements {
            if !statement.is_equal_type(&last_statement)
                && (!statement.is_move() || !last_statement.is_move())
            {
                match Self::generate_optimized_stmt(&last_statement, &mut stmt_count) {
                    Some(statement) => result.push(statement),
                    None => {}
                }
            }
            let mut cloned = statement.clone();
            match statement {
                Statement::MoveLeft(value) => match last_statement {
                    Statement::MoveLeft(_) => {
                        stmt_count += value;
                    }
                    Statement::MoveRight(_) => {
                        if stmt_count < *value {
                            stmt_count = value - stmt_count;
                        } else {
                            stmt_count -= value;
                            cloned = last_statement.clone();
                        }
                    }
                    _ => {
                        stmt_count = *value;
                    }
                },
                Statement::MoveRight(value) => match last_statement {
                    Statement::MoveRight(_) => {
                        stmt_count += value;
                    }
                    Statement::MoveLeft(_) => {
                        if stmt_count < *value {
                            stmt_count = value - stmt_count;
                        } else {
                            stmt_count -= value;
                            cloned = last_statement.clone();
                        }
                    }
                    _ => {
                        stmt_count = *value;
                    }
                },
                Statement::Add(value) => match last_statement {
                    Statement::Add(_) => {
                        stmt_count = value.wrapping_add(stmt_count as u8) as usize;
                    }
                    _ => {
                        stmt_count = *value as usize;
                    }
                },
                stmt @ (Statement::PutChar | Statement::ReadChar) => result.push(stmt.clone()),
                Statement::Loop(code) => {
                    if let Some(optimized) = Self::optimize_rec(&code) {
                        result.push(Statement::new_loop(optimized));
                    }
                }
            }
            last_statement = cloned;
        }
        match Self::generate_optimized_stmt(&last_statement, &mut stmt_count) {
            Some(statement) => result.push(statement),
            None => {}
        }
        Some(result)
    }

    fn optimize_once(&mut self) {
        let opt_result = Self::optimize_rec(&self.statements);
        self.statements = opt_result.unwrap_or_default();
    }

    fn optimize(&mut self, max_iterations: u32) {
        if max_iterations == 0 {
            loop {
                let previous = self.statements.clone();
                self.optimize_once();
                if self.statements == previous {
                    break;
                }
            }
        } else {
            for _ in 0..max_iterations {
                let previous = self.statements.clone();
                self.optimize_once();
                if self.statements == previous {
                    break;
                }
            }
        }
    }

    fn yield_back(self) -> Vec<Statement> {
        self.statements
    }
}

/// A brainfuck interpreter class that reads code from a file / [`BufRead`]
/// instance, parses, optimizes and runs it.
pub struct Interpreter<T: BufRead> {
    parser: Parser<T>,
    machine: BrainfuckMachine,
    console: termios::Termios,
}

impl Interpreter<BufReader<File>> {
    /// Creates a new [`Interpreter<BufReader<File>>`] instance wrapped in a
    /// [`Result`] object. If there were any problems when reading a file
    /// the function will return an [`std::io::Error`] instance.
    pub fn from_file(file_name: &str, machine_size: usize) -> Result<Self> {
        let path = Path::new(file_name);
        if !path.is_file() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Data cannot be read from: {}", file_name),
            ));
        }
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
    /// Creates a new [`Interpreter`] instance from a [`BufRead`] implementor
    /// with a given tape size.
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

    /// Parses the code that was contained within the [`BufRead`] instance
    /// passed to the constructor (or within a given file, if the
    /// [`Interpreter::from_file`] constructor has been
    /// called) and then runs it. This function returns an [`Ok(())`] instance
    /// in case of no issues and a wrapped [`std::io::Error`] if there are any.
    ///
    /// [`Interpreter::from_file`]: ./struct.Interpreter.html#method.from_file
    pub fn run(&mut self) -> Result<()> {
        let statements = self.parser.parse()?;
        self.run_code(&statements);
        Ok(())
    }

    /// Parses the code that was contained within the [`BufRead`] instance
    /// passed to the constructor (or within a given file, if the
    /// [`Interpreter::from_file`] constructor has been
    /// called) and then runs it with a given optimization level. The
    /// `max_iterations` parameter specifies the maximum amount of optimization
    /// iterations that will be run on the code. If `max_iterations` is equal
    /// to `0`, then the code will be optimized fully. This function returns an
    /// [`Ok(())`] instancein case of no issues and a wrapped
    /// [`std::io::Error`] if there are any.
    ///
    /// [`Interpreter::from_file`]: ./struct.Interpreter.html#method.from_file
    pub fn run_with_optimization(&mut self, max_iterations: u32) -> Result<()> {
        let statements = self.parser.parse()?;
        let mut optimizer = Optimizer::new(statements);
        optimizer.optimize(max_iterations);
        let statements = optimizer.yield_back();
        self.run_code(&statements);
        Ok(())
    }

    fn run_code(&mut self, statements: &Vec<Statement>) {
        self.enable_get_char_mode();
        for statement in statements {
            match statement {
                Statement::MoveLeft(value) => self.machine.move_left(*value),
                Statement::MoveRight(value) => self.machine.move_right(*value),
                Statement::Add(value) => self.machine.add(*value),
                Statement::ReadChar => {
                    let chr = self.get_char();
                    self.machine.read_char(chr);
                }
                Statement::PutChar => {
                    let chr = self.machine.put_char();
                    print!("{}", chr);
                }
                Statement::Loop(boxed) => {
                    while self.machine.check_loop() {
                        self.run_code(boxed);
                    }
                }
            }
        }
        self.disable_get_char_mode();
    }

    /// Returns a [`Vec<u8>`] instance represeting the tape of the underlying
    /// [machine].
    ///
    /// [machine]: BrainfuckMachine
    pub fn get_tape(&self) -> Vec<u8> {
        self.machine.get_tape()
    }
}

struct Code<'a> {
    code: &'a Vec<Statement>,
}
impl<'a> Code<'a> {
    fn generate_string(statements: &Vec<Statement>) -> String {
        let mut info: String = String::new();
        for statement in statements {
            let to_push = match statement {
                Statement::Add(value) => format!("{}+ ", *value),
                Statement::MoveLeft(value) => format!("{}< ", *value),
                Statement::MoveRight(value) => format!("{}> ", *value),
                Statement::ReadChar => ", ".to_string(),
                Statement::PutChar => ". ".to_string(),
                Statement::Loop(boxed) => {
                    let loop_stmt = boxed;
                    format!("[ {}] ", Self::generate_string(&loop_stmt))
                }
            };
            info.push_str(&to_push);
        }
        info.trim_end().to_string()
    }
}

impl<'a> std::fmt::Debug for Code<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info: String = Self::generate_string(self.code);
        f.debug_struct("Code").field("code", &info).finish()
    }
}
