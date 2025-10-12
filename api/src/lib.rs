pub mod cli;
pub mod db;
pub mod model;
pub mod mork_api;
pub mod routes;
pub mod schema;

use crate::cli::Cli;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket::fs::FileServer;
use rocket::{http::Method, routes, Build, Rocket};
use rocket_cors::AllowedOrigins;
use std::io::Write;
use std::{env, fs};
use tempfile::Builder;
use tokio::time::Duration;
use url::Url;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
const MORK_BYTES: &[u8] = include_bytes!(env!("MORK_BINARY_PATH"));

async fn spawn_mork_server(mork_url: &str) {
    let url = url::Url::parse(mork_url).expect("Invalid Mork server URL");
    let port = url.port().expect("URL must include a port").to_string();

    let temp_file = Builder::new()
        .prefix("mork_server_")
        .suffix(if cfg!(windows) { ".exe" } else { "" })
        .tempfile()
        .expect("Failed to create temporary file");

    temp_file
        .as_file()
        .write_all(MORK_BYTES)
        .expect("Failed to write mork binary to temp file");

    let temp_path = temp_file.into_temp_path();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)
            .expect("Failed to get file metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms).expect("Failed to set executable permissions");
    }

    let mork_path_str = temp_path.to_str().expect("Invalid UTF-8 in path");
    println!(
        "Starting Mork server from {} on port {}",
        mork_path_str, port
    );

    tokio::spawn(async move {
        let mut cmd = tokio::process::Command::new(&temp_path);
        cmd.env("MORK_SERVER_PORT", port);

        match cmd.spawn() {
            Ok(mut child) => {
                println!("Mork server started with PID: {:?}", child.id());
                if let Err(e) = child.wait().await {
                    eprintln!("Mork server process failed: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to start Mork server: {e}");
            }
        }
    });

    println!("Waiting for Mork server to start...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    env::set_var("METTA_KG_MORK_URL", mork_url);
}

fn build_rocket(cfg: &Cli) -> Rocket<Build> {
    dotenv::dotenv().ok();

    let mut connection = db::establish_connection();
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "https://metta-kg.vercel.app",
        "http://127.0.0.1:3000",
        "http://127.0.0.1:8000",
    ]);

    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete, Method::Options]
            .into_iter()
            .map(From::from)
            .collect(),
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    let url = Url::parse(
        cfg.mettakg_api_url
            .as_deref()
            .expect("Missing --mettakg_api_url"),
    )
    .expect("Invalid -m mettakg_api_url");

    let host: std::net::IpAddr = url
        .host_str()
        .unwrap_or("127.0.0.1")
        .parse()
        .expect("Invalid host IP address");

    let port = url.port().unwrap_or(8000);

    println!("MeTTa-KG server starting at {}:{}", host, port);

    let figment = rocket::Config::figment()
        .merge(("address", host))
        .merge(("port", port));

    rocket::custom(figment)
        .mount(
            "/",
            routes![
                routes::translations::create_from_csv,
                routes::translations::create_from_nt,
                routes::translations::create_from_jsonld,
                routes::translations::create_from_n3,
                routes::tokens::get_all,
                routes::tokens::get,
                routes::tokens::create,
                routes::tokens::update,
                routes::tokens::delete,
                routes::tokens::delete_batch,
                routes::spaces::read,
                routes::spaces::upload,
                routes::spaces::import,
                routes::spaces::transform,
                routes::spaces::explore,
                routes::spaces::export,
                routes::spaces::clear,
            ],
        )
        .attach(cors.clone())
        .manage(cors)
        .mount("/", FileServer::from("ui-dist"))
}

pub async fn rocket(cfg: &Cli) -> Rocket<Build> {
    let mork_url = cfg.mork_server_url.as_ref().unwrap();
    spawn_mork_server(mork_url).await;
    build_rocket(cfg)
}
