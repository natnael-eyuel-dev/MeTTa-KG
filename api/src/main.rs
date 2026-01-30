use clap::Parser;
use metta_kg::{
    cli::{AppConfig, Cli},
    db, launch_setup_server, rocket,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let setup_config = launch_setup_server(cli.mettakg_api_url.clone()).await;

    let raw_api_url = cli.mettakg_api_url.unwrap_or(setup_config.mettakg_api_url);

    let mettakg_api_url = if !raw_api_url.contains("://") {
        format!("http://{}", raw_api_url)
    } else {
        raw_api_url
    };

    let final_config = AppConfig {
        database_url: setup_config.database_url,
        mork_server_url: setup_config.mork_server_url,
        mettakg_api_url,
    };

    db::init_database_url(final_config.database_url.clone());
    let rocket_instance = rocket(&final_config).await;
    rocket_instance.launch().await?;
    Ok(())
}
