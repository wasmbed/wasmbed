// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use anyhow::{Context, Result};

use wasmbed_cert::{DistinguishedName, DnType, ClientAuthority, ServerAuthority};

#[derive(Parser)]
#[command(disable_help_subcommand = true)]
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
        #[arg(long, help = "Output path for the private key (e.g., ca.key)")]
        out_key: PathBuf,
        #[arg(long, help = "Output path for the certificate (e.g., ca.der)")]
        out_cert: PathBuf,
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
        #[arg(
            long,
            help = "Output path for the private key (e.g., identity.key)"
        )]
        out_key: PathBuf,
        #[arg(
            long,
            help = "Output path for the certificate (e.g., identity.der)"
        )]
        out_cert: PathBuf,
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
            kind,
            out_key,
            out_cert,
            ..
        } => {
            let dn = build_distinguished_name(&cli.command);
            match kind {
                CertKind::Server => {
                    let cred = ServerAuthority::new(dn)?;
                    std::fs::write(
                        out_key,
                        cred.private_key().secret_pkcs8_der(),
                    )
                    .with_context(|| format!("failed to write {out_key:?}"))?;
                    std::fs::write(out_cert, cred.certificate()).with_context(
                        || format!("failed to write {out_cert:?}"),
                    )?;
                },
                CertKind::Client => {
                    let cred = ClientAuthority::new(dn)?;
                    std::fs::write(
                        out_key,
                        cred.private_key().secret_pkcs8_der(),
                    )
                    .with_context(|| format!("failed to write {out_key:?}"))?;
                    std::fs::write(out_cert, cred.certificate()).with_context(
                        || format!("failed to write {out_cert:?}"),
                    )?;
                },
            }
        },

        Command::IssueCert {
            kind,
            ca_key,
            ca_cert,
            out_key,
            out_cert,
            ..
        } => {
            let ca_der = std::fs::read(ca_cert).with_context(|| {
                format!("failed to read CA cert from {ca_cert:?}")
            })?;
            let key_der = std::fs::read(ca_key).with_context(|| {
                format!("failed to read CA key from {ca_key:?}")
            })?;
            let dn = build_distinguished_name(&cli.command);

            match kind {
                CertKind::Server => {
                    let ca = ServerAuthority::from_parts(
                        key_der.into(),
                        ca_der.into(),
                    );
                    let issued = ca.issue_certificate(dn)?;
                    std::fs::write(
                        out_key,
                        issued.private_key().secret_pkcs8_der(),
                    )
                    .with_context(|| format!("failed to write {out_key:?}"))?;
                    std::fs::write(out_cert, issued.certificate())
                        .with_context(|| {
                            format!("failed to write {out_cert:?}")
                        })?;
                },
                CertKind::Client => {
                    let ca = ClientAuthority::from_parts(
                        key_der.into(),
                        ca_der.into(),
                    );
                    let issued = ca.issue_certificate(dn)?;
                    std::fs::write(
                        out_key,
                        issued.private_key().secret_pkcs8_der(),
                    )
                    .with_context(|| format!("failed to write {out_key:?}"))?;
                    std::fs::write(out_cert, issued.certificate())
                        .with_context(|| {
                            format!("failed to write {out_cert:?}")
                        })?;
                },
            }
        },
    }

    Ok(())
}
