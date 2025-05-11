use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use rcgen::KeyPair;
use rustls::CertificateError;
use rustls::PeerIncompatible;
use rustls::DigitallySignedStruct;
use rustls::SignatureScheme;
use rustls::crypto::verify_tls13_signature_with_raw_key;
use rustls::RootCertStore;
use rustls::client::AlwaysResolvesClientRawPublicKeys;
use rustls::crypto::{WebPkiSupportedAlgorithms, aws_lc_rs as provider};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{
    UnixTime, ServerName, CertificateDer, PrivateKeyDer,
    SubjectPublicKeyInfoDer,
};
use rustls::sign::CertifiedKey;
use rustls::version::TLS13;
use rustls::{ClientConfig, Error, InconsistentKeys};
use rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};

pub struct TestPki {
    roots: Arc<RootCertStore>,
    ca_cert: rcgen::CertifiedKey,
    client_cert: rcgen::CertifiedKey,
    server_cert: rcgen::CertifiedKey,
}

impl TestPki {
    /// Create a new test PKI using `rcgen`.
    fn new() -> Self {
        // Create an issuer CA cert.
        let alg = &rcgen::PKCS_ECDSA_P256_SHA256;
        let mut ca_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        ca_params
            .distinguished_name
            .push(rcgen::DnType::OrganizationName, "Rustls Server Acceptor");
        ca_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "Example CA");
        ca_params.is_ca =
            rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::CrlSign,
        ];
        let ca_key = KeyPair::generate_for(alg).unwrap();
        let ca_cert = ca_params.self_signed(&ca_key).unwrap();

        // Create a server end entity cert issued by the CA.
        let mut server_ee_params =
            rcgen::CertificateParams::new(vec!["localhost".to_string()])
                .unwrap();
        server_ee_params.is_ca = rcgen::IsCa::NoCa;
        server_ee_params.extended_key_usages =
            vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        let ee_key = KeyPair::generate_for(alg).unwrap();
        let server_cert = server_ee_params
            .signed_by(&ee_key, &ca_cert, &ca_key)
            .unwrap();

        // Create a client end entity cert issued by the CA.
        let mut client_ee_params =
            rcgen::CertificateParams::new(Vec::new()).unwrap();
        client_ee_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "Example Client");
        client_ee_params.is_ca = rcgen::IsCa::NoCa;
        client_ee_params.extended_key_usages =
            vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        client_ee_params.serial_number =
            Some(rcgen::SerialNumber::from(vec![0xC0, 0xFF, 0xEE]));
        let client_key = KeyPair::generate_for(alg).unwrap();
        let client_cert = client_ee_params
            .signed_by(&client_key, &ca_cert, &ca_key)
            .unwrap();

        // Create a root cert store that includes the CA certificate.
        let mut roots = RootCertStore::empty();
        roots.add(ca_cert.der().clone()).unwrap();
        Self {
            roots: roots.into(),
            ca_cert: rcgen::CertifiedKey {
                cert: ca_cert,
                key_pair: ca_key,
            },
            client_cert: rcgen::CertifiedKey {
                cert: client_cert,
                key_pair: client_key,
            },
            server_cert: rcgen::CertifiedKey {
                cert: server_cert,
                key_pair: ee_key,
            },
        }
    }
}
#[derive(Debug)]
struct SimpleRpkServerCertVerifier {
    trusted_spki: Vec<SubjectPublicKeyInfoDer<'static>>,
    supported_algs: WebPkiSupportedAlgorithms,
}

