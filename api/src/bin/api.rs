use clap::Parser;
use metta_kg::{cli::Cli, db, rocket};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    db::init_database_url(cli.database_url());

    let rocket = rocket(&cli);
    rocket.launch().await?;

    Ok(())
}
