use std::iter::zip;

use crate::Statement;
use crate::Token;

use super::{BrainfuckMachine, Lexer, Parser};
#[test]
fn test_machine_index_change_base() {
    let mut machine = BrainfuckMachine::new(10, false, false);
    assert_eq!(machine.index, 0);
    machine.move_right(5);
    assert_eq!(machine.index, 5);
    machine.move_left(3);
    assert_eq!(machine.index, 2);
}
#[test]
fn test_machine_index_change_no_wrap() {
    let mut machine = BrainfuckMachine::new(10, false, false);
    machine.move_left(3);
    assert_eq!(
        machine.index, 0,
        "Left wrap doesn't pass, index: {}.",
        machine.index
    );
    machine.move_right(10);
    assert_eq!(
        machine.index, 9,
        "Right wrap doesn't pass, index: {}.",
        machine.index
    );
}
#[test]
fn test_machine_index_change_wrap() {
    let mut machine = BrainfuckMachine::new(10, true, false);
    machine.move_left(3);
    assert_eq!(
        machine.index, 7,
        "Left wrap doesn't pass, index: {}.",
        machine.index
    );
    machine.move_right(6);
    assert_eq!(
        machine.index, 3,
        "Right wrap doesn't pass, index: {}.",
        machine.index
    );
}
#[test]
fn test_machine_value_change_base() {
    let mut machine = BrainfuckMachine::new(10, false, true);
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
    let mut machine = BrainfuckMachine::new(10, false, true);
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
fn test_machine_value_change_no_wrap() {
    let mut machine = BrainfuckMachine::new(10, false, false);
    assert_eq!(
        machine.tape[machine.index], 0,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
    machine.substract(11);
    assert_eq!(
        machine.tape[machine.index], 0,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
    machine.add(255);
    machine.add(255);
    assert_eq!(
        machine.tape[machine.index], 255,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
}
#[test]
fn test_machine_put_char() {
    let mut machine = BrainfuckMachine::new(10, false, false);
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
    let mut machine = BrainfuckMachine::new(10, false, false);
    machine.read_char('A');
    let result = machine.tape[machine.index];
    assert_eq!(result, 65, "Different char read. Char read: {}.", result);
}
#[test]
fn test_machine_check_loop() {
    let mut machine = BrainfuckMachine::new(10, false, false);
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
    let lexer = Lexer::from_reader(code.as_bytes());
    let expected: Vec<Option<Token>> = vec![None, None, None, None, None, None];
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
fn test_lexer_for_loop() {
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
    for token in lexer {
        actual.push(token);
    }
    assert_eq!(expected.len(), actual.len());
    for (exp, act) in zip(expected, actual) {
        assert_eq!(exp, act);
    }
}

#[test]
fn test_parser_parse_empty_string() {
    let code = String::from("");
    let mut parser = Parser::from_reader(code.as_bytes());
    let result = parser.parse();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_parser_parse_non_loop_statements() {
    let code = String::from("+-><,.");
    let mut parser = Parser::from_reader(code.as_bytes());
    let result = parser.parse();
    let expected = vec![
        Statement::ChangeValue(1),
        Statement::ChangeValue(-1),
        Statement::Move(1),
        Statement::Move(-1),
        Statement::ReadChar,
        Statement::PutChar,
    ];
    assert!(result.is_ok());
    let unwrapped = result.unwrap();
    assert_eq!(unwrapped.len(), expected.len());
    assert_eq!(unwrapped, expected);
}

#[test]
fn test_parser_parse_countable_optimization() {
    let code = String::from("++++----<<<<>>>>");
    let mut parser = Parser::from_reader(code.as_bytes());
    let result = parser.parse();
    let expected = vec![
        Statement::ChangeValue(4),
        Statement::ChangeValue(-4),
        Statement::Move(-4),
        Statement::Move(4),
    ];
    assert!(result.is_ok());
    let unwrapped = result.unwrap();
    assert_eq!(unwrapped.len(), expected.len());
    assert_eq!(unwrapped, expected);
}

#[test]
fn test_parser_parse_loop_valid() {
    let code = String::from("[++++----<<<<>>>>]");
    let mut parser = Parser::from_reader(code.as_bytes());
    let result = parser.parse();
    let expected = vec![
        Statement::ChangeValue(4),
        Statement::ChangeValue(-4),
        Statement::Move(-4),
        Statement::Move(4),
        Statement::JumpIf(0),
    ];
    assert!(result.is_ok());
    let unwrapped = result.unwrap();
    assert_eq!(unwrapped.len(), expected.len());
    assert_eq!(unwrapped, expected);
}

#[test]
fn test_parser_parse_loop_invalid_redundant_left_bracket() {
    let code = String::from("[[++++----<<<<>>>>]");
    let mut parser = Parser::from_reader(code.as_bytes());
    let result = parser.parse();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Error: '[' found with no matching ']'.".to_string()
    );
}

#[test]
fn test_parser_parse_loop_invalid_redundant_right_bracket() {
    let code = String::from("[++++----]<<<<>>>>]");
    let mut parser = Parser::from_reader(code.as_bytes());
    let result = parser.parse();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Error: ']' found with no matching '['.".to_string()
    );
}

#[test]
fn test_parser_parse_loop_optimize_remove_empty_loops() {
    let code = String::from("[][][]");
    let mut parser = Parser::from_reader(code.as_bytes());
    let result = parser.parse();
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_parser_parse_loop_optimize_remove_empty_loops_nested() {
    let code = String::from("[[[]]]");
    let mut parser = Parser::from_reader(code.as_bytes());
    let result = parser.parse();
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
