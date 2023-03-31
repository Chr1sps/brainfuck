use crate::Statement;

use super::utils::test_optimize_once;

#[test]
fn test_optimize_once_no_optimization() {
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
fn test_optimize_once_adds() {
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
fn test_optimize_once_adds_overflow() {
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
fn test_optimize_once_adds_no_add() {
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
fn test_optimize_once_moves_right() {
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
fn test_optimize_once_moves_left_and_right() {
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
fn test_optimize_once_moves_left_and_right_no_shift() {
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
fn test_optimize_once_adds_and_moves() {
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
fn test_optimize_once_adds_with_loop() {
    let input: Vec<Statement> = vec![
        Statement::Add(3),
        Statement::Add(4),
        Statement::Add(3),
        Statement::Add(4),
        Statement::JumpIf(2),
        Statement::JumpIf(0),
    ];
    let output = vec![
        Statement::Add(7),
        Statement::Add(7),
        Statement::JumpIf(1),
        Statement::JumpIf(0),
    ];
    test_optimize_once(&input, &output);
}
