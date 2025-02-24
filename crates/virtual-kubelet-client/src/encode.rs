use minicbor::encode::{Error, Encode, Encoder, Write};
use crate::types::{
    Version,
    Envelope,
    MessageKind,
    Message,
    PodId,
    WasmModule,
    CreatePodRequest,
    CreatePodResult,
    CreatePodResponse,
};

impl<Ctx> Encode<Ctx> for Version {
    fn encode<W>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>>
    where
        W: Write,
    {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<Ctx, T: Encode<()>> Encode<Ctx> for Envelope<T> {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode(&self.version)?
            .encode(&self.body)?;
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for MessageKind {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>> {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for Message {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode(self.kind())?;
        match self {
            Self::CreatePodRequest(v) => e.encode(v)?,
            Self::CreatePodResponse(v) => e.encode(v)?,
        };
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for PodId {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>> {
        e.bytes(self.as_bytes())?;
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for WasmModule {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>> {
        e.bytes(self.as_slice())?;
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for CreatePodRequest {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode(&self.pod_id)?
            .encode(&self.wasm_module)?;
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for CreatePodResult {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>> {
        e.u8(self.as_u8())?;
        Ok(())
    }
}

impl<Ctx> Encode<Ctx> for CreatePodResponse {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx
    ) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode(&self.pod_id)?
            .encode(&self.result)?;
        Ok(())
    }
}
