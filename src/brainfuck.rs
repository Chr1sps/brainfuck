#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::io::BufRead;
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
    wrap_cells: bool,
}

impl BrainfuckMachine {
    /// Creates a `BrainfuckMachine` instance of given tape size and with
    /// preferred tape/cell value wrapping (tape index and cell values will
    /// overflow/wrap around the lower and upper limits accordingly).
    pub fn new(size: usize, wrap_tape: bool, wrap_cells: bool) -> Self {
        let mut result = Self {
            size,
            index: 0,
            tape: Vec::new(),
            wrap_tape,
            wrap_cells,
        };
        result.tape.resize(size, 0);
        result
    }

    /// Returns `(first + other) % modulus`. This implementation avoids
    /// over/underflowing values when performing the inner addition.
    fn add_with_wrap(first: usize, other: usize, modulus: usize) -> usize {
        let negated = modulus - first;
        match negated.cmp(&other) {
            Ordering::Greater => first + other,
            _ => other - negated,
        }
    }

    /// Returns `(first - other) % modulus`. This implementation avoids
    /// over/underflowing values when performing the inner substraction.
    fn sub_with_wrap(first: usize, other: usize, modulus: usize) -> usize {
        match first.cmp(&other) {
            Ordering::Less => {
                let negated = modulus - other;
                negated + first
            }
            _ => first - other,
        }
    }

    /// Moves the header left by a given amount. If `wrap_tape` is true, wraps the current cell
    /// index when encountering the tape margins.
    pub fn move_left(&mut self, shift: usize) {
        self.index = match self.wrap_tape {
            true => Self::sub_with_wrap(self.index, shift, self.size),
            false => self.index.saturating_sub(shift),
        };
    }
    /// Moves the header right by a given amount. If `wrap_tape` is true, wraps the current cell
    /// index when encountering the tape margins.
    pub fn move_right(&mut self, shift: usize) {
        self.index = match self.wrap_tape {
            true => Self::add_with_wrap(self.index, shift, self.size),
            false => self.index.saturating_add(shift).min(self.size - 1),
        };
    }

    /// Adds a given value to the current cell. If `wrap_cells` is true,
    /// upon overflows the value will be wrapped accordingly. Otherwise, the
    /// value shall not exceed the upper bound.
    pub fn add(&mut self, value: u8) {
        let current = self.tape[self.index];
        self.tape[self.index] = match self.wrap_cells {
            true => current.wrapping_add(value),
            false => current.saturating_add(value),
        };
    }

    /// Substracts a given value from the current cell. If `wrap_cells` is
    /// true, upon underflows the value will be wrapped accordingly. Otherwise,
    /// the value shall not exceed the lower bound.
    pub fn substract(&mut self, value: u8) {
        let current = self.tape[self.index];
        self.tape[self.index] = match self.wrap_cells {
            true => current.wrapping_sub(value),
            false => current.saturating_sub(value),
        };
    }

    /// Insert a given char's ASCII value into the current cell.
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

pub struct Lexer<'a> {
    reader: &'a mut dyn BufRead,
    eof: bool,
}

impl<'a> Lexer<'a> {
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
    pub fn from_reader<R: BufRead + Clone + 'a>(reader: &'a mut R) -> Self {
        Self { reader, eof: false }
    }
}

pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
}

impl<'a> Parser<'a> {
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
