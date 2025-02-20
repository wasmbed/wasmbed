use crate::types::{PodId, WasmModule};
use minicbor::encode::{Error, Encode, Encoder, Write};

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
