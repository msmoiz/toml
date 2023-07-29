use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::lexer::{Context, Lexer, Posture, Token};

use crate::error::{Error, Result};
use crate::toml::{Table, Value};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    root: Value,
    current_table_key: Vec<String>,
    predefined_tables: Vec<String>,
    inlined_tables: Vec<String>,
    inlined_arrays: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: Lexer::new(text),
            root: Value::Table(HashMap::new()),
            current_table_key: Vec::new(),
            predefined_tables: Vec::new(),
            inlined_tables: Vec::new(),
            inlined_arrays: Vec::new(),
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
                    let (key, value) = self.key_value_pair()?;
                    self.require_newline_or_eof()?;
                    let mut table = self.current_table_mut()?;
                    let subtable_key = &key[..key.len() - 1];
                    let subtable = Self::find_or_create_subtable_mut(&mut table, subtable_key)?;
                    let last_segment = key.last().unwrap();
                    if subtable.contains_key(last_segment) {
                        return Err(Error::Parse);
                    }
                    subtable.insert(last_segment.clone(), value.clone());
                    let absolute_key =
                        self.absolute_key_string(&self.current_table_key, &key[..key.len() - 1])?;
                    self.predefined_tables.push(absolute_key.clone());
                    if self
                        .inlined_tables
                        .iter()
                        .any(|table| format!("{absolute_key}.{last_segment}").starts_with(table))
                    {
                        return Err(Error::Parse);
                    }
                    if matches!(value, Value::Table(_)) {
                        self.inlined_tables
                            .push(format!("{absolute_key}.{last_segment}"));
                    }
                    if matches!(value, Value::Array(_)) {
                        self.inlined_arrays
                            .push(format!("{absolute_key}.{last_segment}"));
                    }
                }
                Token::LeftBracket => {
                    let mut lookahead = self.lexer.clone();
                    lookahead.next(Context::default())?; // skip first bracket
                    match lookahead.next(Context::default())? {
                        Some(Token::LeftBracket) => {
                            let key = self.array_of_tables()?;
                            let mut table = self.root.as_table_mut();
                            for segment in &key[..key.len() - 1] {
                                table = match table.get(segment) {
                                    Some(Value::Table(_)) => {
                                        table.get_mut(segment).unwrap().as_table_mut()
                                    }
                                    Some(Value::Array(_)) => {
                                        let arr = table.get_mut(segment).unwrap().as_arr_mut();
                                        arr.last_mut().unwrap().as_table_mut()
                                    }
                                    Some(_) => return Err(Error::Parse),
                                    None => {
                                        table.insert(segment.clone(), Value::Table(Table::new()));
                                        table.get_mut(segment).unwrap().as_table_mut()
                                    }
                                }
                            }

                            let last_segment = key.last().unwrap();
                            match table.get_mut(last_segment) {
                                Some(Value::Array(arr)) => arr.push(Value::Table(Table::new())),
                                None => {
                                    table.insert(
                                        last_segment.clone(),
                                        Value::Array(vec![Value::Table(Table::new())]),
                                    );
                                }
                                _ => return Err(Error::Parse),
                            }

                            let absolute_key =
                                self.absolute_key_string(&[], &key[..key.len() - 1])?;
                            if self
                                .inlined_arrays
                                .contains(&format!("{absolute_key}.{last_segment}"))
                            {
                                return Err(Error::Parse);
                            }

                            self.current_table_key = key;
                        }
                        _ => {
                            let key = self.table()?;
                            let mut table = self.root.as_table_mut();
                            Self::find_or_create_subtable_mut(&mut table, &key)?;
                            let abs_key = self.absolute_key_string(&[], &key)?;
                            if self.predefined_tables.contains(&abs_key) {
                                return Err(Error::Parse);
                            }
                            self.predefined_tables.push(abs_key);
                            self.current_table_key = key;
                        }
                    }
                }
                _ => return Err(Error::Parse),
            }
        }
        Ok(self.root.clone())
    }

    fn key_value_pair(&mut self) -> Result<(Vec<String>, Value)> {
        let key = self.key()?;
        self.require(Token::Equal)?;
        let value = self.value()?;
        Ok((key, value))
    }

    fn key(&mut self) -> Result<Vec<String>> {
        let mut key = Vec::new();
        let segment = self.require_string()?;
        key.push(segment);
        while let Some(Token::Dot) = self.lexer.peek(Context::default())? {
            self.require(Token::Dot)?;
            let segment = self.require_string()?;
            key.push(segment);
        }
        Ok(key)
    }

    fn value(&mut self) -> Result<Value> {
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let value = match self.lexer.peek(context.clone())? {
            Some(Token::String(x)) => Value::String(x),
            Some(Token::Integer(x)) => Value::Integer(x),
            Some(Token::Float(x)) => Value::Float(x),
            Some(Token::Bool(x)) => Value::Bool(x),
            Some(Token::OffsetDateTime(x)) => Value::OffsetDateTime(x),
            Some(Token::LocalDateTime(x)) => Value::LocalDateTime(x),
            Some(Token::LocalDate(x)) => Value::LocalDate(x),
            Some(Token::LocalTime(x)) => Value::LocalTime(x),
            Some(Token::LeftBrace) => self.inline_table()?,
            Some(Token::LeftBracket) => self.array()?,
            _ => return Err(Error::Parse),
        };

        if matches!(
            value,
            Value::String(_)
                | Value::Integer(_)
                | Value::Float(_)
                | Value::Bool(_)
                | Value::OffsetDateTime(_)
                | Value::LocalDateTime(_)
                | Value::LocalDate(_)
                | Value::LocalTime(_)
        ) {
            // consume the previously peeked token
            self.lexer.next(context)?;
        }

        Ok(value)
    }

    fn inline_table(&mut self) -> Result<Value> {
        let mut inline_table = Table::new();
        self.require(Token::LeftBrace)?;

        match self.lexer.peek(Context::default())? {
            Some(Token::RightBrace) => {}
            Some(Token::String(_)) => {
                let (key, value) = self.key_value_pair()?;
                let root = &mut inline_table;
                let subtable_key = &key[..key.len() - 1];
                let subtable = Self::find_or_create_subtable_mut(root, subtable_key)?;
                let last_segment = key.last().unwrap();
                if subtable.contains_key(last_segment) {
                    return Err(Error::Parse);
                }
                subtable.insert(last_segment.clone(), value);

                while let Some(Token::Comma) = self.lexer.peek(Context::default())? {
                    self.require(Token::Comma)?;
                    let (key, value) = self.key_value_pair()?;
                    let root = &mut inline_table;
                    let subtable_key = &key[..key.len() - 1];
                    let subtable = Self::find_or_create_subtable_mut(root, subtable_key)?;
                    let last_segment = key.last().unwrap();
                    if subtable.contains_key(last_segment) {
                        return Err(Error::Parse);
                    }
                    subtable.insert(last_segment.clone(), value);
                }
            }
            _ => return Err(Error::Parse),
        }

        self.require(Token::RightBrace)?;
        Ok(Value::Table(inline_table))
    }

    fn array(&mut self) -> Result<Value> {
        let mut array = Vec::new();
        self.require(Token::LeftBracket)?;

        match self.lexer.peek(Context::default())? {
            Some(Token::RightBracket) => {}
            _ => {
                self.skip_newlines()?;
                let value = self.value()?;
                array.push(value);
                self.skip_newlines()?;
                while let Some(Token::Comma) = self.lexer.peek(Context::default())? {
                    self.require(Token::Comma)?;
                    self.skip_newlines()?;
                    match self.lexer.peek(Context::default())? {
                        Some(Token::RightBracket) => break,
                        _ => {
                            let value = self.value()?;
                            array.push(value);
                            self.skip_newlines()?;
                        }
                    }
                }
            }
        }

        self.require(Token::RightBracket)?;
        Ok(Value::Array(array))
    }

    fn table(&mut self) -> Result<Vec<String>> {
        self.require(Token::LeftBracket)?;
        let key = self.key()?;
        self.require(Token::RightBracket)?;
        self.require_newline_or_eof()?;
        Ok(key)
    }

    fn array_of_tables(&mut self) -> Result<Vec<String>> {
        self.require(Token::LeftBracket)?;
        self.require(Token::LeftBracket)?;
        let key = self.key()?;
        self.require(Token::RightBracket)?;
        self.require(Token::RightBracket)?;
        self.require_newline_or_eof()?;
        Ok(key)
    }

    fn find_or_create_subtable_mut<'t>(
        root: &'t mut Table,
        key: &[String],
    ) -> Result<&'t mut Table> {
        let mut table = root;
        for segment in key {
            table = match table.entry(segment.clone()) {
                Entry::Vacant(vacancy) => vacancy.insert(Value::Table(Table::new())).as_table_mut(),
                Entry::Occupied(occupant) => match occupant.into_mut() {
                    Value::Table(table) => table,
                    Value::Array(array) => array.last_mut().unwrap().as_table_mut(),
                    _ => return Err(Error::Parse),
                },
            }
        }
        Ok(table)
    }

    fn current_table_mut(&mut self) -> Result<&mut Table> {
        let mut table = self.root.as_table_mut();
        for segment in &self.current_table_key {
            table = match table.get(segment) {
                Some(Value::Table(_)) => table.get_mut(segment).unwrap().as_table_mut(),
                Some(Value::Array(_)) => {
                    let arr = table.get_mut(segment).unwrap().as_arr_mut();
                    arr.last_mut().unwrap().as_table_mut()
                }
                _ => return Err(Error::Parse),
            }
        }
        Ok(table)
    }

    fn require(&mut self, token: Token) -> Result<()> {
        match self.lexer.next(Context::default())? {
            Some(actual) if actual == token => Ok(()),
            _ => Err(Error::Parse),
        }
    }

    fn require_string(&mut self) -> Result<String> {
        match self.lexer.next(Context::default())? {
            Some(Token::String(string)) => Ok(string),
            _ => Err(Error::Parse),
        }
    }

    fn require_newline_or_eof(&mut self) -> Result<()> {
        match self.lexer.next(Context::default())? {
            Some(Token::Newline) | None => Ok(()),
            _ => Err(Error::Parse),
        }
    }

    fn skip_newlines(&mut self) -> Result<()> {
        while let Some(Token::Newline) = self.lexer.peek(Context::default())? {
            self.lexer.next(Context::default())?;
        }
        Ok(())
    }

    fn absolute_key_string(&self, base_key: &[String], rel_key: &[String]) -> Result<String> {
        let mut string = String::new();
        let mut table = self.root.as_table();
        for segment in base_key
            .iter()
            .chain(rel_key.iter())
            .collect::<Vec<&String>>()
        {
            table = match table.get(segment) {
                Some(Value::Table(table)) => {
                    string.push_str(&format!(".{segment}"));
                    table
                }
                Some(Value::Array(array)) => {
                    string.push_str(&format!(".{segment}.{}", array.len() - 1));
                    array.last().unwrap().as_table()
                }
                _ => return Err(Error::Parse),
            }
        }
        Ok(string)
    }
}
