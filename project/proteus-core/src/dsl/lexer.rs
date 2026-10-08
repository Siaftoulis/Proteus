//! Lexer and token stream for the Proteus Database Scripting DSL.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DslParserError {
    #[error("Syntax error at line {line}: {message}")]
    SyntaxError { message: String, line: usize },

    #[error("Unknown data type '{type_name}' at line {line}")]
    UnknownType { type_name: String, line: usize },

    #[error("Semantic validation error: {message}")]
    ValidationError { message: String },
}

/// Token types extracted during lexical analysis.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    StringLit(String),
    NumberLit(String),
    Colon,
    Comma,
    Dot,
    Arrow, // ->
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
}

/// Lexer that breaks DSL text into a stream of tokens with line numbering.
pub struct Lexer<'a> {
    chars: Vec<(usize, char)>, // (byte_offset, char)
    pos: usize,
    current_line: usize,
    _input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            _input: input,
            chars: input.char_indices().collect(),
            pos: 0,
            current_line: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<(Token, usize)>, DslParserError> {
        let mut tokens = Vec::new();

        while self.pos < self.chars.len() {
            let (_, ch) = self.chars[self.pos];

            if ch == '\n' {
                self.current_line += 1;
                self.pos += 1;
                continue;
            }

            if ch.is_whitespace() {
                self.pos += 1;
                continue;
            }

            // Comments (// or #)
            if ch == '#' || (ch == '/' && self.peek_char() == Some('/')) {
                while self.pos < self.chars.len() && self.chars[self.pos].1 != '\n' {
                    self.pos += 1;
                }
                continue;
            }

            let line = self.current_line;

            if ch == ':' {
                tokens.push((Token::Colon, line));
                self.pos += 1;
            } else if ch == ',' {
                tokens.push((Token::Comma, line));
                self.pos += 1;
            } else if ch == '.' {
                tokens.push((Token::Dot, line));
                self.pos += 1;
            } else if ch == '-' && self.peek_char() == Some('>') {
                tokens.push((Token::Arrow, line));
                self.pos += 2;
            } else if ch == '{' {
                tokens.push((Token::OpenBrace, line));
                self.pos += 1;
            } else if ch == '}' {
                tokens.push((Token::CloseBrace, line));
                self.pos += 1;
            } else if ch == '(' {
                tokens.push((Token::OpenParen, line));
                self.pos += 1;
            } else if ch == ')' {
                tokens.push((Token::CloseParen, line));
                self.pos += 1;
            } else if ch == '[' {
                tokens.push((Token::OpenBracket, line));
                self.pos += 1;
            } else if ch == ']' {
                tokens.push((Token::CloseBracket, line));
                self.pos += 1;
            } else if ch == '"' || ch == '\'' {
                let s = self.read_string_literal(ch)?;
                tokens.push((Token::StringLit(s), line));
            } else if ch.is_ascii_alphanumeric() || ch == '_' {
                let s = self.read_ident_or_number();
                tokens.push((Token::Ident(s), line));
            } else {
                return Err(DslParserError::SyntaxError {
                    message: format!("Unexpected character '{}'", ch),
                    line,
                });
            }
        }

        Ok(tokens)
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos + 1 < self.chars.len() {
            Some(self.chars[self.pos + 1].1)
        } else {
            None
        }
    }

    fn read_string_literal(&mut self, quote: char) -> Result<String, DslParserError> {
        let line = self.current_line;
        self.pos += 1; // skip opening quote
        let mut result = String::new();

        while self.pos < self.chars.len() {
            let (_, ch) = self.chars[self.pos];
            if ch == quote {
                self.pos += 1; // skip closing quote
                return Ok(result);
            }
            if ch == '\n' {
                self.current_line += 1;
            }
            result.push(ch);
            self.pos += 1;
        }

        Err(DslParserError::SyntaxError {
            message: "Unterminated string literal".to_string(),
            line,
        })
    }

    fn read_ident_or_number(&mut self) -> String {
        let mut result = String::new();
        while self.pos < self.chars.len() {
            let (_, ch) = self.chars[self.pos];
            if ch.is_ascii_alphanumeric() || ch == '_' {
                result.push(ch);
                self.pos += 1;
            } else if ch == '.'
                && self.pos + 1 < self.chars.len()
                && self.chars[self.pos + 1].1.is_ascii_digit()
                && !result.is_empty()
                && result.chars().all(|c| c.is_ascii_digit())
            {
                result.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        result
    }
}
