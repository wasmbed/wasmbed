use minicbor::decode::{Decode, Decoder, Error};
use crate::types::{
    PodId,
    WasmModule,
    CreatePodRequest,
    CreatePodResponse,
    CreatePodResult,
};

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

impl<'b, Ctx> Decode<'b, Ctx> for PodId {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        PodId::from_slice(d.bytes()?)
            .ok_or(Error::message(INVALID_UUID_LENGTH_ERROR))
    }
}

impl<'b, Ctx, const N: usize> Decode<'b, Ctx> for WasmModule<N> {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        WasmModule::from_slice(d.bytes()?)
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

        Ok(CreatePodRequest {
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

        Ok(CreatePodResponse {
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
        match d.u8()?{
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            _ => Err(Error::message(INVALID_CREATE_POD_RESULT_TAG_ERROR)),
        }
    }
}
