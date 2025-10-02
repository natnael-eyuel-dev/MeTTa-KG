use diesel::pg::PgConnection;
use diesel::Connection;
use std::env;
use std::fs;

fn is_running_in_docker() -> bool {
    // Explicit overrides
    if env::var("DOCKER").is_ok() || env::var("METTA_KG_IN_DOCKER").is_ok() {
        return true;
    }
    // /.dockerenv exists in Docker containers (Linux)
    if std::path::Path::new("/.dockerenv").exists() {
        return true;
    }
    // cgroup inspection (Linux)
    if let Ok(cgroup) = fs::read_to_string("/proc/1/cgroup") {
        let s = cgroup.to_lowercase();
        if s.contains("docker") || s.contains("containerd") || s.contains("kubepods") {
            return true;
        }
    }
    false
}

pub fn resolve_database_url() -> String {
    // Highest priority: explicit env var
    if let Ok(url) = env::var("METTA_KG_DATABASE_URL") {
        return url;
    }
    // If running in Docker (compose), prefer the internal hostname 'db'
    if is_running_in_docker() {
        return "postgresql://metta-kg-admin:password123@db:5432/metta-kg".to_string();
    }
    // Local default for binary users
    "postgresql://metta-kg-admin:password123@localhost:5432/metta-kg".to_string()
}

pub fn establish_connection() -> PgConnection {
    let database_url = resolve_database_url();

    PgConnection::establish(&database_url)
        .unwrap_or_else(|e| panic!("Error connecting to {database_url} {e}"))
}
