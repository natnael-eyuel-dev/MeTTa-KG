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
use url::Url;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn rocket(cfg: &Cli) -> Rocket<Build> {
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
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete]
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
