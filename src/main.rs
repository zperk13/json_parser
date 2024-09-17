// https://www.json.org/json-en.html

use std::char;

fn is_json_whitespace(c: char) -> bool {
    [' ', '\n', '\r', '\t'].contains(&c)
}

trait CharIterator {
    fn next(&mut self) -> Option<char>;
    fn peek(&mut self) -> Option<char>;
    fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char>;
    fn next_if_eq(&mut self, expected: &char) -> Option<char>;
}

impl CharIterator for std::str::Chars<'_> {
    fn next(&mut self) -> Option<char> {
        std::iter::Iterator::next(self)
    }

    fn peek(&mut self) -> Option<char> {
        std::iter::Peekable::peek(&mut self.peekable()).copied()
    }

    fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        std::iter::Peekable::next_if(&mut self.peekable(), func)
    }

    fn next_if_eq(&mut self, expected: &char) -> Option<char> {
        std::iter::Peekable::next_if_eq(&mut self.peekable(), expected)
    }
}

struct WhitespaceSkippingIndexTrackingIter<CI: CharIterator> {
    previously_outputted_index: Option<usize>,
    inner: CI,
}

impl<CI: CharIterator> WhitespaceSkippingIndexTrackingIter<CI> {
    fn new(ci: CI) -> Self {
        Self {
            previously_outputted_index: None,
            inner: ci,
        }
    }

    fn inc_index(&mut self) {
        match &mut (self.previously_outputted_index) {
            None => self.previously_outputted_index = Some(0),
            Some(i) => *i += 1,
        }
    }

    fn next_any(&mut self) -> Option<char> {
        let out = self.inner.next();
        if out.is_some() {
            self.inc_index();
        }
        out
    }

    fn next_non_whitespace(&mut self) -> Option<char> {
        loop {
            let next = self.next_any()?;
            if !is_json_whitespace(next) {
                return Some(next);
            }
        }
    }

    /// If the next non-whitespace value is not the expected value,
    /// whitespace will still be consumed
    fn next_non_whitespace_if_eq(&mut self, expected: char) -> Option<char> {
        loop {
            let next = self.inner.next_if(|c| is_json_whitespace(*c) || *c == expected)?;
            if is_json_whitespace(next) {
                continue;
            }
            return Some(next);
        }
    }
}

enum ParseError {
    UnexpectedCharacter {
        character: char,
        index: usize,
        expected_characters: Vec<char>,
    },
    UnexpectedEndOfString,
    ControlCharacter {
        control_character: char,
        index: usize,
    },
}

trait JsonType<CI: CharIterator>: Sized {
    fn parse(i: &mut WhitespaceSkippingIndexTrackingIter<CI>) -> Result<Self, ParseError>;
}

enum JsonValue {
    Object(JsonObject),
    Array(JsonArray),
    String(JsonString),
    Number(JsonNumber),
    Bool(JsonBool),
    Null(JsonNull),
}

impl<CI: CharIterator> JsonType<CI> for JsonValue {
    fn parse(i: &mut WhitespaceSkippingIndexTrackingIter<CI>) -> Result<Self, ParseError> {
        todo!();
    }
}

struct JsonArray(Vec<JsonValue>);
impl<CI: CharIterator> JsonType<CI> for JsonArray {
    fn parse(i: &mut WhitespaceSkippingIndexTrackingIter<CI>) -> Result<Self, ParseError> {
        {
            let first_char = i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
            if first_char != '[' {
                return Err(ParseError::UnexpectedCharacter {
                    character: first_char,
                    index: i.previously_outputted_index.unwrap(),
                    expected_characters: vec!['['],
                });
            }
        }
        let mut v = Vec::new();
        let is_empty = i.next_non_whitespace_if_eq(']').is_some();
        if is_empty {
            return Ok(JsonArray(v));
        }
        loop {
            let value = JsonValue::parse(i)?;
            v.push(value);
            let next_char = i
                .next_non_whitespace()
                .ok_or(ParseError::UnexpectedEndOfString)?;
            if next_char == ']' {
                return Ok(JsonArray(v));
            } else if next_char == ',' {
                continue;
            } else {
                return Err(ParseError::UnexpectedCharacter {
                    character: next_char,
                    index: i.previously_outputted_index.unwrap(),
                    expected_characters: vec![']', ','],
                });
            }
        }
    }
}

struct JsonBool(bool);
impl<CI: CharIterator> JsonType<CI> for JsonBool {
    fn parse(i: &mut WhitespaceSkippingIndexTrackingIter<CI>) -> Result<Self, ParseError> {
        todo!();
    }
}

