mod device;

use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use kube::{Api, Client};
use kube::runtime::Controller;
use futures::StreamExt;

use crate::device::Device;

#[derive(Parser)]
#[command(version, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a Custom Resource Definition (CRD) YAML.
    #[command(name = "crd", subcommand)]
    GenerateCrd(Resource),

    /// Run the operator controller.
    #[command(name = "controller")]
    RunController,
}

#[derive(Subcommand)]
enum Resource {
    /// Generate the CRD YAML for the "Device" resource.
    Device,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    use std::io::Write;

    let cli = Cli::parse();
    match cli.command {
        Command::GenerateCrd(resource) => std::io::stdout()
            .write_all(&generate_crd(resource)?.into_bytes())?,
        Command::RunController => {
            let client = Client::try_default().await?;
            let device_api: Api<Device> = Api::all(client.clone());
            run_controller(device_api).await;
        },
    };

    Ok(())
}

fn generate_crd(resource: Resource) -> Result<String, serde_yaml::Error> {
    use kube::CustomResourceExt;

    let definition = match resource {
        Resource::Device => Device::crd(),
    };

    serde_yaml::to_string(&definition)
}

async fn run_controller(api: Api<Device>) {
    use crate::device::{reconcile_device, on_reconcile_device_error};

    let context = Arc::new(());

    Controller::new(api, Default::default())
        .run(
            reconcile_device,
            on_reconcile_device_error,
            Arc::clone(&context),
        )
        .for_each(|_| futures::future::ready(()))
        .await
}
