use rustls::server::ServerConfig;
use std::io::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::net::TcpStream;
use tokio_rustls::{TlsAcceptor, rustls};
use crate::test_pki::TestPki;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt},
    core::crd::CustomResourceExt,
    Client, CustomResource,
};
use wasmbed_protocol::types::{
    ClientMessage, CreatePodResponse, DeletePodResponse, Envelope, Heartbeat,
    HeartbeatAcknowledge, Message, ServerMessage, Version,
};

const ERROR_CLIENT_DECODE_MESSAGE: &'static str =
    "Client Error decoding client message";
const ERROR_CLIENT_INVALID_MESSAGE: &'static str =
    "Client Error invalid client message";

pub async fn process_message(
    acceptor: TlsAcceptor,
    stream: TcpStream,
) -> Result<(), Error> {
    // Accept the TLS connection
    let mut tls_stream = acceptor.accept(stream).await?;

    // Read the request
    let mut buffer = [0; 1024];
    let n = tls_stream.read(&mut buffer).await?;
    let envelope = minicbor::decode::<Envelope>(&buffer[0..n]).map_err(|_| {
        Error::new(std::io::ErrorKind::Other, ERROR_CLIENT_DECODE_MESSAGE)
    });

    match envelope {
        Result::Ok(Envelope {
            version: Version::V0,
            body:
                Message::ClientMessage(ClientMessage::Heartbeat(
                    Heartbeat::Heartbeat,
                )),
        }) => {
            let heartbeat_reply = Envelope {
                version: Version::V0,
                body: Message::ServerMessage(
                    ServerMessage::HeartbeatAcknowledge(
                        HeartbeatAcknowledge::HeartbeatAcknowledge,
                    ),
                ),
            };

            let encoded_message = minicbor::to_vec(heartbeat_reply).unwrap();
            tls_stream.write_all(encoded_message.as_slice()).await?;
            Ok(())
        },

        Result::Ok(Envelope {
            version: Version::V0,
            body:
                Message::ClientMessage(ClientMessage::CreatePodResponse(
                    CreatePodResponse { pod_id, result },
                )),
        }) => {
            // Do Something
            Ok(())
        },

        Result::Ok(Envelope {
            version: Version::V0,
            body:
                Message::ClientMessage(ClientMessage::DeletePodResponse(
                    DeletePodResponse { pod_id, result },
                )),
        }) => {
            // Do Something
            Ok(())
        },

        _ => Err(Error::new(
            std::io::ErrorKind::Other,
            ERROR_CLIENT_INVALID_MESSAGE,
        )),
    }
}

// Move PKI outside of server State
pub struct ServerState {
    pub address: String,
    pub port: u16,
    pub stopped: Arc<AtomicBool>,
    pub config: Arc<ServerConfig>,
    pub test_pki: TestPki,
    pub kube_client: Client
}

impl ServerState {
    pub fn new(address: String, port: u16) -> Self {
        let test_pki = TestPki::new();
        ServerState {
            address,
            port,
            stopped: Arc::new(AtomicBool::new(false)),
            config: test_pki.server_config(),
            test_pki,
        }
    }
}

pub struct Server {
    pub state: ServerState,
}

impl Server {
    pub fn new(address: String, port: u16) -> Self {
        Server {
            state: ServerState::new(address, port),
        }
    }

    pub async fn listen(&self) -> Result<(), Error> {
        let tls_acceptor = TlsAcceptor::from(self.state.config.clone());
        let listener =
            TcpListener::bind((self.state.address.clone(), self.state.port))
                .await
                .unwrap();
        loop {
            // TDODO: Handle a gracefull shutdown where thread
            // stop their task once stopped is triggered
            if self.state.stopped.load(Ordering::SeqCst) {
                return Ok(());
            }
            let (stream, peer_addr) = listener.accept().await?;
            let acceptor = tls_acceptor.clone();

            // Spawn a new task to handle this connection
            tokio::spawn(async move {
                match process_message(acceptor, stream).await {
                    Ok(_) => println!(
                        "Connection with {} handled successfully",
                        peer_addr
                    ),
                    Err(e) => eprintln!(
                        "Error handling connection with {}: {}",
                        peer_addr, e
                    ),
                }
            });
        }
    }

    pub async fn run(&self) {
        match signal::ctrl_c().await {
            Ok(_) => {
                self.state.stopped.store(true, Ordering::SeqCst);
            },
            Err(e) => panic!("Error setting Ctrl-C handler: {}", e),
        }

        let _ = self.listen().await;
    }

    pub fn stop(&self) {
        self.state.stopped.store(true, Ordering::SeqCst);
    }
}
