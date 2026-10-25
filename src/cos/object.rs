use crate::error::{PdfError, PdfResult};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Write};

/// An indirect object identifier in PDF: object number and generation number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId {
    pub num: u32,
    pub generation: u16,
}

impl ObjectId {
    pub const fn new(num: u32, generation: u16) -> Self {
        Self { num, generation }
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} R", self.num, self.generation)
    }
}

/// Representation format for PDF strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringFormat {
    Literal,
    Hex,
}

/// A PDF string object (can be literal or hex formatted).
#[derive(Clone, PartialEq, Eq)]
pub struct COSString {
    pub data: Vec<u8>,
    pub format: StringFormat,
}

impl COSString {
    pub fn literal(data: impl Into<Vec<u8>>) -> Self {
        Self {
            data: data.into(),
            format: StringFormat::Literal,
        }
    }

    pub fn hex(data: impl Into<Vec<u8>>) -> Self {
        Self {
            data: data.into(),
            format: StringFormat::Hex,
        }
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    pub fn to_string_lossy(&self) -> String {
        // PDF strings may be PDFDocEncoding or UTF-16BE with BOM (0xFE 0xFF)
        if self.data.len() >= 2 && self.data[0] == 0xFE && self.data[1] == 0xFF {
            let u16s: Vec<u16> = self.data[2..]
                .chunks_exact(2)
                .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                .collect();
            String::from_utf16_lossy(&u16s)
        } else {
            String::from_utf8_lossy(&self.data).into_owned()
        }
    }
}

impl fmt::Debug for COSString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            StringFormat::Literal => write!(f, "COSString(\"{}\")", self.to_string_lossy()),
            StringFormat::Hex => {
                let hex_str: String = self.data.iter().map(|b| format!("{:02X}", b)).collect();
                write!(f, "COSString(<{}>)", hex_str)
            }
        }
    }
}

/// A PDF Name object (e.g. /Type, /Catalog, /Page).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct COSName(pub String);

impl COSName {
    pub fn new(name: impl Into<String>) -> Self {
        let mut s = name.into();
        if s.starts_with('/') {
            s.remove(0);
        }
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for COSName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.0)
    }
}

impl fmt::Display for COSName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.0)
    }
}

impl<T: Into<String>> From<T> for COSName {
    fn from(s: T) -> Self {
        Self::new(s)
    }
}

/// A PDF Array object.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct COSArray(pub Vec<COSObject>);

impl COSArray {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn push<T: Into<COSObject>>(&mut self, obj: T) {
        self.0.push(obj.into());
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&COSObject> {
        self.0.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, COSObject> {
        self.0.iter()
    }
}

/// A PDF Dictionary object (`/Key Value`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct COSDictionary {
    pub entries: BTreeMap<COSName, COSObject>,
}

impl COSDictionary {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&COSObject> {
        let name = COSName::new(key);
        self.entries.get(&name)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut COSObject> {
        let name = COSName::new(key);
        self.entries.get_mut(&name)
    }

    pub fn set<K: Into<COSName>, V: Into<COSObject>>(&mut self, key: K, val: V) {
        self.entries.insert(key.into(), val.into());
    }

