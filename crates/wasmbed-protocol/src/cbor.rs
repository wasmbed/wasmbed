use minicbor::encode::{Encode, Encoder, Error as EError, Write};
use minicbor::decode::{Decode, Decoder, Error as DError};
use crate::types::{
    Envelope,
    Version,
    Message,
    MessageKind,
    PodId,
    WasmModule,
    CreatePodRequest,
    CreatePodResponse,
    CreatePodResult,
};

const INVALID_VERSION_ERROR: &str =
    "Invalid version";
const INVALID_MESSAGE_KIND_ERROR: &str =
    "Invalid message kind";
const INVALID_UUID_LENGTH_ERROR: &str =
    "Failed to decode PodId: invalid UUID length";
const INDEFINITE_LENGTH_ARRAY_ERROR: &str =
    "Expected a fixed-size array but found an indefinite-length array";
const INVALID_ARRAY_LENGTH_ERROR: &str =
    "Failed to decode array: incorrect length";
const INVALID_CREATE_POD_RESULT_TAG_ERROR: &str =
    "Failed to decode CreatePodResult: unexpected tag";

// Envelope --------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for Envelope {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), EError<W::Error>> {
        e.array(2)?
            .encode(&self.version)?
            .encode(&self.body)?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for Envelope {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
        let len = d.array()?
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
        _ctx: &mut Ctx
    ) -> Result<(), EError<W::Error>>
    where
        W: Write,
    {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for Version {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
        Self::from_u8(d.u8()?)
            .ok_or(DError::message(INVALID_VERSION_ERROR))
    }
}

// Message ---------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for Message {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), EError<W::Error>> {
        e.array(2)?
            .encode(self.kind())?;
        match self {
            Self::CreatePodRequest(v) => e.encode(v)?,
            Self::CreatePodResponse(v) => e.encode(v)?,
        };
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for Message {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
        let len = d.array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(match d.decode()? {
            MessageKind::CreatePodRequest  => Self::CreatePodRequest(d.decode()?),
            MessageKind::CreatePodResponse => Self::CreatePodResponse(d.decode()?),
        })
    }
}

// MessageKind -----------------------------------------------------------------

impl<Ctx> Encode<Ctx> for MessageKind {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), EError<W::Error>> {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for MessageKind {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
        Self::from_u8(d.u8()?)
            .ok_or(DError::message(INVALID_MESSAGE_KIND_ERROR))
    }
}

// PodId -----------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for PodId {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), EError<W::Error>> {
        e.bytes(self.as_bytes())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for PodId {

    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
        Self::from_slice(d.bytes()?)
            .ok_or(DError::message(INVALID_UUID_LENGTH_ERROR))
    }
}

// WasmModule ------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for WasmModule {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), EError<W::Error>> {
        e.bytes(self.as_slice())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for WasmModule {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
        Ok(Self::from_slice(d.bytes()?))
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
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
        let len = d.array()?
            .ok_or(DError::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(DError::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            pod_id: d.decode()?,
            wasm_module: d.decode()?
        })
    }
}

// Message: CreatePodResponse --------------------------------------------------

impl<Ctx> Encode<Ctx> for CreatePodResponse {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), EError<W::Error>> {
        e.array(2)?
            .encode(&self.pod_id)?
            .encode(&self.result)?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for CreatePodResponse {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
        let len = d.array()?
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

impl<Ctx> Encode<Ctx> for CreatePodResult {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), EError<W::Error>> {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for CreatePodResult {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, DError> {
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
        0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2,
        0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    ]);

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
        assert_encode_decode(
            &CreatePodRequest {
                pod_id: POD_ID,
                wasm_module: WasmModule::from_slice(&WASM_MODULE_BYTES),
            }
        );
    }

    #[test]
    fn test_successful_create_pod_response() {
        assert_encode_decode(
            &CreatePodResponse {
                pod_id: POD_ID,
                result: CreatePodResult::Success,
            }
        );
    }

    #[test]
    fn test_unsuccessful_create_pod_response() {
        assert_encode_decode(&CreatePodResponse {
            pod_id: POD_ID,
            result: CreatePodResult::Failure,
        });
    }

    #[test]
    fn test_create_pod_request_message() {
        assert_encode_decode(
            &Message::CreatePodRequest(CreatePodRequest {
                pod_id: POD_ID,
                wasm_module: WasmModule::from_slice(&WASM_MODULE_BYTES),
            })
        );
    }

    #[test]
    fn test_successful_create_pod_response_message() {
        assert_encode_decode(
            &Message::CreatePodResponse(CreatePodResponse {
                pod_id: POD_ID,
                result: CreatePodResult::Success,
            })
        );
    }

    #[test]
    fn test_unsuccessful_create_pod_response_message() {
        assert_encode_decode(
            &Message::CreatePodResponse(CreatePodResponse {
                pod_id: POD_ID,
                result: CreatePodResult::Failure,
            })
        );
    }

    #[test]
    fn test_create_pod_request_message_envelope() {
        assert_encode_decode(
            &Envelope {
                version: Version::V0,
                body: Message::CreatePodRequest(CreatePodRequest {
                    pod_id: POD_ID,
                    wasm_module: WasmModule::from_slice(&WASM_MODULE_BYTES),
                }),
            }
        );
    }

    #[test]
    fn test_successful_create_pod_response_message_envelope() {
        assert_encode_decode(
            &Envelope {
                version: Version::V0,
                body: Message::CreatePodResponse(CreatePodResponse {
                    pod_id: POD_ID,
                    result: CreatePodResult::Success,
                }),
            }
        );
    }

    #[test]
    fn test_unsuccessful_create_pod_response_message_envelope() {
        assert_encode_decode(
            &Envelope {
                version: Version::V0,
                body: Message::CreatePodResponse(CreatePodResponse {
                    pod_id: POD_ID,
                    result: CreatePodResult::Failure,
                }),
            }
        );
    }
}
