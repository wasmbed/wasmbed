// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

#[cfg(all(feature = "schemars", feature = "alloc"))]
use alloc::borrow::ToOwned;

/// An Ed25519 public key encoded in DER format using the X.509
/// `SubjectPublicKeyInfo` structure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "base64", derive(derive_more::Display))]
#[cfg_attr(
    all(feature = "schemars", feature = "std"),
    derive(schemars::JsonSchema)
)]
#[cfg_attr(feature = "base64", display("PublicKey({})", self.to_base64()))]
pub struct PublicKey<'a>(
    #[cfg_attr(
        all(feature = "schemars", feature = "std"),
        schemars(with = "alloc::string::String")
    )]
    rustls_pki_types::SubjectPublicKeyInfoDer<'a>,
);

impl<'a> PublicKey<'a> {
    #[cfg(feature = "alloc")]
    pub fn into_owned(self) -> PublicKey<'static> {
        PublicKey(self.0.into_owned())
    }

    #[cfg(feature = "base64")]
    pub fn to_base64(&self) -> alloc::string::String {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(self.0.as_ref())
    }

    #[cfg(feature = "base64")]
    pub fn from_base64<T: AsRef<[u8]>>(
        s: T,
    ) -> Result<alloc::vec::Vec<u8>, base64::DecodeError> {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(s)
    }
}

impl<'a> core::hash::Hash for PublicKey<'a> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state);
    }
}

impl<'a> From<&'a [u8]> for PublicKey<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        PublicKey(bytes.into())
    }
}

#[cfg(feature = "alloc")]
impl From<alloc::vec::Vec<u8>> for PublicKey<'static> {
    fn from(bytes: alloc::vec::Vec<u8>) -> Self {
        PublicKey(bytes.into())
    }
}

impl<'a> From<rustls_pki_types::SubjectPublicKeyInfoDer<'a>> for PublicKey<'a> {
    fn from(spki: rustls_pki_types::SubjectPublicKeyInfoDer<'a>) -> Self {
        PublicKey(spki)
    }
}

#[cfg(feature = "x509")]
impl<'a> TryFrom<&'a rustls_pki_types::CertificateDer<'a>> for PublicKey<'a> {
    type Error = x509_cert::der::Error;

    fn try_from(
        cert: &'a rustls_pki_types::CertificateDer<'a>,
    ) -> Result<Self, Self::Error> {
        use x509_cert::Certificate;
        use x509_cert::der::{Decode, Encode};

        let certificate = Certificate::from_der(cert)?;
        let spki = certificate.tbs_certificate.subject_public_key_info;

        Ok(spki.to_der()?.into())
    }
}

#[cfg(all(feature = "serde", feature = "base64"))]
impl serde::Serialize for PublicKey<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_base64())
    }
}

#[cfg(all(feature = "serde", feature = "base64"))]
impl<'de> serde::Deserialize<'de> for PublicKey<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let decoded = Self::from_base64(s).map_err(D::Error::custom)?;
        Ok(decoded.into())
    }
}
