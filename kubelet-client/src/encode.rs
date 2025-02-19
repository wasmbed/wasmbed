use crate::types::PayloadId;
use minicbor::encode::{Error, Encode, Encoder, Write};

impl<Ctx> Encode<Ctx> for PayloadId {
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
