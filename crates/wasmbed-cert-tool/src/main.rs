use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use anyhow::{Context, Result};

use wasmbed_cert::{DistinguishedName, DnType, ClientAuthority, ServerAuthority};

#[derive(Parser)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    GenerateCa {
        #[arg(value_enum)]
        kind: CertKind,
        #[arg(long)]
        common_name: String,
        #[arg(long)]
        organization: Option<String>,
        #[arg(long)]
        organizational_unit: Option<String>,
        #[arg(long)]
        country: Option<String>,
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        locality: Option<String>,
        #[arg(long, default_value = "ca")]
        out_prefix: String,
    },

    IssueCert {
        #[arg(value_enum)]
        kind: CertKind,
        #[arg(long)]
        ca_key: PathBuf,
        #[arg(long)]
        ca_cert: PathBuf,
        #[arg(long)]
        common_name: String,
        #[arg(long)]
        organization: Option<String>,
        #[arg(long)]
        organizational_unit: Option<String>,
        #[arg(long)]
        country: Option<String>,
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        locality: Option<String>,
        #[arg(long, default_value = "cert")]
        out_prefix: String,
    },
}

#[derive(ValueEnum, Clone)]
enum CertKind {
    Server,
    Client,
}

fn build_distinguished_name(args: &Command) -> DistinguishedName {
    let mut dn = DistinguishedName::new();
    match args {
        Command::GenerateCa {
            common_name,
            organization,
            organizational_unit,
            country,
            state,
            locality,
            ..
        }
        | Command::IssueCert {
            common_name,
            organization,
            organizational_unit,
            country,
            state,
            locality,
            ..
        } => {
            dn.push(DnType::CommonName, common_name);
            if let Some(v) = organization {
                dn.push(DnType::OrganizationName, v);
            }
            if let Some(v) = organizational_unit {
                dn.push(DnType::OrganizationalUnitName, v);
            }
            if let Some(v) = country {
                dn.push(DnType::CountryName, v);
            }
            if let Some(v) = state {
                dn.push(DnType::StateOrProvinceName, v);
            }
            if let Some(v) = locality {
                dn.push(DnType::LocalityName, v);
            }
        },
    }
    dn
}

fn main() -> Result<()> {
    let cli = Args::parse();

    match &cli.command {
        Command::GenerateCa {
            kind, out_prefix, ..
        } => {
            let dn = build_distinguished_name(&cli.command);
            match kind {
                CertKind::Server => {
                    let cred = ServerAuthority::new(dn)?;
                    fs::write(
                        format!("{}.key", out_prefix),
                        cred.key_pair_der(),
                    )
                    .with_context(|| {
                        format!("failed to write {}.key", out_prefix)
                    })?;
                    fs::write(
                        format!("{}.der", out_prefix),
                        cred.certificate_der(),
                    )
                    .with_context(|| {
                        format!("failed to write {}.der", out_prefix)
                    })?;
                },
                CertKind::Client => {
                    let cred = ClientAuthority::new(dn)?;
                    fs::write(
                        format!("{}.key", out_prefix),
                        cred.key_pair_der(),
                    )
                    .with_context(|| {
                        format!("failed to write {}.key", out_prefix)
                    })?;
                    fs::write(
                        format!("{}.der", out_prefix),
                        cred.certificate_der(),
                    )
                    .with_context(|| {
                        format!("failed to write {}.der", out_prefix)
                    })?;
                },
            }
        },

        Command::IssueCert {
            kind,
            ca_key,
            ca_cert,
            out_prefix,
            ..
        } => {
            let ca_der = fs::read(ca_cert).with_context(|| {
                format!("failed to read CA cert from {:?}", ca_cert)
            })?;
            let key_der = fs::read(ca_key).with_context(|| {
                format!("failed to read CA key from {:?}", ca_key)
            })?;
            let dn = build_distinguished_name(&cli.command);

            match kind {
                CertKind::Server => {
                    let ca = ServerAuthority::from_der(&key_der, &ca_der)?;
                    let issued = ca.issue_certificate(dn)?;
                    fs::write(
                        format!("{}.key", out_prefix),
                        issued.key_pair_der(),
                    )
                    .with_context(|| {
                        format!("failed to write {}.key", out_prefix)
                    })?;
                    fs::write(
                        format!("{}.der", out_prefix),
                        issued.certificate_der(),
                    )
                    .with_context(|| {
                        format!("failed to write {}.der", out_prefix)
                    })?;
                },
                CertKind::Client => {
                    let ca = ClientAuthority::from_der(&key_der, &ca_der)?;
                    let issued = ca.issue_certificate(dn)?;
                    fs::write(
                        format!("{}.key", out_prefix),
                        issued.key_pair_der(),
                    )
                    .with_context(|| {
                        format!("failed to write {}.key", out_prefix)
                    })?;
                    fs::write(
                        format!("{}.der", out_prefix),
                        issued.certificate_der(),
                    )
                    .with_context(|| {
                        format!("failed to write {}.der", out_prefix)
                    })?;
                },
            }
        },
    }

    Ok(())
}
