mod device;

use crate::device::Device;
use clap::{Parser, Subcommand};

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
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::GenerateCrd(resource) => generate_crd(resource),
        Command::RunController => run_controller().await,
    };
}

fn generate_crd(resource: Resource) {
    use std::io::{stdout, Write};
    use kube::CustomResourceExt;

    let definition = match resource {
        Resource::Device => Device::crd(),
    };

    let yaml = serde_yaml::to_string(&definition).unwrap();
    stdout().write_all(&yaml.into_bytes()).unwrap();
}

async fn run_controller() {
    use crate::device::{reconcile_device, on_reconcile_device_error};

    use std::sync::Arc;

    use futures::StreamExt;
    use kube::api::Api;
    use kube::client::Client;
    use kube::runtime::Controller;

    let client = Client::try_default().await.unwrap();
    let client_arc = Arc::new(client.clone());

    let devices: Api<Device> = Api::all(client.clone());

    let device_controller = Controller::new(devices, Default::default())
        .run(reconcile_device, on_reconcile_device_error, Arc::clone(&client_arc))
        .for_each(|_| futures::future::ready(()));

    device_controller.await;
}
