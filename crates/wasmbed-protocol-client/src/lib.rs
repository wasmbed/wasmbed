// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

#![no_std]
#![no_main]
#![deny(unsafe_code)]

mod certs;

use embedded_tls::{Certificate, NoVerify};
use core::marker::PhantomData;
use embassy_net::{
    tcp::{ConnectError, TcpSocket},
    IpEndpoint, Stack, Runner, StackResources, Config,
    driver::{Driver},
};

use embassy_net::tcp::Error as TcpError;
use embedded_tls::{
    Aes128GcmSha256, MaxFragmentLength, TlsConfig, TlsConnection,
    TlsContext, TlsError, Ed25519Provider
};

use rand_core::{CryptoRng, RngCore};
use static_cell::StaticCell;

//use wasmbed_cert::{ServerAuthority, ClientIdentity};
use wasmbed_protocol::{
    ServerEnvelope, ServerMessage, MessageId, ClientEnvelope, ClientMessage,
    Version,
};

use minicbor::{decode, Encode as _, Encoder};

/// Recommended default buffer sizes for MCU applications
const RX_BUFFER_SIZE: usize = 4096;
const TX_BUFFER_SIZE: usize = 4096;
const TLS_RX_BUFFER_SIZE: usize = 16384;
const TLS_TX_BUFFER_SIZE: usize = 16384;

/// Static buffers for TCP socket
static RX_BUFF: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
static TX_BUFF: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();

static TLS_RX_BUFF: StaticCell<[u8; TLS_RX_BUFFER_SIZE]> = StaticCell::new();
static TLS_TX_BUFF: StaticCell<[u8; TLS_TX_BUFFER_SIZE]> = StaticCell::new();

/// Wrapper for Stack Initialization
pub fn init_stack_and_runner<'d, D, const SOCK: usize>(
    driver: D,
    config: Config,
    resources: &'d mut StackResources<SOCK>,
    random_seed: u64,
) -> (Stack<'d>, Runner<'d, D>)
where
    D: Driver + 'd,
{
    embassy_net::new(driver, config, resources, random_seed)
}

/// Tcp + TLS Client
pub struct Client<'d> {
    stack: &'d Stack<'d>,
    tls_connection: Option<TlsConnection<'d, TcpSocket<'d>, Aes128GcmSha256>>,
    next_id: MessageId,
    phantom: PhantomData<&'d ()>,
}

impl<'d> Client<'d> {
    pub fn new(stack: &'d Stack<'d>) -> Self {
        Self {
            stack,
            tls_connection: None,
            next_id: MessageId::default(),
            phantom: PhantomData,
        }
    }

    pub async fn connect_tls<R: CryptoRng + RngCore>(
        &mut self,
        endpoint: IpEndpoint,
        rng: &mut R,
        //server_ca: &ServerAuthority,
        // identity: &ClientIdentity,
        _server_ca: &str,
        _identity: &str,
    ) -> Result<(), ClientError> {
        let rx_buff = RX_BUFF.init([0; RX_BUFFER_SIZE]);
        let tx_buff = TX_BUFF.init([0; TX_BUFFER_SIZE]);
        let tls_rx_buffer = TLS_RX_BUFF.init([0; TLS_RX_BUFFER_SIZE]);
        let tls_tx_buffer = TLS_TX_BUFF.init([0; TLS_TX_BUFFER_SIZE]);
        let mut socket = TcpSocket::new(*self.stack, rx_buff, tx_buff);
        socket
            .connect(endpoint)
            .await
            .map_err(|_| ClientError::NotConnected)?;
        
        let ca_der = certs::SERVER_CA_DER;
        let client_cert = certs::CLIENT_CERT_DER;
        let client_key = certs::CLIENT_PRIVATE_KEY_DER;
        let tls_config = TlsConfig::new()
            .with_ca(Certificate::X509(ca_der))
            .with_cert(Certificate::X509(client_cert))
            .with_priv_key(client_key)
            .with_max_fragment_length(MaxFragmentLength::Bits10);
        
        let tls_context = 
            TlsContext::new(&tls_config, Ed25519Provider::new::<Aes128GcmSha256>(rng));
        let mut tls_connection =
            TlsConnection::<'_>::new(socket, tls_rx_buffer, tls_tx_buffer);
        tls_connection
            .open(tls_context)
            .await
            .map_err(ClientError::from)?; // need a TLSverifier 
        self.tls_connection = Some(tls_connection);
        Ok(())
    }

    pub async fn send_heartbeat(&mut self) -> Result<usize, ClientError> {
        let sent_id = self.next_id;

        let envelope = ClientEnvelope {
            version: Version::V0,
            message_id: sent_id,
            message: ClientMessage::Heartbeat,
        };
        self.next_id = self.next_id.next();

        let mut frame = [0u8; 32];
        let mut enc = Encoder::new(&mut frame[..]);
        let mut ctx = ();
        envelope
            .encode(&mut enc, &mut ctx)
            .map_err(|_| ClientError::BufferOverflow)?;

        let _ = self.send_data(&frame[..]).await;

        let mut resp_buf = [0u8; 32];
        let n = self.recv_data(&mut resp_buf).await?;

        let data = resp_buf.get(..n).ok_or(ClientError::InvalidResponse)?;

        let server_env: ServerEnvelope =
            decode(data).map_err(|_| ClientError::InvalidResponse)?;

        match server_env.message {
            ServerMessage::HeartbeatAck if server_env.message_id == sent_id => {
                Ok(n)
            },
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    async fn send_data(&mut self, data: &[u8]) -> Result<usize, ClientError> {
        match &mut self.tls_connection {
            Some(tls) => tls.write(data).await.map_err(ClientError::from),
            None => Err(ClientError::NotConnected),
        }
    }
    pub async fn recv_data(
        &mut self,
        data: &mut [u8],
    ) -> Result<usize, ClientError> {
        match &mut self.tls_connection {
            Some(tls) => tls.read(data).await.map_err(ClientError::from),
            None => Err(ClientError::AuthenticationFailed),
        }
    }

    pub async fn close(&mut self) -> Result<(), ClientError> {
        if let Some(tls) = self.tls_connection.take() {
            let _ = tls.close().await;
        }
        Ok(())
    }
}


#[derive(Debug)]
pub enum ClientError {
    TlsError(TlsError),
    TcpError(TcpError),
    ConnectError(ConnectError),
    HeartbeatFailed,
    AuthenticationFailed,
    Timeout,
    BufferOverflow,
    NotConnected,
    InvalidConfiguration,
    UnexpectedResponse,
    InvalidResponse,
}

impl From<TlsError> for ClientError {
    fn from(err: TlsError) -> Self {
        ClientError::TlsError(err)
    }
}

impl From<TcpError> for ClientError {
    fn from(err: TcpError) -> Self {
        ClientError::TcpError(err)
    }
}

impl From<ConnectError> for ClientError {
    fn from(err: ConnectError) -> Self {
        ClientError::ConnectError(err)
    }
}