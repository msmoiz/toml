use std::collections::HashMap;

use crate::lexer::{Context, Lexer, Posture, Token};

use crate::error::{Error, Result};
use crate::toml::{Table, Value};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    root: Value,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: Lexer::new(text),
            root: Value::Table(HashMap::new()),
        }
    }

    pub fn from_str(text: &'a str) -> Result<Value> {
        let mut parser = Parser::new(text);
        parser.toml()
    }

    fn toml(&mut self) -> Result<Value> {
        while let Some(token) = self.lexer.peek(Context::default())? {
            match token {
                Token::Newline => {
                    self.lexer.next(Context::default())?;
                }
                Token::BareKey(_) | Token::String(_) => self.key_val()?,
                _ => return Err(Error::Parse),
            }
        }
        Ok(self.root.clone())
    }

    fn key_val(&mut self) -> Result<()> {
        let mut table = &mut self.root;
        let mut key = match self.lexer.next(Context::default())? {
            Some(Token::BareKey(key)) | Some(Token::String(key)) => key,
            _ => {
                return Err(Error::Parse);
            }
        };

        // dotted keys
        while let Some(Token::Dot) = self.lexer.peek(Context::default())? {
            self.lexer.next(Context::default())?; // ignore dot
            key = match self.lexer.next(Context::default())? {
                Some(Token::BareKey(next_key)) | Some(Token::String(next_key)) => {
                    table = if table.as_table().contains_key(&key) {
                        if !matches!(table[&key], Value::Table(_)) {
                            // prev key has already been defined as something
                            // other than a table
                            return Err(Error::Parse);
                        }
                        table.as_table_mut().get_mut(&key).unwrap()
                    } else {
                        table
                            .as_table_mut()
                            .insert(key.clone(), Value::Table(Table::new()));
                        table.as_table_mut().get_mut(&key).unwrap()
                    };
                    next_key
                }
                _ => {
                    return Err(Error::Parse);
                }
            };
        }

        if table.as_table().contains_key(&key) {
            // key may not be redefined
            return Err(Error::Parse);
        }

        let Some(Token::Equal) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };

        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let value = match self.lexer.next(context)? {
            Some(value) => match value {
                Token::Newline
                | Token::BareKey(_)
                | Token::Equal
                | Token::Dot
                | Token::Comma
                | Token::RightBrace
                | Token::RightBracket => return Err(Error::Parse),
                Token::String(x) => Value::String(x),
                Token::Integer(x) => Value::Integer(x),
                Token::Float(x) => Value::Float(x),
                Token::Bool(x) => Value::Bool(x),
                Token::DateTime(x) => Value::OffsetDateTime(x),
                Token::DateTimeLocal(x) => Value::LocalDateTime(x),
                Token::DateLocal(x) => Value::LocalDate(x),
                Token::TimeLocal(x) => Value::LocalTime(x),
                _ => todo!(),
            },
            None => return Err(Error::Parse),
        };

        match self.lexer.next(Context::default())? {
            Some(Token::Newline) | None => {}
            _ => return Err(Error::Parse),
        }

        table.insert(key, value);

        Ok(())
    }
}
