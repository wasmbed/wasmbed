use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use kube::{Api, Client};
use tokio_util::sync::CancellationToken;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

use wasmbed_cert::ServerIdentity;
use wasmbed_k8s_resource::{Device, DeviceStatusUpdate};
use wasmbed_protocol::{ClientMessage, ServerMessage};
use wasmbed_protocol_server::{
    AuthorizationResult, MessageContext, OnClientConnect, OnClientDisconnect,
    OnClientMessage, Server, ServerConfig,
};
use wasmbed_types::{GatewayReference, PublicKey};

#[derive(Parser)]
#[command(disable_help_subcommand = true)]
struct Args {
    #[arg(long, env = "WASMBED_GATEWAY_BIND_ADDR")]
    bind_addr: SocketAddr,
    #[arg(long, env = "WASMBED_GATEWAY_PRIVATE_KEY")]
    private_key: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_CERTIFICATE")]
    certificate: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_CLIENT_CA")]
    client_ca: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_NAMESPACE")]
    namespace: String,
    #[arg(long, env = "WASMBED_GATEWAY_POD_NAMESPACE")]
    pod_namespace: String,
    #[arg(long, env = "WASMBED_GATEWAY_POD_NAME")]
    pod_name: String,
}

struct Callbacks {
    api: Api<Device>,
    gateway_reference: GatewayReference,
}

impl Callbacks {
    fn on_connect(&self) -> Box<OnClientConnect> {
        let api = self.api.clone();
        let gateway_reference = self.gateway_reference.clone();
        Box::new(move |public_key: PublicKey<'static>| {
            let api = api.clone();
            let gateway_reference = gateway_reference.clone();
            Box::pin(async move {
                match Device::find(api.clone(), public_key).await {
                    Ok(Some(device)) => {
                        if let Err(e) = DeviceStatusUpdate::default()
                            .mark_connected(gateway_reference)
                            .apply(api.clone(), device)
                            .await
                        {
                            error!("Error updating DeviceStatus: {e}");
                        }
                        AuthorizationResult::Authorized
                    },
                    Ok(None) => AuthorizationResult::Unauthorized,
                    Err(e) => {
                        error!("Unable to find Device: {e}");
                        AuthorizationResult::Unauthorized
                    },
                }
            })
        })
    }

    fn on_disconnect(&self) -> Box<OnClientDisconnect> {
        Box::new(move |_public_key: PublicKey<'static>| Box::pin(async move {}))
    }

    fn on_message(&self) -> Box<OnClientMessage> {
        Box::new(move |ctx: MessageContext| {
            Box::pin(async move {
                match ctx.message() {
                    ClientMessage::Heartbeat => {
                        let _ = ctx.reply(ServerMessage::HeartbeatAck);
                    },
                }
            })
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    let private_key_bytes =
        std::fs::read(&args.private_key).with_context(|| {
            format!(
                "Failed to read private key from {}",
                args.private_key.display()
            )
        })?;
    let certificate_bytes =
        std::fs::read(&args.certificate).with_context(|| {
            format!(
                "Failed to read certificate from {}",
                args.certificate.display()
            )
        })?;
    let client_ca_bytes =
        std::fs::read(&args.client_ca).with_context(|| {
            format!(
                "Failed to read client CA certificate from {}",
                args.client_ca.display()
            )
        })?;

    let identity = ServerIdentity::from_parts(
        private_key_bytes.into(),
        certificate_bytes.into(),
    );
    let client_ca = client_ca_bytes.into();

    let gateway_reference =
        GatewayReference::new(&args.pod_namespace, &args.pod_name);

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

    let client = Client::try_default().await?;
    let api: Api<Device> = Api::namespaced(client.clone(), &args.namespace);

    let callbacks = Callbacks {
        api: api.clone(),
        gateway_reference: gateway_reference.clone(),
    };

    let config = ServerConfig {
        bind_addr: args.bind_addr,
        identity,
        client_ca,
        on_client_connect: Arc::from(callbacks.on_connect()),
        on_client_disconnect: Arc::from(callbacks.on_disconnect()),
        on_client_message: Arc::from(callbacks.on_message()),
        shutdown,
    };

    let server = Server::new(config);
    info!("Starting server on {}", args.bind_addr);
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
    }

    Ok(())
}
