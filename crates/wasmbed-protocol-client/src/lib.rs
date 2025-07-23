#![no_std]
#![no_main]
#![deny(unsafe_code)]


use core::marker::PhantomData;
use embassy_net::{
    tcp::{ConnectError, TcpSocket}, IpEndpoint, Stack,
};

use embassy_net::tcp::Error as TcpError;
use embedded_tls::{
    Aes128GcmSha256,  MaxFragmentLength, NoVerify, TlsConfig, TlsConnection, TlsContext, TlsError,
};
use rand_core::{CryptoRng, RngCore};
use static_cell::StaticCell;


//use wasmbed_cert::{ServerAuthority, ClientIdentity};
use wasmbed_protocol::{ServerEnvelope, ServerMessage, MessageId, ClientEnvelope, ClientMessage, Version};


use heapless::Vec;
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

// Maybe is not useful
/*  
pub fn init_stack(driver: &'static dyn Driver) -> &'static Stack<'static> {
    let config = Config::dhcpv4(Default::default());
    let resources = RESOURCES.init(StackResources::new());
    STACK.init(Stack::new(driver, config, resources))
}


pub struct NoNameServerVerifier<'a> {
    ca_cert_der: &'a [u8],
}

impl<'a> NoNameServerVerifier<'a> {
    pub fn new(ca_cert: &'a CertificateDer<'static>) -> Self {
        Self {
            ca_cert_der: ca_cert.as_ref(),
        }
    }
}


impl<'a, CipherSuite> TlsVerifier<'a, CipherSuite> for NoNameServerVerifier 
where 
    CipherSuite: TlsCipherSuite,
{

    fn new(_host: &Option<&str>, ca_cert_der: &'a [u8]) -> Self {
        NoNameServerVerifier { ca_cert_der }
    }

    fn verify_certificate(
        &mut self,
        _transcript: &CipherSuite::Hash,
        _ca: &Option<Certificate<'_>>,
        _cert: CertificateRef<'_>,
    ) -> Result<(), TlsError>{
        
        let (_, server_cert) = X509Certificate::from_der(cert.raw)
            .map_err(|_| TlsError::CertificateParsingFailed)?;

        let (_, ca_cert) = X509Certificate::from_der(self.ca_cert_der)
            .map_err(|_| TlsError::CertificateParsingFailed)?;

        server_cert
            .verify_signature(Some(&ca_cert.tbs_certificate.subject_pki))
            .map_err(|_| TlsError::CertificateVerificationFailed)?;

        Ok(())
    }

    fn verify_signature(
        &mut self,
        _verify: CertificateVerify<'_>,
    ) -> Result<(), TlsError> {
        todo!()
    }
} 

*/
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

    pub async fn connect_tls<R: RngCore + CryptoRng>(
        &mut self,
        endpoint: IpEndpoint,
        rng: &mut R,
        //server_ca: &ServerAuthority,
       // identity: &ClientIdentity,
        server_ca: &str,
        identity: &str
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

    pub async fn send_heartbeat(
        &mut self,
    ) -> Result<usize, ClientError> {

        let sent_id = self.next_id;

        let envelope = ClientEnvelope {
            version: Version::V0,
            message_id: sent_id,
            message: ClientMessage::Heartbeat,
        };
        self.next_id = self.next_id.next();
        let buf = Vec::<u8, 32>::new();
        {
            let mut frame = [0u8; 32];
            let mut enc = Encoder::new(&mut frame[..]);
            let mut ctx = ();
            envelope
                .encode(&mut enc,&mut ctx)
                .map_err(|_| ClientError::BufferOverflow)?;
        };
        let _ = self.send_data(&buf).await;

        let mut resp_buf = [0u8; 32];
        let n = self.recv_data(&mut resp_buf).await?;

        let data = resp_buf
            .get(..n)
            .ok_or(ClientError::InvalidResponse)?;        // evita panic da slice

        let server_env: ServerEnvelope =
            decode(data).map_err(|_| ClientError::InvalidResponse)?;

        match server_env.message {
            ServerMessage::HeartbeatAck if server_env.message_id == sent_id => Ok(n),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    async fn send_data(
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
       // server_ca: &'a ServerAuthority,
       // identity: &'a ClientIdentity,
       _server_ca: &str,
       _identity: &str
    ) -> Result<TlsConfig<'a, Aes128GcmSha256>, ClientError> {
        let config = TlsConfig::new()
           // .with_ca(Certificate::X509(server_ca.certificate().as_ref()))
            //.with_cert(Certificate::X509(identity.certificate().as_ref()))
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