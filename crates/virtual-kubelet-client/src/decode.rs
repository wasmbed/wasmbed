use minicbor::decode::{Decode, Decoder, Error};
use crate::types::{
    Version,
    Envelope,
    MessageKind,
    Message,
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
const INVALID_WASM_MODULE_SIZE_ERROR: &str =
    "Failed to decode WasmModule: incorrect size";
const INDEFINITE_LENGTH_ARRAY_ERROR: &str =
    "Expected a fixed-size array but found an indefinite-length array";
const INVALID_ARRAY_LENGTH_ERROR: &str =
    "Failed to decode array: incorrect length";
const INVALID_CREATE_POD_RESULT_TAG_ERROR: &str =
    "Failed to decode CreatePodResult: unexpected tag";

impl<'b, Ctx> Decode<'b, Ctx> for Version {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        Self::from_u8(d.u8()?)
            .ok_or(Error::message(INVALID_VERSION_ERROR))
    }
}

impl<'b, Ctx, T: Decode<'b, ()>> Decode<'b, Ctx> for Envelope<T> {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let len = d.array()?
            .ok_or(Error::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(Error::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            version: d.decode()?,
            body: d.decode()?,
        })
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for MessageKind {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        Self::from_u8(d.u8()?)
            .ok_or(Error::message(INVALID_MESSAGE_KIND_ERROR))
    }
}

impl<'b, Ctx, const WASM_MODULE_SIZE: usize> Decode<'b, Ctx>
    for Message<WASM_MODULE_SIZE>
{
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let len = d.array()?
            .ok_or(Error::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(Error::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(match d.decode()? {
            MessageKind::CreatePodRequest  => Self::CreatePodRequest(d.decode()?),
            MessageKind::CreatePodResponse => Self::CreatePodResponse(d.decode()?),
        })
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for PodId {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        Self::from_slice(d.bytes()?)
            .ok_or(Error::message(INVALID_UUID_LENGTH_ERROR))
    }
}

impl<'b, Ctx, const N: usize> Decode<'b, Ctx> for WasmModule<N> {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        Self::from_slice(d.bytes()?)
            .ok_or(Error::message(INVALID_WASM_MODULE_SIZE_ERROR))
    }
}

impl<'b, Ctx, const WASM_MODULE_SIZE: usize> Decode<'b, Ctx>
    for CreatePodRequest<WASM_MODULE_SIZE>
{
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let len = d.array()?
            .ok_or(Error::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(Error::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            pod_id: d.decode()?,
            wasm_module: d.decode()?
        })
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for CreatePodResponse {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let len = d.array()?
            .ok_or(Error::message(INDEFINITE_LENGTH_ARRAY_ERROR))?;
        if len != 2 {
            return Err(Error::message(INVALID_ARRAY_LENGTH_ERROR));
        }

        Ok(Self {
            pod_id: d.decode()?,
            result: d.decode()?,
        })
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for CreatePodResult {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        Self::from_u8(d.u8()?)
            .ok_or(Error::message(INVALID_CREATE_POD_RESULT_TAG_ERROR))
    }
}
