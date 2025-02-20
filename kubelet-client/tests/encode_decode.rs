use kubelet_client::types::{PodId, WasmModule};

#[test]
fn pod_id() {
    const V: PodId = PodId::from_bytes([
        0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2,
        0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    ]);

    let encoded = minicbor::to_vec(&V).unwrap();
    let decoded = minicbor::decode::<PodId>(&encoded).unwrap();

    assert_eq!(V, decoded);
}

#[test]
fn wasm_module() {
    const N: usize = 24;
    const V: WasmModule<N> = WasmModule::from_bytes([
        0x00, 0x61, 0x73, 0x6D,             // Magic Header "\0asm"
        0x01, 0x00, 0x00, 0x00,             // Wasm Version (1)
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Type section (empty function)
        0x03, 0x02, 0x01, 0x00,             // Function section (one function)
        0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B, // Code section
    ]);

    let encoded = minicbor::to_vec(&V).unwrap();
    let decoded = minicbor::decode::<WasmModule<N>>(&encoded).unwrap();

    assert_eq!(V, decoded);
}