struct JsonNull;
impl<CI: CharIterator> JsonType<CI> for JsonNull {
    fn parse(i: &mut WhitespaceSkippingIndexTrackingIter<CI>) -> Result<Self, ParseError> {
        todo!();
    }
}

struct JsonNumber(f64);
impl<CI: CharIterator> JsonType<CI> for JsonNumber {
    fn parse(i: &mut WhitespaceSkippingIndexTrackingIter<CI>) -> Result<Self, ParseError> {
        todo!();
    }
}

struct JsonObject(std::collections::HashMap<JsonString, JsonValue>);
impl<CI: CharIterator> JsonType<CI> for JsonObject {
    fn parse(i: &mut WhitespaceSkippingIndexTrackingIter<CI>) -> Result<Self, ParseError> {
        {
            let first_char = i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
            if first_char != '{' {
                return Err(ParseError::UnexpectedCharacter {
                    character: first_char,
                    index: i.previously_outputted_index.unwrap(),
                    expected_characters: vec!['{'],
                });
            }
        }
        let mut hashmap = std::collections::HashMap::new();
        let mut is_first = true;
        loop {
            let next_char = i
                .next_non_whitespace()
                .ok_or(ParseError::UnexpectedEndOfString)?;
            if next_char == '}' {
                return Ok(Self(hashmap));
            }
            if next_char != ',' && !is_first {
                return Err(ParseError::UnexpectedCharacter {
                    character: next_char,
                    index: i.previously_outputted_index.unwrap(),
                    expected_characters: vec![',', '}'],
                });
            }
            let key = JsonString::parse(i)?;
            let next_char = i
                .next_non_whitespace()
                .ok_or(ParseError::UnexpectedEndOfString)?;
            if next_char != ':' {
                return Err(ParseError::UnexpectedCharacter {
                    character: next_char,
                    index: i.previously_outputted_index.unwrap(),
                    expected_characters: vec![':'],
                });
            }
            let value = JsonValue::parse(i)?;
            hashmap.insert(key, value);
            is_first = false;
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
struct JsonString(String);
impl<CI: CharIterator> JsonType<CI> for JsonString {
    fn parse(i: &mut WhitespaceSkippingIndexTrackingIter<CI>) -> Result<Self, ParseError> {
        {
            let first_char = i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
            if first_char != '"' {
                return Err(ParseError::UnexpectedCharacter {
                    character: first_char,
                    index: i.previously_outputted_index.unwrap(),
                    expected_characters: vec!['"'],
                });
            }
        }
        let mut string = String::new();
        loop {
            let next_char = i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
            if next_char == '"' {
                return Ok(JsonString(string));
            } else if next_char == '\\' {
                let escaped_character = i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
                if escaped_character == '"' {
                    string.push('"');
                } else if escaped_character == '\\' {
                    string.push('\\');
                } else if escaped_character == 'b' {
                    todo!("Figure out the best way to implement \\b");
                } else if escaped_character == 'f' {
                    todo!("Figure out the best way to implement \\f");
                } else if escaped_character == 'n' {
                    string.push('\n');
                } else if escaped_character == 'r' {
                    string.push('\r');
                } else if escaped_character == 't' {
                    string.push('\t');
                } else if escaped_character == 'u' {
                    todo!("Figure out the best way to implement parsing 4 hex digits")
                } else {
                    return Err(ParseError::UnexpectedCharacter {
                        character: escaped_character,
                        index: i.previously_outputted_index.unwrap(),
                        expected_characters: vec!['"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u'],
                    });
                }
            } else if next_char.is_control() {
                return Err(ParseError::ControlCharacter {
                    control_character: next_char,
                    index: i.previously_outputted_index.unwrap(),
                });
            } else {
                string.push(next_char);
            }
        }
    }
}

fn hex_to_char(chars: [char; 4]) -> char {
    let bytes = chars.map(|c| match c {
        '0' => 0u8,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' | 'A' => 0xA,
        'b' | 'B' => 0xB,
        'c' | 'C' => 0xC,
        'd' | 'D' => 0xD,
        'e' | 'E' => 0xE,
        'f' | 'F' => 0xF,
        _ => panic!()
    });
    let b00 = bytes[0] << 4;
    let b01 = bytes[1];
    let b0 = b00 | b01;
    let b10 = bytes[2] << 4;
    let b11 = bytes[3];
    let b1 = b10 | b11;
    let bytes = [0, 0, b0, b1];
    char::from_u32(u32::from_be_bytes(bytes)).unwrap()
}

fn main() {
    println!("{}", hex_to_char(['0', '1', '9', 'B']));
}

#[cfg(test)]
mod tests {}
