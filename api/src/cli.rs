use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(name = "metta-kg", version, about = "MeTTa-KG Server/Frontend")]
pub struct Cli {
    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long)]
    pub address: String,

    #[arg(long)]
    pub port: Option<u16>,
}

impl Cli {
    pub fn database_url(&self) -> Option<String> {
        self.database_url
            .clone()
            .or_else(|| env::var("DATABASE_URL").ok())
    }
}
