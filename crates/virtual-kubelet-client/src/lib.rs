#![no_std]

pub mod decode;
pub mod encode;
pub mod types;

pub const WASM_MODULE_SIZE: usize = 64_000;

pub type Message = crate::types::Message<WASM_MODULE_SIZE>;
pub type Envelope = crate::types::Envelope<Message>;

pub fn decode<
    T: for <'b> minicbor::Decode<'b, ()>
>(
    bytes: &[u8]
) -> Result<T, minicbor::decode::Error> {
    minicbor::decode(bytes)
}
