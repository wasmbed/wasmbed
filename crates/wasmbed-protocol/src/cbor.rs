use minicbor::encode::{Encode, Encoder, Error as EError, Write};
use minicbor::decode::{Decode, Decoder, Error as DError};
use crate::types::{
    ClientPodResult, ClientMessage, ClientMessageKind, CreatePodRequest,
    CreatePodResponse, DeletePodRequest, DeletePodResponse, Envelope,
    Heartbeat, HeartbeatAcknowledge, Message, MessageKind, PodId,
    ServerMessage, ServerMessageKind, Version, WasmModule,
};

const INVALID_VERSION_ERROR: &str = "Invalid version";
const INVALID_MESSAGE_KIND_ERROR: &str = "Invalid message kind";
const INVALID_UUID_LENGTH_ERROR: &str =
    "Failed to decode PodId: invalid UUID length";
const INDEFINITE_LENGTH_ARRAY_ERROR: &str =
    "Expected a fixed-size array but found an indefinite-length array";
const INVALID_ARRAY_LENGTH_ERROR: &str =
    "Failed to decode array: incorrect length";
const INVALID_CREATE_POD_RESULT_TAG_ERROR: &str =
    "Failed to decode CreatePodResult: unexpected tag";
const INVALID_HEARTBEAT_ERROR: &str = "Invalid heartbeat";
const INVALID_HEARTBEAT_ACKNOWLEDGE_ERROR: &str =
    "Invalid heartbeat acknowledge";
const INVALID_CLIENT_MESSAGE_KIND_ERROR: &str = "Invalid client message kind";
const INVALID_SERVER_MESSAGE_KIND_ERROR: &str = "Invalid server message kind";

// Envelope --------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for Envelope {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.array(2)?.encode(&self.version)?.encode(&self.body)?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for Envelope {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        let len = d
            .array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            version: d.decode()?,
            body: d.decode()?,
        })
    }
}

// Version ---------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for Version {
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>>
    where
        W: Write,
    {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for Version {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_u8(d.u8()?).ok_or(DError::message(INVALID_VERSION_ERROR))
    }
}

// Message ---------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for Message {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.array(2)?.encode(self.kind())?;
        match self {
            Self::ClientMessage(v) => e.encode(v)?,
            Self::ServerMessage(v) => e.encode(v)?,
        };
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for Message {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        let len = d
            .array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(match d.decode()? {
            MessageKind::ClientMessage => Self::ClientMessage(d.decode()?),
            MessageKind::ServerMessage => Self::ServerMessage(d.decode()?),
        })
    }
}

// MessageKind -----------------------------------------------------------------

impl<Ctx> Encode<Ctx> for MessageKind {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for MessageKind {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_u8(d.u8()?)
            .ok_or(DError::message(INVALID_MESSAGE_KIND_ERROR))
    }
}

// ClientMessage -----------------------------------------------------------------

impl<Ctx> Encode<Ctx> for ClientMessage {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.array(3)?.encode(self.kind())?;
        match self {
            Self::CreatePodResponse(v) => e.encode(v)?,
            Self::DeletePodResponse(v) => e.encode(v)?,
            Self::Heartbeat(v) => e.encode(v.as_u8())?,
        };
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for ClientMessage {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        let len = d
            .array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 3 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(match d.decode()? {
            ClientMessageKind::CreatePodResponse => {
                Self::CreatePodResponse(d.decode()?)
            },
            ClientMessageKind::DeletePodResponse => {
                Self::DeletePodResponse(d.decode()?)
            },
            ClientMessageKind::Heartbeat => Self::Heartbeat(d.decode()?),
        })
    }
}

// ClientMessageKind -----------------------------------------------------------------

impl<Ctx> Encode<Ctx> for ClientMessageKind {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for ClientMessageKind {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_u8(d.u8()?)
            .ok_or(DError::message(INVALID_CLIENT_MESSAGE_KIND_ERROR))
    }
}

// ServerMessage -----------------------------------------------------------------

