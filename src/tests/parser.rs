use std::io::{Error, ErrorKind};

use crate::Statement;

use super::utils::{test_parser, test_parser_error};

#[test]
fn test_parse_empty_string() {
    let code = String::from("");
    let expected: Vec<Statement> = Vec::new();
    test_parser(&code, &expected);
}

#[test]
fn test_parse_non_loop_statements() {
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
fn test_parse_countable_optimization() {
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
fn test_parse_loop_valid() {
    let code = String::from("[+-<>]+");
    let expected = vec![
        Statement::new_loop(vec![
            Statement::Add(1),
            Statement::Add(255),
            Statement::MoveLeft(1),
            Statement::MoveRight(1),
        ]),
        Statement::Add(1),
    ];
    test_parser(&code, &expected);
}

#[test]
fn test_parse_loop_end_of_file() {
    let code = String::from("[+-<>]");
    let expected = vec![Statement::new_loop(vec![
        Statement::Add(1),
        Statement::Add(255),
        Statement::MoveLeft(1),
        Statement::MoveRight(1),
    ])];
    test_parser(&code, &expected);
}

#[test]
fn test_parse_loop_invalid_redundant_left_bracket() {
    let code = String::from("[[++++----<<<<>>>>]");
    let error = Error::new(
        ErrorKind::InvalidData,
        "Error: '[' found with no matching ']'.".to_string(),
    );
    test_parser_error(&code, &error);
}

#[test]
fn test_parse_loop_invalid_redundant_right_bracket() {
    let code = String::from("[++++----]<<<<>>>>]");
    let error = Error::new(
        ErrorKind::InvalidData,
        "Error: ']' found with no matching '['.".to_string(),
    );
    test_parser_error(&code, &error);
}

#[test]
fn test_parse_loop_optimize_remove_empty_loops() {
    let code = String::from("[][][]");
    let result: Vec<Statement> = Vec::new();
    test_parser(&code, &result);
}

#[test]
fn test_parse_loop_optimize_remove_empty_loops_nested() {
    let code = String::from("[[[]]]");
    let result: Vec<Statement> = Vec::new();
    test_parser(&code, &result);
}
