#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "schemars")]
use alloc::{string::String, borrow::ToOwned};

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "k8s")]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct GatewayReference(
    pub k8s_openapi::api::core::v1::TypedObjectReference,
);

#[cfg(all(feature = "k8s", feature = "alloc"))]
impl GatewayReference {
    pub fn new(namespace: &str, name: &str) -> Self {
        use alloc::string::ToString;
        use alloc::borrow::ToOwned;
        use k8s_openapi::api::core::v1::TypedObjectReference;

        Self(TypedObjectReference {
            api_group: None,
            kind: "Pod".to_string(),
            name: name.to_owned(),
            namespace: Some(namespace.to_owned()),
        })
    }
}

/// An Ed25519 public key encoded in DER format using the X.509
/// `SubjectPublicKeyInfo` structure.
#[cfg(feature = "cert")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PublicKey<'a>(
    #[cfg_attr(feature = "serde", serde(with = "crate::serde::spki_der"))]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    rustls_pki_types::SubjectPublicKeyInfoDer<'a>,
);

#[cfg(feature = "cert")]
impl<'a> PublicKey<'a> {
    pub fn into_owned(self) -> PublicKey<'static> {
        PublicKey(self.0.into_owned())
    }
}

#[cfg(feature = "cert")]
impl<'a> core::hash::Hash for PublicKey<'a> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state);
    }
}

#[cfg(feature = "cert")]
impl<'a> From<&'a [u8]> for PublicKey<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        PublicKey(bytes.into())
    }
}

#[cfg(all(feature = "cert", feature = "alloc"))]
impl From<alloc::vec::Vec<u8>> for PublicKey<'static> {
    fn from(bytes: alloc::vec::Vec<u8>) -> Self {
        PublicKey(bytes.into())
    }
}

#[cfg(feature = "cert")]
impl<'a> From<rustls_pki_types::SubjectPublicKeyInfoDer<'a>> for PublicKey<'a> {
    fn from(spki: rustls_pki_types::SubjectPublicKeyInfoDer<'a>) -> Self {
        PublicKey(spki)
    }
}

#[cfg(feature = "cert")]
impl<'a> TryFrom<&'a rustls_pki_types::CertificateDer<'a>> for PublicKey<'a> {
    type Error = x509_parser::error::X509Error;

    fn try_from(
        cert: &'a rustls_pki_types::CertificateDer<'a>,
    ) -> Result<Self, Self::Error> {
        use x509_parser::certificate::X509Certificate;
        use x509_parser::prelude::FromDer;

        let cert_bytes: &'a [u8] = cert.as_ref();
        let (_, x509): (_, X509Certificate<'a>) =
            X509Certificate::from_der(cert_bytes)?;

        let spki_raw = x509.tbs_certificate.subject_pki.raw;
        Ok(spki_raw.into())
    }
}
