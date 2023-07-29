use std::collections::HashMap;

use crate::lexer::{Context, Lexer, Posture, Token};

use crate::error::{Error, Result};
use crate::toml::{Table, Value};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    root: Value,
    current_table_chain: Vec<String>,
    explicitly_defined_tables: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: Lexer::new(text),
            root: Value::Table(HashMap::new()),
            current_table_chain: Vec::new(),
            explicitly_defined_tables: Vec::new(),
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
                Token::String(_) => self.key_val()?,
                Token::LeftBracket => self.table()?,
                _ => return Err(Error::Parse),
            }
        }
        Ok(self.root.clone())
    }

    fn key_val(&mut self) -> Result<()> {
        let keychain = self.keychain()?;
        let Some(Token::Equal) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        let value = self.value()?;
        self.newline_or_eof()?;

        let mut table = self.current_table_mut()?;
        for key in &keychain[..keychain.len() - 1] {
            table = match table.get(key) {
                Some(Value::Table(_)) => table.get_mut(key).unwrap().as_table_mut(),
                Some(_) => return Err(Error::Parse),
                None => {
                    table.insert(key.clone(), Value::Table(Table::new()));
                    table.get_mut(key).unwrap().as_table_mut()
                }
            }
        }

        let leaf_key = keychain.last().unwrap();
        if table.contains_key(leaf_key) {
            return Err(Error::Parse);
        }

        table.insert(leaf_key.clone(), value);

        Ok(())
    }

    fn keychain(&mut self) -> Result<Vec<String>> {
        let Some(Token::String(key)) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        let mut chain = vec![key];
        while let Some(Token::Dot) = self.lexer.peek(Context::default())? {
            self.lexer.next(Context::default())?; // skip dot
            match self.lexer.next(Context::default())? {
                Some(Token::String(key)) => chain.push(key),
                _ => return Err(Error::Parse),
            }
        }
        Ok(chain)
    }

    fn value(&mut self) -> Result<Value> {
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let value = match self.lexer.next(context)? {
            Some(value) => match value {
                Token::Newline
                | Token::Equal
                | Token::Dot
                | Token::Comma
                | Token::RightBrace
                | Token::RightBracket => return Err(Error::Parse),
                Token::String(x) => Value::String(x),
                Token::Integer(x) => Value::Integer(x),
                Token::Float(x) => Value::Float(x),
                Token::Bool(x) => Value::Bool(x),
                Token::OffsetDateTime(x) => Value::OffsetDateTime(x),
                Token::LocalDateTime(x) => Value::LocalDateTime(x),
                Token::LocalDate(x) => Value::LocalDate(x),
                Token::LocalTime(x) => Value::LocalTime(x),
                _ => todo!(),
            },
            None => return Err(Error::Parse),
        };
        Ok(value)
    }

    fn newline_or_eof(&mut self) -> Result<()> {
        match self.lexer.next(Context::default())? {
            Some(Token::Newline) | None => Ok(()),
            _ => Err(Error::Parse),
        }
    }

    fn table(&mut self) -> Result<()> {
        let Some(Token::LeftBracket) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        let keychain = self.keychain()?;
        let Some(Token::RightBracket) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        self.newline_or_eof()?;

        if self.explicitly_defined_tables.contains(&keychain.join(".")) {
            return Err(Error::Parse);
        }

        let mut table = self.root.as_table_mut();
        for key in &keychain {
            table = match table.get(key) {
                Some(Value::Table(_)) => table.get_mut(key).unwrap().as_table_mut(),
                Some(_) => return Err(Error::Parse),
                None => {
                    table.insert(key.clone(), Value::Table(Table::new()));
                    table.get_mut(key).unwrap().as_table_mut()
                }
            }
        }

        self.explicitly_defined_tables.push(keychain.join("."));
        self.current_table_chain = keychain;

        Ok(())
    }

    fn current_table_mut(&mut self) -> Result<&mut Table> {
        let mut table = self.root.as_table_mut();
        for key in &self.current_table_chain {
            table = match table.get(key) {
                Some(Value::Table(_)) => table.get_mut(key).unwrap().as_table_mut(),
                Some(_) => return Err(Error::Parse),
                None => return Err(Error::Parse),
            }
        }
        Ok(table)
    }
}
