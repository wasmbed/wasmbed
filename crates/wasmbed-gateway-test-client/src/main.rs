// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Error, Result};
use clap::Parser;
use tokio::net::TcpStream;
use rustls::{DigitallySignedStruct, RootCertStore};
use rustls::client::{ClientConfig, WebPkiServerVerifier};
use rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
use tokio_rustls::TlsConnector;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    address: SocketAddr,
    #[arg(long)]
    server_ca: PathBuf,
    #[arg(long)]
    private_key: PathBuf,
    #[arg(long)]
    certificate: PathBuf,
}

#[derive(Debug)]
pub struct NoServerNameVerification {
    inner: Arc<WebPkiServerVerifier>,
}

impl NoServerNameVerification {
    pub fn new(inner: Arc<WebPkiServerVerifier>) -> Self {
        Self { inner }
    }
}

impl ServerCertVerifier for NoServerNameVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        match self.inner.verify_server_cert(
            _end_entity,
            _intermediates,
            _server_name,
            _ocsp,
            _now,
        ) {
            Ok(scv) => Ok(scv),
            Err(rustls::Error::InvalidCertificate(cert_error)) => {
                match cert_error {
                    rustls::CertificateError::NotValidForName
                    | rustls::CertificateError::NotValidForNameContext {
                        ..
                    } => Ok(ServerCertVerified::assertion()),
                    _ => Err(rustls::Error::InvalidCertificate(cert_error)),
                }
            },
            Err(e) => Err(e),
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let server_ca_bytes =
        std::fs::read(&args.server_ca).with_context(|| {
            format!(
                "Failed to read server CA certificate from {}",
                args.server_ca.display()
            )
        })?;
    let private_key_bytes =
        std::fs::read(&args.private_key).with_context(|| {
            format!(
                "Failed to read private key from {}",
                args.private_key.display()
            )
        })?;
    let certificate_bytes =
        std::fs::read(&args.certificate).with_context(|| {
            format!(
                "Failed to read certificate from {}",
                args.certificate.display()
            )
        })?;

    let mut root_store = RootCertStore::empty();
    root_store
        .add(server_ca_bytes.into())
        .context("failed to add CA certificate to root store")?;

    let mut config = ClientConfig::builder()
        .with_root_certificates(root_store.clone())
        .with_client_auth_cert(
            vec![certificate_bytes.into()],
            private_key_bytes.try_into().map_err(Error::msg)?,
        )
        .context("invalid client cert or key")?;

    config.dangerous().set_certificate_verifier(Arc::new(
        NoServerNameVerification::new(
            WebPkiServerVerifier::builder(Arc::new(root_store.clone()))
                .build()?,
        ),
    ));

    let connector = TlsConnector::from(Arc::new(config));

    let stream = TcpStream::connect(args.address)
        .await
        .context("failed to connect")?;

    let _tls_stream = connector
        .connect("example.com".try_into()?, stream)
        .await
        .context("TLS handshake failed")?;

    println!("Successfully connected and verified TLS");

    Ok(())
}
