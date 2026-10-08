//! Recursive descent parser assembling tokens into a structured DslSchema AST.

use crate::dsl::ast::{
    DslEntity, DslField, DslFieldType, DslIndex, DslRelation, DslSchema, RelationKind,
};
use crate::dsl::lexer::{DslParserError, Lexer, Token};

pub struct DslParser {
    tokens: Vec<(Token, usize)>,
    cursor: usize,
}

impl DslParser {
    pub fn new(tokens: Vec<(Token, usize)>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_schema(&mut self) -> Result<DslSchema, DslParserError> {
        let mut schema = DslSchema::default();

        while !self.is_eof() {
            let (tok, line) = self.current();

            match tok {
                Token::Ident(ref name) if name.eq_ignore_ascii_case("entity") => {
                    self.cursor += 1;
                    let entity = self.parse_entity()?;
                    schema.entities.push(entity);
                }
                Token::Ident(ref name) if name.eq_ignore_ascii_case("relation") => {
                    self.cursor += 1;
                    let relation = self.parse_relation()?;
                    schema.relations.push(relation);
                }
                _ => {
                    return Err(DslParserError::SyntaxError {
                        message: format!("Expected 'entity' or 'relation', found {:?}", tok),
                        line: *line,
                    });
                }
            }
        }

        self.validate_schema(&schema)?;
        Ok(schema)
    }

    fn parse_entity(&mut self) -> Result<DslEntity, DslParserError> {
        let (name_tok, line) = self.advance()?;
        let entity_name = match name_tok {
            Token::Ident(s) => s,
            other => {
                return Err(DslParserError::SyntaxError {
                    message: format!("Expected entity name identifier, found {:?}", other),
                    line,
                })
            }
        };

        self.expect(Token::OpenBrace)?;
        let mut entity = DslEntity::new(entity_name);

        while !self.check(&Token::CloseBrace) && !self.is_eof() {
            let (tok, _) = self.current();

            if let Token::Ident(ref id) = tok {
                if id.eq_ignore_ascii_case("index") {
                    self.cursor += 1;
                    let index = self.parse_index()?;
                    entity.indexes.push(index);
                    continue;
                }
            }

            let field = self.parse_field()?;
            entity.fields.push(field);

            if self.check(&Token::Comma) {
                self.cursor += 1;
            }
        }

        self.expect(Token::CloseBrace)?;
        Ok(entity)
    }

    fn parse_field(&mut self) -> Result<DslField, DslParserError> {
        let (name_tok, line) = self.advance()?;
        let field_name = match name_tok {
            Token::Ident(s) => s,
            other => {
                return Err(DslParserError::SyntaxError {
                    message: format!("Expected field name, found {:?}", other),
                    line,
                })
            }
        };

        self.expect(Token::Colon)?;

        let (type_tok, type_line) = self.advance()?;
        let type_str = match type_tok {
            Token::Ident(s) => s,
            other => {
                return Err(DslParserError::SyntaxError {
                    message: format!("Expected type name, found {:?}", other),
                    line: type_line,
                })
            }
        };

        let field_type = DslFieldType::parse_type_name(&type_str).ok_or_else(|| {
            DslParserError::UnknownType {
                type_name: type_str,
                line: type_line,
            }
        })?;

        let mut is_primary = false;
        let mut is_required = false;
        let mut is_unique = false;
        let mut default_value = None;

        while !self.check(&Token::Comma) && !self.check(&Token::CloseBrace) && !self.is_eof() {
            let (mod_tok, _) = self.current();
            match mod_tok {
                Token::Ident(ref s) if s.eq_ignore_ascii_case("primary") => {
                    is_primary = true;
                    is_required = true;
                    self.cursor += 1;
                }
                Token::Ident(ref s) if s.eq_ignore_ascii_case("required") => {
                    is_required = true;
                    self.cursor += 1;
                }
                Token::Ident(ref s) if s.eq_ignore_ascii_case("unique") => {
                    is_unique = true;
                    self.cursor += 1;
                }
                Token::Ident(ref s) if s.eq_ignore_ascii_case("default") => {
                    self.cursor += 1;
                    self.expect(Token::OpenParen)?;
                    let mut val_parts = Vec::new();
                    while !self.check(&Token::CloseParen) && !self.is_eof() {
                        let (tok, _) = self.advance()?;
                        match tok {
                            Token::Ident(v) | Token::StringLit(v) | Token::NumberLit(v) => val_parts.push(v),
                            Token::Dot => val_parts.push(".".to_string()),
                            Token::Comma => val_parts.push(",".to_string()),
                            _ => {}
                        }
                    }
                    self.expect(Token::CloseParen)?;
                    default_value = Some(val_parts.join(""));
                }
                _ => break,
            }
        }

        Ok(DslField {
            name: field_name,
            field_type,
            is_primary,
            is_required,
            is_unique,
            default_value,
        })
    }

    fn parse_index(&mut self) -> Result<DslIndex, DslParserError> {
        let (name_tok, line) = self.advance()?;
        let idx_name = match name_tok {
            Token::Ident(s) => s,
            other => {
                return Err(DslParserError::SyntaxError {
                    message: format!("Expected index name, found {:?}", other),
                    line,
                })
            }
        };

        self.expect(Token::OpenParen)?;
        let mut fields = Vec::new();

        while !self.check(&Token::CloseParen) && !self.is_eof() {
            let (f_tok, f_line) = self.advance()?;
            if let Token::Ident(f_name) = f_tok {
                fields.push(f_name);
            } else {
                return Err(DslParserError::SyntaxError {
                    message: "Expected indexed field identifier".to_string(),
                    line: f_line,
                });
            }

            if self.check(&Token::Comma) {
                self.cursor += 1;
            }
        }

        self.expect(Token::CloseParen)?;

        let mut is_unique = false;
        if let (Token::Ident(ref s), _) = self.current() {
            if s.eq_ignore_ascii_case("unique") {
                is_unique = true;
                self.cursor += 1;
            }
        }

        Ok(DslIndex {
            name: idx_name,
            fields,
            is_unique,
        })
    }

    fn parse_relation(&mut self) -> Result<DslRelation, DslParserError> {
        let (from_ent_tok, line) = self.advance()?;
        let from_entity = match from_ent_tok {
            Token::Ident(s) => s,
            other => {
                return Err(DslParserError::SyntaxError {
                    message: format!("Expected source entity name, found {:?}", other),
                    line,
                })
            }
        };

        self.expect(Token::Dot)?;

        let (from_f_tok, f_line) = self.advance()?;
        let from_field = match from_f_tok {
            Token::Ident(s) => s,
            _ => return Err(DslParserError::SyntaxError { message: "Expected field".into(), line: f_line }),
        };

        self.expect(Token::Arrow)?;

        let (to_ent_tok, t_line) = self.advance()?;
        let to_entity = match to_ent_tok {
            Token::Ident(s) => s,
            _ => return Err(DslParserError::SyntaxError { message: "Expected target entity".into(), line: t_line }),
        };

        self.expect(Token::Dot)?;

        let (to_f_tok, tf_line) = self.advance()?;
        let to_field = match to_f_tok {
            Token::Ident(s) => s,
            _ => return Err(DslParserError::SyntaxError { message: "Expected target field".into(), line: tf_line }),
        };

        self.expect(Token::OpenBracket)?;

        let (kind_tok, k_line) = self.advance()?;
        let kind_str = match kind_tok {
            Token::Ident(s) => s,
            _ => return Err(DslParserError::SyntaxError { message: "Expected relation kind".into(), line: k_line }),
        };

        let kind = RelationKind::parse_kind(&kind_str).ok_or_else(|| {
            DslParserError::SyntaxError {
                message: format!("Invalid relation kind '{}'", kind_str),
                line: k_line,
            }
        })?;

        self.expect(Token::CloseBracket)?;

        Ok(DslRelation {
            from_entity,
            from_field,
            to_entity,
            to_field,
            kind,
        })
    }

    fn validate_schema(&self, schema: &DslSchema) -> Result<(), DslParserError> {
        for rel in &schema.relations {
            if schema.find_entity(&rel.from_entity).is_none() {
                return Err(DslParserError::ValidationError {
                    message: format!("Relation references unknown source entity '{}'", rel.from_entity),
                });
            }
            if schema.find_entity(&rel.to_entity).is_none() {
                return Err(DslParserError::ValidationError {
                    message: format!("Relation references unknown target entity '{}'", rel.to_entity),
                });
            }
        }
        Ok(())
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn current(&self) -> &(Token, usize) {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            static EOF: (Token, usize) = (Token::CloseBrace, 0);
            &EOF
        }
    }

    fn advance(&mut self) -> Result<(Token, usize), DslParserError> {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            Ok(tok)
        } else {
            Err(DslParserError::SyntaxError {
                message: "Unexpected end of input".to_string(),
                line: 0,
            })
        }
    }

    fn check(&self, target: &Token) -> bool {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor].0 == target
        } else {
            false
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), DslParserError> {
        let (tok, line) = self.advance()?;
        if tok == expected {
            Ok(())
        } else {
            Err(DslParserError::SyntaxError {
                message: format!("Expected {:?}, found {:?}", expected, tok),
                line,
            })
        }
    }
}

/// Convenience function that lexes and parses a DSL script.
pub fn parse_dsl(script: &str) -> Result<DslSchema, DslParserError> {
    let mut lexer = Lexer::new(script);
    let tokens = lexer.tokenize()?;
    let mut parser = DslParser::new(tokens);
    parser.parse_schema()
}
