mod tests;

use std::io::{BufRead, Read, Result};
use std::ops::Sub;
#[derive(Copy, Clone)]
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
    // post-optimization tokens
    JumpTo(usize),
    Add(u8),
    Move(usize),
}

pub struct BrainfuckMachine {
    /// Size of the tape vector.
    size: usize,
    /// Current cell index.
    index: usize,
    /// Tape vector.
    tape: Vec<u8>,
    /// If true, wraps the tape index on overflows/underflows.
    wrap_tape: bool,
    /// If true, wraps the cell values on overflows/underflows.
    wrap_values: bool,
}

impl BrainfuckMachine {
    pub fn new(size: usize, wrap_tape: bool, wrap_values: bool) -> Self {
        let mut result = Self {
            size,
            index: 0,
            tape: Vec::new(),
            wrap_tape,
            wrap_values,
        };
        result.tape.resize(size, 0);
        result
    }

    /// Moves the header left by a given amount. If `wrap_tape` is true, wraps the current cell
    /// index when encountering the tape margins.
    pub fn move_left(&mut self, shift: usize) {
        self.index = match self.wrap_tape {
            true => self.index.wrapping_sub(shift),
            false => self.index.saturating_sub(shift),
        };
    }
    /// Moves the header right by a given amount. If `wrap_tape` is true, wraps the current cell
    /// index when encountering the tape margins.
    pub fn move_right(&mut self, shift: usize) {
        self.index = match self.wrap_tape {
            true => self.index.wrapping_add(shift),
            false => self.index.saturating_add(shift),
        };
    }

    pub fn add(&mut self, value: u8) {
        let current = self.tape[self.index];
        self.tape[self.index] = match self.wrap_values {
            true => current.wrapping_add(value),
            false => current.saturating_add(value),
        };
    }
    pub fn substract(&mut self, value: u8) {
        let current = self.tape[self.index];
        self.tape[self.index] = match self.wrap_values {
            true => current.wrapping_sub(value),
            false => current.saturating_sub(value),
        };
    }

    pub fn read_char(&mut self, input: char) {
        self.tape[self.index] = input as u8
    }
    pub fn put_char(&self) -> char {
        self.tape[self.index] as char
    }
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

pub struct Lexer {
    reader: Box<dyn Read>,
}

impl Lexer {
    fn next_token(&mut self) -> Option<Token> {
        let mut buf: [u8; 1] = [0];
        if let Err(msg) = self.reader.read_exact(&mut buf) {
            panic!("Error when reading a token: {}", msg);
        } else {
            let ascii = buf[0];
            let to_token = ascii as char;
            Self::tokenize(&to_token)
        }
    }
    // fn eof(&self) -> bool {}
    fn tokenize(input: &char) -> Option<Token> {
        use crate::brainfuck::Token::*;

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
    pub fn from_string(input: &'static str) -> Self {
        Self {
            reader: Box::new(input.as_bytes()),
        }
    }
    pub fn from_reader(reader: impl Read + 'static) -> Self {
        Self {
            reader: Box::new(reader),
        }
    }
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    // fn parse(&mut self) -> Result<Vec<Token>> {
    //     let token = self.get_next_token();
    // }
    fn get_next_token(&mut self) -> Option<Token> {
        self.lexer.next_token()
    }
    fn parse_tokens(&mut self, tokens: Vec<Token>) -> Option<Vec<Token>> {
        let mut loop_stack: Vec<usize> = Vec::new();
        let mut result: Vec<Token> = Vec::new();
        for (addr, token) in tokens.iter().enumerate() {
            match token {
                Token::StartLoop => {
                    loop_stack.push(addr);
                    result.push(token.clone());
                }
                Token::EndLoop => match loop_stack.len() {
                    0 => {
                        return None;
                    }
                    _ => {}
                },
                _ => (),
            };
        }
        None
    }
}
