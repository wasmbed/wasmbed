use alloc::vec::Vec;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct Envelope<T> {
    pub version: Version,
    pub body: T,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    CreatePodRequest(CreatePodRequest),
    CreatePodResponse(CreatePodResponse),
}

impl Message {
    pub fn kind(&self) -> MessageKind {
        match self {
            Message::CreatePodRequest(_)  => MessageKind::CreatePodRequest,
            Message::CreatePodResponse(_) => MessageKind::CreatePodResponse,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct WasmModule(Vec<u8>);

impl WasmModule {
    pub fn from_slice(bytes: &[u8]) -> Self {
        WasmModule(bytes.to_vec())
    }

    pub fn as_slice(&self) -> &[u8]{
        self.0.as_slice()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreatePodRequest {
    pub pod_id: PodId,
    pub wasm_module: WasmModule,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct CreatePodResponse {
    pub pod_id: PodId,
    pub result: CreatePodResult,
}
