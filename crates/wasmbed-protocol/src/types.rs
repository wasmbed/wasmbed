use alloc::vec::Vec;
use uuid::Uuid;

// Envelope --------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Envelope {
    pub version: Version,
    // body become type
    // inside type an enum Message Client or Server
    // this enum will be of type CreatePod... , same for the Server
    pub body: Message,
}

// Version ---------------------------------------------------------------------

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

// Message ---------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    ClientMessage(ClientMessage),
    ServerMessage(ServerMessage),
}

impl Message {
    pub fn kind(&self) -> MessageKind {
        match self {
            Message::ClientMessage(_) => MessageKind::ClientMessage,
            Message::ServerMessage(_) => MessageKind::ServerMessage,
        }
    }
}

// Client Message -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum ClientMessage {
    CreatePodResponse(CreatePodResponse),
    DeletePodResponse(DeletePodResponse),
    Heartbeat(Heartbeat),
}

impl ClientMessage {
    pub fn kind(&self) -> ClientMessageKind {
        match self {
            ClientMessage::CreatePodResponse(_) => {
                ClientMessageKind::CreatePodResponse
            },
            ClientMessage::DeletePodResponse(_) => {
                ClientMessageKind::DeletePodResponse
            },
            ClientMessage::Heartbeat(_) => ClientMessageKind::Heartbeat,
        }
    }
}

// Heartbeat -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Heartbeat {
    Heartbeat,
}

impl Heartbeat {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Heartbeat),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Heartbeat => 0,
        }
    }
}

// Server Messsage -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum ServerMessage {
    CreatePodRequest(CreatePodRequest),
    DeletePodRequest(DeletePodRequest),
    HeartbeatAcknowledge(HeartbeatAcknowledge),
}

impl ServerMessage {
    pub fn kind(&self) -> ServerMessageKind {
        match self {
            ServerMessage::CreatePodRequest(_) => {
                ServerMessageKind::CreatePodRequest
            },
            ServerMessage::DeletePodRequest(_) => {
                ServerMessageKind::DeletePodRequest
            },
            ServerMessage::HeartbeatAcknowledge(_) => {
                ServerMessageKind::HeartbeatAcknowledge
            },
        }
    }
}

// HeartbeatAcknowledge -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum HeartbeatAcknowledge {
    HeartbeatAcknowledge,
}

impl HeartbeatAcknowledge {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::HeartbeatAcknowledge),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::HeartbeatAcknowledge => 0,
        }
    }
}

// MessageKind -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum MessageKind {
    ClientMessage,
    ServerMessage,
}

impl MessageKind {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::ClientMessage),
            1 => Some(Self::ServerMessage),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::ClientMessage => 0,
            Self::ServerMessage => 1,
        }
    }
}

// ClientMessageKind -----------------------------------------------------------------

pub enum ClientMessageKind {
    CreatePodResponse,
    DeletePodResponse,
    Heartbeat,
}

impl ClientMessageKind {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::CreatePodResponse),
            1 => Some(Self::DeletePodResponse),
            2 => Some(Self::Heartbeat),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::CreatePodResponse => 0,
            Self::DeletePodResponse => 1,
            Self::Heartbeat => 2,
        }
    }
}

// ServerMessageKind -----------------------------------------------------------------

pub enum ServerMessageKind {
    CreatePodRequest,
    DeletePodRequest,
    HeartbeatAcknowledge,
}

impl ServerMessageKind {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::CreatePodRequest),
            1 => Some(Self::DeletePodRequest),
            2 => Some(Self::HeartbeatAcknowledge),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::CreatePodRequest => 0,
            Self::DeletePodRequest => 1,
            Self::HeartbeatAcknowledge => 2,
        }
    }
}

// PodId -----------------------------------------------------------------------

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

// WasmModule ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct WasmModule(Vec<u8>);

impl WasmModule {
    pub fn from_slice(bytes: &[u8]) -> Self {
        WasmModule(bytes.to_vec())
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

// Message: CreatePodRequest ---------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct CreatePodRequest {
    pub pod_id: PodId,
    pub wasm_module: WasmModule,
}

// Message: DeletePodRequest ---------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct DeletePodRequest {
    pub pod_id: PodId,
}

// Message: CreatePodResponse --------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct CreatePodResponse {
    pub pod_id: PodId,
    pub result: ClientPodResult,
}

// Message: DeletePodResponse ---------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct DeletePodResponse {
    pub pod_id: PodId,
    pub result: ClientPodResult,
}

// ClientCreatePodResult -------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum ClientPodResult {
    Success,
    Failure,
}

impl ClientPodResult {
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

// -------------------------------------------------------------
