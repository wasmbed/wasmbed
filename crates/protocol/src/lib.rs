#![no_std]

extern crate alloc;

pub mod decode;
pub mod encode;
pub mod types;

pub type Envelope = crate::types::Envelope<crate::types::Message>;

pub fn decode<
    T: for <'b> minicbor::Decode<'b, ()>
>(
    bytes: &[u8]
) -> Result<T, minicbor::decode::Error> {
    minicbor::decode(bytes)
}
