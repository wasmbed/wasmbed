use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use kube::CustomResourceExt;
use rustls_pki_types::CertificateDer;

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
        /// Path to the device's certificate in DER format.
        #[arg(long = "cert", value_name = "FILE")]
        certificate: PathBuf,
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
            ManifestResource::Device { name, certificate } => {
                let cert_bytes =
                    std::fs::read(&certificate).with_context(|| {
                        format!(
                            "Failed to read certificate from {}",
                            certificate.display()
                        )
                    })?;
                let cert = CertificateDer::from_slice(&cert_bytes);
                let public_key: PublicKey = (&cert).try_into()?;

                let device = Device::new(
                    &name,
                    DeviceSpec {
                        public_key: public_key.into_owned(),
                    },
                );

                std::io::stdout()
                    .write_all(&serde_yaml::to_string(&device)?.into_bytes())?;
            },
        },
    };

    Ok(())
}
