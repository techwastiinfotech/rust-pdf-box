use crate::cos::lexer::{Lexer, Token};
use crate::cos::object::COSObject;
use crate::error::PdfResult;
use crate::pd::document::PDDocument;

/// Text extraction engine inspired by Apache PDFBox PDFTextStripper.
pub struct PDFTextStripper;

impl PDFTextStripper {
    /// Extracts all text from all pages of the document.
    pub fn extract_text(doc: &PDDocument) -> PdfResult<String> {
        let mut full_text = String::new();
        for i in 0..doc.page_count() {
            let page_text = Self::extract_page_text(doc, i)?;
            if !page_text.is_empty() {
                if !full_text.is_empty() {
                    full_text.push_str("\n\n");
                }
                full_text.push_str(&page_text);
            }
        }
        Ok(full_text)
    }

    /// Extracts text from a single page by 0-based index.
    pub fn extract_page_text(doc: &PDDocument, page_index: usize) -> PdfResult<String> {
        let page = doc.get_page(page_index)?;
        let content_ids = page.content_stream_ids();

        let mut page_text = String::new();

        for id in content_ids {
            if let Some(COSObject::Stream(stream)) = doc.cos_doc.get_object(id) {
                if let Ok(data) = stream.decode() {
                    let stream_text = Self::extract_text_from_stream(&data);
                    if !stream_text.is_empty() {
                        if !page_text.is_empty() {
                            page_text.push('\n');
                        }
                        page_text.push_str(&stream_text);
                    }
                }
            }
        }

        Ok(page_text)
    }

    /// Parses content stream operators to extract text strings.
    fn extract_text_from_stream(stream_data: &[u8]) -> String {
        let mut lexer = Lexer::new(stream_data);
        let mut extracted = String::new();
        let mut token_stack: Vec<Token> = Vec::new();
        let mut in_text_object = false;

        while let Ok(token) = lexer.next_token() {
            match token {
                Token::EOF => break,
                Token::Keyword(ref k) => match k.as_str() {
                    "BT" => {
                        in_text_object = true;
                        token_stack.clear();
                    }
                    "ET" => {
                        in_text_object = false;
                        token_stack.clear();
                        if !extracted.ends_with('\n') && !extracted.is_empty() {
                            extracted.push('\n');
                        }
                    }
                    "T*" => {
                        extracted.push('\n');
                        token_stack.clear();
                    }
                    "Td" | "TD" => {
                        // If vertical translation is negative, it's typically a new line
                        if token_stack.len() >= 2 {
                            if let Token::Real(ty) = token_stack[token_stack.len() - 1] {
                                if ty < 0.0 && !extracted.ends_with('\n') {
                                    extracted.push('\n');
                                }
                            } else if let Token::Integer(ty) = token_stack[token_stack.len() - 1] {
                                if ty < 0 && !extracted.ends_with('\n') {
                                    extracted.push('\n');
                                }
                            }
                        }
                        token_stack.clear();
                    }
                    "Tj" => {
                        if let Some(Token::String(bytes, _)) = token_stack.pop() {
                            let text = String::from_utf8_lossy(&bytes);
                            extracted.push_str(&text);
                        }
                        token_stack.clear();
                    }
                    "'" => {
                        extracted.push('\n');
                        if let Some(Token::String(bytes, _)) = token_stack.pop() {
                            let text = String::from_utf8_lossy(&bytes);
                            extracted.push_str(&text);
                        }
                        token_stack.clear();
                    }
                    "\"" => {
                        extracted.push('\n');
                        if let Some(Token::String(bytes, _)) = token_stack.pop() {
                            let text = String::from_utf8_lossy(&bytes);
                            extracted.push_str(&text);
                        }
                        token_stack.clear();
                    }
                    "TJ" => {
                        // Array of strings and kerning numbers: [(Hello) -100 (World)] TJ
                        if let Some(Token::ArrayEnd) = token_stack.pop() {
                            let mut elements = Vec::new();
                            while let Some(top) = token_stack.pop() {
                                if top == Token::ArrayStart {
                                    break;
                                }
                                elements.push(top);
                            }
                            elements.reverse();

                            for el in elements {
                                match el {
                                    Token::String(bytes, _) => {
                                        let text = String::from_utf8_lossy(&bytes);
                                        extracted.push_str(&text);
                                    }
                                    Token::Integer(kern) if kern < -150 => {
                                        // Large negative kerning indicates a space
                                        if !extracted.ends_with(' ') {
                                            extracted.push(' ');
                                        }
                                    }
                                    Token::Real(kern) if kern < -150.0 => {
                                        if !extracted.ends_with(' ') {
                                            extracted.push(' ');
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        token_stack.clear();
                    }
                    _ => {
                        token_stack.clear();
                    }
                },
                other => {
                    if in_text_object {
                        token_stack.push(other);
                    }
                }
            }
        }

        extracted.trim().to_string()
    }
}
