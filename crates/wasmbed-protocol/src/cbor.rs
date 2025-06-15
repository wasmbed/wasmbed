use derive_more::{Display, Error};
use minicbor::{Decode, Decoder, Encode, Encoder};
use minicbor::encode::{Error as EncodeError, Write};
use minicbor::decode::Error as DecodeError;
use crate::{ClientMessage, ServerMessage};

const CLIENT_HEARTBEAT: u32 = 0;
const SERVER_HEARTBEAT_ACK: u32 = 1;

#[derive(Debug, Display, Error)]
enum MessageDecodeError {
    #[display(
        "Unexpected array length: it should be {expected} but it is {actual}"
    )]
    UnexpectedArrayLength {
        expected: u64,
        actual: u64,
    },
    #[display("Unknown tag: {tag}")]
    UnknownTag {
        tag: u32,
    },
    #[display("Unexpected indefinite length array")]
    UnexpectedIndefiniteLengthArray,
}

impl Encode<()> for ClientMessage {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        match self {
            ClientMessage::Heartbeat => {
                e.array(1)?.u32(CLIENT_HEARTBEAT)?;
            },
        }
        Ok(())
    }
}

impl<'b> Decode<'b, ()> for ClientMessage {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut ()) -> Result<Self, DecodeError> {
        let array_len = d.array()?.ok_or_else(|| {
            DecodeError::custom(
                MessageDecodeError::UnexpectedIndefiniteLengthArray,
            )
        })?;

        let tag = d.u32()?;
        match (tag, array_len) {
            (CLIENT_HEARTBEAT, 1) => Ok(ClientMessage::Heartbeat),
            (CLIENT_HEARTBEAT, _) => Err(DecodeError::custom(
                MessageDecodeError::UnexpectedArrayLength {
                    expected: 1,
                    actual: array_len,
                },
            )),
            _ => {
                Err(DecodeError::custom(MessageDecodeError::UnknownTag { tag }))
            },
        }
    }
}

impl Encode<()> for ServerMessage {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        match self {
            ServerMessage::HeartbeatAck => {
                e.array(1)?.u32(SERVER_HEARTBEAT_ACK)?;
            },
        }
        Ok(())
    }
}

impl<'b> Decode<'b, ()> for ServerMessage {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut ()) -> Result<Self, DecodeError> {
        let array_len = d.array()?.ok_or_else(|| {
            DecodeError::custom(
                MessageDecodeError::UnexpectedIndefiniteLengthArray,
            )
        })?;

        let tag = d.u32()?;
        match (tag, array_len) {
            (SERVER_HEARTBEAT_ACK, 1) => Ok(ServerMessage::HeartbeatAck),
            (SERVER_HEARTBEAT_ACK, _) => Err(DecodeError::custom(
                MessageDecodeError::UnexpectedArrayLength {
                    expected: 1,
                    actual: array_len,
                },
            )),
            _ => {
                Err(DecodeError::custom(MessageDecodeError::UnknownTag { tag }))
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use wasmbed_test_utils::minicbor::assert_encode_decode;

    #[test]
    fn test_client_message_heartbeat() {
        assert_encode_decode(&ClientMessage::Heartbeat);
    }

    #[test]
    fn test_server_message_heartbeat_ack() {
        assert_encode_decode(&ServerMessage::HeartbeatAck);
    }
}
