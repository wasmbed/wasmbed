use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Version {
    V0,
}

impl Version {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::V0),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::V0 => 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Envelope<T> {
    pub version: Version,
    pub body: T,
}

#[derive(Debug, PartialEq)]
pub enum MessageKind {
    CreatePodRequest,
    CreatePodResponse,
}

impl MessageKind {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::CreatePodRequest),
            1 => Some(Self::CreatePodResponse),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::CreatePodRequest  => 0,
            Self::CreatePodResponse => 1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Message<const WASM_MODULE_SIZE: usize> {
    CreatePodRequest(CreatePodRequest<WASM_MODULE_SIZE>),
    CreatePodResponse(CreatePodResponse),
}

impl<const WASM_MESSAGE_SIZE: usize> Message<WASM_MESSAGE_SIZE> {
    pub fn kind(&self) -> MessageKind {
        match self {
            Message::CreatePodRequest(_)  => MessageKind::CreatePodRequest,
            Message::CreatePodResponse(_) => MessageKind::CreatePodResponse,
        }
    }
}

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
pub enum CreatePodResult {
    Success,
    Failure,
}

impl CreatePodResult {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Success),
            1 => Some(Self::Failure),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CreatePodResponse {
    pub pod_id: PodId,
    pub result: CreatePodResult,
}
