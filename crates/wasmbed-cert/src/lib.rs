#![no_std]

extern crate alloc;

use alloc::{vec, vec::Vec};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, Error,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, PKCS_ED25519,
};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

pub use rcgen::{DistinguishedName, DnType};

/// Core cryptographic credential containing a key pair and certificate.
struct Credential {
    key_pair: KeyPair,
    certificate: Certificate,
}

/// Certificate Authority capable of issuing certificates.
struct Authority(Credential);

/// End-entity certificate for client or server authentication.
struct Identity(Credential);

/// Certificate Authority for issuing server certificates.
pub struct ServerAuthority(Authority);

/// Certificate Authority for issuing client certificates.
pub struct ClientAuthority(Authority);

/// Server certificate with ServerAuth extended key usage.
pub struct ServerIdentity(Identity);

/// Client certificate with ClientAuth extended key usage.
pub struct ClientIdentity(Identity);

impl Credential {
    /// Creates a self-signed certificate with the given parameters.
    fn self_signed(params: CertificateParams) -> Result<Self, Error> {
        let key_pair = KeyPair::generate_for(&PKCS_ED25519)?;
        let certificate = params.self_signed(&key_pair)?;
        Ok(Self {
            key_pair,
            certificate,
        })
    }

    /// Creates a certificate signed by this credential.
    fn signed(&self, params: CertificateParams) -> Result<Self, Error> {
        let key_pair = KeyPair::generate_for(&PKCS_ED25519)?;
        let certificate =
            params.signed_by(&key_pair, &self.certificate, &self.key_pair)?;

        Ok(Self {
            key_pair,
            certificate,
        })
    }

    fn key_pair_der(&self) -> &[u8] {
        self.key_pair.serialized_der()
    }

    fn certificate_der(&self) -> &[u8] {
        self.certificate.der()
    }

    /// Reconstructs a credential from DER-encoded private key and certificate.
    ///
    /// Note: This recreates the certificate from parsed parameters, which may
    /// not preserve all original certificate fields exactly. See
    /// [`from_ca_cert_der`](rcgen::CertificateParams::from_ca_cert_der).
    fn from_der(
        private_key_der: &[u8],
        certificate_der: &[u8],
    ) -> Result<Self, Error> {
        let private_key = PrivateKeyDer::Pkcs8(private_key_der.into());
        let key_pair =
            KeyPair::from_der_and_sign_algo(&private_key, &PKCS_ED25519)?;
        let certificate_der = CertificateDer::from_slice(certificate_der);
        let certificate_params =
            CertificateParams::from_ca_cert_der(&certificate_der)?;
        let certificate = certificate_params.self_signed(&key_pair)?;
        Ok(Self {
            key_pair,
            certificate,
        })
    }
}

impl Authority {
    /// Creates a new Certificate Authority with the given distinguished name.
    fn new(distinguished_name: DistinguishedName) -> Result<Self, Error> {
        let mut params = CertificateParams::default();
        params.distinguished_name = distinguished_name;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
        ];

        Ok(Self(Credential::self_signed(params)?))
    }

    /// Issues a certificate signed by this authority.
    fn issue_certificate(
        &self,
        distinguished_name: DistinguishedName,
        key_usages: Vec<KeyUsagePurpose>,
        extended_key_usages: Vec<ExtendedKeyUsagePurpose>,
    ) -> Result<Identity, Error> {
        let mut params = CertificateParams::default();
        params.distinguished_name = distinguished_name;
        params.key_usages = key_usages;
        params.extended_key_usages = extended_key_usages;

        Ok(Identity(self.0.signed(params)?))
    }

    fn key_pair_der(&self) -> &[u8] {
        self.0.key_pair_der()
    }

    fn certificate_der(&self) -> &[u8] {
        self.0.certificate_der()
    }

    /// Reconstructs an authority from DER-encoded materials.
    fn from_der(
        private_key_der: &[u8],
        certificate_der: &[u8],
    ) -> Result<Self, Error> {
        Ok(Self(Credential::from_der(
            private_key_der,
            certificate_der,
        )?))
    }
}

