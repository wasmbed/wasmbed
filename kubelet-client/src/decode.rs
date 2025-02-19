use crate::types::PayloadId;
use minicbor::decode::{Decode, Decoder, Error};

impl<'b, Ctx> Decode<'b, Ctx> for PayloadId {
    fn decode(
        d: &mut Decoder<'b>,
        _ctx: &mut Ctx
    ) -> Result<Self, Error> {
        let bytes = d.bytes()?;
        PayloadId::from_slice(bytes)
            .ok_or_else(|| Error::message("Invalid UUID length"))
    }
}
