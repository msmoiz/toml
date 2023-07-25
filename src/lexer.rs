#![allow(dead_code)]

use lazy_static::lazy_static;
use regex::Regex;

use crate::error::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum Token {
    Comment,
    BareKey(String),
    Equal,
    Dot,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    String(String),
    Integer(i64),
}

lazy_static! {
    static ref COMMENT_RE: Regex = Regex::new("^#.*(?:\n|$)").expect("comment re should be valid");
    static ref BARE_KEY_RE: Regex =
        Regex::new("^[a-zA-Z0-9_-]+").expect("bare key re should be valid");
    static ref LITERAL_STR_RE: Regex =
        Regex::new("^'([^'\n]*)'").expect("literal str re should be valid");
    static ref MULTILINE_LITERAL_STR_RE: Regex =
        Regex::new("^'''(?s)(.*)'''").expect("multiline literal str re should be valid");
    static ref BASIC_STR_RE: Regex =
        Regex::new(r#"^"([^"\n]*)""#).expect("basic re should be valid");
    static ref MULTILINE_BASIC_STR_RE: Regex =
        Regex::new(r#"^"""(?s)(.*)""""#).expect("multiline basic re should be valid");
    static ref INTEGER_RE: Regex =
        Regex::new("^(?:\\+|-)?(?:0|[1-9](_?[0-9])*)").expect("integer re should be valid");
    static ref INTEGER_HEX_RE: Regex =
        Regex::new("^0x[a-fA-F0-9](?:_?[a-fA-F0-9])*").expect("integer hex re should be valid");
    static ref INTEGER_OCTAL_RE: Regex =
        Regex::new("^0o[0-7](?:_?[0-7])*").expect("integer octal re should be valid");
    static ref INTEGER_BINARY_RE: Regex =
        Regex::new("^0b[0-1](?:_?[0-1])*").expect("integer binary re should be valid");
}

pub enum Posture {
    Key,
    Value,
}

#[derive(Default)]
pub struct Context {
    posture: Option<Posture>,
}

pub struct Lexer<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }

    pub fn next(&mut self, context: Context) -> Result<Option<Token>> {
        let len = self.scan_whitespace();
        self.pos += len;

        if self.pos == self.text.len() {
            return Ok(None);
        }

        if let Some(len) = self.scan_comment() {
            self.pos += len;
            return Ok(Some(Token::Comment));
        }

        if let Some(punct) = self.scan_punct() {
            self.pos += 1;
            return Ok(Some(punct));
        }

        if !matches!(context.posture, Some(Posture::Value)) {
            if let Some(len) = self.scan_bare_key() {
                let key = &self.text[self.pos..self.pos + len];
                self.pos += len;
                return Ok(Some(Token::BareKey(key.into())));
            }
        }

        if let Some(len) = self.scan_multiline_basic_string() {
            let str = &self.text[self.pos + 3..self.pos + len - 3];
            self.pos += len;
            return Ok(Some(Token::String(str.into())));
        }

        if let Some(len) = self.scan_basic_string() {
            let str = &self.text[self.pos + 1..self.pos + len - 1];
            self.pos += len;
            return Ok(Some(Token::String(str.into())));
        }

        if let Some(len) = self.scan_multiline_literal_string() {
            let str = &self.text[self.pos + 3..self.pos + len - 3];
            self.pos += len;
            return Ok(Some(Token::String(str.into())));
        }

        if let Some(len) = self.scan_literal_string() {
            let str = &self.text[self.pos + 1..self.pos + len - 1];
            self.pos += len;
            return Ok(Some(Token::String(str.into())));
        }

        if let Some(len) = self.scan_integer_hex() {
            let int = &self.text[self.pos + 2..self.pos + len].replace("_", "");
            let int = i64::from_str_radix(int, 16).map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::Integer(int)));
        }

        if let Some(len) = self.scan_integer_octal() {
            let int = &self.text[self.pos + 2..self.pos + len].replace("_", "");
            let int = i64::from_str_radix(int, 8).map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::Integer(int)));
        }

        if let Some(len) = self.scan_integer_binary() {
            let int = &self.text[self.pos + 2..self.pos + len].replace("_", "");
            let int = i64::from_str_radix(int, 2).map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::Integer(int)));
        }

        if let Some(len) = self.scan_integer() {
            let int = &self.text[self.pos..self.pos + len].replace("_", "");
            let int = int.parse::<i64>().map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::Integer(int)));
        }

        Err(Error::Parse)
    }

    fn scan_whitespace(&self) -> usize {
        let mut len = 0;
        let mut ix = 0;
        while let Some(' ') | Some('\t') = &self.text[self.pos + ix..].chars().next() {
            len += 1;
            ix += 1;
        }
        len
    }

    fn scan_comment(&self) -> Option<usize> {
        match COMMENT_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_punct(&self) -> Option<Token> {
        match &self.text[self.pos..].chars().next() {
            Some('=') => Some(Token::Equal),
            Some('.') => Some(Token::Dot),
            Some('{') => Some(Token::LeftBrace),
            Some('}') => Some(Token::RightBrace),
            Some('[') => Some(Token::LeftBracket),
            Some(']') => Some(Token::RightBracket),
            _ => None,
        }
    }

    fn scan_bare_key(&self) -> Option<usize> {
        match BARE_KEY_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_basic_string(&self) -> Option<usize> {
        match BASIC_STR_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_multiline_basic_string(&self) -> Option<usize> {
        match MULTILINE_BASIC_STR_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_literal_string(&self) -> Option<usize> {
        match LITERAL_STR_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_multiline_literal_string(&self) -> Option<usize> {
        match MULTILINE_LITERAL_STR_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_integer(&self) -> Option<usize> {
        match INTEGER_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_integer_hex(&self) -> Option<usize> {
        match INTEGER_HEX_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_integer_octal(&self) -> Option<usize> {
        match INTEGER_OCTAL_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_integer_binary(&self) -> Option<usize> {
        match INTEGER_BINARY_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::{Error, Result},
        lexer::{Context, Lexer, Posture, Token},
    };

    #[test]
    fn empty() -> Result<()> {
        let text = "";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, None);
        Ok(())
    }

    #[test]
    fn whitespace() -> Result<()> {
        let text = " \t";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, None);
        Ok(())
    }

    #[test]
    fn full_line_comment() -> Result<()> {
        let text = "# This is a comment";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::Comment));
        Ok(())
    }

    #[test]
    fn inline_comment() -> Result<()> {
        let text = "  # This is a comment at the end of a line";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::Comment));
        Ok(())
    }

    #[test]
    fn bare_key() -> Result<()> {
        let text = "key";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::BareKey("key".into())));
        Ok(())
    }

    #[test]
    fn equal() -> Result<()> {
        let text = "=";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::Equal));
        Ok(())
    }

    #[test]
    fn dot() -> Result<()> {
        let text = ".";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::Dot));
        Ok(())
    }

    #[test]
    fn left_brace() -> Result<()> {
        let text = "{";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::LeftBrace));
        Ok(())
    }

    #[test]
    fn right_brace() -> Result<()> {
        let text = "}";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::RightBrace));
        Ok(())
    }

    #[test]
    fn left_bracket() -> Result<()> {
        let text = "[";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::LeftBracket));
        Ok(())
    }

    #[test]
    fn right_bracket() -> Result<()> {
        let text = "]";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::RightBracket));
        Ok(())
    }

    #[test]
    fn literal_string() -> Result<()> {
        let text = "'foo'";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::String("foo".into())));
        Ok(())
    }

    #[test]
    fn literal_string_newline() -> Result<()> {
        let text = "'foo\n'";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context), Err(Error::Parse));
        Ok(())
    }

    #[test]
    fn multiline_literal_string() -> Result<()> {
        let text = "'''foo\nbar'''";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::String("foo\nbar".into())));
        Ok(())
    }

    #[test]
    fn basic_string() -> Result<()> {
        let text = r#""foo""#;
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::String("foo".into())));
        Ok(())
    }

    #[test]
    fn multiline_basic_string() -> Result<()> {
        let text = "\"\"\"foo\nbar\"\"\"";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::String("foo\nbar".into())));
        Ok(())
    }

    #[test]
    fn integer() -> Result<()> {
        let text = "123";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Integer(123)));
        Ok(())
    }

    #[test]
    fn integer_positive() -> Result<()> {
        let text = "+123";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::Integer(123)));
        Ok(())
    }

    #[test]
    fn integer_negative() -> Result<()> {
        let text = "-123";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Integer(-123)));
        Ok(())
    }

    #[test]
    fn integer_zero() -> Result<()> {
        let text = "0";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Integer(0)));
        Ok(())
    }

    #[test]
    fn integer_underscore() -> Result<()> {
        let text = "1_000";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Integer(1000)));
        Ok(())
    }

    #[test]
    fn integer_hex() -> Result<()> {
        let text = "0xDEADBEEF";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Integer(3735928559)));
        Ok(())
    }

    #[test]
    fn integer_octal() -> Result<()> {
        let text = "0o01234567";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Integer(342391)));
        Ok(())
    }

    #[test]
    fn integer_binary() -> Result<()> {
        let text = "0b11010110";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Integer(214)));
        Ok(())
    }
}
