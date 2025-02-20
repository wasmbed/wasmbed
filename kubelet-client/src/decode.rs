use crate::types::{PodId, WasmModule};
use minicbor::decode::{Decode, Decoder, Error};

impl<'b, Ctx> Decode<'b, Ctx> for PodId {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let bytes = d.bytes()?;
        PodId::from_slice(bytes)
            .ok_or_else(|| Error::message("Invalid UUID length"))
    }
}

impl<'b, Ctx, const N: usize> Decode<'b, Ctx> for WasmModule<N> {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let bytes = d.bytes()?;
        WasmModule::from_slice(bytes)
            .ok_or_else(|| Error::message("Invalid WasmModule size"))
    }
}
