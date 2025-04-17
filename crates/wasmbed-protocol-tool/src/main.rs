use std::io::Read;

use anyhow::{Result, Context};
use clap::{Parser, ValueEnum};
use minicbor::decode;

use wasmbed_protocol::types::Envelope;

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, value_enum, help = "Input format")]
    format: Format,
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

fn main() -> Result<()> {
    let cli = Args::parse();

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)
        .context("could not read from stdin")?;

    let bytes = match &cli.format {
        Format::Hexadecimal =>
            hex::decode(input.trim())
                .context("coult not parse hexadecimal input")?,
        Format::Decimal => input
            .split_whitespace()
            .map(
                |s|
                s.parse::<u8>()
                 .context(format!("coult not parse decimal value: {}", s))
            )
            .collect::<Result<Vec<u8>>>()?,
        Format::Binary => input.trim().into(),
    };

    let decoded = decode::<Envelope>(&bytes)
        .context("coult not decode envelope")?;

    println!("{:#?}", decoded);

    Ok(())
}