impl SimpleRpkServerCertVerifier {
    fn new(trusted_spki: Vec<SubjectPublicKeyInfoDer<'static>>) -> Self {
        SimpleRpkServerCertVerifier {
            trusted_spki,
            supported_algs: Arc::new(provider::default_provider())
                .clone()
                .signature_verification_algorithms,
        }
    }
}
impl ServerCertVerifier for SimpleRpkServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let end_entity_as_spki =
            SubjectPublicKeyInfoDer::from(end_entity.as_ref());
        match self.trusted_spki.contains(&end_entity_as_spki) {
            false => Err(rustls::Error::InvalidCertificate(
                CertificateError::UnknownIssuer,
            )),
            true => Ok(ServerCertVerified::assertion()),
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Err(rustls::Error::PeerIncompatible(
            PeerIncompatible::Tls12NotOffered,
        ))
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature_with_raw_key(
            message,
            &SubjectPublicKeyInfoDer::from(cert.as_ref()),
            dss,
            &self.supported_algs,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.supported_algs.supported_schemes()
    }

    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

fn write_pem(path: &str, pem: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(pem.as_bytes()).unwrap();
}

fn make_client_config(
    client_private_key: &str,
    server_pub_key: &str,
) -> ClientConfig {
    let client_private_key = Arc::new(provider::default_provider())
        .key_provider
        .load_private_key(
            PrivateKeyDer::from_pem_file(client_private_key)
                .expect("cannot open private key file"),
        )
        .expect("cannot load signing key");
    let client_public_key = client_private_key
        .public_key()
        .ok_or(Error::InconsistentKeys(InconsistentKeys::Unknown))
        .expect("cannot load public key");
    let client_public_key_as_cert =
        CertificateDer::from(client_public_key.to_vec());

    let server_raw_key = SubjectPublicKeyInfoDer::from_pem_file(server_pub_key)
        .expect("cannot open pub key file");

    let certified_key = Arc::new(CertifiedKey::new(
        vec![client_public_key_as_cert],
        client_private_key,
    ));

    ClientConfig::builder_with_protocol_versions(&[&TLS13])
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(
            SimpleRpkServerCertVerifier::new(vec![server_raw_key]),
        ))
        .with_client_cert_resolver(Arc::new(
            AlwaysResolvesClientRawPublicKeys::new(certified_key),
        ))
}

#[cfg(test)]
mod certificate_gen {
    use std::fs;

    use super::*;

    const CA_PATH: &'static str = "./test-certs/ca/ca.pem";
    const SERVER_PATH: &'static str = "./test-certs/server/server.pem";
    const CLIENT_PATH: &'static str = "./test-certs/client/client.pem";

    #[test]
    fn generate_certs() {
        let test_pki = TestPki::new();
        write_pem(CA_PATH, &test_pki.ca_cert.cert.pem());
        write_pem(SERVER_PATH, &test_pki.client_cert.cert.pem());
        write_pem(CLIENT_PATH, &test_pki.client_cert.key_pair.serialize_pem());

        assert_eq!(true, fs::exists(CA_PATH).unwrap_or(false));
        assert_eq!(true, fs::exists(SERVER_PATH).unwrap_or(false));
        assert_eq!(true, fs::exists(CLIENT_PATH).unwrap_or(false));

        fs::remove_file(CA_PATH).unwrap();
        fs::remove_file(CLIENT_PATH).unwrap();
        fs::remove_file(SERVER_PATH).unwrap();

        assert_eq!(false, fs::exists(CA_PATH).unwrap_or(false));
        assert_eq!(false, fs::exists(SERVER_PATH).unwrap_or(false));
        assert_eq!(false, fs::exists(CLIENT_PATH).unwrap_or(false));
    }
}

#[cfg(test)]
mod client {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::Arc;
    use rustls::{ClientConnection, Stream};

    use crate::make_client_config;

    const SERVER_PORT: u32 = 6443;
    const SERVER_ADDRESS: &'static str = "127.0.0.1";
    const SERVER_PATH: &'static str = "./test-certs/server/server.pem";
    const CLIENT_PATH: &'static str = "./test-certs/client/client.pem";

    #[test]
    #[ignore = "Incomplete"]
    fn test_client() {
        let config = make_client_config(CLIENT_PATH, SERVER_PATH);
        let server_name = SERVER_ADDRESS.try_into().unwrap();
        let mut conn =
            ClientConnection::new(Arc::new(config), server_name).unwrap();
        let mut sock =
            TcpStream::connect(format!("[::]:{}", SERVER_PORT)).unwrap();
        let mut tls = Stream::new(&mut conn, &mut sock);

        let mut buf = vec![0; 128];
        let len = tls.read(&mut buf).unwrap();
        let received_message = String::from_utf8_lossy(&buf[..len]).to_string();

        let bytes_written = tls
            .write("Hello from the client".as_bytes())
            .unwrap_or("".len());
        assert!(bytes_written > 0);
    }
}
