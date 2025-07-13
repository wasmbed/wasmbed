// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use rustls::{Error as RustlsError, RootCertStore, ServerConfig as RustlsConfig};
use rustls::server::WebPkiClientVerifier;
use rustls_pki_types::CertificateDer;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::mpsc::error::SendError;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn, debug};

use wasmbed_cert::ServerIdentity;
use wasmbed_protocol::{
    ClientEnvelope, ClientMessage, MessageId, ServerEnvelope, ServerMessage,
    Version,
};
use wasmbed_types::PublicKey;

/// Maximum message size to prevent DoS attacks (16MB)
const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

type Clients = Arc<RwLock<HashMap<PublicKey<'static>, Sender>>>;
type LastMessageId = Arc<RwLock<MessageId>>;
pub type OnClientConnect = dyn Send
    + Sync
    + Fn(
        PublicKey<'static>,
    ) -> Pin<Box<dyn Future<Output = AuthorizationResult> + Send>>;
pub type OnClientDisconnect = dyn Send
    + Sync
    + Fn(PublicKey<'static>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
pub type OnClientMessage = dyn Send
    + Sync
    + Fn(MessageContext) -> Pin<Box<dyn Future<Output = ()> + Send>>;
type Sender = UnboundedSender<ServerEnvelope>;

pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub identity: ServerIdentity,
    pub client_ca: CertificateDer<'static>,
    pub on_client_connect: Arc<OnClientConnect>,
    pub on_client_disconnect: Arc<OnClientDisconnect>,
    pub on_client_message: Arc<OnClientMessage>,
    pub shutdown: CancellationToken,
}

pub enum AuthorizationResult {
    Authorized,
    Unauthorized,
}

pub enum MessageDeliveryError {
    ClientNotFound(PublicKey<'static>),
    SendError(SendError<ServerEnvelope>),
}

pub struct MessageContext {
    envelope: ClientEnvelope,
    sender: Sender,
}

impl MessageContext {
    pub fn message(&self) -> ClientMessage {
        self.envelope.message.clone()
    }

    pub fn reply(
        &self,
        message: ServerMessage,
    ) -> Result<(), SendError<ServerEnvelope>> {
        self.sender.send(ServerEnvelope {
            version: Version::V0,
            message_id: self.envelope.message_id,
            message,
        })
    }
}

pub struct Server {
    config: ServerConfig,
    clients: Clients,
    last_message_id: LastMessageId,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            clients: Default::default(),
            last_message_id: Default::default(),
        }
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(&self.config.bind_addr).await?;
        let acceptor = Arc::new(
            build_tls_acceptor(&self.config.identity, &self.config.client_ca)
                .map_err(std::io::Error::other)?,
        );

        info!("Server listening on {}", self.config.bind_addr);

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("Accepted connection from {}", addr);
                            let acceptor = Arc::clone(&acceptor);
                            let clients = Arc::clone(&self.clients);
                            let on_client_connect = Arc::clone(&self.config.on_client_connect);
                            let on_client_disconnect = Arc::clone(&self.config.on_client_disconnect);
                            let on_client_message = Arc::clone(&self.config.on_client_message);
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(
                                    stream,
                                    acceptor,
                                    clients,
                                    &*on_client_connect,
                                    &*on_client_disconnect,
                                    &*on_client_message,
                                ).await {
                                    error!("Client handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = self.config.shutdown.cancelled() => {
                    info!("Server shutdown requested");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn send(
        &self,
        client_key: &PublicKey<'static>,
        message: ServerMessage,
    ) -> Result<MessageId, MessageDeliveryError> {
        let message_id = self.next_message_id().await;
        let envelope = ServerEnvelope {
            version: Version::V0,
            message_id,
            message,
        };

        match self.clients.read().await.get(client_key) {
            Some(client) => {
                client
                    .send(envelope)
                    .map_err(MessageDeliveryError::SendError)?;
                Ok(message_id)
            },
            None => {
                Err(MessageDeliveryError::ClientNotFound(client_key.clone()))
            },
        }
    }

    async fn next_message_id(&self) -> MessageId {
        let mut last = self.last_message_id.write().await;
        *last = last.next();
        *last
    }
}

fn build_tls_acceptor(
    identity: &ServerIdentity,
    client_ca: &CertificateDer<'static>,
) -> Result<TlsAcceptor, RustlsError> {
    let mut root_store = RootCertStore::empty();
    root_store.add(client_ca.clone())?;

    let verifier = WebPkiClientVerifier::builder(root_store.into())
        .build()
        .map_err(|e| RustlsError::Other(rustls::OtherError(Arc::new(e))))?;

    let config = RustlsConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![identity.certificate().clone()],
            identity.private_key().clone_key().into(),
        )?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

fn extract_client_public_key(
    tls_stream: &TlsStream<TcpStream>,
) -> Option<PublicKey<'static>> {
    let (_, session) = tls_stream.get_ref();
    let client_cert = session.peer_certificates()?.first()?;
    Some(PublicKey::try_from(client_cert).ok()?.into_owned())
}

async fn handle_client(
    stream: TcpStream,
    acceptor: Arc<TlsAcceptor>,
    clients: Clients,
    on_client_connect: &OnClientConnect,
    on_client_disconnect: &OnClientDisconnect,
    on_client_message: &OnClientMessage,
) -> std::io::Result<()> {
    let tls_stream = acceptor.accept(stream).await?;

    let public_key =
        extract_client_public_key(&tls_stream).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to extract client public key",
            )
        })?;

    info!("Client connected: {}", public_key);

    if matches!(
        on_client_connect(public_key.clone()).await,
        AuthorizationResult::Unauthorized
    ) {
        warn!("Client authorization failed: {:?}", public_key);
        return Ok(());
    }

    info!("Client authorized: {}", public_key);

    let (tx, rx) = unbounded_channel::<ServerEnvelope>();
    register_client(&clients, &public_key, tx.clone()).await;

    let result = client_handler(
        tls_stream,
        &public_key,
        &clients,
        rx,
        on_client_message,
    )
    .await;

    unregister_client(&clients, &public_key).await;
    info!("Client disconnected: {}", public_key);
    on_client_disconnect(public_key.clone()).await;

    result
}

