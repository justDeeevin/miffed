use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg()]
    pub file: PathBuf,
    #[arg(short, long)]
    /// Enable branch delay slot
    pub delay_slot: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
