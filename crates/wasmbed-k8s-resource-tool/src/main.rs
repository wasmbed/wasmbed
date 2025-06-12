use anyhow::Result;
use clap::{Parser, Subcommand};

use wasmbed_k8s_resource::Device;

#[derive(Parser)]
#[command(version, disable_help_subcommand = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(name = "crd", subcommand)]
    Generate(Resource),
}

#[derive(Subcommand)]
enum Resource {
    /// Generate the CRD YAML for the "Device" resource.
    Device,
}

pub fn main() -> Result<()> {
    use std::io::Write;
    use kube::CustomResourceExt;

    let cli = Args::parse();
    match cli.command {
        Command::Generate(Resource::Device) => std::io::stdout()
            .write_all(&serde_yaml::to_string(&Device::crd())?.into_bytes())?,
    };

    Ok(())
}