impl ServerAuthority {
    /// Creates a new Server Certificate Authority.
    pub fn new(distinguished_name: DistinguishedName) -> Result<Self, Error> {
        Ok(ServerAuthority(Authority::new(distinguished_name)?))
    }

    /// Issues a server certificate with ServerAuth extended key usage.
    pub fn issue_certificate(
        &self,
        distinguished_name: DistinguishedName,
    ) -> Result<ServerIdentity, Error> {
        Ok(ServerIdentity(self.0.issue_certificate(
            distinguished_name,
            vec![
                KeyUsagePurpose::DigitalSignature,
                KeyUsagePurpose::KeyEncipherment,
            ],
            vec![ExtendedKeyUsagePurpose::ServerAuth],
        )?))
    }

    pub fn key_pair_der(&self) -> &[u8] {
        self.0.key_pair_der()
    }

    pub fn certificate_der(&self) -> &[u8] {
        self.0.certificate_der()
    }

    /// Reconstructs a server authority from DER-encoded materials.
    pub fn from_der(
        private_key_der: &[u8],
        certificate_der: &[u8],
    ) -> Result<Self, Error> {
        Ok(Self(Authority::from_der(private_key_der, certificate_der)?))
    }
}

impl ClientAuthority {
    /// Creates a new Client Certificate Authority.
    pub fn new(distinguished_name: DistinguishedName) -> Result<Self, Error> {
        Ok(ClientAuthority(Authority::new(distinguished_name)?))
    }

    /// Issues a client certificate with ClientAuth extended key usage.
    pub fn issue_certificate(
        &self,
        distinguished_name: DistinguishedName,
    ) -> Result<ClientIdentity, Error> {
        Ok(ClientIdentity(self.0.issue_certificate(
            distinguished_name,
            vec![KeyUsagePurpose::DigitalSignature],
            vec![ExtendedKeyUsagePurpose::ClientAuth],
        )?))
    }

    pub fn key_pair_der(&self) -> &[u8] {
        self.0.key_pair_der()
    }

    pub fn certificate_der(&self) -> &[u8] {
        self.0.certificate_der()
    }

    /// Reconstructs a client authority from DER-encoded materials.
    pub fn from_der(
        private_key_der: &[u8],
        certificate_der: &[u8],
    ) -> Result<Self, Error> {
        Ok(Self(Authority::from_der(private_key_der, certificate_der)?))
    }
}

impl Identity {
    fn key_pair_der(&self) -> &[u8] {
        self.0.key_pair_der()
    }

    fn certificate_der(&self) -> &[u8] {
        self.0.certificate_der()
    }
}

impl ServerIdentity {
    pub fn key_pair_der(&self) -> &[u8] {
        self.0.key_pair_der()
    }

    pub fn certificate_der(&self) -> &[u8] {
        self.0.certificate_der()
    }
}

impl ClientIdentity {
    pub fn key_pair_der(&self) -> &[u8] {
        self.0.key_pair_der()
    }

    pub fn certificate_der(&self) -> &[u8] {
        self.0.certificate_der()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::DnType;

    fn create_server_ca_dn() -> DistinguishedName {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Test Server CA");
        dn.push(DnType::OrganizationName, "Test Org");
        dn
    }

    fn create_client_ca_dn() -> DistinguishedName {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Test Client CA");
        dn.push(DnType::OrganizationName, "Test Org");
        dn
    }

    fn create_server_dn() -> DistinguishedName {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "test.example.com");
        dn
    }

