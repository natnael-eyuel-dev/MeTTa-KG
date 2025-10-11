use diesel::pg::PgConnection;
use diesel::Connection;
use once_cell::sync::OnceCell;
use std::env;

static DATABASE_URL: OnceCell<String> = OnceCell::new();

pub fn init_database_url(cli_url: Option<String>) {
    if let Some(url) = cli_url {
        DATABASE_URL
            .set(url)
            .expect("DATABASE_URL already initialized");
    }
}

pub fn establish_connection() -> PgConnection {
    let url = DATABASE_URL.get().cloned().unwrap_or_else(|| {
        let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
        let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
        let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        format!(
            "postgresql://{}:{}@{}:5432/{}",
            user, password, host, db_name
        )
    });

    PgConnection::establish(&url).unwrap_or_else(|e| panic!("Error connecting to {}: {}", url, e))
}
