use std::borrow::Cow;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use kube::CustomResourceExt;

use wasmbed_k8s_resource::{Device, DeviceSpec};
use wasmbed_types::PublicKey;

#[derive(Parser)]
#[command(disable_help_subcommand = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate the CRD YAML for a given resource.
    #[command(name = "crd", subcommand)]
    GenerateCrd(Resource),
    /// Generate a resource manifest YAML.
    #[command(name = "manifest", subcommand)]
    GenerateManifest(ManifestResource),
}

#[derive(Subcommand)]
enum Resource {
    /// Generate the CRD YAML for the "Device" resource.
    Device,
}

#[derive(Subcommand)]
enum ManifestResource {
    /// Generate a manifest for the "Device" resource.
    Device {
        /// Metadata.name of the resource.
        #[arg(long)]
        name: String,
        /// Path to the device's public key in DER format.
        #[arg(long, value_name = "FILE")]
        public_key: PathBuf,
    },
}

pub fn main() -> Result<()> {
    use std::io::Write;

    let args = Args::parse();
    match args.command {
        Command::GenerateCrd(resource) => match resource {
            Resource::Device => {
                std::io::stdout().write_all(
                    &serde_yaml::to_string(&Device::crd())?.into_bytes(),
                )?;
            },
        },

        Command::GenerateManifest(resource) => match resource {
            ManifestResource::Device { name, public_key } => {
                let pubkey_bytes =
                    std::fs::read(&public_key).with_context(|| {
                        format!(
                            "Failed to read public key from {}",
                            public_key.display()
                        )
                    })?;

                let device = Device::new(
                    &name,
                    DeviceSpec {
                        public_key: PublicKey(Cow::Owned(pubkey_bytes)),
                    },
                );

                std::io::stdout()
                    .write_all(&serde_yaml::to_string(&device)?.into_bytes())?;
            },
        },
    };

    Ok(())
}
