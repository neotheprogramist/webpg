use anyhow::Result;
use chrono::Datelike;
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use salvo::conn::rustls::{Keycert, RustlsConfig};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;

pub struct CertificateMaterial {
    cert_pem: String,
    key_pem: String,
    pub fingerprint_hex: String,
}

impl CertificateMaterial {
    pub fn load_or_generate(data_dir: &Path) -> Result<Self> {
        let cert_path = data_dir.join("cert.pem");
        let key_path = data_dir.join("key.pem");

        if cert_path.exists() && key_path.exists() {
            let cert_pem = std::fs::read_to_string(&cert_path)?;
            let key_pem = std::fs::read_to_string(&key_path)?;
            let cert_der = pem::parse(&cert_pem)?.into_contents();
            let fingerprint_hex = fingerprint_sha256_hex(&cert_der);
            return Ok(Self {
                cert_pem,
                key_pem,
                fingerprint_hex,
            });
        }

        let material = Self::generate()?;
        std::fs::write(&cert_path, &material.cert_pem)?;
        std::fs::write(&key_path, &material.key_pem)?;
        Ok(material)
    }

    pub fn generate() -> Result<Self> {
        let key = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)?;
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "localhost");
        params.distinguished_name = dn;
        params.subject_alt_names = vec![
            SanType::DnsName("localhost".try_into()?),
            SanType::IpAddress(Ipv4Addr::LOCALHOST.into()),
            SanType::IpAddress(Ipv6Addr::LOCALHOST.into()),
        ];

        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::days(14);
        params.not_before = rcgen::date_time_ymd(now.year(), now.month() as u8, now.day() as u8);
        params.not_after = rcgen::date_time_ymd(exp.year(), exp.month() as u8, exp.day() as u8);

        let cert: Certificate = params.self_signed(&key)?;
        let cert_der = cert.der().to_vec();
        let cert_pem = pem::encode(&pem::Pem::new("CERTIFICATE", cert_der.clone()));
        let key_pem = pem::encode(&pem::Pem::new("PRIVATE KEY", key.serialize_der()));

        Ok(Self {
            cert_pem,
            key_pem,
            fingerprint_hex: fingerprint_sha256_hex(&cert_der),
        })
    }

    pub fn salvo_rustls_config(&self) -> RustlsConfig {
        RustlsConfig::new(
            Keycert::new()
                .cert(self.cert_pem.as_bytes())
                .key(self.key_pem.as_bytes()),
        )
    }
}

fn fingerprint_sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
