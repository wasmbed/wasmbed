// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

#![no_std]

mod cbor;

use minicbor::{Decode, Encode};

/// A protocol message wrapper that provides versioning and correlation tracking.
#[derive(Debug, Clone, PartialEq, Decode, Encode)]
pub struct Envelope<T> {
    /// Protocol version for backward compatibility
    #[cbor(n(0))]
    pub version: Version,
    /// Unique identifier for request/response correlation
    #[cbor(n(1))]
    pub message_id: MessageId,
    /// The actual message payload
    #[cbor(n(2))]
    pub message: T,
}

/// Type alias for envelopes containing client-originated messages
pub type ClientEnvelope = Envelope<ClientMessage>;

/// Type alias for envelopes containing server-originated messages
pub type ServerEnvelope = Envelope<ServerMessage>;

/// Protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Decode, Encode)]
#[cbor(index_only)]
pub enum Version {
    #[cbor(n(0))]
    V0,
}

/// Unique identifier for correlating requests with responses
#[derive(Debug, Default, Clone, Copy, PartialEq, Decode, Encode)]
#[cbor(transparent)]
pub struct MessageId(u32);

impl MessageId {
    pub fn next(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

/// Messages sent from client to server
#[derive(Debug, Clone, PartialEq)]
pub enum ClientMessage {
    /// Periodic heartbeat to maintain connection liveness
    Heartbeat,
}

/// Messages sent from server to client
#[derive(Debug, Clone, PartialEq)]
pub enum ServerMessage {
    /// Acknowledgment of a client heartbeat
    HeartbeatAck,
}
