use std::io::{Error, ErrorKind};
use std::iter::zip;

use crate::Statement;
use crate::Token;
use utils::*;

use super::{BrainfuckMachine, Lexer};
#[test]
fn test_machine_index_change_base() {
    let mut machine = BrainfuckMachine::new(10);
    assert_eq!(machine.index, 0);
    machine.move_right(5);
    assert_eq!(machine.index, 5);
    machine.move_left(3);
    assert_eq!(machine.index, 2);
}

#[test]
#[should_panic = "Index out of bounds."]
fn test_machine_index_change_left_panic() {
    let mut machine = BrainfuckMachine::new(10);
    machine.move_left(3);
}
#[test]
#[should_panic = "Index out of bounds."]
fn test_machine_index_change_right_panic() {
    let mut machine = BrainfuckMachine::new(10);
    machine.move_right(11);
}

#[test]
fn test_machine_value_change_base() {
    let mut machine = BrainfuckMachine::new(10);
    assert_eq!(
        machine.tape[machine.index], 0,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
    machine.add(69);
    assert_eq!(machine.tape[machine.index], 69);
    machine.substract(42);
    assert_eq!(machine.tape[machine.index], 27);
}
#[test]
fn test_machine_value_change_wrap() {
    let mut machine = BrainfuckMachine::new(10);
    assert_eq!(
        machine.tape[machine.index], 0,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
    machine.substract(11);
    assert_eq!(
        machine.tape[machine.index], 245,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
    machine.add(23);
    assert_eq!(
        machine.tape[machine.index], 12,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
}

#[test]
fn test_machine_put_char() {
    let mut machine = BrainfuckMachine::new(10);
    machine.add(65);
    let result: char = machine.put_char();
    assert_eq!(
        result, 'A',
        "Different char read. Char read: {}.",
        result as u8
    );
}
#[test]
fn test_machine_read_char() {
    let mut machine = BrainfuckMachine::new(10);
    machine.read_char('A');
    let result = machine.tape[machine.index];
    assert_eq!(result, 65, "Different char read. Char read: {}.", result);
}
#[test]
fn test_machine_check_loop() {
    let mut machine = BrainfuckMachine::new(10);
    assert!(!machine.check_loop());
    machine.add(4);
    assert!(machine.check_loop());
    machine.substract(4);
    assert!(!machine.check_loop());
}

#[test]
fn test_lexer_eof_true() {
    let code = String::from("");
    let mut lexer = Lexer::from_reader(code.as_bytes());
    assert!(lexer.eof());
}

#[test]
fn test_lexer_eof_false() {
    let code = String::from(".");
    let mut bytes = code.as_bytes();
    let mut lexer = Lexer::from_reader(&mut bytes);
    assert!(!lexer.eof());
}

#[test]
fn test_lexer_next_token_valid_tokens() {
    let code = String::from("><,.+-[]");
    let mut lexer = Lexer::from_reader(code.as_bytes());
    let expected: Vec<Token> = vec![
        Token::ShiftRight,
        Token::ShiftLeft,
        Token::ReadChar,
        Token::PutChar,
        Token::Increment,
        Token::Decrement,
        Token::StartLoop,
        Token::EndLoop,
    ];
    for exp in expected {
        assert!(!lexer.eof());
        let token = lexer.next_token();
        assert!(token.is_some());
        assert_eq!(token.unwrap(), exp);
    }
    assert!(lexer.eof());
    let token = lexer.next_token();
    assert!(token.is_none());
    assert!(lexer.eof());
}

#[test]
fn test_lexer_next_token_other_symbols() {
    let code = String::from("abcdef");
    let mut lexer = Lexer::from_reader(code.as_bytes());
    while !lexer.eof() {
        let token = lexer.next_token();
        assert!(token.is_none());
    }
}

#[test]
fn test_lexer_iter_valid_tokens() {
    let code = String::from("><,.+-[]");
    let lexer = Lexer::from_reader(code.as_bytes());
    let expected: Vec<Option<Token>> = vec![
        Some(Token::ShiftRight),
        Some(Token::ShiftLeft),
        Some(Token::ReadChar),
        Some(Token::PutChar),
        Some(Token::Increment),
        Some(Token::Decrement),
        Some(Token::StartLoop),
        Some(Token::EndLoop),
    ];
    let mut actual: Vec<Option<Token>> = Vec::new();
    for token in lexer.iter() {
        actual.push(token);
    }
    assert_eq!(expected.len(), actual.len());
    for (exp, act) in zip(expected, actual) {
        assert_eq!(exp, act);
    }
}

#[test]
fn test_lexer_iter_other_symbols() {
    let code = String::from("abcdef");
    let expected: Vec<Option<Token>> = vec![None, None, None, None, None, None];
    test_lexer(&code, &expected);
}

#[test]
fn test_lexer_for_loop() {
    let code = String::from("><,.+-[]");
    let expected: Vec<Option<Token>> = vec![
        Some(Token::ShiftRight),
        Some(Token::ShiftLeft),
        Some(Token::ReadChar),
        Some(Token::PutChar),
        Some(Token::Increment),
        Some(Token::Decrement),
        Some(Token::StartLoop),
        Some(Token::EndLoop),
    ];
    test_lexer(&code, &expected);
}

#[test]
fn test_parser_parse_empty_string() {
    let code = String::from("");
    let expected: Vec<Statement> = Vec::new();
    test_parser(&code, &expected);
}

#[test]
fn test_parser_parse_non_loop_statements() {
    let code = String::from("+-><,.");
    let expected = vec![
        Statement::Add(1),
        Statement::Add(255),
        Statement::MoveRight(1),
        Statement::MoveLeft(1),
        Statement::ReadChar,
        Statement::PutChar,
    ];
    test_parser(&code, &expected);
}

#[test]
fn test_parser_parse_countable_optimization() {
    let code = String::from("++++----<<<<>>>>");
    let expected = vec![
        Statement::Add(1),
        Statement::Add(1),
        Statement::Add(1),
        Statement::Add(1),
        Statement::Add(255),
        Statement::Add(255),
        Statement::Add(255),
        Statement::Add(255),
        Statement::MoveLeft(1),
        Statement::MoveLeft(1),
        Statement::MoveLeft(1),
        Statement::MoveLeft(1),
        Statement::MoveRight(1),
        Statement::MoveRight(1),
        Statement::MoveRight(1),
        Statement::MoveRight(1),
    ];
    test_parser(&code, &expected);
}

#[test]
fn test_parser_parse_loop_valid() {
    let code = String::from("[+-<>]");
    let expected = vec![
        Statement::Add(1),
        Statement::Add(255),
        Statement::MoveLeft(1),
        Statement::MoveRight(1),
        Statement::JumpIf(0),
    ];
    test_parser(&code, &expected);
}

#[test]
fn test_parser_parse_loop_invalid_redundant_left_bracket() {
    let code = String::from("[[++++----<<<<>>>>]");
    let error = Error::new(
        ErrorKind::InvalidData,
        "Error: '[' found with no matching ']'.".to_string(),
    );
    test_parser_error(&code, &error);
}

#[test]
fn test_parser_parse_loop_invalid_redundant_right_bracket() {
    let code = String::from("[++++----]<<<<>>>>]");
    let error = Error::new(
        ErrorKind::InvalidData,
        "Error: ']' found with no matching '['.".to_string(),
    );
    test_parser_error(&code, &error);
}

#[test]
fn test_parser_parse_loop_optimize_remove_empty_loops() {
    let code = String::from("[][][]");
    let result: Vec<Statement> = Vec::new();
    test_parser(&code, &result);
}

#[test]
fn test_parser_parse_loop_optimize_remove_empty_loops_nested() {
    let code = String::from("[[[]]]");
    let result: Vec<Statement> = Vec::new();
    test_parser(&code, &result);
}

#[test]
fn test_optimizer_optimize_once_no_optimization() {
    let statements: Vec<Statement> = vec![
        Statement::ReadChar,
        Statement::PutChar,
        Statement::MoveRight(1),
        Statement::Add(1),
        Statement::MoveLeft(1),
        Statement::JumpIf(0),
    ];
    test_optimize_once(&statements, &statements);
}

#[test]
fn test_optimizer_optimize_once_adds() {
    let input: Vec<Statement> = vec![
        Statement::Add(1),
        Statement::Add(2),
        Statement::Add(3),
        Statement::Add(4),
    ];
    let output = vec![Statement::Add(10)];
    test_optimize_once(&input, &output);
}

#[test]
fn test_optimizer_optimize_once_adds_overflow() {
    let input: Vec<Statement> = vec![
        Statement::Add(3),
        Statement::Add(254),
        Statement::Add(4),
        Statement::Add(250),
    ];
    let output = vec![Statement::Add(255)];
    test_optimize_once(&input, &output);
}

#[test]
fn test_optimizer_optimize_once_adds_no_add() {
    let input: Vec<Statement> = vec![
        Statement::Add(3),
        Statement::Add(255),
        Statement::Add(4),
        Statement::Add(250),
    ];
    let output: Vec<Statement> = Vec::new();
    test_optimize_once(&input, &output);
}

#[test]
fn test_optimizer_optimize_once_moves_right() {
    let input: Vec<Statement> = vec![
        Statement::MoveRight(3),
        Statement::MoveRight(4),
        Statement::MoveRight(5),
        Statement::MoveRight(6),
    ];
    let output = vec![Statement::MoveRight(18)];
    test_optimize_once(&input, &output);
}

#[test]
fn test_optimizer_optimize_once_moves_left_and_right() {
    let input: Vec<Statement> = vec![
        Statement::MoveLeft(3),
        Statement::MoveLeft(4),
        Statement::MoveRight(5),
        Statement::MoveRight(6),
    ];
    let output = vec![Statement::MoveRight(4)];
    test_optimize_once(&input, &output);
}

#[test]
fn test_optimizer_optimize_once_moves_left_and_right_no_shift() {
    let input: Vec<Statement> = vec![
        Statement::MoveRight(3),
        Statement::MoveLeft(4),
        Statement::MoveLeft(5),
        Statement::MoveRight(6),
    ];
    let output: Vec<Statement> = Vec::new();
    test_optimize_once(&input, &output);
}

#[test]
fn test_optimizer_optimize_once_adds_and_moves() {
    let input: Vec<Statement> = vec![
        Statement::MoveRight(3),
        Statement::MoveLeft(4),
        Statement::Add(3),
        Statement::Add(4),
        Statement::MoveLeft(5),
        Statement::MoveRight(6),
    ];
    let output = vec![
        Statement::MoveLeft(1),
        Statement::Add(7),
        Statement::MoveRight(1),
    ];
    test_optimize_once(&input, &output);
}

#[test]
fn test_optimizer_optimize_once_adds_with_loop() {
    let input: Vec<Statement> = vec![
        Statement::Add(3),
        Statement::Add(4),
        Statement::Add(3),
        Statement::Add(4),
        Statement::JumpIf(2),
    ];
    let output = vec![Statement::Add(7), Statement::Add(7), Statement::JumpIf(1)];
    test_optimize_once(&input, &output);
}

// helper testing functions
mod utils {
    use crate::{Lexer, Optimizer, Parser, Statement, Token};
    use std::io::Error;
    pub(in crate::tests) fn test_lexer(code: &String, expected: &Vec<Option<Token>>) {
        let lexer = Lexer::from_reader(code.as_bytes());
        let mut actual: Vec<Option<Token>> = Vec::new();
        for token in lexer {
            actual.push(token);
        }
        assert_eq!(*expected, actual);
    }
    pub(in crate::tests) fn test_parser(code: &String, expected: &Vec<Statement>) {
        let mut parser = Parser::from_reader(code.as_bytes());
        let parsed = parser.parse().unwrap();
        assert_eq!(parsed, *expected);
    }

    pub fn test_parser_error(code: &String, error: &Error) {
        let mut parser = Parser::from_reader(code.as_bytes());
        let parsed = parser.parse().unwrap_err();
        assert_eq!(parsed.kind(), error.kind());
        assert_eq!(parsed.to_string(), error.to_string(),);
    }

    pub(in crate::tests) fn test_optimize_once(input: &Vec<Statement>, output: &Vec<Statement>) {
        let mut optimizer = Optimizer::new(input.clone());
        optimizer.optimize_once();
        let optimized = optimizer.yield_back();
        assert_eq!(*optimized, *output);
    }
}
