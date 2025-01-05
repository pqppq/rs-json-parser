use std::iter::Peekable;

use crate::tokenizer::{JToken, Number, Tokenizer};
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JValue {
    Null,
    Bool(bool),
    String(String),
    Number(Number),
    Array(Vec<JValue>),
    Object(IndexMap<String, JValue>),
}

pub struct Parser {
    t: Peekable<Tokenizer>,
}

enum State {
    Key,
    Value,
}

impl Parser {
    pub fn new(s: String) -> Self {
        let t = Tokenizer::new(s).peekable();
        Self { t }
    }

    pub fn parse(&mut self) -> JValue {
        let token = self.t.peek();
        let value = match token {
            Some(JToken::LeftBrace) => self.parse_object(),
            Some(JToken::LeftBracket) => self.parse_array(),
            _ => panic!(""),
        };

        value
    }

    pub fn parse_object(&mut self) -> JValue {
        let token = self.t.next();
        assert_eq!(token, Some(JToken::LeftBrace));

        let mut m = IndexMap::<String, JValue>::new();
        loop {
            let next = self.t.peek().cloned();
            if next == Some(JToken::RightBrace) {
                self.t.next();
                break;
            }

            let key = self.t.next().unwrap();
            let collon = self.t.next().unwrap();
            assert!(matches!(key, JToken::String(_)));
            assert!(matches!(collon, JToken::Collon));

            let next = self.t.peek().cloned();
            let value = match next.unwrap() {
                JToken::Null => {
                    self.t.next();
                    JValue::Null
                }
                JToken::Bool(b) => {
                    self.t.next();
                    JValue::Bool(b)
                }
                JToken::String(s) => {
                    self.t.next();
                    JValue::String(s.clone())
                }
                JToken::Number(n) => {
                    self.t.next();
                    JValue::Number(n.clone())
                }
                JToken::LeftBrace => self.parse_object(),
                JToken::LeftBracket => self.parse_array(),
                _ => panic!("invalid json."),
            };

            match key {
                JToken::String(s) => {
                    m.insert(s, value);
                }
                _ => panic!(""),
            }

            let next = self.t.peek().cloned();
            if next != Some(JToken::Comma) && next != Some(JToken::RightBrace) {
                panic!("invalid object: {:?}", next);
            }
            if next == Some(JToken::Comma) {
                self.t.next();
            }
        }
        JValue::Object(m)
    }

    pub fn parse_array(&mut self) -> JValue {
        let mut arr = Vec::<JValue>::new();

        let token = self.t.next();
        assert_eq!(token, Some(JToken::LeftBracket));

        loop {
            let next = self.t.peek().cloned();
            if next == Some(JToken::RightBracket) {
                self.t.next();
                break;
            }

            let next = self.t.peek().cloned();
            let value = match next.unwrap() {
                JToken::Null => {
                    self.t.next();
                    JValue::Null
                }
                JToken::Bool(b) => {
                    self.t.next();
                    JValue::Bool(b)
                }
                JToken::String(s) => {
                    self.t.next();
                    JValue::String(s.clone())
                }
                JToken::Number(n) => {
                    self.t.next();
                    JValue::Number(n.clone())
                }
                JToken::LeftBrace => self.parse_object(),
                JToken::LeftBracket => self.parse_array(),
                _ => panic!("invalid json."),
            };

            arr.push(value);

            let next = self.t.peek().cloned();
            if next != Some(JToken::Comma) && next != Some(JToken::RightBracket) {
                panic!("invalid array.")
            }
            if next == Some(JToken::Comma) {
                self.t.next();
            }
        }
        JValue::Array(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_object() {
        let input = "{}".to_string();
        let mut parser = Parser::new(input);
        let m = IndexMap::<String, JValue>::new();
        let expected = JValue::Object(m);

        assert_eq!(parser.parse(), expected);
    }

    #[test]
    fn test_empty_array() {
        let input = "[]".to_string();
        let mut parser = Parser::new(input);
        let arr = Vec::<JValue>::new();
        let expected = JValue::Array(arr);

        assert_eq!(parser.parse(), expected);
    }

    #[test]
    fn test_object() {
        let input = "{\"foo\": \"bar\"}".to_string();
        let mut parser = Parser::new(input);
        let mut m = IndexMap::<String, JValue>::new();
        m.insert("foo".to_string(), JValue::String("bar".to_string()));
        let expected = JValue::Object(m);

        assert_eq!(parser.parse(), expected);
    }

    #[test]
    fn test_object_with_multiple_keys() {
        let input = "{\"foo\": \"bar\", \"active\": true, \"arr\": [1, 2, 3]}".to_string();
        let mut parser = Parser::new(input);
        let mut m = IndexMap::<String, JValue>::new();
        m.insert("foo".to_string(), JValue::String("bar".to_string()));
        m.insert("active".to_string(), JValue::Bool(true));
        m.insert(
            "arr".to_string(),
            JValue::Array(vec![
                JValue::Number(Number::new(1, None, None)),
                JValue::Number(Number::new(2, None, None)),
                JValue::Number(Number::new(3, None, None)),
            ]),
        );
        let expected = JValue::Object(m);

        assert_eq!(parser.parse(), expected);
    }

    #[test]
    fn test_nested_object() {
        let input = "{\"foo\": { \"bar\": true, \"arr\": [1, 2, 3]}}".to_string();
        let mut parser = Parser::new(input);
        let mut m = IndexMap::<String, JValue>::new();
        let mut mm = IndexMap::<String, JValue>::new();
        mm.insert("bar".to_string(), JValue::Bool(true));
        mm.insert(
            "arr".to_string(),
            JValue::Array(vec![
                JValue::Number(Number::new(1, None, None)),
                JValue::Number(Number::new(2, None, None)),
                JValue::Number(Number::new(3, None, None)),
            ]),
        );
        m.insert("foo".to_string(), JValue::Object(mm));
        let expected = JValue::Object(m);

        assert_eq!(parser.parse(), expected);
    }
}
