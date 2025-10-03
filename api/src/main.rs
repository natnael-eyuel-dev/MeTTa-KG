use std::net::IpAddr;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use mime_guess::from_path;
use rocket::fairing::AdHoc;
use rocket::http::{ContentType, Method};
use rocket::{self, get, routes, Build, Config, Rocket, State};
use rocket_cors::AllowedOrigins;
use rust_embed::RustEmbed;
use rocket::{catch, catchers};

mod db;
mod model;
mod mork_api;
mod routes;
mod schema;

// embed the UI assets
#[derive(RustEmbed)]
#[folder = "ui-dist/"]
struct UiAssets;

#[derive(Debug, Clone)]
struct UiConfig {
    base_path: String,
}

#[derive(Parser, Debug)]
#[command(name = "metta-kg", version, about = "MeTTa-KG: single-binary server", long_about = None)]
struct Cli {
    /// address to bind HTTP server
    #[arg(long, default_value = "127.0.0.1")]
    address: String,

    /// port to bind HTTP server
    #[arg(long, default_value_t = 3030)]
    port: u16,

    /// serve UI under a base path 
    #[arg(long, default_value = "/")]
    base_path: String,

    /// do not open a browser window on startup
    #[arg(long, default_value_t = false)]
    no_browser: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// run the server (default)
    Run,

    /// import data 
    Import {
        /// path or URI
        #[arg()]
        _source: Option<String>,
    },

    /// query data 
    Query {
        #[arg()]
        _query: Option<String>,
    },
}

#[get("/")]
fn index(ui: &State<UiConfig>) -> Option<(ContentType, Vec<u8>)> {
    // load index.html and adjust paths if base_path is set
    let file = UiAssets::get("index.html")?;
    let mut html = String::from_utf8(file.data.to_vec()).ok()?;
    let base = ui.base_path.trim_end_matches('/');
    if !base.is_empty() && base != "/" {
        let prefix = base.to_string();
        html = html.replace("href=\"/assets", &format!("href=\"{}/assets", prefix));
        html = html.replace("src=\"/assets", &format!("src=\"{}/assets", prefix));
        html = html.replace("href=\"/", &format!("href=\"{}/", prefix));
        html = html.replace("src=\"/", &format!("src=\"{}/", prefix));
    }
    Some((ContentType::HTML, html.into_bytes()))
}

#[get("/assets/<path..>")]
fn asset(path: PathBuf, _ui: &State<UiConfig>) -> Option<(ContentType, Vec<u8>)> {
    let path_str = Path::new("assets").join(path);
    ui_asset_response(path_str.to_string_lossy().as_ref())
}

// fallback to index.html for SPA routes
#[get("/<_path..>", rank = 255)]
fn spa_fallback(_path: PathBuf, _ui: &State<UiConfig>) -> Option<(ContentType, Vec<u8>)> {
    ui_asset_response("index.html")
}

fn ui_asset_response(path: &str) -> Option<(ContentType, Vec<u8>)> {
    let normalized = path.trim_start_matches('/');
    let file = UiAssets::get(normalized)?;
    let mime = from_path(normalized).first_or_octet_stream();
    let ct = rocket::http::ContentType::parse_flexible(mime.as_ref())
        .unwrap_or(rocket::http::ContentType::Binary);
    Some((ct, file.data.into_owned()))
}

fn build_cors() -> rocket_cors::Cors {
    // define allowed origins
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "http://127.0.0.1:3000",
        "http://localhost:3030",
        "http://127.0.0.1:3030",
        "https://metta-kg.vercel.app",
    ]);

    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        ..Default::default()
    }
    .to_cors()
    .expect("CORS configuration invalid")
}

#[catch(401)]
fn unauthorized() -> &'static str {
    "Unauthorized: token missing or invalid"
}

#[catch(403)]
fn forbidden() -> &'static str {
    "CORS Forbidden"
}

#[catch(422)]
fn unprocessable() -> &'static str {
    "Unprocessable Entity"
}

async fn build_rocket(cfg: &Cli) -> Rocket<Build> {
    let address: IpAddr = cfg
        .address
        .parse()
        .expect("--address must be a valid IP address");

    let figment = Config::figment()
        .merge(("address", address))
        .merge(("port", cfg.port));

    let cors = build_cors();
    let ui_base = cfg.base_path.clone();

    rocket::custom(figment)
        .attach(cors.clone())
        .register("/", catchers![forbidden, unprocessable, unauthorized])
        .attach(AdHoc::on_ignite("prepare-fs", |rocket| Box::pin(async move {
            let _ = std::fs::create_dir_all("temp");
            rocket
        })))
        .manage(cors)
        .manage(UiConfig {
            base_path: cfg.base_path.clone(),
        })
        .mount(
            "/",
            routes![
                // API routes
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
                routes::spaces::status_sse,
                routes::spaces::status,
                routes::spaces::explore,
                routes::spaces::export,
                routes::spaces::clear,
            ],
        )
        .mount(ui_base, routes![index, asset, spa_fallback])
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv::dotenv().ok();

    let cfg = Cli::parse();

    // default to "Run" if no subcommand is provided
    let command = cfg.command.as_ref().unwrap_or(&Commands::Run);

    match command {
        Commands::Run => {
            let url = format!("http://{}:{}", cfg.address, cfg.port);
            if !cfg.no_browser {
                // open the browser
                let _ = open::that_detached(&url);
            }

            let rocket = build_rocket(&cfg).await;
            rocket.launch().await?;
        }
        Commands::Import { .. } => {
            eprintln!("import subcommand not yet implemented");
        }
        Commands::Query { .. } => {
            eprintln!("query subcommand not yet implemented");
        }
    }

    Ok(())
}
