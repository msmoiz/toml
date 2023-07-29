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
                Token::String(_) => {
                    let (keychain, value) = self.key_val()?;
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
                }
                Token::LeftBracket => {
                    let mut lookahead = self.lexer.clone();
                    lookahead.next(Context::default())?; // skip first bracket
                    if let Some(Token::LeftBracket) = lookahead.next(Context::default())? {
                        self.array_of_tables()?
                    } else {
                        self.table()?
                    }
                }
                _ => return Err(Error::Parse),
            }
        }
        Ok(self.root.clone())
    }

    fn key_val(&mut self) -> Result<(Vec<String>, Value)> {
        let keychain = self.keychain()?;
        let Some(Token::Equal) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        let value = self.value()?;
        Ok((keychain, value))
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
        let value = match self.lexer.peek(context.clone())? {
            Some(value) => match value {
                Token::Newline
                | Token::Equal
                | Token::Dot
                | Token::Comma
                | Token::RightBrace
                | Token::RightBracket => return Err(Error::Parse),
                Token::String(x) => {
                    self.lexer.next(context)?;
                    Value::String(x)
                }
                Token::Integer(x) => {
                    self.lexer.next(context)?;
                    Value::Integer(x)
                }
                Token::Float(x) => {
                    self.lexer.next(context)?;
                    Value::Float(x)
                }
                Token::Bool(x) => {
                    self.lexer.next(context)?;
                    Value::Bool(x)
                }
                Token::OffsetDateTime(x) => {
                    self.lexer.next(context)?;
                    Value::OffsetDateTime(x)
                }
                Token::LocalDateTime(x) => {
                    self.lexer.next(context)?;
                    Value::LocalDateTime(x)
                }
                Token::LocalDate(x) => {
                    self.lexer.next(context)?;
                    Value::LocalDate(x)
                }
                Token::LocalTime(x) => {
                    self.lexer.next(context)?;
                    Value::LocalTime(x)
                }
                Token::LeftBrace => self.inline_table()?,
                Token::LeftBracket => self.array()?,
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

    fn inline_table(&mut self) -> Result<Value> {
        let mut table = Table::new();

        let Some(Token::LeftBrace) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };

        match self.lexer.peek(Context::default())? {
            Some(Token::String(_)) => {
                let (keychain, value) = self.key_val()?;
                let mut subtable = &mut table;
                for key in &keychain[..keychain.len() - 1] {
                    subtable = match subtable.get(key) {
                        Some(Value::Table(_)) => subtable.get_mut(key).unwrap().as_table_mut(),
                        Some(_) => return Err(Error::Parse),
                        None => {
                            subtable.insert(key.clone(), Value::Table(Table::new()));
                            subtable.get_mut(key).unwrap().as_table_mut()
                        }
                    }
                }

                let leaf_key = keychain.last().unwrap();
                if subtable.contains_key(leaf_key) {
                    return Err(Error::Parse);
                }

                subtable.insert(leaf_key.clone(), value);

                while let Some(Token::Comma) = self.lexer.peek(Context::default())? {
                    self.lexer.next(Context::default())?; // skip comma
                    let (keychain, value) = self.key_val()?;
                    let mut subtable = &mut table;
                    for key in &keychain[..keychain.len() - 1] {
                        subtable = match subtable.get(key) {
                            Some(Value::Table(_)) => subtable.get_mut(key).unwrap().as_table_mut(),
                            Some(_) => return Err(Error::Parse),
                            None => {
                                subtable.insert(key.clone(), Value::Table(Table::new()));
                                subtable.get_mut(key).unwrap().as_table_mut()
                            }
                        }
                    }

                    let leaf_key = keychain.last().unwrap();
                    if subtable.contains_key(leaf_key) {
                        return Err(Error::Parse);
                    }

                    subtable.insert(leaf_key.clone(), value);
                }
            }
            Some(Token::RightBrace) => {}
            _ => return Err(Error::Parse),
        }

        let Some(Token::RightBrace) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };

        Ok(Value::Table(table))
    }

    fn array(&mut self) -> Result<Value> {
        let mut array = Vec::new();

        let Some(Token::LeftBracket) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };

        match self.lexer.peek(Context::default())? {
            Some(Token::RightBracket) => {}
            _ => {
                while let Some(Token::Newline) = self.lexer.peek(Context::default())? {
                    self.lexer.next(Context::default())?; // skip newline
                }

                let value = self.value()?;
                array.push(value);

                while let Some(Token::Newline) = self.lexer.peek(Context::default())? {
                    self.lexer.next(Context::default())?; // skip newline
                }

                while let Some(Token::Comma) = self.lexer.peek(Context::default())? {
                    self.lexer.next(Context::default())?; // skip comma

                    while let Some(Token::Newline) = self.lexer.peek(Context::default())? {
                        self.lexer.next(Context::default())?; // skip newline
                    }

                    if let Some(Token::RightBracket) = self.lexer.peek(Context::default())? {
                        break;
                    };

                    let value = self.value()?;
                    array.push(value);

                    while let Some(Token::Newline) = self.lexer.peek(Context::default())? {
                        self.lexer.next(Context::default())?; // skip newline
                    }
                }
            }
        }

        let Some(Token::RightBracket) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };

        Ok(Value::Array(array))
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
                Some(Value::Array(_)) => table
                    .get_mut(key)
                    .unwrap()
                    .as_arr_mut()
                    .last_mut()
                    .unwrap()
                    .as_table_mut(),
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

    fn array_of_tables(&mut self) -> Result<()> {
        let Some(Token::LeftBracket) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        let Some(Token::LeftBracket) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        let keychain = self.keychain()?;
        let Some(Token::RightBracket) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        let Some(Token::RightBracket) = self.lexer.next(Context::default())? else {
            return Err(Error::Parse);
        };
        self.newline_or_eof()?;

        let mut table = self.root.as_table_mut();
        for key in &keychain[..keychain.len() - 1] {
            table = match table.get(key) {
                Some(Value::Table(_)) => table.get_mut(key).unwrap().as_table_mut(),
                Some(Value::Array(_)) => table
                    .get_mut(key)
                    .unwrap()
                    .as_arr_mut()
                    .last_mut()
                    .unwrap()
                    .as_table_mut(),
                Some(_) => return Err(Error::Parse),
                None => {
                    table.insert(key.clone(), Value::Table(Table::new()));
                    table.get_mut(key).unwrap().as_table_mut()
                }
            }
        }

        let leaf_key = keychain.last().unwrap();
        match table.get_mut(leaf_key) {
            Some(Value::Array(arr)) => arr.push(Value::Table(Table::new())),
            None => {
                table.insert(
                    leaf_key.clone(),
                    Value::Array(vec![Value::Table(Table::new())]),
                );
            }
            _ => {}
        }

        self.current_table_chain = keychain;

        Ok(())
    }

    fn current_table_mut(&mut self) -> Result<&mut Table> {
        let mut table = self.root.as_table_mut();
        println!("{table:?}");
        for key in &self.current_table_chain {
            table = match table.get(key) {
                Some(Value::Table(_)) => table.get_mut(key).unwrap().as_table_mut(),
                Some(Value::Array(_)) => table
                    .get_mut(key)
                    .unwrap()
                    .as_arr_mut()
                    .last_mut()
                    .unwrap()
                    .as_table_mut(),
                Some(_) => return Err(Error::Parse),
                None => return Err(Error::Parse),
            }
        }
        Ok(table)
    }
}
