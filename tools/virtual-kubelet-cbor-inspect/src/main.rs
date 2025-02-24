use clap::{Parser, ValueEnum};
use virtual_kubelet_client::{Envelope, decode};

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

fn main() {
    let cli = Args::parse();
    let input = std::io::read_to_string(std::io::stdin()).unwrap();

    let bytes = match &cli.format {
        Format::Hexadecimal =>
            hex::decode(input.trim()).unwrap(),
        Format::Decimal => input
            .split_whitespace()
            .map(|s| s.parse::<u8>().unwrap())
            .collect(),
        Format::Binary => input.trim().into(),
    };

    let decoded = decode::<Envelope>(&bytes).unwrap();
    println!("{:#?}", decoded);
}
