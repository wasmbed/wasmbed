use core::panic;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use crate::pool::ThreadPool;
use wasmbed_protocol::types::Envelope;
use rustls::client::danger::HandshakeSignatureValid;
use rustls::server::AlwaysResolvesServerRawPublicKeys;
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::sign::CertifiedKey;
use rustls::version::TLS13;
use rustls::ServerConnection;
use rustls::{
    CertificateError, DigitallySignedStruct, DistinguishedName, Error,
    InconsistentKeys, PeerIncompatible, ServerConfig, SignatureScheme,
};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{
    CertificateDer, PrivateKeyDer, SubjectPublicKeyInfoDer, UnixTime,
};
use rustls::crypto::{
    WebPkiSupportedAlgorithms, aws_lc_rs, verify_tls13_signature_with_raw_key,
};

const NUM_WORKERS: usize = 16;

pub fn make_config(client_pub_key: &str, server_prv_key: &str) -> ServerConfig {
    let client_key_raw = SubjectPublicKeyInfoDer::from_pem_file(client_pub_key)
        .expect("cannot open pub key file");
    let server_private_key = aws_lc_rs::default_provider()
        .key_provider
        .load_private_key(
            PrivateKeyDer::from_pem_file(server_prv_key)
                .expect("cannot open private key file"),
        )
        .expect("cannot load signing key");

    let server_public_key = server_private_key
        .public_key()
        .ok_or(Error::InconsistentKeys(InconsistentKeys::Unknown))
        .expect("cannot load public key");
    let server_public_key_as_cert =
        CertificateDer::from(server_public_key.to_vec());

    let certified_key = Arc::new(CertifiedKey::new(
        vec![server_public_key_as_cert],
        server_private_key,
    ));
    let client_cert_verifier =
        Arc::new(SimpleRpkClientCertVerifier::new(vec![client_key_raw]));
    let server_cert_resolver =
        Arc::new(AlwaysResolvesServerRawPublicKeys::new(certified_key));
    ServerConfig::builder_with_protocol_versions(&[&TLS13])
        .with_client_cert_verifier(client_cert_verifier)
        .with_cert_resolver(server_cert_resolver)
}

#[derive(Debug)]
struct SimpleRpkClientCertVerifier {
    trusted_spki: Vec<SubjectPublicKeyInfoDer<'static>>,
    supported_algs: WebPkiSupportedAlgorithms,
}

impl SimpleRpkClientCertVerifier {
    pub fn new(trusted_spki: Vec<SubjectPublicKeyInfoDer<'static>>) -> Self {
        Self {
            trusted_spki,
            supported_algs: Arc::new(aws_lc_rs::default_provider())
                .clone()
                .signature_verification_algorithms,
        }
    }
}

impl ClientCertVerifier for SimpleRpkClientCertVerifier {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        let end_entity_as_spki =
            SubjectPublicKeyInfoDer::from(end_entity.as_ref());
        match self.trusted_spki.contains(&end_entity_as_spki) {
            false => Err(rustls::Error::InvalidCertificate(
                CertificateError::UnknownIssuer,
            )),
            true => Ok(ClientCertVerified::assertion()),
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

pub struct Server {
    address: String,
    port: u16,
    stopped: Arc<AtomicBool>,
    config: Arc<ServerConfig>,
    pool: ThreadPool,
}

impl Server {
    pub fn new(address: String, port: u16) -> Self {
        Server {
            address,
            port,
            stopped: Arc::new(AtomicBool::new(false)),
            config: Arc::new(make_config(
                "to_be_added".as_ref(),
                "to_be_added".as_ref(),
            )),
            pool: ThreadPool::new(NUM_WORKERS),
        }
    }

    pub fn listen(&self) {
        let listener =
            TcpListener::bind((self.address.clone(), self.port)).unwrap();

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(mut stream) => {
                    let config = self.config.clone();
                    // Handle the message:
                    // Auth using TLS (decrypt the message)
                    // Read the CBOR message from the client
                    self.pool.execute(move || {
                        let mut conn =
                            ServerConnection::new(config.clone()).unwrap();
                        conn.complete_io(&mut stream).unwrap();

                        let mut buf: Vec<u8> = Vec::new();
                        if stream.read_to_end(&mut buf).is_ok() {}
                        match minicbor::decode::<Envelope>(buf.as_slice()) {
                            Result::Ok(envelope) => {
                                println!("Envelope message: {:?}", envelope);
                            },
                            Err(_) => (),
                        }
                    });
                },
                Err(_) => (),
            }
        }
    }

    pub fn run(&mut self) {
        let server_state = Arc::clone(&self.stopped);
        match ctrlc::try_set_handler(move || {
            server_state.store(true, Ordering::SeqCst);
        }) {
            Ok(_) => {},
            Err(ctrlc::Error::MultipleHandlers) => {},
            Err(e) => panic!("Error setting Ctrl-C handler: {}", e),
        }
        self.listen();
        loop {}
    }

    pub fn stop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
}
