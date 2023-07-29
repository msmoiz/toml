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

    pub fn as_int(&self) -> i64 {
        match self {
            Value::Integer(int) => *int,
            _ => panic!("not a int value"),
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            Value::Float(float) => *float,
            _ => panic!("not a float value"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(bool) => *bool,
            _ => panic!("not a bool value"),
        }
    }

    pub fn as_offset_date_time(&self) -> DateTime<FixedOffset> {
        match self {
            Value::OffsetDateTime(x) => *x,
            _ => panic!("wrong type"),
        }
    }

    pub fn as_local_date_time(&self) -> NaiveDateTime {
        match self {
            Value::LocalDateTime(x) => *x,
            _ => panic!("wrong type"),
        }
    }

    pub fn as_local_date(&self) -> NaiveDate {
        match self {
            Value::LocalDate(x) => *x,
            _ => panic!("wrong type"),
        }
    }

    pub fn as_local_time(&self) -> NaiveTime {
        match self {
            Value::LocalTime(x) => *x,
            _ => panic!("wrong type"),
        }
    }

    pub fn as_table(&self) -> &HashMap<String, Value> {
        match self {
            Value::Table(table) => table,
            _ => panic!("not a table value"),
        }
    }

    pub fn as_table_mut(&mut self) -> &mut HashMap<String, Value> {
        match self {
            Value::Table(table) => table,
            _ => panic!("not a table value"),
        }
    }

    pub fn as_arr(&self) -> &[Value] {
        match self {
            Value::Array(array) => array,
            _ => panic!("not an array value"),
        }
    }

    pub fn as_arr_mut(&mut self) -> &mut [Value] {
        match self {
            Value::Array(array) => array,
            _ => panic!("not an array value"),
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
            _ => panic!("index by string only supported for table values"),
        }
    }
}

impl IndexMut<&str> for Value {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        match self {
            Value::Table(table) => table.get_mut(index).unwrap(),
            _ => panic!("index by string only supported for table values"),
        }
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Value::Array(array) => &array[index],
            _ => panic!("index by int only supported for array values"),
        }
    }
}

impl IndexMut<usize> for Value {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            Value::Array(array) => &mut array[index],
            _ => panic!("index by int only supported for array values"),
        }
    }
}
