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

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PublicKey<'a>(
    #[cfg_attr(feature = "serde", serde(with = "crate::serde::base64_bytes"))]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub Cow<'a, [u8]>,
);
