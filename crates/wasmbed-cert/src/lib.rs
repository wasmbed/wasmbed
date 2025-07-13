// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

#![no_std]

extern crate alloc;

use alloc::{vec, vec::Vec};
use rcgen::{
    BasicConstraints, CertificateParams, Error, ExtendedKeyUsagePurpose, IsCa,
    KeyPair, KeyUsagePurpose, PKCS_ED25519,
};
use rustls_pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use wasmbed_types::PublicKey;

pub use rcgen::{DistinguishedName, DnType};

/// Core cryptographic credential containing a private key and certificate.
struct Credential {
    private_key: PrivatePkcs8KeyDer<'static>,
    certificate: CertificateDer<'static>,
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
            private_key: key_pair.serialize_der().into(),
            certificate: certificate.der().clone(),
        })
    }

    /// Creates a certificate signed by the provided authority credential.
    fn signed(
        authority: &Self,
        params: CertificateParams,
    ) -> Result<Self, Error> {
        let key_pair = KeyPair::generate_for(&PKCS_ED25519)?;
        let authority_key_pair = KeyPair::from_pkcs8_der_and_sign_algo(
            &authority.private_key,
            &PKCS_ED25519,
        )?;
        let authority_cert_params =
            CertificateParams::from_ca_cert_der(&authority.certificate)?;
        let authority_cert =
            authority_cert_params.self_signed(&authority_key_pair)?;

        let certificate = params.signed_by(
            &key_pair,
            &authority_cert,
            &authority_key_pair,
        )?;

        Ok(Self {
            private_key: key_pair.serialize_der().into(),
            certificate: certificate.der().clone(),
        })
    }

    /// The private key in PKCS#8 format.
    fn private_key(&self) -> &PrivatePkcs8KeyDer<'static> {
        &self.private_key
    }

    /// The public key in X.509 SubjectPublicKeyInfo format.
    fn public_key(&self) -> Result<PublicKey<'static>, Error> {
        let key_pair = KeyPair::from_pkcs8_der_and_sign_algo(
            &self.private_key,
            &PKCS_ED25519,
        )?;
        Ok(key_pair.public_key_der().into())
    }

    /// The X.509 certificate.
    fn certificate(&self) -> &CertificateDer<'static> {
        &self.certificate
    }

    /// Reconstructs a credential from private key and certificate.
    fn from_parts(
        private_key: PrivatePkcs8KeyDer<'static>,
        certificate: CertificateDer<'static>,
    ) -> Self {
        Self {
            private_key,
            certificate,
        }
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

        Ok(Identity(Credential::signed(&self.0, params)?))
    }

    /// The private key in PKCS#8 format.
    fn private_key(&self) -> &PrivatePkcs8KeyDer<'static> {
        self.0.private_key()
    }

    /// The public key in X.509 SubjectPublicKeyInfo format.
    fn public_key(&self) -> Result<PublicKey<'static>, Error> {
        self.0.public_key()
    }

    /// The X.509 certificate.
    fn certificate(&self) -> &CertificateDer<'static> {
        self.0.certificate()
    }

    /// Reconstructs an authority from private key and certificate.
    fn from_parts(
        private_key: PrivatePkcs8KeyDer<'static>,
        certificate: CertificateDer<'static>,
    ) -> Self {
        Self(Credential::from_parts(private_key, certificate))
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

    /// The private key in PKCS#8 format.
    pub fn private_key(&self) -> &PrivatePkcs8KeyDer<'static> {
        self.0.private_key()
    }

    /// The public key in X.509 SubjectPublicKeyInfo format.
    pub fn public_key(&self) -> Result<PublicKey<'static>, Error> {
        self.0.public_key()
    }

    /// The X.509 certificate.
    pub fn certificate(&self) -> &CertificateDer<'static> {
        self.0.certificate()
    }

    /// Reconstructs a server authority from private key and certificate.
    pub fn from_parts(
        private_key: PrivatePkcs8KeyDer<'static>,
        certificate: CertificateDer<'static>,
    ) -> Self {
        Self(Authority::from_parts(private_key, certificate))
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

    /// The private key in PKCS#8 format.
    pub fn private_key(&self) -> &PrivatePkcs8KeyDer<'static> {
        self.0.private_key()
    }

    /// The public key in X.509 SubjectPublicKeyInfo format.
    pub fn public_key(&self) -> Result<PublicKey<'static>, Error> {
        self.0.public_key()
    }

    /// The X.509 certificate.
    pub fn certificate(&self) -> &CertificateDer<'static> {
        self.0.certificate()
    }

    /// Reconstructs a client authority from private key and certificate.
    pub fn from_parts(
        private_key: PrivatePkcs8KeyDer<'static>,
        certificate: CertificateDer<'static>,
    ) -> Self {
        Self(Authority::from_parts(private_key, certificate))
    }
}

