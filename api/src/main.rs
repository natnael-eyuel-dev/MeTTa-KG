mod cli;

use clap::Parser;
use metta_kg::{db, rocket};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let cli = metta_kg::cli::Cli::parse();
    db::init_database_url(cli.database_url.clone());
    let rocket_instance = rocket(&cli).await;
    rocket_instance.launch().await?;

    Ok(())
}
