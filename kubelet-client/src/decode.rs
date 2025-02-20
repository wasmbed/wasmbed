use crate::types::{PodId, WasmModule, CreatePodRequest};
use minicbor::decode::{Decode, Decoder, Error};

impl<'b, Ctx> Decode<'b, Ctx> for PodId {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let bytes = d.bytes()?;
        PodId::from_slice(bytes)
            .ok_or(Error::message("Invalid UUID length"))
    }
}

impl<'b, Ctx, const N: usize> Decode<'b, Ctx> for WasmModule<N> {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let bytes = d.bytes()?;
        WasmModule::from_slice(bytes)
            .ok_or(Error::message("Invalid WasmModule size"))
    }
}

impl<'b, Ctx, const WASM_MODULE_SIZE: usize> Decode<'b, Ctx>
    for CreatePodRequest<WASM_MODULE_SIZE>
{
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let len = d.array()?.ok_or(Error::message("Expected sized array"))?;
        if len != 2 {
            return Err(Error::message("Unexpected array length"));
        }

        Ok(CreatePodRequest {
            pod_id: d.decode()?,
            wasm_module: d.decode()?
        })
    }
}
