use rocket::http::Method;
use rocket::{self, launch, routes, Build, Rocket};
use rocket_cors::AllowedOrigins;

mod db;
mod model;
mod mork_api;
mod routes;
mod schema;
mod static_files;

#[launch]
async fn rocket() -> Rocket<Build> {
    // TODO: move hardcoded allowed origins to database,
    // or get backend and frontend hosted under same domain

    dotenv::dotenv().ok();

    tokio::spawn(async {
        let mut cmd = tokio::process::Command::new("./mork_server");
        cmd.env("MORK_SERVER_PORT", "8001");

        match cmd.spawn() {
            Ok(mut child) => {
                if let Err(e) = child.wait().await {
                    eprintln!("MORK server process failed: {e}");
                }
            }
            Err(e) => eprintln!("Failed to start MORK server: {e}"),
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    std::env::set_var("METTA_KG_MORK_URL", "http://localhost:8001");

    let allowed_origins =
        AllowedOrigins::some_exact(&["http://localhost:8000", "http://127.0.0.1:8000"]);

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

    rocket::build()
        .mount(
            "/api",
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
        // .mount("/public", FileServer::from("static"))
        .mount("/", static_files::routes())
        .attach(cors.clone())
        .manage(cors)
}
