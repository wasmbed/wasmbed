use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio_util::sync::CancellationToken;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

use wasmbed_cert::ServerIdentity;
use wasmbed_protocol::{ClientMessage, ServerMessage};
use wasmbed_protocol_server::{
    AuthorizationResult, MessageContext, Server, ServerConfig,
};
use wasmbed_types::PublicKey;

#[derive(Parser)]
#[command(disable_help_subcommand = true)]
struct Args {
    #[arg(long, env = "BIND_ADDR")]
    bind_addr: SocketAddr,
    #[arg(long, env = "PRIVATE_KEY")]
    private_key: String,
    #[arg(long, env = "CERTIFICATE")]
    certificate: String,
    #[arg(long, env = "CLIENT_CA")]
    client_ca: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let private_key_bytes = std::fs::read(&args.private_key)?;
    let certificate_bytes = std::fs::read(&args.certificate)?;
    let client_ca_bytes = std::fs::read(&args.client_ca)?;

    let identity = ServerIdentity::from_parts(
        private_key_bytes.into(),
        certificate_bytes.into(),
    );
    let client_ca = client_ca_bytes.into();

    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down...");
                shutdown_clone.cancel();
            },
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            },
        }
    });

    let config = ServerConfig {
        bind_addr: args.bind_addr,
        identity,
        client_ca,
        on_client_connect: Arc::new(|public_key: PublicKey| {
            Box::pin(async move {
                info!("Client connecting: {:?}", public_key);
                AuthorizationResult::Authorized
            })
        }),
        on_client_message: Arc::new(|ctx: MessageContext| {
            Box::pin(async move {
                match ctx.message() {
                    ClientMessage::Heartbeat => {
                        // FIXME: error handling?
                        let _ = ctx.reply(ServerMessage::HeartbeatAck);
                    },
                }
            })
        }),
        shutdown,
    };

    let server = Server::new(config);
    info!("Starting server on {}", args.bind_addr);
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
    }

    Ok(())
}
