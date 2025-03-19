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