impl<Ctx> Encode<Ctx> for ServerMessage {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.array(3)?.encode(self.kind())?;
        match self {
            Self::CreatePodRequest(v) => e.encode(v)?,
            Self::DeletePodRequest(v) => e.encode(v)?,
            Self::HeartbeatAcknowledge(v) => e.encode(v.as_u8())?,
        };
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for ServerMessage {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        let len = d
            .array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 3 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(match d.decode()? {
            ServerMessageKind::CreatePodRequest => {
                Self::CreatePodRequest(d.decode()?)
            },
            ServerMessageKind::DeletePodRequest => {
                Self::DeletePodRequest(d.decode()?)
            },
            ServerMessageKind::HeartbeatAcknowledge => {
                Self::HeartbeatAcknowledge(d.decode()?)
            },
        })
    }
}

// ServerMessageKind -----------------------------------------------------------------

impl<Ctx> Encode<Ctx> for ServerMessageKind {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for ServerMessageKind {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_u8(d.u8()?)
            .ok_or(DError::message(INVALID_SERVER_MESSAGE_KIND_ERROR))
    }
}

// PodId -----------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for PodId {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.bytes(self.as_bytes())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for PodId {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_slice(d.bytes()?)
            .ok_or(DError::message(INVALID_UUID_LENGTH_ERROR))
    }
}

// WasmModule ------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for WasmModule {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.bytes(self.as_slice())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for WasmModule {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Ok(Self::from_slice(d.bytes()?))
    }
}

// Heartbeat ---------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for Heartbeat {
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>>
    where
        W: Write,
    {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for Heartbeat {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_u8(d.u8()?).ok_or(DError::message(INVALID_HEARTBEAT_ERROR))
    }
}

// HeartbeatAcknowledge ---------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for HeartbeatAcknowledge {
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>>
    where
        W: Write,
    {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for HeartbeatAcknowledge {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_u8(d.u8()?)
            .ok_or(DError::message(INVALID_HEARTBEAT_ACKNOWLEDGE_ERROR))
    }
}
// Message: CreatePodRequest ---------------------------------------------------

impl<Ctx> Encode<Ctx> for CreatePodRequest {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.array(2)?
            .encode(&self.pod_id)?
            .encode(&self.wasm_module)?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for CreatePodRequest {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        let len = d
            .array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            pod_id: d.decode()?,
            wasm_module: d.decode()?,
        })
    }
}

// Message: DeletePodRequest ---------------------------------------------------

impl<Ctx> Encode<Ctx> for DeletePodRequest {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.array(1)?.encode(&self.pod_id)?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for DeletePodRequest {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        let len = d
            .array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 1 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            pod_id: d.decode()?,
        })
    }
}

// Message: CreatePodResponse --------------------------------------------------

impl<Ctx> Encode<Ctx> for CreatePodResponse {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.array(2)?.encode(&self.pod_id)?.encode(&self.result)?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for CreatePodResponse {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        let len = d
            .array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            pod_id: d.decode()?,
            result: d.decode()?,
        })
    }
}

// Message: DeletePodRequest ---------------------------------------------------

impl<Ctx> Encode<Ctx> for DeletePodResponse {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.array(2)?.encode(&self.pod_id)?.encode(&self.result)?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for DeletePodResponse {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        let len = d
            .array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            pod_id: d.decode()?,
            result: d.decode()?,
        })
    }
}

// CreatePodResult -------------------------------------------------------------

impl<Ctx> Encode<Ctx> for ClientPodResult {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for ClientPodResult {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_u8(d.u8()?)
            .ok_or(DError::message(INVALID_CREATE_POD_RESULT_TAG_ERROR))
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wasmbed_test_utils::minicbor::assert_encode_decode;

    const POD_ID: PodId = PodId::from_bytes([
        0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3, 0xd4,
        0xd5, 0xd6, 0xd7, 0xd8,
    ]);

    #[rustfmt::skip]
    const WASM_MODULE_BYTES: [u8; 24] = [
        0x00, 0x61, 0x73, 0x6D,             // Magic Header "\0asm"
        0x01, 0x00, 0x00, 0x00,             // Wasm Version (1)
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Type section (empty function)
        0x03, 0x02, 0x01, 0x00,             // Function section (one function)
        0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B, // Code section
    ];

    #[test]
    fn test_pod_id() {
        assert_encode_decode(&POD_ID);
    }

    #[test]
    fn test_wasm_module() {
        assert_encode_decode(&WasmModule::from_slice(&WASM_MODULE_BYTES));
    }

    #[test]
    fn test_create_pod_request() {
        assert_encode_decode(&CreatePodRequest {
            pod_id: POD_ID,
            wasm_module: WasmModule::from_slice(&WASM_MODULE_BYTES),
        });
    }

