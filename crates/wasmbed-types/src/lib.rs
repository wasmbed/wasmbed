#[cfg(feature = "serde")]
mod serde;

use std::borrow::Cow;

#[cfg(feature = "k8s")]
use k8s_openapi::api::core::v1::TypedObjectReference;

#[cfg(feature = "k8s")]
#[derive(
    Clone,
    Debug,
    PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize,
    schemars::JsonSchema,
)]
pub struct GatewayReference(pub TypedObjectReference);

#[cfg(feature = "k8s")]
impl GatewayReference {
    pub fn new(namespace: &str, name: &str) -> Self {
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
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PublicKey<'a>(
    #[cfg_attr(feature = "serde", serde(with = "crate::serde::base64_bytes"))]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub Cow<'a, [u8]>,
);

impl<'a> From<Cow<'a, [u8]>> for PublicKey<'a> {
    fn from(cow: Cow<'a, [u8]>) -> Self {
        PublicKey(cow)
    }
}

impl<'a> From<&'a [u8]> for PublicKey<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Cow::Borrowed(bytes).into()
    }
}

impl From<Vec<u8>> for PublicKey<'static> {
    fn from(bytes: Vec<u8>) -> Self {
        Cow::<'static, [u8]>::Owned(bytes).into()
    }
}
