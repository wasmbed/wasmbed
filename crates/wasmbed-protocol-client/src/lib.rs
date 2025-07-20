#![no_std]
#![no_main]
#![deny(unsafe_code)]

extern crate alloc;
use core::marker::PhantomData;
use embassy_net::{
    tcp::{ConnectError, TcpSocket},
    IpEndpoint, Stack,
};

use embassy_net::tcp::Error as TcpError;
use embedded_tls::{
    Aes128GcmSha256, NoVerify, TlsConfig, TlsConnection, TlsContext,
    MaxFragmentLength, Certificate, TlsError,
};
use rand_core::{CryptoRng, RngCore};
use static_cell::StaticCell;
use wasmbed_cert::{ServerAuthority, ClientIdentity};

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

/// Tcp + TLS Client
pub struct Client<'d> {
    stack: &'d Stack<'d>,
    tls_connection: Option<TlsConnection<'d, TcpSocket<'d>, Aes128GcmSha256>>,
    phantom: PhantomData<&'d ()>,
}

impl<'d> Client<'d> {
    pub fn new(stack: &'d Stack<'d>) -> Self {
        Self {
            stack,
            tls_connection: None,
            phantom: PhantomData,
        }
    }

    pub async fn connect_tls<R: RngCore + CryptoRng>(
        &mut self,
        endpoint: IpEndpoint,
        rng: &mut R,
        server_ca: &ServerAuthority,
        identity: &ClientIdentity,
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
        let tls_config = self.build_tls_config(server_ca, identity)?;
        let tls_context = TlsContext::new(&tls_config, rng);
        let mut tls_connection =
            TlsConnection::<'d>::new(socket, tls_rx_buffer, tls_tx_buffer);
        tls_connection
            .open::<_, NoVerify>(tls_context)
            .await
            .map_err(ClientError::from)?; // need a TLSverifier 
        self.tls_connection = Some(tls_connection);
        Ok(())
    }

    pub async fn send_data(
        &mut self,
        data: &[u8],
    ) -> Result<usize, ClientError> {
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

    fn build_tls_config<'a>(
        &self,
        server_ca: &'a ServerAuthority,
        identity: &'a ClientIdentity,
    ) -> Result<TlsConfig<'a, Aes128GcmSha256>, ClientError> {
        let config = TlsConfig::new()
            .with_ca(Certificate::X509(server_ca.certificate().as_ref()))
            .with_cert(Certificate::X509(identity.certificate().as_ref()))
            .with_max_fragment_length(MaxFragmentLength::Bits10);
        Ok(config)
    }
}

/*
impl <'d> Drop for Client<'d> {
    fn drop(&mut self) {
        if let Some(mut tls) = self.tls_connection.take() {
            #[cfg(feature = "defmt")]
            defmt::warn!("TLS connection dropped without proper close");

        }
    }
}
*/

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
