mod lexer;
mod machine;
mod optimizer;
mod parser;

// helper testing functions
mod utils {
    use crate::{Lexer, Optimizer, Parser, Statement, Token};
    use std::io::Error;
    pub(in crate::tests) fn test_lexer(code: &String, expected: &Vec<Option<Token>>) {
        let lexer = Lexer {
            reader: code.as_bytes(),
        };
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
