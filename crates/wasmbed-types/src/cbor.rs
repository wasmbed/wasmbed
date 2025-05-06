use minicbor::encode::{Encode, Encoder, Error as EError, Write};
use minicbor::decode::{Decode, Decoder, Error as DError};
use crate::types::DeviceId;

// PodId -----------------------------------------------------------------------

impl<Ctx> Encode<Ctx> for DeviceId {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut Ctx,
    ) -> Result<(), EError<W::Error>> {
        e.bytes(self.as_bytes())?;
        Ok(())
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for DeviceId {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut Ctx) -> Result<Self, DError> {
        Self::from_slice(d.bytes()?)
            .ok_or(DError::message("Invalid UUID length"))
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wasmbed_test_utils::minicbor::assert_encode_decode;

    #[rustfmt::skip]
    const POD_ID: DeviceId = DeviceId::from_bytes([
        0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2,
        0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    ]);

    #[test]
    fn test_device_id() {
        assert_encode_decode(&POD_ID);
    }
}
