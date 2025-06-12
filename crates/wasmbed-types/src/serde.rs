pub(crate) mod base64_bytes {
    use std::borrow::Cow;
    use serde::{Serializer, Deserialize, Deserializer};
    use serde::de::Error;
    use base64::{engine::general_purpose::STANDARD, Engine};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = STANDARD.encode(bytes.as_ref());
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Cow<'static, [u8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let decoded = STANDARD.decode(s).map_err(D::Error::custom)?;
        Ok(Cow::Owned(decoded))
    }
}
