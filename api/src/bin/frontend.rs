use clap::Parser;
use metta_kg::cli::Cli;
use rocket::fs::FileServer;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let cli = Cli::parse();

    let address: std::net::IpAddr = cli.address.parse().unwrap_or_else(|_| {
        eprintln!("Invalid --address: must be a numeric IP like 127.0.0.1");
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
    });

    let port = cli.port.unwrap_or(3000);

    let figment = rocket::Config::figment()
        .merge(("address", address))
        .merge(("port", port));

    println!("Serving frontend at http://{}:{}", cli.address, port);

    rocket::custom(figment)
        .mount("/", FileServer::from("ui-dist"))
        .launch()
        .await?;

    Ok(())
}
