use kubelet_client::types::PayloadId;

#[test]
fn payload_id() {
    let payload_id = PayloadId::from_bytes([
        0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2,
        0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    ]);

    let encoded = minicbor::to_vec(&payload_id).unwrap();
    let decoded: PayloadId = minicbor::decode(&encoded).unwrap();

    assert_eq!(payload_id, decoded);
}
