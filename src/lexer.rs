#![allow(dead_code)]

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone};
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
    Float(f64),
    Bool(bool),
    DateTime(DateTime<FixedOffset>),
    DateTimeLocal(NaiveDateTime),
    DateLocal(NaiveDate),
    TimeLocal(NaiveTime),
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
    static ref FLOAT_RE: Regex =
        Regex::new("^(?:\\+|-)?(?:0|[1-9](?:_?[0-9])*)(\\.[0-9](?:_?[0-9])*)?([eE](?:\\+|-)?(?:0|[1-9](?:_?[0-9])*))?").expect("float re should be valid");
    static ref FLOAT_INF_RE: Regex =
        Regex::new("^(?:\\+|-)?inf").expect("float inf re should be valid");
    static ref TRUE_RE: Regex = Regex::new("^true(?:$|\\s)").expect("true re should be valid");
    static ref FALSE_RE: Regex = Regex::new("^false(?:$|\\s)").expect("false re should be valid");
    static ref DATE_TIME_RE: Regex = Regex::new("^((?:(\\d{4}-\\d{2}-\\d{2})[T ](\\d{2}:\\d{2}:\\d{2}(?:\\.\\d+)?))(Z|[\\+-]\\d{2}:\\d{2}))").expect("date time re should be valid");
    static ref DATE_TIME_LOCAL_RE: Regex = Regex::new("^((?:(\\d{4}-\\d{2}-\\d{2})[T ](\\d{2}:\\d{2}:\\d{2}(?:\\.\\d+)?)))").expect("date time local re should be valid");
    static ref DATE_LOCAL_RE: Regex = Regex::new("^((?:(\\d{4}-\\d{2}-\\d{2})))").expect("date local re should be valid");
    static ref TIME_LOCAL_RE: Regex = Regex::new("^((?:(\\d{2}:\\d{2}:\\d{2}(?:\\.\\d+)?)))").expect("time local re should be valid");
}

#[derive(Clone)]
pub enum Posture {
    Key,
    Value,
}

#[derive(Default, Clone)]
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

        if let Some(len) = self.scan_date_time() {
            let dt = &self.text[self.pos..self.pos + len];
            let dt = DateTime::parse_from_rfc3339(dt).map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::DateTime(dt)));
        }

        if let Some(len) = self.scan_date_time_local() {
            let dt = &self.text[self.pos..self.pos + len];
            let dt =
                NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M:%S").map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::DateTimeLocal(dt)));
        }

        if let Some(len) = self.scan_date_local() {
            let dt = &self.text[self.pos..self.pos + len];
            let dt = NaiveDate::parse_from_str(dt, "%Y-%m-%d").map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::DateLocal(dt)));
        }

        if let Some(len) = self.scan_time_local() {
            let dt = &self.text[self.pos..self.pos + len];
            let dt = NaiveTime::parse_from_str(dt, "%H:%M:%S").map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::TimeLocal(dt)));
        }

        if let Some(len) = self.scan_float() {
            let float = &self.text[self.pos..self.pos + len].replace("_", "");
            let float = float.parse::<f64>().map_err(|_| Error::Parse)?;
            self.pos += len;
            return Ok(Some(Token::Float(float)));
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

        if let Some(len) = self.scan_true() {
            self.pos += len;
            return Ok(Some(Token::Bool(true)));
        }

        if let Some(len) = self.scan_false() {
            self.pos += len;
            return Ok(Some(Token::Bool(false)));
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

    fn scan_float(&self) -> Option<usize> {
        let text = &self.text[self.pos..];

        if let Some(cap) = FLOAT_RE.captures(text) {
            if let Some(_) = cap.get(1) {
                return Some(cap.get(0).unwrap().len());
            } else if let Some(_) = cap.get(2) {
                return Some(cap.get(0).unwrap().len());
            }
        }

        if let Some(cap) = FLOAT_INF_RE.captures(text) {
            return Some(cap.get(0).unwrap().len());
        }

        None
    }

    fn scan_true(&self) -> Option<usize> {
        match TRUE_RE.captures(&self.text[self.pos..]) {
            Some(_) => Some(4),
            None => None,
        }
    }

    fn scan_false(&self) -> Option<usize> {
        match FALSE_RE.captures(&self.text[self.pos..]) {
            Some(_) => Some(5),
            None => None,
        }
    }

    fn scan_date_time(&self) -> Option<usize> {
        match DATE_TIME_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_date_time_local(&self) -> Option<usize> {
        match DATE_TIME_LOCAL_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_date_local(&self) -> Option<usize> {
        match DATE_LOCAL_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }

    fn scan_time_local(&self) -> Option<usize> {
        match TIME_LOCAL_RE.captures(&self.text[self.pos..]) {
            Some(cap) => Some(cap.get(0).unwrap().len()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone};

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

    #[test]
    fn float() -> Result<()> {
        let text = "+1.0";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Float(1.0)));
        Ok(())
    }

    #[test]
    fn float_exp() -> Result<()> {
        let text = "6.26e-34";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Float(6.26e-34)));
        Ok(())
    }

    #[test]
    fn float_underscore() -> Result<()> {
        let text = "224_617.445_991_228";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Float(224617.445991228)));
        Ok(())
    }

    #[test]
    fn float_inf() -> Result<()> {
        let text = "inf +inf -inf";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(
            lexer.next(context.clone())?,
            Some(Token::Float(f64::INFINITY))
        );
        assert_eq!(
            lexer.next(context.clone())?,
            Some(Token::Float(f64::INFINITY))
        );
        assert_eq!(lexer.next(context)?, Some(Token::Float(f64::NEG_INFINITY)));
        Ok(())
    }

    #[test]
    fn bool_true() -> Result<()> {
        let text = "true";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Bool(true)));
        Ok(())
    }

    #[test]
    fn bool_false() -> Result<()> {
        let text = "false";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        assert_eq!(lexer.next(context)?, Some(Token::Bool(false)));
        Ok(())
    }

    #[test]
    fn date_time_offset() -> Result<()> {
        let text = "1979-05-27T07:32:00Z";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let dt = FixedOffset::west_opt(0)
            .unwrap()
            .with_ymd_and_hms(1979, 05, 27, 07, 32, 00)
            .unwrap();
        assert_eq!(lexer.next(context)?, Some(Token::DateTime(dt)));
        Ok(())
    }

    #[test]
    fn date_time_local() -> Result<()> {
        let text = "1979-05-27T07:32:00";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let dt = NaiveDate::from_ymd_opt(1979, 05, 27)
            .unwrap()
            .and_hms_opt(07, 32, 00)
            .unwrap();
        assert_eq!(lexer.next(context)?, Some(Token::DateTimeLocal(dt)));
        Ok(())
    }

    #[test]
    fn date_local() -> Result<()> {
        let text = "1979-05-27";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let dt = NaiveDate::from_ymd_opt(1979, 05, 27).unwrap();
        assert_eq!(lexer.next(context)?, Some(Token::DateLocal(dt)));
        Ok(())
    }

    #[test]
    fn time_local() -> Result<()> {
        let text = "07:32:00";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let dt = NaiveTime::from_hms_opt(07, 32, 00).unwrap();
        assert_eq!(lexer.next(context)?, Some(Token::TimeLocal(dt)));
        Ok(())
    }
}
