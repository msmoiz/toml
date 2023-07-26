use std::collections::HashMap;

use crate::lexer::{Context, Lexer, Token};

use crate::error::{Error, Result};
use crate::toml::Value;

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
        let key = match self.lexer.next(Context::default())? {
            Some(Token::BareKey(key)) | Some(Token::String(key)) => key,
            _ => {
                return Err(Error::Parse);
            }
        };

        if self.root.as_table().contains_key(&key) {
            // key may not be redefined
            return Err(Error::Parse);
        }

        let Some(Token::Equal) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };

        let Some(Token::String(value)) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };

        match self.lexer.next(Context::default())? {
            Some(Token::Newline) | None => {}
            _ => return Err(Error::Parse),
        }

        self.root.insert(key, Value::String(value));

        Ok(())
    }
}
