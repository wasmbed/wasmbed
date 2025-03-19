use minicbor::encode::Encode;
use minicbor::decode::Decode;
use wasmbed_protocol::types::{
    Version,
    Envelope,
    Message,
    PodId,
    WasmModule,
    CreatePodRequest,
    CreatePodResponse,
    CreatePodResult,
};

fn encode_decode<T>(v: &T)
where
    T: PartialEq
       + std::fmt::Debug
       + Encode<()>
       + for<'b> Decode<'b, ()>
{
    let encoded = minicbor::to_vec(v).unwrap();
    let decoded = minicbor::decode(&encoded).unwrap();
    assert_eq!(*v, decoded);
}

#[test]
fn test() {
    let pod_id = PodId::from_bytes([
        0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2,
        0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8,
    ]);

    let wasm_module = WasmModule::from_slice(&[
        0x00, 0x61, 0x73, 0x6D,             // Magic Header "\0asm"
        0x01, 0x00, 0x00, 0x00,             // Wasm Version (1)
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Type section (empty function)
        0x03, 0x02, 0x01, 0x00,             // Function section (one function)
        0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B, // Code section
    ]);

    let create_pod_request = CreatePodRequest {
        pod_id: pod_id.clone(),
        wasm_module: wasm_module.clone()
    };

    let successful_create_pod_response = CreatePodResponse {
        pod_id: pod_id.clone(),
        result: CreatePodResult::Success,
    };

    let unsuccessful_create_pod_response = CreatePodResponse {
        pod_id: pod_id.clone(),
        result: CreatePodResult::Failure,
    };

    let create_pod_request_message =
        Message::CreatePodRequest(create_pod_request.clone());

    let successful_create_pod_response_message =
        Message::CreatePodResponse(successful_create_pod_response.clone());

    let unsuccessful_create_pod_response_message =
        Message::CreatePodResponse(unsuccessful_create_pod_response.clone());

    let create_pod_request_message_envelope =
        Envelope {
            version: Version::V0,
            body: Message::CreatePodRequest(create_pod_request.clone()),
        };

    let successful_create_pod_response_message_envelope =
        Envelope {
            version: Version::V0,
            body: Message::CreatePodResponse(successful_create_pod_response.clone()),
        };

    let unsuccessful_create_pod_response_message_envelope =
        Envelope {
            version: Version::V0,
            body: Message::CreatePodResponse(unsuccessful_create_pod_response.clone()),
        };

    encode_decode(&pod_id);
    encode_decode(&wasm_module);
    encode_decode(&create_pod_request);
    encode_decode(&successful_create_pod_response);
    encode_decode(&unsuccessful_create_pod_response);
    encode_decode(&create_pod_request_message);
    encode_decode(&successful_create_pod_response_message);
    encode_decode(&unsuccessful_create_pod_response_message);
    encode_decode(&create_pod_request_message_envelope);
    encode_decode(&successful_create_pod_response_message_envelope);
    encode_decode(&unsuccessful_create_pod_response_message_envelope);
}