impl Identity {
    /// The private key in PKCS#8 format.
    fn private_key(&self) -> &PrivatePkcs8KeyDer<'static> {
        self.0.private_key()
    }

    /// The public key in X.509 SubjectPublicKeyInfo format.
    fn public_key(&self) -> Result<PublicKey<'static>, Error> {
        self.0.public_key()
    }

    /// The X.509 certificate.
    fn certificate(&self) -> &CertificateDer<'static> {
        self.0.certificate()
    }

    /// Reconstructs an identity from private key and certificate.
    pub fn from_parts(
        private_key: PrivatePkcs8KeyDer<'static>,
        certificate: CertificateDer<'static>,
    ) -> Self {
        Self(Credential::from_parts(private_key, certificate))
    }
}

impl ServerIdentity {
    /// The private key in PKCS#8 format.
    pub fn private_key(&self) -> &PrivatePkcs8KeyDer<'static> {
        self.0.private_key()
    }

    /// The public key in X.509 SubjectPublicKeyInfo format.
    pub fn public_key(&self) -> Result<PublicKey<'static>, Error> {
        self.0.public_key()
    }

    /// The X.509 certificate.
    pub fn certificate(&self) -> &CertificateDer<'static> {
        self.0.certificate()
    }

    /// Reconstructs a server identity from private key and certificate.
    pub fn from_parts(
        private_key: PrivatePkcs8KeyDer<'static>,
        certificate: CertificateDer<'static>,
    ) -> Self {
        Self(Identity::from_parts(private_key, certificate))
    }
}

impl ClientIdentity {
    /// The private key in PKCS#8 format.
    pub fn private_key(&self) -> &PrivatePkcs8KeyDer<'static> {
        self.0.private_key()
    }

    /// The public key in X.509 SubjectPublicKeyInfo format.
    pub fn public_key(&self) -> Result<PublicKey<'static>, Error> {
        self.0.public_key()
    }

    /// The X.509 certificate.
    pub fn certificate(&self) -> &CertificateDer<'static> {
        self.0.certificate()
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

    #[test]
    fn test_server_authority_serialization() {
        let original_ca = ServerAuthority::new(create_server_ca_dn()).unwrap();

        let restored_ca = ServerAuthority::from_parts(
            original_ca.private_key().clone_key(),
            original_ca.certificate().clone(),
        );

        // Keys should be identical
        assert_eq!(original_ca.private_key(), restored_ca.private_key());
        assert_eq!(original_ca.public_key(), restored_ca.public_key());
    }

    #[test]
    fn test_client_authority_serialization() {
        let original_ca = ClientAuthority::new(create_client_ca_dn()).unwrap();

        let restored_ca = ClientAuthority::from_parts(
            original_ca.private_key().clone_key(),
            original_ca.certificate().clone(),
        );

        assert_eq!(original_ca.private_key(), restored_ca.private_key());
        assert_eq!(original_ca.public_key(), restored_ca.public_key());
    }
}
