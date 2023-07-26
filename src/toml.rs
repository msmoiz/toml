use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};

pub type Table = HashMap<String, Value>;
pub type Array = Vec<Value>;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    OffsetDateTime(DateTime<FixedOffset>),
    LocalDateTime(NaiveDateTime),
    LocalDate(NaiveDate),
    LocalTime(NaiveTime),
    Array(Array),
    Table(Table),
}

impl Value {
    pub fn as_str(&self) -> &str {
        match self {
            Value::String(str) => str,
            _ => panic!("not a string value"),
        }
    }

    pub fn as_table(&self) -> &HashMap<String, Value> {
        match self {
            Value::Table(table) => table,
            _ => panic!("not a string value"),
        }
    }

    pub fn insert(&mut self, key: String, value: Value) {
        match self {
            Value::Table(table) => {
                table.insert(key, value);
            }
            _ => panic!("not a table value"),
        }
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Value::Table(table) => &table[index],
            _ => panic!("index only supported for array and table values"),
        }
    }
}

impl IndexMut<&str> for Value {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        match self {
            Value::Table(table) => table.get_mut(index).unwrap(),
            _ => panic!("index only supported for array and table values"),
        }
    }
}
