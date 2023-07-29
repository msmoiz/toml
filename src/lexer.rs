#![allow(dead_code)]

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use lazy_static::lazy_static;
use regex::Regex;

use crate::error::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum Token {
    Newline,
    Equal,
    Dot,
    Comma,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    OffsetDateTime(DateTime<FixedOffset>),
    LocalDateTime(NaiveDateTime),
    LocalDate(NaiveDate),
    LocalTime(NaiveTime),
}

#[derive(Clone)]
pub enum Posture {
    Any,
    Value,
}

#[derive(Default, Clone)]
pub struct Context {
    pub posture: Option<Posture>,
}

#[derive(Clone)]
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

        if let Some(len) = self.scan_comment() {
            self.pos += len;
        }

        if self.pos == self.text.len() {
            return Ok(None);
        }

        if let Some((token, len)) = self.scan_newline() {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some(token) = self.scan_punct() {
            self.pos += 1;
            return Ok(Some(token));
        }

        if !matches!(context.posture, Some(Posture::Value)) {
            if let Some((token, len)) = self.scan_bare_key() {
                self.pos += len;
                return Ok(Some(token));
            }
        }

        if let Some((token, len)) = self.scan_multiline_basic_string() {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_basic_string() {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_multiline_literal_string() {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_literal_string() {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_offset_date_time()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_local_date_time()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_local_date()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_local_time()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_float()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_integer_hex()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_integer_octal()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_integer_binary()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_integer()? {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_true() {
            self.pos += len;
            return Ok(Some(token));
        }

        if let Some((token, len)) = self.scan_false() {
            self.pos += len;
            return Ok(Some(token));
        }

        Err(Error::Parse)
    }

    pub fn peek(&mut self, context: Context) -> Result<Option<Token>> {
        let start = self.pos;
        let token = self.next(context);
        self.pos = start;
        token
    }

    fn remainder(&self) -> &str {
        &self.text[self.pos..]
    }

    fn scan_whitespace(&self) -> usize {
        let mut len = 0;
        let mut chars = self.remainder().chars();
        while let Some(' ') | Some('\t') = chars.next() {
            len += 1;
        }
        len
    }

    fn scan_comment(&self) -> Option<usize> {
        lazy_static! {
            static ref COMMENT_RE: Regex = Regex::new(
                "(?x)
                ^               # start
                \\#             # delimiter
                .*              # body
                (\r\n|\n|$)     # newline or eof
                "
            )
            .expect("comment re should be valid");
        }
        let captures = COMMENT_RE.captures(&self.remainder())?;
        let comment = captures.get(0)?.as_str();
        let ending_len = captures.get(1)?.len();
        Some(comment.len() - ending_len)
    }

    fn scan_newline(&self) -> Option<(Token, usize)> {
        let mut chars = self.remainder().chars();
        match (chars.next(), chars.next()) {
            (Some('\r'), Some('\n')) => Some((Token::Newline, 2)),
            (Some('\n'), _) => Some((Token::Newline, 1)),
            _ => None,
        }
    }

    fn scan_punct(&self) -> Option<Token> {
        match &self.remainder().chars().next() {
            Some('=') => Some(Token::Equal),
            Some('.') => Some(Token::Dot),
            Some('{') => Some(Token::LeftBrace),
            Some('}') => Some(Token::RightBrace),
            Some('[') => Some(Token::LeftBracket),
            Some(']') => Some(Token::RightBracket),
            Some(',') => Some(Token::Comma),
            _ => None,
        }
    }

    fn scan_bare_key(&self) -> Option<(Token, usize)> {
        lazy_static! {
            static ref BARE_KEY_RE: Regex =
                Regex::new("^[[:alnum:]-_]+").expect("bare key re should be valid");
        }
        let captures = BARE_KEY_RE.captures(&self.remainder())?;
        let key = captures.get(0)?.as_str();
        Some((Token::String(key.into()), key.len()))
    }

    fn scan_basic_string(&self) -> Option<(Token, usize)> {
        lazy_static! {
            static ref BASIC_STR_RE: Regex = Regex::new(
                r#"(?x)
                ^       # start
                "       # open quote
                (       # content
                    (?:
                        [^"\\\n]                            # general
                        |\\(?:[btnfr"\\]|u[0-9a-fA-F]{4})   # escapes
                    )*
                )
                "       # close quote
                "#
            )
            .expect("basic re should be valid");
        }
        let captures = BASIC_STR_RE.captures(&self.remainder())?;
        let text = captures.get(0)?.as_str();
        let str = captures.get(1)?.as_str();
        let str = str
            .replace("\\b", "\u{0008}")
            .replace("\\t", "\t")
            .replace("\\n", "\n")
            .replace("\\f", "\u{000C}")
            .replace("\\r", "\r")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
        Some((Token::String(str), text.len()))
    }

    fn scan_multiline_basic_string(&self) -> Option<(Token, usize)> {
        lazy_static! {
            static ref MULTILINE_BASIC_STR_RE: Regex = Regex::new(
                r#"(?xs)
                ^           # start
                "{3}        # open delim
                .*          # content
                "{3,}       # close delim
                "#
            )
            .expect("multiline basic re should be valid");
            static ref LINE_ENDING_SLASH: Regex = Regex::new(
                r#"(?x)
                \\              # delim
                [ \t]*          # trailing whitespace
                (?:\r\n|\n)     # newline
                [[:space:]]*    # following whitespace
                "#
            )
            .expect("line ending slash re should be valid");
        }
        let captures = MULTILINE_BASIC_STR_RE.captures(&self.remainder())?;
        let text = captures.get(0)?.as_str();
        let content = &text[3..text.len() - 3];
        let content = content.strip_prefix("\n").unwrap_or(content);
        let content = LINE_ENDING_SLASH.replace_all(content, "");

        if Lexer::contains_three_consec_delims(&content) {
            return None;
        }

        let content = content
            .replace("\\b", "\u{0008}")
            .replace("\\t", "\t")
            .replace("\\n", "\n")
            .replace("\\f", "\u{000C}")
            .replace("\\r", "\r")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
        Some((Token::String(content.into()), text.len()))
    }

    fn scan_literal_string(&self) -> Option<(Token, usize)> {
        lazy_static! {
            static ref LITERAL_STR_RE: Regex = Regex::new(
                r#"(?x)
                ^          # start
                '          # open quote
                ([^'\n]*)  # content
                '          # close quote
                "#
            )
            .expect("literal str re should be valid");
        }
        let captures = LITERAL_STR_RE.captures(&self.remainder())?;
        let text = captures.get(0).unwrap().as_str();
        let str = captures.get(1).unwrap().as_str();
        Some((Token::String(str.into()), text.len()))
    }

    fn scan_multiline_literal_string(&self) -> Option<(Token, usize)> {
        lazy_static! {
            static ref MULTILINE_LITERAL_STR_RE: Regex = Regex::new(
                r"(?xs)
                ^       # start
                '{3}    # open delim
                \n?     # initial newline
                .*?     # content
                '{3,}   # close delim
                "
            )
            .expect("multiline literal str re should be valid");
        }
        let captures = MULTILINE_LITERAL_STR_RE.captures(&self.remainder())?;
        let text = captures.get(0)?.as_str();
        let content = &text[3..text.len() - 3];
        let content = content.strip_prefix("\n").unwrap_or(content);
        Some((Token::String(content.into()), text.len()))
    }

    fn scan_integer(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref INTEGER_RE: Regex = Regex::new(
                "(?x)
                ^                       # start
                (?:\\+|-)?              # sign
                (?:                     # number
                    0                   # zero
                    |[1-9](_?[0-9])*    # digits and underscores
                )
                "
            )
            .expect("integer re should be valid");
        }
        let Some(captures) = INTEGER_RE.captures(&self.remainder()) else {
            return Ok(None);
        };
        let raw = captures.get(0).unwrap().as_str();
        let replaced = raw.replace("_", "");
        Ok(Some((
            Token::Integer(replaced.parse().map_err(|_| Error::Parse)?),
            raw.len(),
        )))
    }

    fn scan_integer_hex(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref INTEGER_HEX_RE: Regex = Regex::new(
                "(?x)
                ^                       # start
                0x                      # prefix
                [[:xdigit:]]            # first digit
                (?:_?[[:xdigit:]])*     # digits and underscores
                "
            )
            .expect("integer hex re should be valid");
        }
        let Some(captures) = INTEGER_HEX_RE.captures(&self.remainder()) else {
            return Ok(None);
        };
        let raw = captures.get(0).unwrap().as_str();
        let replaced = raw.replace("_", "");
        Ok(Some((
            Token::Integer(i64::from_str_radix(&replaced[2..], 16).map_err(|_| Error::Parse)?),
            raw.len(),
        )))
    }

    fn scan_integer_octal(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref INTEGER_OCTAL_RE: Regex = Regex::new(
                "(?x)
                ^               # start
                0o              # prefix
                [0-7]           # first digit
                (?:_?[0-7])*    # digits and underscores
                "
            )
            .expect("integer octal re should be valid");
        }
        let Some(captures) = INTEGER_OCTAL_RE.captures(&self.remainder()) else {
            return Ok(None);
        };
        let raw = captures.get(0).unwrap().as_str();
        let replaced = raw.replace("_", "");
        Ok(Some((
            Token::Integer(i64::from_str_radix(&replaced[2..], 8).map_err(|_| Error::Parse)?),
            raw.len(),
        )))
    }

    fn scan_integer_binary(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref INTEGER_BINARY_RE: Regex = Regex::new(
                "(?x)
                ^               # start
                0b              # prefix
                [0-1]           # first digit
                (?:_?[0-1])*    # digits and underscores
                "
            )
            .expect("integer binary re should be valid");
        }
        let Some(captures) = INTEGER_BINARY_RE.captures(&self.remainder()) else {
            return Ok(None);
        };
        let text = captures.get(0).unwrap().as_str();
        let replaced = text.replace("_", "");
        Ok(Some((
            Token::Integer(i64::from_str_radix(&replaced[2..], 2).map_err(|_| Error::Parse)?),
            text.len(),
        )))
    }

    fn scan_float(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref FLOAT_RE: Regex = Regex::new(
                "(?x)
                    ^                                       # start
                    (?:\\+|-)?                              # sign
                    (?:0|[1-9](?:_?[0-9])*)                 # whole number
                    (\\.[0-9](?:_?[0-9])*)?                 # fraction
                    ([eE](?:\\+|-)?(?:[0-9](?:_?[0-9])*))?  # exponent
                    "
            )
            .expect("float re should be valid");
            static ref FLOAT_INF_RE: Regex =
                Regex::new("^(?:\\+|-)?inf").expect("float inf re should be valid");
            static ref FLOAT_NAN_RE: Regex =
                Regex::new("^(?:\\+|-)?nan").expect("float nan re should be valid");
        }
        let Some(text) = ({
            if let Some(captures) = FLOAT_RE.captures(self.remainder()) {
                if captures.get(1).is_some() || captures.get(2).is_some() {
                    Some(captures.get(0).unwrap().as_str())
                } else {
                    None
                }
            } else if let Some(captures) = FLOAT_INF_RE.captures(self.remainder()) {
                Some(captures.get(0).unwrap().as_str())
            } else if let Some(captures) = FLOAT_NAN_RE.captures(self.remainder()) {
                Some(captures.get(0).unwrap().as_str())
            } else {
                None
            }
        }) else {
            return Ok(None);
        };

        let float = text.replace("_", "");
        let float = float.parse::<f64>().map_err(|_| Error::Parse)?;

        Ok(Some((Token::Float(float), text.len())))
    }

    fn scan_true(&self) -> Option<(Token, usize)> {
        lazy_static! {
            static ref TRUE_RE: Regex =
                Regex::new("^true(?:$|\\s)").expect("true re should be valid");
        }
        if TRUE_RE.is_match(&self.remainder()) {
            Some((Token::Bool(true), 4))
        } else {
            None
        }
    }

    fn scan_false(&self) -> Option<(Token, usize)> {
        lazy_static! {
            static ref FALSE_RE: Regex =
                Regex::new("^false(?:$|\\s)").expect("false re should be valid");
        }
        if FALSE_RE.is_match(&self.remainder()) {
            Some((Token::Bool(false), 5))
        } else {
            None
        }
    }

    fn scan_offset_date_time(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref OFFSET_DATE_TIME_RE: Regex = Regex::new(
                r"(?x)
                ^                               # start
                \d{4}-\d{2}-\d{2}               # date
                [T\ ]                           # separator
                \d{2}:\d{2}:\d{2}(?:\.\d+)?     # time
                (?:Z|[\+-]\d{2}:\d{2})          # offset
                "
            )
            .expect("date time re should be valid");
        }
        let Some(captures) = OFFSET_DATE_TIME_RE.captures(&self.remainder()) else {
            return Ok(None);
        };
        let text = captures.get(0).unwrap().as_str();
        let text = text.replace(" ", "T");
        let dt = DateTime::parse_from_rfc3339(&text).map_err(|_| Error::Parse)?;
        Ok(Some((Token::OffsetDateTime(dt), text.len())))
    }

    fn scan_local_date_time(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref LOCAL_DATE_TIME_RE: Regex = Regex::new(
                r"(?x)
                ^                               # start
                \d{4}-\d{2}-\d{2}               # date
                [T\ ]                           # separator
                \d{2}:\d{2}:\d{2}(?:\.\d+)?     # time
                "
            )
            .expect("date time local re should be valid");
        }
        let Some(captures) = LOCAL_DATE_TIME_RE.captures(&self.remainder()) else {
            return Ok(None);
        };
        let text = captures.get(0).unwrap().as_str();
        let dt = NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f")
            .map_err(|_| Error::Parse)?;
        Ok(Some((Token::LocalDateTime(dt), text.len())))
    }

    fn scan_local_date(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref LOCAL_DATE_RE: Regex =
                Regex::new(r"^\d{4}-\d{2}-\d{2}").expect("date local re should be valid");
        }
        let Some(captures) = LOCAL_DATE_RE.captures(&self.remainder()) else {
            return Ok(None);
        };
        let text = captures.get(0).unwrap().as_str();
        let date = NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| Error::Parse)?;
        Ok(Some((Token::LocalDate(date), text.len())))
    }

    fn scan_local_time(&self) -> Result<Option<(Token, usize)>> {
        lazy_static! {
            static ref LOCAL_TIME_RE: Regex =
                Regex::new(r"^\d{2}:\d{2}:\d{2}(?:\.\d+)?").expect("time local re should be valid");
        }
        let Some(captures) = LOCAL_TIME_RE.captures(&self.remainder()) else {
            return Ok(None);
        };
        let text = captures.get(0).unwrap().as_str();
        let time = NaiveTime::parse_from_str(text, "%H:%M:%S%.f").map_err(|_| Error::Parse)?;
        Ok(Some((Token::LocalTime(time), text.len())))
    }

    fn contains_three_consec_delims(text: &str) -> bool {
        let mut count = 0;
        let mut ix = 0;
        while ix < text.len() {
            if text[ix..].starts_with("\"") {
                count += 1;
                ix += 1;
            } else if text[ix..].starts_with("\\\"") {
                count = 0;
                ix += 2;
            } else {
                count = 0;
                ix += 1;
            }
            if count == 3 {
                return true;
            }
        }

        if count == 3 {
            return true;
        }

        false
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
    fn newline() -> Result<()> {
        let text = "\n";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::Newline));
        Ok(())
    }

    #[test]
    fn bare_key() -> Result<()> {
        let text = "key";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::String("key".into())));
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
    fn comma() -> Result<()> {
        let text = ",";
        let mut lexer = Lexer::new(text);
        let context = Context::default();
        assert_eq!(lexer.next(context)?, Some(Token::Comma));
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
        assert_eq!(lexer.next(context)?, Some(Token::OffsetDateTime(dt)));
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
        assert_eq!(lexer.next(context)?, Some(Token::LocalDateTime(dt)));
        Ok(())
    }

    #[test]
    fn date_local() -> Result<()> {
        let text = "1979-05-27";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let dt = NaiveDate::from_ymd_opt(1979, 05, 27).unwrap();
        assert_eq!(lexer.next(context)?, Some(Token::LocalDate(dt)));
        Ok(())
    }

    #[test]
    fn time_local() -> Result<()> {
        let text = "07:32:00";
        let mut lexer = Lexer::new(text);
        let mut context = Context::default();
        context.posture = Some(Posture::Value);
        let dt = NaiveTime::from_hms_opt(07, 32, 00).unwrap();
        assert_eq!(lexer.next(context)?, Some(Token::LocalTime(dt)));
        Ok(())
    }
}
