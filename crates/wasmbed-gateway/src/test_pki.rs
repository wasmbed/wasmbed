use std::sync::Arc;
use rustls::RootCertStore;
use rcgen::KeyPair;
use rustls::server::{ServerConfig, WebPkiClientVerifier};
use rustls::pki_types::PrivatePkcs8KeyDer;

pub struct TestPki {
    pub roots: Arc<RootCertStore>,
    pub ca_cert: rcgen::CertifiedKey,
    pub client_cert: rcgen::CertifiedKey,
    pub server_cert: rcgen::CertifiedKey,
}

impl TestPki {
    /// Create a new test PKI using `rcgen`.
    pub fn new() -> Self {
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
            rcgen::CertificateParams::new(vec!["127.0.0.1".to_string()])
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

    pub fn server_config(&self) -> Arc<ServerConfig> {
        // Read the latest CRL from disk. The CRL is being periodically updated by the crl_updater
        // thread.

        // Construct a fresh verifier using the test PKI roots, and the updated CRL.
        let verifier = WebPkiClientVerifier::builder(self.roots.clone())
            .build()
            .unwrap();

        // Build a server config using the fresh verifier. If necessary, this could be customized
        // based on the ClientHello (e.g. selecting a different certificate, or customizing
        // supported algorithms/protocol versions).
        let server_config = ServerConfig::builder()
            .with_client_cert_verifier(verifier)
            .with_single_cert(
                vec![self.server_cert.cert.der().clone()],
                PrivatePkcs8KeyDer::from(
                    self.server_cert.key_pair.serialize_der(),
                )
                .into(),
            )
            .unwrap();

        // Allow using SSLKEYLOGFILE.
        // server_config.key_log = Arc::new(rustls::KeyLogFile::new());

        Arc::new(server_config)
    }
}
