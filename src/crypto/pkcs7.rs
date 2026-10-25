use crate::crypto::asn1::DerElement;
use crate::crypto::x509::SigningKey;
use crate::error::PdfResult;
use chrono::Utc;
use rsa::pkcs1v15::SigningKey as RsaSigningKey;
use rsa::signature::{SignatureEncoding, Signer as RsaSignerTrait};
use sha2::Sha256;

/// Generates a detached PKCS#7 / CMS SignedData structure for PDF digital signing.
pub fn generate_pkcs7_detached(key: &SigningKey, byte_range_sha256: &[u8]) -> PdfResult<Vec<u8>> {
    let id_data = vec![1, 2, 840, 113549, 1, 7, 1];
    let id_signed_data = vec![1, 2, 840, 113549, 1, 7, 2];
    let id_sha256 = vec![2, 16, 840, 1, 101, 3, 4, 2, 1];
    let rsa_encryption = vec![1, 2, 840, 113549, 1, 1, 1];
    let id_content_type = vec![1, 2, 840, 113549, 1, 9, 3];
    let id_message_digest = vec![1, 2, 840, 113549, 1, 9, 4];
    let id_signing_time = vec![1, 2, 840, 113549, 1, 9, 5];

    // 1. Signed Attributes (SET OF Attribute)
    // Attribute 1: contentType -> id-data
    let attr_content_type = DerElement::Sequence(vec![
        DerElement::ObjectIdentifier(id_content_type),
        DerElement::Set(vec![DerElement::ObjectIdentifier(id_data.clone())]),
    ]);

    // Attribute 2: messageDigest -> hash of PDF byte ranges
    let attr_message_digest = DerElement::Sequence(vec![
        DerElement::ObjectIdentifier(id_message_digest),
        DerElement::Set(vec![DerElement::OctetString(byte_range_sha256.to_vec())]),
    ]);

    // Attribute 3: signingTime
    let now = Utc::now();
    let time_str = now.format("%y%m%d%H%M%SZ").to_string();
    let attr_signing_time = DerElement::Sequence(vec![
        DerElement::ObjectIdentifier(id_signing_time),
        DerElement::Set(vec![DerElement::UtcTime(time_str)]),
    ]);

    // In DER, attributes in a SET must be sorted by DER encoded bytes
    let mut raw_attrs = vec![attr_content_type, attr_message_digest, attr_signing_time];
    raw_attrs.sort_by_key(|a| a.to_der());

    let signed_attrs_set = DerElement::Set(raw_attrs.clone());
    let signed_attrs_der = signed_attrs_set.to_der();

    // 2. Sign the signed attributes DER bytes
    let signer = RsaSigningKey::<Sha256>::new(key.private_key.clone());
    let signature = signer.sign(&signed_attrs_der);

    // 3. SignerIdentifier: IssuerAndSerialNumber
    let issuer_seq = DerElement::Sequence(vec![DerElement::Set(vec![DerElement::Sequence(vec![
        DerElement::ObjectIdentifier(vec![2, 5, 4, 3]),
        DerElement::Utf8String("Techwasti Infotech".to_string()),
    ])])]);

    let issuer_and_serial = DerElement::Sequence(vec![
        issuer_seq,
        DerElement::integer_from_bytes(&key.serial_number),
    ]);

    // 4. SignerInfo
    // [0] tag for authenticatedAttributes in SignerInfo
    let auth_attrs_tagged =
        DerElement::ContextSpecific(0, Box::new(DerElement::Set(raw_attrs)), true);

    let signer_info = DerElement::Sequence(vec![
        DerElement::integer_from_u64(1), // version = 1
        issuer_and_serial,
        DerElement::Sequence(vec![
            DerElement::ObjectIdentifier(id_sha256.clone()),
            DerElement::Null,
        ]),
        auth_attrs_tagged,
        DerElement::Sequence(vec![
            DerElement::ObjectIdentifier(rsa_encryption),
            DerElement::Null,
        ]),
        DerElement::OctetString(signature.to_vec()),
    ]);

    // 5. EncapsulatedContentInfo: id-data, no content (detached)
    let encap_content_info = DerElement::Sequence(vec![DerElement::ObjectIdentifier(id_data)]);

    // 6. Certificates [0] IMPLICIT
    // We embed the raw DER bytes of the signer's certificate directly
    let cert_tagged = DerElement::ContextSpecific(
        0,
        Box::new(DerElement::OctetString(key.certificate_der.clone())),
        true,
    );

    // 7. SignedData
    let signed_data = DerElement::Sequence(vec![
        DerElement::integer_from_u64(1), // version = 1
        DerElement::Set(vec![DerElement::Sequence(vec![
            DerElement::ObjectIdentifier(id_sha256),
            DerElement::Null,
        ])]),
        encap_content_info,
        cert_tagged,
        DerElement::Set(vec![signer_info]),
    ]);

    // 8. Outer ContentInfo
    let content_info = DerElement::Sequence(vec![
        DerElement::ObjectIdentifier(id_signed_data),
        DerElement::ContextSpecific(0, Box::new(signed_data), true),
    ]);

    Ok(content_info.to_der())
}