async fn register_client<'a>(
    clients: &Clients,
    public_key: &PublicKey<'a>,
    sender: UnboundedSender<ServerEnvelope>,
) {
    let mut guard = clients.write().await;
    guard.insert(public_key.clone().into_owned(), sender);
}

async fn unregister_client(clients: &Clients, client_key: &PublicKey<'static>) {
    let mut guard = clients.write().await;
    guard.remove(client_key);
}

async fn client_handler<'a>(
    tls_stream: TlsStream<TcpStream>,
    client_key: &PublicKey<'a>,
    clients: &Clients,
    mut rx: UnboundedReceiver<ServerEnvelope>,
    on_client_message: &OnClientMessage,
) -> std::io::Result<()> {
    let (mut reader, mut writer) = tokio::io::split(tls_stream);

    let writer_task = tokio::spawn(async move {
        while let Some(envelope) = rx.recv().await {
            if let Err(e) = write_envelope(&mut writer, &envelope).await {
                error!("Failed to write envelope: {}", e);
                break;
            }
        }
    });

    let result = loop {
        tokio::select! {
            result = read_envelope(&mut reader) => {
                match result {
                    Ok(envelope) => {
                        let sender = {
                            let guard = clients.read().await;
                            guard.get(client_key).cloned()
                        };

                        if let Some(sender) = sender {
                            let ctx = MessageContext {
                                envelope,
                                sender,
                            };

                            on_client_message(ctx).await;
                        } else {
                            error!("Client sender not found: {client_key:?}");
                            break Err(std::io::Error::other(
                                format!("Client not found: {client_key:?}")
                            ));
                        }
                    }
                    Err(e) => {
                        error!("Failed to read envelope: {e}");
                        break Err(e);
                    }
                }
            }
        }
    };

    writer_task.abort();
    result
}

async fn read_envelope(
    reader: &mut (impl AsyncReadExt + Unpin),
) -> std::io::Result<ClientEnvelope> {
    // Read length prefix (4 bytes, big endian)
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > MAX_MESSAGE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Message too large: {len} bytes (max: {MAX_MESSAGE_SIZE})"),
        ));
    }

    let mut data = vec![0u8; len];
    reader.read_exact(&mut data).await?;

    minicbor::decode(&data).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("CBOR decode error: {e}"),
        )
    })
}

async fn write_envelope(
    writer: &mut (impl AsyncWriteExt + Unpin),
    envelope: &ServerEnvelope,
) -> std::io::Result<()> {
    let data = minicbor::to_vec(envelope).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("CBOR encode error: {e}"),
        )
    })?;

    // Write length prefix
    let len: u32 = data.len().try_into().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid message length: {e}"),
        )
    })?;
    writer.write_all(&len.to_be_bytes()).await?;

    // Write message data
    writer.write_all(&data).await?;
    writer.flush().await?;

    Ok(())
}
