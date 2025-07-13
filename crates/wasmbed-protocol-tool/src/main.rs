// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use anyhow::{Result, Context};
use clap::{Parser, ValueEnum};
use minicbor::decode;
use wasmbed_protocol::{ClientEnvelope, ServerEnvelope};

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, value_enum, help = "Input format")]
    format: Format,

    #[arg(long, value_enum, help = "Message type to decode")]
    message_type: MessageType,
}

#[derive(Clone, ValueEnum)]
enum Format {
    #[clap(name = "dec")]
    Decimal,
    #[clap(name = "hex")]
    Hexadecimal,
    #[clap(name = "bin")]
    Binary,
}

#[derive(Clone, ValueEnum)]
enum MessageType {
    #[clap(name = "client")]
    Client,
    #[clap(name = "server")]
    Server,
}

fn main() -> Result<()> {
    use std::io::Read;

    let cli = Args::parse();

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("could not read from stdin")?;

    let bytes = match &cli.format {
        Format::Hexadecimal => hex::decode(input.trim())
            .context("could not parse hexadecimal input")?,
        Format::Decimal => input
            .split_whitespace()
            .map(|s| {
                s.parse::<u8>()
                    .context(format!("could not parse decimal value: {s}"))
            })
            .collect::<Result<Vec<u8>>>()?,
        Format::Binary => input.trim().as_bytes().to_vec(),
    };

    match cli.message_type {
        MessageType::Client => {
            let decoded: ClientEnvelope =
                decode(&bytes).context("could not decode client envelope")?;
            println!("{decoded:#?}");
        },
        MessageType::Server => {
            let decoded: ServerEnvelope =
                decode(&bytes).context("could not decode server envelope")?;
            println!("{decoded:#?}");
        },
    }

    Ok(())
}
