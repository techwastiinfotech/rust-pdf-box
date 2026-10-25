use rust_pdf_box::cos::{
    COSArray, COSDictionary, COSDocument, COSName, COSStream, COSString,
    Parser, Writer,
};

#[test]
fn test_cos_string_escaping_and_hex() {
    let mut doc = COSDocument::new();
    let id = doc.allocate_id();

    let mut dict = COSDictionary::new();
    // Literal string with parentheses and escapes
    dict.set("EscapedLiteral", COSString::literal("Line 1\nLine 2\t(Nested) \\ Backslash"));
    // Hex string
    dict.set("HexStr", COSString::hex(vec![0xDE, 0xAD, 0xBE, 0xEF]));
    // Name with spaces and special chars
    dict.set(COSName::new("Custom Name #1"), 100);

    doc.set_object(id, dict);
    doc.trailer.set("Root", id);

    let mut buf = Vec::new();
    let mut writer = Writer::new(&mut buf);
    writer.write_document(&doc).expect("write failed");

    let parser = Parser::new(&buf);
    let parsed_doc = parser.parse().expect("parse failed");

    let parsed_obj = parsed_doc.get_object(id).expect("object exists");
    let parsed_dict = parsed_obj.as_dict().expect("is dict");

    let lit = parsed_dict.get_string("EscapedLiteral").expect("has string");
    assert_eq!(lit.to_string_lossy(), "Line 1\nLine 2\t(Nested) \\ Backslash");

    let hex = parsed_dict.get_string("HexStr").expect("has hex");
    assert_eq!(hex.data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn test_cos_stream_flate_compression() {
    let mut doc = COSDocument::new();
    let id = doc.allocate_id();

    let original_data = b"Hello, this is a highly compressible repetitive string! ".repeat(50);
    let mut stream = COSStream::new(COSDictionary::new(), original_data.clone());
    stream.encode_flate().expect("compression failed");

    assert!(stream.data.len() < original_data.len());
    assert_eq!(stream.dict.get_name("Filter"), Some("FlateDecode"));

    let decompressed = stream.decode().expect("decompression failed");
    assert_eq!(decompressed, original_data);

    doc.set_object(id, stream);
    doc.trailer.set("Root", id);

    let mut buf = Vec::new();
    let mut writer = Writer::new(&mut buf);
    writer.write_document(&doc).expect("write failed");

    let parser = Parser::new(&buf);
    let parsed_doc = parser.parse().expect("parse failed");
    let parsed_stream = parsed_doc.get_object(id).unwrap().as_stream().unwrap();
    assert_eq!(parsed_stream.decode().unwrap(), original_data);
}

#[test]
fn test_cos_array_and_nested_dictionaries() {
    let mut doc = COSDocument::new();
    let id = doc.allocate_id();

    let mut inner_dict = COSDictionary::new();
    inner_dict.set("KeyA", "ValA");
    inner_dict.set("KeyB", 999);

    let mut arr = COSArray::new();
    arr.push(1);
    arr.push(2.5);
    arr.push("StringItem");
    arr.push(inner_dict);

    let mut root_dict = COSDictionary::new();
    root_dict.set("MyArray", arr);

    doc.set_object(id, root_dict);
    doc.trailer.set("Root", id);

    let mut buf = Vec::new();
    let mut writer = Writer::new(&mut buf);
    writer.write_document(&doc).expect("write failed");

    let parser = Parser::new(&buf);
    let parsed = parser.parse().expect("parse failed");
    let root = parsed.get_object(id).unwrap().as_dict().unwrap();

    let parsed_arr = root.get_array("MyArray").unwrap();
    assert_eq!(parsed_arr.len(), 4);
    assert_eq!(parsed_arr.get(0).unwrap().as_integer(), Some(1));
    assert_eq!(parsed_arr.get(1).unwrap().as_float(), Some(2.5));
}
