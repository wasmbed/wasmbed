use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub struct PodId(Uuid);

impl PodId {
    pub const fn from_bytes(bytes: uuid::Bytes) -> Self {
        Self(Uuid::from_bytes(bytes))
    }

    pub const fn as_bytes(&self) -> &uuid::Bytes {
        self.0.as_bytes()
    }

    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        Uuid::from_slice(bytes).ok().map(Self)
    }
}

#[derive(Debug, PartialEq)]
pub struct WasmModule<const N: usize>([u8; N]);

impl<const N: usize> WasmModule<N> {
    pub const fn from_bytes(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }

    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        bytes.try_into().ok().map(Self)
    }
}

#[derive(Debug, PartialEq)]
pub struct CreatePodRequest<const WASM_MODULE_SIZE: usize> {
    pub pod_id: PodId,
    pub wasm_module: WasmModule<WASM_MODULE_SIZE>,
}

#[derive(Debug, PartialEq)]
pub struct CreatePodResponse {
    pub pod_id: PodId,
    pub result: CreatePodResult,
}

#[derive(Debug, PartialEq)]
pub enum CreatePodResult {
    Success,
    Failure,
}
