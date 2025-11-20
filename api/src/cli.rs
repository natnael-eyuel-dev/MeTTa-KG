use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "metta-kg", version, about = "MeTTa-KG Server/Frontend")]
pub struct Cli {
    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long)]
    pub mettakg_frontend_url: Option<String>,

    #[arg(long)]
    pub mork_server_url: Option<String>,

    #[arg(long)]
    pub mettakg_api_url: Option<String>,
}
