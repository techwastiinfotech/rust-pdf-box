use crate::cos::document::COSDocument;
use crate::cos::object::{
    COSArray, COSDictionary, COSName, COSObject, COSStream, COSString, ObjectId, StringFormat,
};
use crate::error::PdfResult;
use std::collections::BTreeMap;
use std::io::Write;

pub struct Writer<W: Write> {
    writer: W,
    written_bytes: usize,
}

impl<W: Write> Writer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            written_bytes: 0,
        }
    }

    pub fn with_offset(writer: W, offset: usize) -> Self {
        Self {
            writer,
            written_bytes: offset,
        }
    }

    pub fn written_bytes(&self) -> usize {
        self.written_bytes
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> PdfResult<()> {
        self.writer.write_all(bytes)?;
        self.written_bytes += bytes.len();
        Ok(())
    }

    pub fn write_str(&mut self, s: &str) -> PdfResult<()> {
        self.write_bytes(s.as_bytes())
    }

    pub fn write_line(&mut self, s: &str) -> PdfResult<()> {
        self.write_str(s)?;
        self.write_bytes(b"\n")
    }

    /// Serializes an entire COSDocument into standard PDF format.
    pub fn write_document(&mut self, doc: &COSDocument) -> PdfResult<()> {
        // 1. Header
        self.write_line(&format!("%PDF-{}", doc.version))?;
        // 4 binary bytes > 127 to indicate binary file
        self.write_bytes(b"%\xE2\xE3\xCF\xD3\n")?;

        let mut offsets: BTreeMap<ObjectId, usize> = BTreeMap::new();

        // 2. Body: write all indirect objects in order
        for (&id, obj) in &doc.objects {
            offsets.insert(id, self.written_bytes);
            self.write_indirect_object(id, obj)?;
        }

        // 3. Cross-reference table
        let xref_offset = self.written_bytes;
        let size = (doc.max_object_number + 1) as usize;

        self.write_line("xref")?;
        self.write_line(&format!("0 {}", size))?;

        // Object 0 (head of free list)
        self.write_str("0000000000 65535 f \n")?;

        for num in 1..=doc.max_object_number {
            let id = ObjectId::new(num, 0);
            if let Some(&off) = offsets.get(&id) {
                self.write_str(&format!("{:010} {:05} n \n", off, id.generation))?;
            } else {
                self.write_str("0000000000 00000 f \n")?;
            }
        }

        // 4. Trailer
        let mut trailer = doc.trailer.clone();
        trailer.set("Size", size as i64);

        self.write_line("trailer")?;
        self.write_object(&COSObject::Dictionary(trailer))?;
        self.write_line("")?;

        // 5. Startxref & EOF
        self.write_line("startxref")?;
        self.write_line(&format!("{}", xref_offset))?;
        self.write_line("%%EOF")?;

        Ok(())
    }

    /// Appends an incremental update to an existing PDF buffer.
    pub fn write_incremental_update(
        &mut self,
        doc: &COSDocument,
        prev_startxref: usize,
    ) -> PdfResult<()> {
        let mut updated_offsets: BTreeMap<ObjectId, usize> = BTreeMap::new();

        // Write only dirty objects
        for &id in &doc.dirty_objects {
            if let Some(obj) = doc.get_object(id) {
                updated_offsets.insert(id, self.written_bytes);
                self.write_indirect_object(id, obj)?;
            }
        }

        let xref_offset = self.written_bytes;
        self.write_line("xref")?;

        // Group updated objects into contiguous sections or single entries
        for (&id, &off) in &updated_offsets {
            self.write_line(&format!("{} 1", id.num))?;
            self.write_str(&format!("{:010} {:05} n \n", off, id.generation))?;
        }

        // Trailer pointing to previous xref
        let mut trailer = doc.trailer.clone();
        let size = (doc.max_object_number + 1) as i64;
        trailer.set("Size", size);
        trailer.set("Prev", prev_startxref as i64);

        self.write_line("trailer")?;
        self.write_object(&COSObject::Dictionary(trailer))?;
        self.write_line("")?;

        // Startxref & EOF
        self.write_line("startxref")?;
        self.write_line(&format!("{}", xref_offset))?;
        self.write_line("%%EOF")?;

        Ok(())
    }

    pub fn write_indirect_object(&mut self, id: ObjectId, obj: &COSObject) -> PdfResult<()> {
        self.write_line(&format!("{} {} obj", id.num, id.generation))?;
        self.write_object(obj)?;
        self.write_line("\nendobj")?;
        Ok(())
    }

    pub fn write_object(&mut self, obj: &COSObject) -> PdfResult<()> {
        match obj {
            COSObject::Null => self.write_str("null"),
            COSObject::Boolean(b) => self.write_str(if *b { "true" } else { "false" }),
            COSObject::Integer(i) => self.write_str(&format!("{}", i)),
            COSObject::Real(f) => {
                if f.fract() == 0.0 {
                    self.write_str(&format!("{:.1}", f))
                } else {
                    let mut s = format!("{:.5}", f);
                    while s.ends_with('0') && s.contains('.') {
                        s.pop();
                    }
                    if s.ends_with('.') {
                        s.pop();
                    }
                    self.write_str(&s)
                }
            }
            COSObject::String(s) => self.write_string(s),
            COSObject::Name(n) => self.write_name(n),
            COSObject::Array(a) => self.write_array(a),
            COSObject::Dictionary(d) => self.write_dictionary(d),
            COSObject::Stream(s) => self.write_stream(s),
            COSObject::Reference(id) => self.write_str(&format!("{} {} R", id.num, id.generation)),
        }
    }

    pub fn write_name(&mut self, name: &COSName) -> PdfResult<()> {
        self.write_str("/")?;
        for &b in name.as_str().as_bytes() {
            if matches!(b, 33..=126)
                && !matches!(
                    b,
                    b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%' | b'#'
                )
            {
                self.write_bytes(&[b])?;
            } else {
                self.write_str(&format!("#{:02X}", b))?;
            }
        }
        Ok(())
    }

    pub fn write_string(&mut self, s: &COSString) -> PdfResult<()> {
        match s.format {
            StringFormat::Literal => {
                self.write_str("(")?;
                for &b in &s.data {
                    match b {
                        b'(' => self.write_str(r"\(")?,
                        b')' => self.write_str(r"\)")?,
                        b'\\' => self.write_str(r"\\")?,
                        b'\r' => self.write_str(r"\r")?,
                        b'\n' => self.write_str(r"\n")?,
                        _ => self.write_bytes(&[b])?,
                    }
                }
                self.write_str(")")
            }
            StringFormat::Hex => {
                self.write_str("<")?;
                for &b in &s.data {
                    self.write_str(&format!("{:02X}", b))?;
                }
                self.write_str(">")
            }
        }
    }

    pub fn write_array(&mut self, arr: &COSArray) -> PdfResult<()> {
        self.write_str("[")?;
        for (i, item) in arr.iter().enumerate() {
            if i > 0 {
                self.write_str(" ")?;
            }
            self.write_object(item)?;
        }
        self.write_str("]")
    }

    pub fn write_dictionary(&mut self, dict: &COSDictionary) -> PdfResult<()> {
        self.write_line("<<")?;
        for (key, val) in &dict.entries {
            self.write_str("  ")?;
            self.write_name(key)?;
            self.write_str(" ")?;
            self.write_object(val)?;
            self.write_line("")?;
        }
        self.write_str(">>")
    }

    pub fn write_stream(&mut self, stream: &COSStream) -> PdfResult<()> {
        let mut dict = stream.dict.clone();
        dict.set("Length", stream.data.len() as i64);

        self.write_dictionary(&dict)?;
        self.write_line("")?;
        self.write_line("stream")?;
        self.write_bytes(&stream.data)?;
        self.write_line("")?;
        self.write_str("endstream")
    }
}
