/// Lightweight ASN.1 DER encoder for X.509 certificates and PKCS#7 SignedData.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerElement {
    Boolean(bool),
    Integer(Vec<u8>),       // signed big-endian integer bytes
    BitString(Vec<u8>, u8), // data and unused bits count
    OctetString(Vec<u8>),
    Null,
    ObjectIdentifier(Vec<u64>),
    Utf8String(String),
    PrintableString(String),
    UtcTime(String),         // YYMMDDHHMMSSZ
    GeneralizedTime(String), // YYYYMMDDHHMMSSZ
    Sequence(Vec<DerElement>),
    Set(Vec<DerElement>),
    ContextSpecific(u8, Box<DerElement>, bool), // tag number, inner element, is_constructed
}

impl DerElement {
    pub fn integer_from_u64(val: u64) -> Self {
        let bytes = val.to_be_bytes();
        let mut start = 0;
        while start < bytes.len() - 1 && bytes[start] == 0 {
            start += 1;
        }
        let mut slice = bytes[start..].to_vec();
        // If highest bit is 1, prepend 0x00 so it is positive
        if (slice[0] & 0x80) != 0 {
            slice.insert(0, 0x00);
        }
        Self::Integer(slice)
    }

    pub fn integer_from_bytes(bytes: &[u8]) -> Self {
        let mut v = bytes.to_vec();
        if v.is_empty() {
            v.push(0);
        } else if (v[0] & 0x80) != 0 {
            v.insert(0, 0x00);
        }
        Self::Integer(v)
    }

    pub fn to_der(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            Self::Boolean(b) => {
                out.push(0x01);
                out.push(0x01);
                out.push(if *b { 0xFF } else { 0x00 });
            }
            Self::Integer(bytes) => {
                out.push(0x02);
                encode_length(&mut out, bytes.len());
                out.extend_from_slice(bytes);
            }
            Self::BitString(bytes, unused_bits) => {
                out.push(0x03);
                encode_length(&mut out, bytes.len() + 1);
                out.push(*unused_bits);
                out.extend_from_slice(bytes);
            }
            Self::OctetString(bytes) => {
                out.push(0x04);
                encode_length(&mut out, bytes.len());
                out.extend_from_slice(bytes);
            }
            Self::Null => {
                out.push(0x05);
                out.push(0x00);
            }
            Self::ObjectIdentifier(arcs) => {
                out.push(0x06);
                let mut body = Vec::new();
                if arcs.len() >= 2 {
                    body.push((arcs[0] * 40 + arcs[1]) as u8);
                    for &arc in &arcs[2..] {
                        encode_oid_arc(&mut body, arc);
                    }
                }
                encode_length(&mut out, body.len());
                out.extend_from_slice(&body);
            }
            Self::Utf8String(s) => {
                out.push(0x0C);
                encode_length(&mut out, s.len());
                out.extend_from_slice(s.as_bytes());
            }
            Self::PrintableString(s) => {
                out.push(0x13);
                encode_length(&mut out, s.len());
                out.extend_from_slice(s.as_bytes());
            }
            Self::UtcTime(s) => {
                out.push(0x17);
                encode_length(&mut out, s.len());
                out.extend_from_slice(s.as_bytes());
            }
            Self::GeneralizedTime(s) => {
                out.push(0x18);
                encode_length(&mut out, s.len());
                out.extend_from_slice(s.as_bytes());
            }
            Self::Sequence(elements) => {
                let mut body = Vec::new();
                for el in elements {
                    body.extend_from_slice(&el.to_der());
                }
                out.push(0x30); // 0x20 | 0x10 (Constructed | Sequence)
                encode_length(&mut out, body.len());
                out.extend_from_slice(&body);
            }
            Self::Set(elements) => {
                let mut body = Vec::new();
                for el in elements {
                    body.extend_from_slice(&el.to_der());
                }
                out.push(0x31); // 0x20 | 0x11 (Constructed | Set)
                encode_length(&mut out, body.len());
                out.extend_from_slice(&body);
            }
            Self::ContextSpecific(tag, inner, is_constructed) => {
                let body = inner.to_der();
                let tag_byte = 0x80 | (if *is_constructed { 0x20 } else { 0x00 }) | (tag & 0x1F);
                out.push(tag_byte);
                // For explicit tagging, inner DER includes its own tag and length
                encode_length(&mut out, body.len());
                out.extend_from_slice(&body);
            }
        }
        out
    }
}

fn encode_length(out: &mut Vec<u8>, len: usize) {
    if len < 128 {
        out.push(len as u8);
    } else {
        let mut len_bytes = Vec::new();
        let mut temp = len;
        while temp > 0 {
            len_bytes.push((temp & 0xFF) as u8);
            temp >>= 8;
        }
        len_bytes.reverse();
        out.push(0x80 | (len_bytes.len() as u8));
        out.extend_from_slice(&len_bytes);
    }
}

fn encode_oid_arc(out: &mut Vec<u8>, mut arc: u64) {
    let mut chunks = Vec::new();
    chunks.push((arc & 0x7F) as u8);
    arc >>= 7;
    while arc > 0 {
        chunks.push(0x80 | ((arc & 0x7F) as u8));
        arc >>= 7;
    }
    chunks.reverse();
    out.extend_from_slice(&chunks);
}
