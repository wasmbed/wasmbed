#[cfg(feature = "serde")]
mod serde;

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PublicKey<'a>(
    #[cfg_attr(feature = "serde", serde(with = "crate::serde::base64_bytes"))]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub Cow<'a, [u8]>,
);
