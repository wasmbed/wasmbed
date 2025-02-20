use minicbor::encode::{Error, Encode, Encoder, Write};
use crate::types::{
    PodId,
    WasmModule,
    CreatePodRequest,
    CreatePodResponse,
    CreatePodResult,
};

impl<Ctx> Encode<Ctx> for PodId {
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>>
    where
        W: Write,
    {
        e.bytes(self.as_bytes())?;
        Ok(())
    }
}

impl<Ctx, const N: usize> Encode<Ctx> for WasmModule<N> {
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>>
    where
        W: Write,
    {
        e.bytes(self.as_bytes())?;
        Ok(())
    }
}

impl<Ctx, const WASM_MODULE_SIZE: usize> Encode<Ctx>
    for CreatePodRequest<WASM_MODULE_SIZE>
{
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), Error<W::Error>>
    where
        W: Write,
    {
        e.array(2)?
            .encode(&self.pod_id)?
            .encode(&self.wasm_module)?;
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for CreatePodResponse {
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>>
    where
        W: Write,
    {
        e.array(2)?
            .encode(&self.pod_id)?
            .encode(&self.result)?;
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for CreatePodResult {
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>>
    where
        W: Write,
    {
        match *self {
            Self::Success => e.u8(0)?,
            Self::Failure => e.u8(1)?,
        };
        Ok(())
    }
}
