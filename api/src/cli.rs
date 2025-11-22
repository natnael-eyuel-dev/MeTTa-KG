use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "metta-kg", version, about = "MeTTa-KG Server/Frontend")]
pub struct Cli {
    #[arg(long)]
    #[cfg_attr(
        feature = "sqlite",
        arg(help = "Database file path (e.g., metta_kg.db)")
    )]
    #[cfg_attr(
        feature = "postgres",
        arg(help = "PostgreSQL connection URL (e.g., postgres://user:pass@localhost/dbname)")
    )]
    pub database_url: Option<String>,

    #[arg(long)]
    pub mettakg_frontend_url: Option<String>,

    #[arg(long)]
    pub mork_server_url: Option<String>,

    #[arg(long)]
    pub mettakg_api_url: Option<String>,
}
