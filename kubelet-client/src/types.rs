use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub struct PayloadId(Uuid);

impl PayloadId {
    pub fn as_bytes(&self) -> &uuid::Bytes {
        self.0.as_bytes()
    }

    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        Uuid::from_slice(bytes).ok().map(PayloadId)
    }

    pub const fn from_bytes(bytes: uuid::Bytes) -> Self {
        PayloadId(Uuid::from_bytes(bytes))
    }
}
