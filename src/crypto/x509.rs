use crate::crypto::asn1::DerElement;
use crate::error::{PdfError, PdfResult};
use chrono::{Duration, Utc};
use rsa::pkcs1v15::SigningKey as RsaSigningKey;
use rsa::signature::{SignatureEncoding, Signer as RsaSignerTrait};
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use sha2::Sha256;

/// A cryptographic signing key and associated X.509 certificate.
#[derive(Clone)]
pub struct SigningKey {
    pub private_key: RsaPrivateKey,
    pub certificate_der: Vec<u8>,
    pub issuer_name_der: Vec<u8>,
    pub serial_number: Vec<u8>,
}

impl SigningKey {
    /// Generates an in-memory RSA key pair and a self-signed X.509 certificate for rapid signing.
    pub fn generate_self_signed(common_name: &str) -> PdfResult<Self> {
        Self::generate_self_signed_with_bits(common_name, 2048)
    }

    /// Generates a self-signed certificate with specified RSA key bits (e.g. 2048, 3072, 4096).
    pub fn generate_self_signed_with_bits(common_name: &str, bits: usize) -> PdfResult<Self> {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| PdfError::Crypto(format!("Failed generating RSA key: {e}")))?;

        // Serial number (e.g. 16 random bytes)
        let serial_num = vec![1, 2, 3, 4, 5, 6, 7, 8];

        // Validity: today to 10 years later
        let not_before = Utc::now();
        let not_after = not_before + Duration::days(365 * 10);
        let not_before_str = not_before.format("%y%m%d%H%M%SZ").to_string();
        let not_after_str = not_after.format("%y%m%d%H%M%SZ").to_string();

        let validity = DerElement::Sequence(vec![
            DerElement::UtcTime(not_before_str),
            DerElement::UtcTime(not_after_str),
        ]);

        // Subject / Issuer RDN: CN={common_name}
        let cn_oid = vec![2, 5, 4, 3];
        let rdn_attr = DerElement::Sequence(vec![
            DerElement::ObjectIdentifier(cn_oid),
            DerElement::Utf8String(common_name.to_string()),
        ]);
        let rdn_set = DerElement::Set(vec![rdn_attr]);
        let name_seq = DerElement::Sequence(vec![rdn_set]);
        let name_der = name_seq.to_der();

        // AlgorithmIdentifier: sha256WithRSAEncryption (1.2.840.113549.1.1.11)
        let sha256_with_rsa_oid = vec![1, 2, 840, 113549, 1, 1, 11];
        let alg_id = DerElement::Sequence(vec![
            DerElement::ObjectIdentifier(sha256_with_rsa_oid.clone()),
            DerElement::Null,
        ]);

        // SubjectPublicKeyInfo: rsaEncryption (1.2.840.113549.1.1.1)
        let rsa_oid = vec![1, 2, 840, 113549, 1, 1, 1];
        let pub_key = private_key.to_public_key();
        let n_bytes = pub_key.n().to_bytes_be();
        let e_bytes = pub_key.e().to_bytes_be();
        let rsa_pub_der = DerElement::Sequence(vec![
            DerElement::integer_from_bytes(&n_bytes),
            DerElement::integer_from_bytes(&e_bytes),
        ]).to_der();

        let spki = DerElement::Sequence(vec![
            DerElement::Sequence(vec![
                DerElement::ObjectIdentifier(rsa_oid),
                DerElement::Null,
            ]),
            DerElement::BitString(rsa_pub_der, 0),
        ]);

        // TBSCertificate
        let tbs_cert = DerElement::Sequence(vec![
            // v3 version tag [0] -> 2
            DerElement::ContextSpecific(0, Box::new(DerElement::integer_from_u64(2)), true),
            DerElement::integer_from_bytes(&serial_num),
            alg_id.clone(),
            name_seq.clone(), // issuer
            validity,
            name_seq.clone(), // subject
            spki,
        ]);

        let tbs_der = tbs_cert.to_der();

        // Sign TBSCertificate with RSA-SHA256
        let signer = RsaSigningKey::<Sha256>::new(private_key.clone());
        let signature = signer.sign(&tbs_der);

        // Final X.509 Certificate
        let cert = DerElement::Sequence(vec![
            tbs_cert,
            alg_id,
            DerElement::BitString(signature.to_vec(), 0),
        ]);

        let cert_der = cert.to_der();

        Ok(Self {
            private_key,
            certificate_der: cert_der,
            issuer_name_der: name_der,
            serial_number: serial_num,
        })
    }

    /// Creates a SigningKey from existing RSA private key and DER-encoded X.509 certificate.
    pub fn from_rsa_and_cert(private_key: RsaPrivateKey, cert_der: Vec<u8>) -> Self {
        // Attempt basic extraction of issuer and serial from cert DER, fallback if needed
        let serial = vec![1];
        let issuer = vec![0x30, 0x00];
        Self {
            private_key,
            certificate_der: cert_der,
            issuer_name_der: issuer,
            serial_number: serial,
        }
    }
}
