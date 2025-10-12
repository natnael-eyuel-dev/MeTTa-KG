use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(name = "metta-kg", version, about = "MeTTa-KG Server/Frontend")]
pub struct Cli {
    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long, default_value = "http://127.0.0.1:3000")]
    pub mettakg_frontend_url: Option<String>,

    #[arg(long, default_value = "http://127.0.0.1:8001")]
    pub mork_server_url: Option<String>,

    #[arg(long, default_value = "http://127.0.0.1:8000")]
    pub mettakg_api_url: Option<String>,
}

impl Cli {
    pub fn database_url(&self) -> Option<String> {
        self.database_url
            .clone()
            .or_else(|| env::var("DATABASE_URL").ok())
    }
}
