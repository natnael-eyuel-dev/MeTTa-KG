mod cli;
mod config_ui;

use clap::Parser;
use config_ui::{launch_config_server, ConfigPageData};
use metta_kg::{cli::Cli, db, rocket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = Cli::parse();

    let all_provided = cli.database_url.is_some()
        && cli.mork_server_url.is_some()
        && cli.mettakg_api_url.is_some();

    if !all_provided {
        let needs_config = cli.database_url.is_none()
            || cli.mork_server_url.is_none()
            || cli.mettakg_api_url.is_none();

        if needs_config {
            println!("Launching configuration web interface at http://127.0.0.1:8080");
            println!("Please open your browser and configure the server.");

            let preset_config = ConfigPageData {
                database_url: cli.database_url.clone(),
                mettakg_api_url: cli.mettakg_api_url.clone(),
                mork_server_url: cli.mork_server_url.clone(),
                error: None,
            };

            let config_cli = launch_config_server(preset_config).await;

            cli.database_url = cli.database_url.or(config_cli.database_url);
            cli.mork_server_url = cli.mork_server_url.or(config_cli.mork_server_url);
            cli.mettakg_api_url = cli.mettakg_api_url.or(config_cli.mettakg_api_url);
        }
    }

    let database_url = cli.database_url.expect("Database URL must be provided");
    let mork_server_url = cli
        .mork_server_url
        .unwrap_or_else(|| "http://127.0.0.1:8001".to_string());
    let mettakg_api_url = cli
        .mettakg_api_url
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());

    let final_cli = Cli {
        database_url: Some(database_url.clone()),
        mork_server_url: Some(mork_server_url),
        mettakg_api_url: Some(mettakg_api_url),
    };

    db::init_database_url(database_url);
    let rocket_instance = rocket(&final_cli).await;
    rocket_instance.launch().await?;
    Ok(())
}
