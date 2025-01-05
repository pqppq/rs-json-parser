use std::iter::Peekable;
use std::vec::IntoIter;

fn main() {
    println!("Hello, world!");
}

#[derive(Debug, Clone, PartialEq)]
pub enum JToken {
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Collon,         // :
    Comma,          // ,
    Null,           // null
    Bool(bool),     // true, false
    Number(Number), // number
    String(String), // "string"
}

#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    int: i32,
    frac: Option<f32>,
    exponent: Option<i32>,
}

pub struct Tokenizer {
    input: Peekable<IntoIter<char>>,
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        let cs = input.chars().collect::<Vec<char>>();
        Self {
            input: cs.into_iter().peekable(),
        }
    }

    pub fn consume_string(&mut self) -> JToken {
        let c = self.input.next();
        assert_eq!(c, Some('"'));

        let mut s = "".to_string();
        loop {
            let c = self.input.next();
            match c {
                Some('"') => break,
                Some(c) => s.push(c),
                None => panic!("unclosed string."),
            }
        }
        JToken::String(s)
    }

    fn consume_int(&mut self) -> i32 {
        let mut n = "".to_string();
        loop {
            let c = self.input.peek();
            match c {
                Some(&c) if c == '-' || c == '+' => {
                    if n.is_empty() {
                        self.input.next();
                        n.push(c);
                    } else {
                        panic!("invalid sign position.");
                    }
                }
                Some(&c) if c.is_numeric() => {
                    self.input.next();
                    n.push(c);
                }
                _ => break,
            }
        }

        n.parse::<i32>().unwrap_or(0)
    }

    fn consume_frac(&mut self) -> Option<f32> {
        let c = self.input.peek();
        match c {
            Some(&c) if c == '.' => {
                self.input.next();
            }
            _ => return None,
        }

        let mut n = ".".to_string();
        loop {
            let c = self.input.peek();
            match c {
                Some(&c) if c.is_numeric() => {
                    self.input.next();
                    n.push(c);
                }
                _ => break,
            }
        }

        n.parse::<f32>().ok()
    }

    fn consume_exponent(&mut self) -> Option<i32> {
        let c = self.input.peek();
        let mut n = "".to_string();

        match c {
            Some(&c) if c == 'e' || c == 'E' => {
                self.input.next();
            }
            _ => return None,
        }

        loop {
            let c = self.input.peek();
            match c {
                Some(&c) if c == '-' || c == '+' => {
                    if n.is_empty() {
                        self.input.next();
                        n.push(c);
                    } else {
                        panic!("invalid sign position.");
                    }
                }
                Some(&c) if c.is_numeric() => {
                    self.input.next();
                    n.push(c);
                }
                _ => break,
            }
        }

        n.parse::<i32>().ok()
    }

    pub fn consume_number(&mut self) -> JToken {
        let int = self.consume_int();
        let frac = self.consume_frac();
        let exponent = self.consume_exponent();

        JToken::Number(Number {
            int,
            frac,
            exponent,
        })
    }

    pub fn consume_keyword(&mut self) -> JToken {
        let mut s = "".to_string();
        loop {
            let c = self.input.peek();
            match c {
                Some(&c) if c.is_ascii_lowercase() => {
                    self.input.next();
                    s.push(c);
                }
                _ => break,
            }
        }

        match s.as_str() {
            "null" => JToken::Null,
            "true" => JToken::Bool(true),
            "false" => JToken::Bool(false),
            _ => panic!("invalid keyword {:?}", s),
        }
    }
}

impl Iterator for Tokenizer {
    type Item = JToken;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.input.peek();

            if c.is_none() {
                return None;
            }

