// https://www.json.org/json-en.html
#![allow(dead_code)]
#![allow(unused_variables)]

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
            let next = self
                .inner
                .next_if(|c| is_json_whitespace(*c) || *c == expected)?;
            if is_json_whitespace(next) {
                continue;
            }
            return Some(next);
        }
    }

    fn expect_specific_char(&mut self, expected: char) -> Result<(), ParseError> {
        let c = self.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
        if c == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedCharacter {
                character: c,
                index: self.previously_outputted_index.unwrap(),
                expected_characters: vec![expected],
            })
        }
    }
    fn expect_specific_char_ignore_whitespace(&mut self, expected: char) -> Result<(), ParseError> {
        let c = self
            .next_non_whitespace()
            .ok_or(ParseError::UnexpectedEndOfString)?;
        if c == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedCharacter {
                character: c,
                index: self.previously_outputted_index.unwrap(),
                expected_characters: vec![expected],
            })
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
    UnexpectedNonHexCharacter {
        character: char,
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
        i.expect_specific_char('[')?;
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
        i.expect_specific_char('{')?;
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
            i.expect_specific_char_ignore_whitespace(':')?;
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
        i.expect_specific_char('"')?;
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
                    fn parse4hex<CI: CharIterator>(
                        i: &mut WhitespaceSkippingIndexTrackingIter<CI>,
                    ) -> Result<u16, ParseError> {
                        let mut next_char =
                            i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
                        let b00 = hex_digit_to_byte(next_char);
                        let b00 = b00.ok_or(ParseError::UnexpectedNonHexCharacter {
                            character: next_char,
                            index: i.previously_outputted_index.unwrap(),
                        })?;
                        next_char = i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
                        let b01 = hex_digit_to_byte(next_char);
                        let b01 = b01.ok_or(ParseError::UnexpectedNonHexCharacter {
                            character: next_char,
                            index: i.previously_outputted_index.unwrap(),
                        })?;

                        next_char = i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
                        let b10 = hex_digit_to_byte(next_char);
                        let b10 = b10.ok_or(ParseError::UnexpectedNonHexCharacter {
                            character: next_char,
                            index: i.previously_outputted_index.unwrap(),
                        })?;
                        next_char = i.next_any().ok_or(ParseError::UnexpectedEndOfString)?;
                        let b11 = hex_digit_to_byte(next_char);
                        let b11 = b11.ok_or(ParseError::UnexpectedNonHexCharacter {
                            character: next_char,
                            index: i.previously_outputted_index.unwrap(),
                        })?;

                        let b0 = (b00 << 4) | b01;
                        let b1 = (b10 << 4) | b11;

                        Ok(((b0 as u16) << 8) | (b1 as u16))
                    }
                    let w0 = parse4hex(i)?;
                    if (0xD800..=0xDFFF).contains(&w0) {
                        i.expect_specific_char('\\')?;
                        i.expect_specific_char('u')?;
                        let w1 = parse4hex(i)?;
                        string.push(char::decode_utf16([w0, w1]).next().unwrap().unwrap());
                    } else {
                        string.push(
                            char::decode_utf16(std::iter::once(w0))
                                .next()
                                .unwrap()
                                .unwrap(),
                        );
                    }
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

fn hex_digit_to_byte(hex_digit: char) -> Option<u8> {
    match hex_digit {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'a' | 'A' => Some(0xA),
        'b' | 'B' => Some(0xB),
        'c' | 'C' => Some(0xC),
        'd' | 'D' => Some(0xD),
        'e' | 'E' => Some(0xE),
        'f' | 'F' => Some(0xF),
        _ => None,
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    #[test]
    fn all_non_surrogates_are_valid() {
        fn test(x: u16) {
            let mut iter = char::decode_utf16(std::iter::once(x));
            assert!(iter.next().unwrap().is_ok());
            let second_next = iter.next();
            assert!(second_next.is_none());
        }
        for x in 0x0000..=0xD7FF {
            test(x)
        }
        for x in 0xE000..=0xFFFF {
            test(x)
        }
    }
}
