mod cli;

use clap::Parser;
use metta_kg::{db, rocket};

#[rocket::launch]
fn rocket_main() -> _ {
    let cli = metta_kg::cli::Cli::parse();
    db::init_database_url(cli.database_url());
    rocket(&cli)
}
