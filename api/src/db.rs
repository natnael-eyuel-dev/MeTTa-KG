#[cfg(feature = "sqlite")]
use diesel::sqlite::SqliteConnection as DbConnection;

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use diesel::pg::PgConnection as DbConnection;

use diesel::Connection;
use std::env;
use std::sync::OnceLock;

static DATABASE_URL: OnceLock<String> = OnceLock::new();

pub fn init_database_url(url: String) {
    DATABASE_URL
        .set(url)
        .expect("DATABASE_URL already initialized");
}

pub fn establish_connection() -> DbConnection {
    let url = DATABASE_URL.get().cloned().unwrap_or_else(|| {
        #[cfg(feature = "sqlite")]
        {
            env::var("DATABASE_URL").unwrap_or_else(|_| "metta_kg.db".to_string())
        }

        #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
        {
            let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
            let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
            let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
            let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                user, password, host, port, db_name
            )
        }
    });

    #[allow(unused_mut)]
    let mut conn = DbConnection::establish(&url)
        .unwrap_or_else(|e| panic!("Error connecting to {}: {}", url, e));

    #[cfg(feature = "sqlite")]
    {
        use diesel::connection::SimpleConnection;
        conn.batch_execute("PRAGMA foreign_keys = ON").unwrap();
    }

    conn
}
