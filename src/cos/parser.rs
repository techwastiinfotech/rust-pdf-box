use crate::cos::document::COSDocument;
use crate::cos::lexer::{Lexer, Token};
use crate::cos::object::{
    COSArray, COSDictionary, COSName, COSObject, COSStream, COSString, ObjectId,
};
use crate::error::{PdfError, PdfResult};
use std::collections::BTreeMap;

pub struct Parser<'a> {
    data: &'a [u8],
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum XRefEntry {
    Free,
    Normal {
        offset: usize,
        generation: u16,
    },
    Compressed {
        obj_stm_num: u32,
        index_in_stream: u32,
    },
}

impl<'a> Parser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Parses an entire PDF file buffer into a `COSDocument`.
    pub fn parse(&self) -> PdfResult<COSDocument> {
        let version = self.parse_header()?;
        let startxref_offset = self.find_startxref()?;

        let mut doc = COSDocument::new();
        doc.version = version;

        // Traverse all xref tables / streams starting from the last startxref
        let mut xref_entries = BTreeMap::new();
        let mut current_xref_offset = Some(startxref_offset);
        let mut final_trailer = COSDictionary::new();

        while let Some(offset) = current_xref_offset {
            let mut lexer = Lexer::with_pos(self.data, offset);
            lexer.skip_whitespace_and_comments();

            let token = lexer.next_token()?;
            match token {
                Token::Keyword(ref k) if k == "xref" => {
                    let trailer = self.parse_classic_xref_section(&mut lexer, &mut xref_entries)?;
                    if final_trailer.entries.is_empty() {
                        final_trailer = trailer.clone();
                    } else {
                        // Merge trailer entries without overwriting newer entries
                        for (k, v) in &trailer.entries {
                            doc.trailer
                                .entries
                                .entry(k.clone())
                                .or_insert_with(|| v.clone());
                        }
                    }

                    if let Some(prev) = trailer.get_integer("Prev") {
                        current_xref_offset = Some(prev as usize);
                    } else {
                        current_xref_offset = None;
                    }
                }
                Token::Integer(_num) => {
                    // Modern XRef Stream: "num gen obj << /Type /XRef ... >> stream ... endstream"
                    let gen_token = lexer.next_token()?;
                    let obj_token = lexer.next_token()?;
                    if matches!(gen_token, Token::Integer(_))
                        && matches!(obj_token, Token::Keyword(ref k) if k == "obj")
                    {
                        let stream_obj = self.parse_direct_object(&mut lexer)?;
                        if let COSObject::Stream(s) = stream_obj {
                            let trailer = s.dict.clone();
                            self.parse_xref_stream(&s, &mut xref_entries)?;
                            if final_trailer.entries.is_empty() {
                                final_trailer = trailer.clone();
                            }
                            if let Some(prev) = trailer.get_integer("Prev") {
                                current_xref_offset = Some(prev as usize);
                            } else {
                                current_xref_offset = None;
                            }
                        } else {
                            return Err(PdfError::Parse {
                                offset: lexer.pos(),
                                message: "Expected XRef stream object".to_string(),
                            });
                        }
                    } else {
                        return Err(PdfError::Parse {
                            offset: lexer.pos(),
                            message: "Invalid xref table/stream start".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(PdfError::Parse {
                        offset: lexer.pos(),
                        message: format!("Expected 'xref' or xref stream, found {:?}", token),
                    });
                }
            }
        }

        doc.trailer = final_trailer;

        // Load all normal objects
        for (&_obj_id, entry) in &xref_entries {
            if let XRefEntry::Normal {
                offset,
                generation: _,
            } = entry
            {
                if *offset < self.data.len() {
                    let mut obj_lexer = Lexer::with_pos(self.data, *offset);
                    if let Ok((id, obj)) = self.parse_indirect_object(&mut obj_lexer) {
                        doc.set_object(id, obj);
                    }
                }
            }
        }

        // Load compressed objects from object streams (/ObjStm)
        let mut obj_streams: BTreeMap<u32, COSStream> = BTreeMap::new();
        for (id, obj) in &doc.objects {
            if let COSObject::Stream(s) = obj {
                if s.dict.get_name("Type") == Some("ObjStm") {
                    obj_streams.insert(id.num, s.clone());
                }
            }
        }

        for (&_obj_id, entry) in &xref_entries {
            if let XRefEntry::Compressed {
                obj_stm_num,
                index_in_stream,
            } = entry
            {
                if let Some(stm) = obj_streams.get(obj_stm_num) {
                    let decomp_res: PdfResult<Vec<u8>> = stm.decode();
                    if let Ok(decompressed) = decomp_res {
                        let n = stm.dict.get_integer("N").unwrap_or(0) as usize;
                        let first = stm.dict.get_integer("First").unwrap_or(0) as usize;

                        let mut lex = Lexer::new(&decompressed);
                        let mut offsets = Vec::new();
                        for _ in 0..n {
                            let obj_num_tok = lex.next_token()?;
                            let offset_tok = lex.next_token()?;
                            if let (Token::Integer(on), Token::Integer(off)) =
                                (obj_num_tok, offset_tok)
                            {
                                offsets.push((on as u32, first + off as usize));
                            }
                        }

                        if let Some(&(target_num, byte_off)) =
                            offsets.get(*index_in_stream as usize)
                        {
                            if byte_off < decompressed.len() {
                                let mut obj_lex = Lexer::with_pos(&decompressed, byte_off);
                                if let Ok(obj) = self.parse_direct_object(&mut obj_lex) {
                                    doc.set_object(ObjectId::new(target_num, 0), obj);
                                }
                            }
                        }
                    }
                }
            }
        }

        doc.clear_dirty();
        Ok(doc)
    }

    fn parse_header(&self) -> PdfResult<String> {
        let header_prefix = b"%PDF-";
        if let Some(idx) = self
            .data
            .windows(header_prefix.len())
            .position(|w| w == header_prefix)
        {
            let start = idx + header_prefix.len();
            let mut end = start;
            while end < self.data.len() && self.data[end] != b'\r' && self.data[end] != b'\n' {
                end += 1;
            }
            let ver = String::from_utf8_lossy(&self.data[start..end]).into_owned();
            Ok(ver)
        } else {
            Err(PdfError::InvalidFormat("Missing %PDF header".to_string()))
        }
    }

    pub fn find_startxref(&self) -> PdfResult<usize> {
        let pattern = b"startxref";
        let search_window = 2048.min(self.data.len());
        let start = self.data.len() - search_window;
        let slice = &self.data[start..];

        if let Some(rel_pos) = slice.windows(pattern.len()).rposition(|w| w == pattern) {
            let pos = start + rel_pos + pattern.len();
            let mut lexer = Lexer::with_pos(self.data, pos);
            let token = lexer.next_token()?;
            if let Token::Integer(offset) = token {
                return Ok(offset as usize);
            }
        }

        Err(PdfError::InvalidFormat(
            "Could not locate 'startxref'".to_string(),
        ))
    }

    fn parse_classic_xref_section(
        &self,
        lexer: &mut Lexer<'a>,
        entries: &mut BTreeMap<ObjectId, XRefEntry>,
    ) -> PdfResult<COSDictionary> {
        loop {
            let token = lexer.next_token()?;
            match token {
                Token::Keyword(ref k) if k == "trailer" => {
                    let trailer_obj = self.parse_direct_object(lexer)?;
                    if let COSObject::Dictionary(dict) = trailer_obj {
                        return Ok(dict);
                    } else {
                        return Err(PdfError::Parse {
                            offset: lexer.pos(),
                            message: "Expected trailer dictionary".to_string(),
                        });
                    }
                }
                Token::Integer(start_num) => {
                    let count_token = lexer.next_token()?;
                    if let Token::Integer(count) = count_token {
                        for i in 0..count {
                            let obj_num = (start_num + i) as u32;
                            let off_token = lexer.next_token()?;
                            let gen_token = lexer.next_token()?;
                            let type_token = lexer.next_token()?;

                            if let (
                                Token::Integer(off),
                                Token::Integer(gen_num),
                                Token::Keyword(t),
                            ) = (off_token, gen_token, type_token)
                            {
                                let obj_id = ObjectId::new(obj_num, gen_num as u16);
                                if t == "n" {
                                    entries.entry(obj_id).or_insert(XRefEntry::Normal {
                                        offset: off as usize,
                                        generation: gen_num as u16,
                                    });
                                } else if t == "f" {
                                    entries.entry(obj_id).or_insert(XRefEntry::Free);
                                }
                            }
                        }
                    }
                }
                Token::EOF => {
                    return Err(PdfError::Parse {
                        offset: lexer.pos(),
                        message: "Unexpected EOF while parsing xref table".to_string(),
                    });
                }
                _ => {}
            }
        }
    }

    fn parse_xref_stream(
        &self,
        stream: &COSStream,
        entries: &mut BTreeMap<ObjectId, XRefEntry>,
    ) -> PdfResult<()> {
        let w_arr = stream
            .dict
            .get_array("W")
            .ok_or_else(|| PdfError::InvalidFormat("Missing /W in xref stream".to_string()))?;
        if w_arr.len() < 3 {
            return Err(PdfError::InvalidFormat(
                "Invalid /W array in xref stream".to_string(),
            ));
        }

        let w1 = w_arr.get(0).and_then(|o| o.as_integer()).unwrap_or(0) as usize;
        let w2 = w_arr.get(1).and_then(|o| o.as_integer()).unwrap_or(0) as usize;
        let w3 = w_arr.get(2).and_then(|o| o.as_integer()).unwrap_or(0) as usize;
        let entry_size = w1 + w2 + w3;

        let index_arr = stream.dict.get_array("Index");
        let ranges: Vec<(u32, u32)> = if let Some(arr) = index_arr {
            let mut list = Vec::new();
            let mut iter = arr.iter();
            while let (Some(a), Some(b)) = (iter.next(), iter.next()) {
                if let (Some(start), Some(count)) = (a.as_integer(), b.as_integer()) {
                    list.push((start as u32, count as u32));
                }
            }
            list
        } else {
            let size = stream.dict.get_integer("Size").unwrap_or(0) as u32;
            vec![(0, size)]
        };

        let decoded = stream.decode()?;
        let mut byte_offset = 0;

        for (start_num, count) in ranges {
            for i in 0..count {
                if byte_offset + entry_size > decoded.len() {
                    break;
                }
                let obj_num = start_num + i;
                let entry_bytes = &decoded[byte_offset..byte_offset + entry_size];
                byte_offset += entry_size;

                let read_int = |slice: &[u8]| -> u64 {
                    let mut val = 0u64;
                    for &b in slice {
                        val = (val << 8) | (b as u64);
                    }
                    val
                };

                let t = if w1 > 0 {
                    read_int(&entry_bytes[0..w1])
                } else {
                    1
                };
                let f2 = read_int(&entry_bytes[w1..w1 + w2]);
                let f3 = read_int(&entry_bytes[w1 + w2..w1 + w2 + w3]);

                match t {
                    0 => {
                        entries
                            .entry(ObjectId::new(obj_num, f3 as u16))
                            .or_insert(XRefEntry::Free);
                    }
                    1 => {
                        entries.entry(ObjectId::new(obj_num, f3 as u16)).or_insert(
                            XRefEntry::Normal {
                                offset: f2 as usize,
                                generation: f3 as u16,
                            },
                        );
                    }
                    2 => {
                        entries
                            .entry(ObjectId::new(obj_num, 0))
                            .or_insert(XRefEntry::Compressed {
                                obj_stm_num: f2 as u32,
                                index_in_stream: f3 as u32,
                            });
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn parse_indirect_object(&self, lexer: &mut Lexer<'a>) -> PdfResult<(ObjectId, COSObject)> {
        let num = match lexer.next_token()? {
            Token::Integer(i) => i as u32,
            tok => {
                return Err(PdfError::Parse {
                    offset: lexer.pos(),
                    message: format!("Expected object number, found {:?}", tok),
                });
            }
        };

        let gen_num = match lexer.next_token()? {
            Token::Integer(i) => i as u16,
            tok => {
                return Err(PdfError::Parse {
                    offset: lexer.pos(),
                    message: format!("Expected generation number, found {:?}", tok),
                });
            }
        };

        match lexer.next_token()? {
            Token::Keyword(ref k) if k == "obj" => {}
            tok => {
                return Err(PdfError::Parse {
                    offset: lexer.pos(),
                    message: format!("Expected 'obj' keyword, found {:?}", tok),
                });
            }
        };

        let obj = self.parse_direct_object(lexer)?;
        let id = ObjectId::new(num, gen_num);
        Ok((id, obj))
    }

    pub fn parse_direct_object(&self, lexer: &mut Lexer<'a>) -> PdfResult<COSObject> {
        let token = lexer.next_token()?;
        match token {
            Token::Keyword(k) => match k.as_str() {
                "null" => Ok(COSObject::Null),
                "true" => Ok(COSObject::Boolean(true)),
                "false" => Ok(COSObject::Boolean(false)),
                _ => Ok(COSObject::Name(COSName::new(k))),
            },
            Token::Integer(i) => {
                // Lookahead to check if this is an indirect reference "num gen R"
                let saved_pos = lexer.pos();
                let next1 = lexer.next_token()?;
                if let Token::Integer(gen_num) = next1 {
                    let next2 = lexer.next_token()?;
                    if let Token::Keyword(ref r) = next2 {
                        if r == "R" {
                            return Ok(COSObject::Reference(ObjectId::new(
                                i as u32,
                                gen_num as u16,
                            )));
                        }
                    }
                }
                // Backtrack if not an indirect reference
                lexer.set_pos(saved_pos);
                Ok(COSObject::Integer(i))
            }
            Token::Real(f) => Ok(COSObject::Real(f)),
            Token::String(bytes, fmt) => Ok(COSObject::String(COSString {
                data: bytes,
                format: fmt,
            })),
            Token::Name(name) => Ok(COSObject::Name(COSName::new(name))),
            Token::ArrayStart => {
                let mut arr = COSArray::new();
                loop {
                    lexer.skip_whitespace_and_comments();
                    if lexer.peek_byte() == Some(b']') {
                        lexer.next_byte();
                        break;
                    }
                    let item = self.parse_direct_object(lexer)?;
                    arr.push(item);
                }
                Ok(COSObject::Array(arr))
            }
            Token::DictStart => {
                let mut dict = COSDictionary::new();
                loop {
                    lexer.skip_whitespace_and_comments();
                    if lexer.peek_byte() == Some(b'>') {
                        lexer.next_token()?; // consume DictEnd
                        break;
                    }
                    let key_tok = lexer.next_token()?;
                    let key_name = match key_tok {
                        Token::Name(n) => n,
                        Token::DictEnd => break,
                        _ => {
                            return Err(PdfError::Parse {
                                offset: lexer.pos(),
                                message: format!(
                                    "Expected dictionary key name, found {:?}",
                                    key_tok
                                ),
                            });
                        }
                    };

                    let val = self.parse_direct_object(lexer)?;
                    dict.set(COSName::new(key_name), val);
                }

                // Check if this dictionary is followed by 'stream'
                let saved_pos = lexer.pos();
                let next_tok = lexer.next_token()?;
                if let Token::Keyword(ref k) = next_tok {
                    if k == "stream" {
                        let stream_len = dict.get_integer("Length").map(|l| l as usize);
                        let stream_data = lexer.read_stream_data(stream_len)?;
                        // skip 'endstream'
                        lexer.skip_whitespace_and_comments();
                        let end_tok = lexer.next_token()?;
                        if !matches!(end_tok, Token::Keyword(ref s) if s == "endstream") {
                            // Some files don't strictly separate endstream, that's fine
                        }
                        return Ok(COSObject::Stream(COSStream::new(dict, stream_data)));
                    }
                }

                // Not a stream, backtrack
                lexer.set_pos(saved_pos);
                Ok(COSObject::Dictionary(dict))
            }
            Token::DictEnd | Token::ArrayEnd | Token::EOF => Err(PdfError::Parse {
                offset: lexer.pos(),
                message: format!("Unexpected token: {:?}", token),
            }),
        }
    }
}
