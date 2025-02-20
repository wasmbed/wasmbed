use kubelet_client::types::{PodId, WasmModule, CreatePodRequest};
use minicbor::encode::Encode;
use minicbor::decode::Decode;

const POD_ID: PodId = PodId::from_bytes([
    0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2,
    0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
]);

const WASM_MODULE_SIZE: usize = 24;
const WASM_MODULE: WasmModule<WASM_MODULE_SIZE> = WasmModule::from_bytes([
    0x00, 0x61, 0x73, 0x6D,             // Magic Header "\0asm"
    0x01, 0x00, 0x00, 0x00,             // Wasm Version (1)
    0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Type section (empty function)
    0x03, 0x02, 0x01, 0x00,             // Function section (one function)
    0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B, // Code section
]);

const CREATE_POD_REQUEST: CreatePodRequest<WASM_MODULE_SIZE> = CreatePodRequest {
    pod_id: POD_ID,
    wasm_module: WASM_MODULE
};

fn encode_decode<T>(v: &T)
where
    T: PartialEq
       + std::fmt::Debug
       + Encode<()>
       + for <'b> Decode<'b, ()>
{
    let encoded = minicbor::to_vec(v).unwrap();
    let decoded = minicbor::decode(&encoded).unwrap();
    assert_eq!(*v, decoded);
}

#[test]
fn test() {
    encode_decode(&POD_ID);
    encode_decode(&WASM_MODULE);
    encode_decode(&CREATE_POD_REQUEST);
}