    fn create_client_dn() -> DistinguishedName {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "client-001");
        dn
    }

    #[test]
    fn test_server_authority_creation() {
        let ca = ServerAuthority::new(create_server_ca_dn()).unwrap();
        assert!(!ca.key_pair_der().is_empty());
        assert!(!ca.certificate_der().is_empty());
    }

    #[test]
    fn test_client_authority_creation() {
        let ca = ClientAuthority::new(create_client_ca_dn()).unwrap();
        assert!(!ca.key_pair_der().is_empty());
        assert!(!ca.certificate_der().is_empty());
    }

    #[test]
    fn test_server_certificate_issuance() {
        let ca = ServerAuthority::new(create_server_ca_dn()).unwrap();
        let server_cert = ca.issue_certificate(create_server_dn()).unwrap();

        assert!(!server_cert.key_pair_der().is_empty());
        assert!(!server_cert.certificate_der().is_empty());
    }

    #[test]
    fn test_client_certificate_issuance() {
        let ca = ClientAuthority::new(create_client_ca_dn()).unwrap();
        let client_cert = ca.issue_certificate(create_client_dn()).unwrap();

        assert!(!client_cert.key_pair_der().is_empty());
        assert!(!client_cert.certificate_der().is_empty());
    }

    #[test]
    fn test_server_authority_serialization() {
        let original_ca = ServerAuthority::new(create_server_ca_dn()).unwrap();
        let key_der = original_ca.key_pair_der();
        let cert_der = original_ca.certificate_der();

        let restored_ca = ServerAuthority::from_der(key_der, cert_der).unwrap();

        // Keys should be identical
        assert_eq!(original_ca.key_pair_der(), restored_ca.key_pair_der());

        // Certificates may differ due to reconstruction, but should be valid
        assert!(!restored_ca.certificate_der().is_empty());
    }

    #[test]
    fn test_client_authority_serialization() {
        let original_ca = ClientAuthority::new(create_client_ca_dn()).unwrap();
        let key_der = original_ca.key_pair_der();
        let cert_der = original_ca.certificate_der();

        let restored_ca = ClientAuthority::from_der(key_der, cert_der).unwrap();

        assert_eq!(original_ca.key_pair_der(), restored_ca.key_pair_der());
        assert!(!restored_ca.certificate_der().is_empty());
    }

    #[test]
    fn test_serialization_roundtrip_authority_functionality() {
        let original_ca = ServerAuthority::new(create_server_ca_dn()).unwrap();
        let restored_ca = ServerAuthority::from_der(
            original_ca.key_pair_der(),
            original_ca.certificate_der(),
        )
        .unwrap();

        // Both should be able to issue certificates
        let cert1 = original_ca.issue_certificate(create_server_dn()).unwrap();
        let cert2 = restored_ca.issue_certificate(create_server_dn()).unwrap();

        assert!(!cert1.certificate_der().is_empty());
        assert!(!cert2.certificate_der().is_empty());
    }

    #[test]
    fn test_cross_authority_type_safety() {
        let server_ca = ServerAuthority::new(create_server_ca_dn()).unwrap();
        let client_ca = ClientAuthority::new(create_client_ca_dn()).unwrap();

        let server_cert =
            server_ca.issue_certificate(create_server_dn()).unwrap();
        let client_cert =
            client_ca.issue_certificate(create_client_dn()).unwrap();

        // Ensure different types produce different certificates
        assert_ne!(
            server_cert.certificate_der(),
            client_cert.certificate_der()
        );
    }

    #[test]
    fn test_invalid_der_handling() {
        let invalid_key = b"invalid key data";
        let invalid_cert = b"invalid cert data";

        assert!(ServerAuthority::from_der(invalid_key, invalid_cert).is_err());
        assert!(ClientAuthority::from_der(invalid_key, invalid_cert).is_err());
    }

    #[test]
    fn test_empty_der_handling() {
        let empty_data = b"";

        assert!(ServerAuthority::from_der(empty_data, empty_data).is_err());
        assert!(ClientAuthority::from_der(empty_data, empty_data).is_err());
    }
}
