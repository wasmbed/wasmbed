#[cfg(feature = "cert")]
pub(crate) mod spki_der {
    use rustls_pki_types::SubjectPublicKeyInfoDer;
    use serde::{Serializer, Deserializer, Deserialize};
    use serde::de::Error;
    use base64::{engine::general_purpose::STANDARD, Engine};

    pub fn serialize<S>(
        spki: &SubjectPublicKeyInfoDer,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = STANDARD.encode(spki.as_ref());
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<SubjectPublicKeyInfoDer<'static>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let decoded = STANDARD.decode(s).map_err(D::Error::custom)?;
        Ok(decoded.into())
    }
}
