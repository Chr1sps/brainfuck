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
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Statement {
    Move(isize),
    ChangeValue(i8),
    JumpIf(usize),
    PutChar,
    ReadChar,
}

impl Token {
    fn is_countable(&self) -> bool {
        use self::Token::*;
        match self {
            &(Increment | Decrement | ShiftLeft | ShiftRight) => true,
            _ => false,
        }
    }
    fn is_loop(&self) -> bool {
        use self::Token::*;
        match self {
            &(StartLoop | EndLoop) => true,
            _ => false,
        }
    }
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
        Self {
            lexer: Lexer { reader },
        }
    }
    fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut result = Vec::new();
        let mut token_count: usize = 0;
        let mut last_token = Token::ShiftLeft;
        let mut loop_stack: Vec<usize> = Vec::new();

        for opt_token in &mut self.lexer {
            match opt_token {
                Some(token) if token == last_token => {
                    if token.is_countable() {
                        token_count += 1;
                    }
                    if let Token::StartLoop = token {
                        loop_stack.push(result.len());
                    } else if let Token::EndLoop = token {
                        let address_opt = loop_stack.pop();
                        match address_opt {
                            Some(address) if address == result.len() => {}
                            Some(address) => {
                                result.push(Statement::JumpIf(address));
                            }
                            None => {
                                return Err("Error: ']' found with no matching '['.".to_string())
                            }
                        }
                    }
                }
                Some(token) => {
                    if token_count != 0 {
                        let stmt_to_push = Self::generate_optimized_stmt(last_token, token_count);
                        result.push(stmt_to_push);
                    }
                    if let Token::StartLoop = token {
                        loop_stack.push(result.len());
                    } else if let Token::EndLoop = token {
                        let address_opt = loop_stack.pop();
                        match address_opt {
                            Some(address) if address == result.len() => {}
                            Some(address) => {
                                result.push(Statement::JumpIf(address));
                            }
                            None => {
                                return Err("Error: ']' found with no matching '['.".to_string())
                            }
                        }
                    }
                    if token.is_countable() {
                        token_count = 1;
                    } else {
                        token_count = 0;
                    }
                    last_token = token;
                }
                None => {}
            }
        }
        if !loop_stack.is_empty() {
            Err("Error: '[' found with no matching ']'.".to_string())
        } else {
            if token_count != 0 {
                let stmt_to_push = Self::generate_optimized_stmt(last_token, token_count);
                result.push(stmt_to_push);
            }
            Ok(result)
        }
    }
    fn generate_optimized_stmt(token: Token, count: usize) -> Statement {
        match token {
            Token::Increment => Statement::ChangeValue(count as i8),
            Token::Decrement => Statement::ChangeValue(-(count as i8)),
            Token::ShiftLeft => Statement::Move(-(count as isize)),
            Token::ShiftRight => Statement::Move(count as isize),
            _ => panic!("Ehmmm... wtf?"),
        }
    }
}