            match c.unwrap() {
                ' ' | '\t' | '\n' => {
                    self.input.next();
                    continue;
                }
                '{' => {
                    self.input.next();
                    return Some(JToken::LeftBrace);
                }
                '}' => {
                    self.input.next();
                    return Some(JToken::RightBrace);
                }
                '[' => {
                    self.input.next();
                    return Some(JToken::LeftBracket);
                }
                ']' => {
                    self.input.next();
                    return Some(JToken::RightBracket);
                }
                ':' => {
                    self.input.next();
                    return Some(JToken::Collon);
                }
                ',' => {
                    self.input.next();
                    return Some(JToken::Comma);
                }
                '"' => {
                    let token = self.consume_string();
                    return Some(token);
                }
                '0'..='9' | '-' | '+' | '.' => {
                    let n = self.consume_number();
                    return Some(n);
                }
                'a'..='z' | 'A'..='Z' => {
                    let token = self.consume_keyword();
                    return Some(token);
                }
                c => {
                    panic!("cannot parse input: {:?}", c);
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brace() {
        let json = "{}".to_string();
        let mut t = Tokenizer::new(json);
        let expected = [JToken::LeftBrace, JToken::RightBrace];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_string() {
        let json = "{\"key\": \"value\"}".to_string();
        let mut t = Tokenizer::new(json);
        let expected = [
            JToken::LeftBrace,
            JToken::String("key".to_string()),
            JToken::Collon,
            JToken::String("value".to_string()),
            JToken::RightBrace,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_array() {
        let json = "{\"key\": [\"value1\", \"value2\"]}".to_string();
        let mut t = Tokenizer::new(json);
        let expected = [
            JToken::LeftBrace,
            JToken::String("key".to_string()),
            JToken::Collon,
            JToken::LeftBracket,
            JToken::String("value1".to_string()),
            JToken::Comma,
            JToken::String("value2".to_string()),
            JToken::RightBracket,
            JToken::RightBrace,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_number() {
        let json = "{\"key\": [123, 123.456, -1.0, +1.2, .123, 1E-2, 123.456e+3]}".to_string();
        let mut t = Tokenizer::new(json);
        let expected = [
            JToken::LeftBrace,
            JToken::String("key".to_string()),
            JToken::Collon,
            JToken::LeftBracket,
            JToken::Number(Number {
                int: 123,
                frac: None,
                exponent: None,
            }),
            JToken::Comma,
            JToken::Number(Number {
                int: 123,
                frac: Some(0.456),
                exponent: None,
            }),
            JToken::Comma,
            JToken::Number(Number {
                int: -1,
                frac: Some(0.0),
                exponent: None,
            }),
            JToken::Comma,
            JToken::Number(Number {
                int: 1,
                frac: Some(0.2),
                exponent: None,
            }),
            JToken::Comma,
            JToken::Number(Number {
                int: 0,
                frac: Some(0.123),
                exponent: None,
            }),
            JToken::Comma,
            JToken::Number(Number {
                int: 1,
                frac: None,
                exponent: Some(-2),
            }),
            JToken::Comma,
            JToken::Number(Number {
                int: 123,
                frac: Some(0.456),
                exponent: Some(3),
            }),
            JToken::RightBracket,
            JToken::RightBrace,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_keyword() {
        let json = "{\"key\": [null, true, false]}".to_string();
        let mut t = Tokenizer::new(json);
        let expected = [
            JToken::LeftBrace,
            JToken::String("key".to_string()),
            JToken::Collon,
            JToken::LeftBracket,
            JToken::Null,
            JToken::Comma,
            JToken::Bool(true),
            JToken::Comma,
            JToken::Bool(false),
            JToken::RightBracket,
            JToken::RightBrace,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_misc() {
        let json =
            "{\"foo\": [123.456E-2, \"bar\"], \"foobar\": true, \"fizz\": { \"buzz\": null }}"
                .to_string();
        let mut t = Tokenizer::new(json);
        let expected = [
            JToken::LeftBrace,
            JToken::String("foo".to_string()),
            JToken::Collon,
            JToken::LeftBracket,
            JToken::Number(Number {
                int: 123,
                frac: Some(0.456),
                exponent: Some(-2),
            }),
            JToken::Comma,
            JToken::String("bar".to_string()),
            JToken::RightBracket,
            JToken::Comma,
            JToken::String("foobar".to_string()),
            JToken::Collon,
            JToken::Bool(true),
            JToken::Comma,
            JToken::String("fizz".to_string()),
            JToken::Collon,
            JToken::LeftBrace,
            JToken::String("buzz".to_string()),
            JToken::Collon,
            JToken::Null,
            JToken::RightBrace,
            JToken::RightBrace,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }
}
