pub use crate::toml::Value;
pub use error::Result;
use parser::Parser;

mod error;
mod lexer;
mod parser;
mod toml;

pub fn from_str(text: &str) -> Result<Value> {
    Parser::from_str(text)
}