    pub fn remove(&mut self, key: &str) -> Option<COSObject> {
        let name = COSName::new(key);
        self.entries.remove(&name)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn get_name(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(COSObject::Name(n)) => Some(n.as_str()),
            _ => None,
        }
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(COSObject::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.get(key) {
            Some(COSObject::Real(f)) => Some(*f),
            Some(COSObject::Integer(i)) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key) {
            Some(COSObject::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<&COSString> {
        match self.get(key) {
            Some(COSObject::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_dict(&self, key: &str) -> Option<&COSDictionary> {
        match self.get(key) {
            Some(COSObject::Dictionary(d)) => Some(d),
            _ => None,
        }
    }

    pub fn get_dict_mut(&mut self, key: &str) -> Option<&mut COSDictionary> {
        let name = COSName::new(key);
        match self.entries.get_mut(&name) {
            Some(COSObject::Dictionary(d)) => Some(d),
            _ => None,
        }
    }

    pub fn get_array(&self, key: &str) -> Option<&COSArray> {
        match self.get(key) {
            Some(COSObject::Array(a)) => Some(a),
            _ => None,
        }
    }

    pub fn get_reference(&self, key: &str) -> Option<ObjectId> {
        match self.get(key) {
            Some(COSObject::Reference(id)) => Some(*id),
            _ => None,
        }
    }
}

/// A PDF Stream object (Dictionary + raw data).
#[derive(Debug, Clone, PartialEq)]
pub struct COSStream {
    pub dict: COSDictionary,
    pub data: Vec<u8>,
}

impl COSStream {
    pub fn new(dict: COSDictionary, data: Vec<u8>) -> Self {
        let mut s = Self { dict, data };
        s.update_length();
        s
    }

    pub fn update_length(&mut self) {
        self.dict.set(
            COSName::new("Length"),
            COSObject::Integer(self.data.len() as i64),
        );
    }

    /// Decodes stream contents based on /Filter entries.
    pub fn decode(&self) -> PdfResult<Vec<u8>> {
        let filter = self.dict.get_name("Filter");
        match filter {
            None => Ok(self.data.clone()),
            Some("FlateDecode") => {
                let mut decoder = ZlibDecoder::new(&self.data[..]);
                let mut decoded = Vec::new();
                decoder.read_to_end(&mut decoded).map_err(|e| {
                    PdfError::StreamCompression(format!("Flate decompression error: {e}"))
                })?;
                Ok(decoded)
            }
            Some(other) => Err(PdfError::Unsupported(format!(
                "Unsupported stream filter: {other}"
            ))),
        }
    }

    /// Compresses uncompressed stream contents with FlateDecode.
    pub fn encode_flate(&mut self) -> PdfResult<()> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&self.data)
            .map_err(|e| PdfError::StreamCompression(format!("Flate compression error: {e}")))?;
        let compressed = encoder.finish().map_err(|e| {
            PdfError::StreamCompression(format!("Flate compression finish error: {e}"))
        })?;

        self.data = compressed;
        self.dict
            .set(COSName::new("Filter"), COSName::new("FlateDecode"));
        self.update_length();
        Ok(())
    }
}

/// The fundamental COS Object enum in PDF.
#[derive(Debug, Clone, PartialEq)]
pub enum COSObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(COSString),
    Name(COSName),
    Array(COSArray),
    Dictionary(COSDictionary),
    Stream(COSStream),
    Reference(ObjectId),
}

impl COSObject {
    pub const fn null() -> Self {
        Self::Null
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean(_) => "boolean",
            Self::Integer(_) => "integer",
            Self::Real(_) => "real",
            Self::String(_) => "string",
            Self::Name(_) => "name",
            Self::Array(_) => "array",
            Self::Dictionary(_) => "dictionary",
            Self::Stream(_) => "stream",
            Self::Reference(_) => "reference",
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Real(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            Self::Name(n) => Some(n.as_str()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&COSString> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&COSDictionary> {
        match self {
            Self::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_dict_mut(&mut self) -> Option<&mut COSDictionary> {
        match self {
            Self::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&COSArray> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_stream(&self) -> Option<&COSStream> {
        match self {
            Self::Stream(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_reference(&self) -> Option<ObjectId> {
        match self {
            Self::Reference(id) => Some(*id),
            _ => None,
        }
    }
}

// Ergonomic From implementations
impl From<bool> for COSObject {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<i64> for COSObject {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<i32> for COSObject {
    fn from(i: i32) -> Self {
        Self::Integer(i as i64)
    }
}

impl From<u32> for COSObject {
    fn from(i: u32) -> Self {
        Self::Integer(i as i64)
    }
}

impl From<usize> for COSObject {
    fn from(i: usize) -> Self {
        Self::Integer(i as i64)
    }
}

impl From<f64> for COSObject {
    fn from(f: f64) -> Self {
        Self::Real(f)
    }
}

impl From<f32> for COSObject {
    fn from(f: f32) -> Self {
        Self::Real(f as f64)
    }
}

impl From<COSName> for COSObject {
    fn from(n: COSName) -> Self {
        Self::Name(n)
    }
}

impl From<COSString> for COSObject {
    fn from(s: COSString) -> Self {
        Self::String(s)
    }
}

impl From<COSDictionary> for COSObject {
    fn from(d: COSDictionary) -> Self {
        Self::Dictionary(d)
    }
}

impl From<COSArray> for COSObject {
    fn from(a: COSArray) -> Self {
        Self::Array(a)
    }
}

impl From<COSStream> for COSObject {
    fn from(s: COSStream) -> Self {
        Self::Stream(s)
    }
}

impl From<ObjectId> for COSObject {
    fn from(id: ObjectId) -> Self {
        Self::Reference(id)
    }
}

impl From<&str> for COSObject {
    fn from(s: &str) -> Self {
        Self::String(COSString::literal(s))
    }
}

impl From<String> for COSObject {
    fn from(s: String) -> Self {
        Self::String(COSString::literal(s))
    }
}