    #[test]
    fn test_heartbeat() {
        assert_encode_decode(&ClientMessage::Heartbeat(Heartbeat::Heartbeat));
    }

    #[test]
    fn test_heartbeat_ack() {
        assert_encode_decode(&ServerMessage::HeartbeatAcknowledge(
            HeartbeatAcknowledge::HeartbeatAcknowledge,
        ));
    }

    #[test]
    fn test_successful_create_pod_response() {
        assert_encode_decode(&CreatePodResponse {
            pod_id: POD_ID,
            result: ClientPodResult::Success,
        });
    }

    #[test]
    fn test_unsuccessful_create_pod_response() {
        assert_encode_decode(&CreatePodResponse {
            pod_id: POD_ID,
            result: ClientPodResult::Failure,
        });
    }

    #[test]
    fn test_create_pod_request_message() {
        assert_encode_decode(&ServerMessage::CreatePodRequest(
            CreatePodRequest {
                pod_id: POD_ID,
                wasm_module: WasmModule::from_slice(&WASM_MODULE_BYTES),
            },
        ));
    }

    #[test]
    fn test_successful_create_pod_response_message() {
        assert_encode_decode(&ClientMessage::CreatePodResponse(
            CreatePodResponse {
                pod_id: POD_ID,
                result: ClientPodResult::Success,
            },
        ));
    }

    #[test]
    fn test_unsuccessful_create_pod_response_message() {
        assert_encode_decode(&ClientMessage::CreatePodResponse(
            CreatePodResponse {
                pod_id: POD_ID,
                result: ClientPodResult::Success,
            },
        ));
    }

    #[test]
    fn test_heartbeat_message_envelope() {
        assert_encode_decode(&Envelope {
            version: Version::V0,
            body: Message::ClientMessage(ClientMessage::Heartbeat(
                Heartbeat::Heartbeat,
            )),
        });
    }

    #[test]
    fn test_heartbeat_ack_message_envelope() {
        assert_encode_decode(&Envelope {
            version: Version::V0,
            body: Message::ServerMessage(ServerMessage::HeartbeatAcknowledge(
                HeartbeatAcknowledge::HeartbeatAcknowledge,
            )),
        });
    }

    #[test]
    fn test_create_pod_request_message_envelope() {
        assert_encode_decode(&Envelope {
            version: Version::V0,
            body: Message::ServerMessage(ServerMessage::CreatePodRequest(
                CreatePodRequest {
                    pod_id: POD_ID,
                    wasm_module: WasmModule::from_slice(&WASM_MODULE_BYTES),
                },
            )),
        });
    }

    #[test]
    fn test_delete_pod_request_message_envelope() {
        assert_encode_decode(&Envelope {
            version: Version::V0,
            body: Message::ServerMessage(ServerMessage::DeletePodRequest(
                DeletePodRequest { pod_id: POD_ID },
            )),
        });
    }

    #[test]
    fn test_successful_create_pod_response_message_envelope() {
        assert_encode_decode(&Envelope {
            version: Version::V0,
            body: Message::ClientMessage(ClientMessage::CreatePodResponse(
                CreatePodResponse {
                    pod_id: POD_ID,
                    result: ClientPodResult::Success,
                },
            )),
        });
    }

    #[test]
    fn test_successful_delete_pod_response_message_envelope() {
        assert_encode_decode(&Envelope {
            version: Version::V0,
            body: Message::ClientMessage(ClientMessage::DeletePodResponse(
                DeletePodResponse {
                    pod_id: POD_ID,
                    result: ClientPodResult::Success,
                },
            )),
        });
    }

    #[test]
    fn test_unsuccessful_create_pod_response_message_envelope() {
        assert_encode_decode(&Envelope {
            version: Version::V0,
            body: Message::ClientMessage(ClientMessage::CreatePodResponse(
                CreatePodResponse {
                    pod_id: POD_ID,
                    result: ClientPodResult::Failure,
                },
            )),
        });
    }

    #[test]
    fn test_unsuccessful_delete_pod_response_message_envelope() {
        assert_encode_decode(&Envelope {
            version: Version::V0,
            body: Message::ClientMessage(ClientMessage::DeletePodResponse(
                DeletePodResponse {
                    pod_id: POD_ID,
                    result: ClientPodResult::Failure,
                },
            )),
        });
    }
}
