use crate::cos::object::StringFormat;
use crate::error::{PdfError, PdfResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Integer(i64),
    Real(f64),
    String(Vec<u8>, StringFormat),
    Name(String),
    DictStart, // <<
    DictEnd,   // >>
    ArrayStart,// [
    ArrayEnd,  // ]
    EOF,
}

pub struct Lexer<'a> {
    data: &'a [u8],
    pub pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn with_pos(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos.min(self.data.len());
    }

    pub fn peek_byte(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    pub fn next_byte(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }

    pub fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            if matches!(b, 0 | 9 | 10 | 12 | 13 | 32) {
                self.pos += 1;
            } else if b == b'%' {
                // Comment until \r or \n
                self.pos += 1;
                while self.pos < self.data.len() {
                    let c = self.data[self.pos];
                    self.pos += 1;
                    if c == b'\r' || c == b'\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> PdfResult<Token> {
        self.skip_whitespace_and_comments();
        if self.pos >= self.data.len() {
            return Ok(Token::EOF);
        }

        let start_pos = self.pos;
        let b = self.data[self.pos];

        match b {
            b'<' => {
                self.pos += 1;
                if self.pos < self.data.len() && self.data[self.pos] == b'<' {
                    self.pos += 1;
                    Ok(Token::DictStart)
                } else {
                    self.read_hex_string(start_pos)
                }
            }
            b'>' => {
                self.pos += 1;
                if self.pos < self.data.len() && self.data[self.pos] == b'>' {
                    self.pos += 1;
                    Ok(Token::DictEnd)
                } else {
                    Err(PdfError::Parse {
                        offset: start_pos,
                        message: "Unexpected single '>' character".to_string(),
                    })
                }
            }
            b'[' => {
                self.pos += 1;
                Ok(Token::ArrayStart)
            }
            b']' => {
                self.pos += 1;
                Ok(Token::ArrayEnd)
            }
            b'(' => {
                self.read_literal_string(start_pos)
            }
            b'/' => {
                self.read_name()
            }
            _ => self.read_number_or_keyword(start_pos),
        }
    }

    fn read_name(&mut self) -> PdfResult<Token> {
        self.pos += 1; // skip leading '/'
        let mut name_bytes = Vec::new();

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            if matches!(b, 0 | 9 | 10 | 12 | 13 | 32 | b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%') {
                break;
            }
            if b == b'#' {
                self.pos += 1;
                if self.pos + 1 < self.data.len() {
                    let hex_chars = std::str::from_utf8(&self.data[self.pos..self.pos + 2])
                        .map_err(|_| PdfError::Parse {
                            offset: self.pos,
                            message: "Invalid hex sequence in PDF Name".to_string(),
                        })?;
                    let val = u8::from_str_radix(hex_chars, 16).map_err(|_| PdfError::Parse {
                        offset: self.pos,
                        message: format!("Invalid hex digits in PDF Name: #{hex_chars}"),
                    })?;
                    name_bytes.push(val);
                    self.pos += 2;
                } else {
                    name_bytes.push(b'#');
                }
            } else {
                name_bytes.push(b);
                self.pos += 1;
            }
        }

        let name_str = String::from_utf8_lossy(&name_bytes).into_owned();
        Ok(Token::Name(name_str))
    }

    fn read_literal_string(&mut self, start_pos: usize) -> PdfResult<Token> {
        self.pos += 1; // skip '('
        let mut depth = 1;
        let mut result = Vec::new();

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;

            if b == b'\\' {
                if self.pos >= self.data.len() {
                    break;
                }
                let esc = self.data[self.pos];
                self.pos += 1;
                match esc {
                    b'n' => result.push(b'\n'),
                    b'r' => result.push(b'\r'),
                    b't' => result.push(b'\t'),
                    b'b' => result.push(0x08),
                    b'f' => result.push(0x0C),
                    b'(' => result.push(b'('),
                    b')' => result.push(b')'),
                    b'\\' => result.push(b'\\'),
                    b'\r' => {
                        // line continuation, skip following \n if present
                        if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
                            self.pos += 1;
                        }
                    }
                    b'\n' => {} // line continuation
                    b'0'..=b'7' => {
                        // Octal sequence up to 3 digits
                        let mut oct_val = (esc - b'0') as u32;
                        for _ in 0..2 {
                            if self.pos < self.data.len() && (b'0'..=b'7').contains(&self.data[self.pos]) {
                                oct_val = oct_val * 8 + (self.data[self.pos] - b'0') as u32;
                                self.pos += 1;
                            } else {
                                break;
                            }
                        }
                        result.push((oct_val & 0xFF) as u8);
                    }
                    other => result.push(other),
                }
            } else if b == b'(' {
                depth += 1;
                result.push(b);
            } else if b == b')' {
                depth -= 1;
                if depth == 0 {
                    return Ok(Token::String(result, StringFormat::Literal));
                } else {
                    result.push(b);
                }
            } else {
                result.push(b);
            }
        }

        Err(PdfError::Parse {
            offset: start_pos,
            message: "Unterminated literal string".to_string(),
        })
    }

    fn read_hex_string(&mut self, start_pos: usize) -> PdfResult<Token> {
        let mut hex_digits = Vec::new();

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;

            if b == b'>' {
                break;
            } else if matches!(b, 0 | 9 | 10 | 12 | 13 | 32) {
                continue;
            } else if (b'0'..=b'9').contains(&b) || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b) {
                hex_digits.push(b);
            } else {
                return Err(PdfError::Parse {
                    offset: self.pos - 1,
                    message: format!("Invalid character in hex string: {:02X}", b),
                });
            }
        }

        if hex_digits.len() % 2 != 0 {
            hex_digits.push(b'0'); // odd length padded with 0
        }

        let mut bytes = Vec::with_capacity(hex_digits.len() / 2);
        for chunk in hex_digits.chunks_exact(2) {
            let s = std::str::from_utf8(chunk).unwrap();
            let byte = u8::from_str_radix(s, 16).map_err(|_| PdfError::Parse {
                offset: start_pos,
                message: "Failed parsing hex byte".to_string(),
            })?;
            bytes.push(byte);
        }

        Ok(Token::String(bytes, StringFormat::Hex))
    }

    fn read_number_or_keyword(&mut self, _start_pos: usize) -> PdfResult<Token> {
        let mut token_bytes = Vec::new();

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            if matches!(b, 0 | 9 | 10 | 12 | 13 | 32 | b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%') {
                break;
            }
            token_bytes.push(b);
            self.pos += 1;
        }

        let s = String::from_utf8_lossy(&token_bytes).into_owned();

        // Check if integer
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Token::Integer(i));
        }

        // Check if real float
        if let Ok(f) = s.parse::<f64>() {
            if s.contains('.') {
                return Ok(Token::Real(f));
            }
        }

        Ok(Token::Keyword(s))
    }

    /// Reads raw bytes for stream contents up to length or endstream keyword.
    pub fn read_stream_data(&mut self, length: Option<usize>) -> PdfResult<Vec<u8>> {
        // Skip whitespace after 'stream' keyword
        if self.pos < self.data.len() && self.data[self.pos] == b'\r' {
            self.pos += 1;
        }
        if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
            self.pos += 1;
        }

        if let Some(len) = length {
            if self.pos + len <= self.data.len() {
                let stream_bytes = self.data[self.pos..self.pos + len].to_vec();
                self.pos += len;
                return Ok(stream_bytes);
            }
        }

        // Fallback: search for 'endstream'
        let start = self.pos;
        let pattern = b"endstream";
        if let Some(idx) = self.data[start..].windows(pattern.len()).position(|w| w == pattern) {
            let mut end = start + idx;
            // Trim trailing \r and \n before endstream
            if end > start && self.data[end - 1] == b'\n' {
                end -= 1;
            }
            if end > start && self.data[end - 1] == b'\r' {
                end -= 1;
            }
            let stream_bytes = self.data[start..end].to_vec();
            self.pos = start + idx;
            Ok(stream_bytes)
        } else {
            Err(PdfError::Parse {
                offset: start,
                message: "Missing endstream keyword".to_string(),
            })
        }
    }
}
