use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg()]
    pub file: PathBuf,
}

pub fn parse() -> Args {
    Args::parse()
}
