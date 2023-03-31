use std::iter::zip;

use crate::{Lexer, Token};

use super::utils::test_lexer;

#[test]
fn test_eof_true() {
    let code = String::from("");
    let mut lexer = Lexer {
        reader: code.as_bytes(),
    };
    assert!(lexer.eof());
}

#[test]
fn test_eof_false() {
    let code = String::from(".");
    let mut lexer = Lexer {
        reader: code.as_bytes(),
    };
    assert!(!lexer.eof());
}

#[test]
fn test_next_token_valid_tokens() {
    let code = String::from("><,.+-[]");
    let mut lexer = Lexer {
        reader: code.as_bytes(),
    };
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
fn test_next_token_other_symbols() {
    let code = String::from("abcdef");
    let mut lexer = Lexer {
        reader: code.as_bytes(),
    };
    while !lexer.eof() {
        let token = lexer.next_token();
        assert!(token.is_none());
    }
}

#[test]
fn test_iter_valid_tokens() {
    let code = String::from("><,.+-[]");
    let lexer = Lexer {
        reader: code.as_bytes(),
    };
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
fn test_iter_other_symbols() {
    let code = String::from("abcdef");
    let expected: Vec<Option<Token>> = vec![None, None, None, None, None, None];
    test_lexer(&code, &expected);
}

#[test]
fn test_for_loop() {
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
