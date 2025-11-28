use clap::Parser;
use httpmock::prelude::*;
use httpmock::Regex;
use metta_kg::rocket;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use serial_test::serial;

#[path = "common.rs"]
mod common;
// use crate::common;

#[tokio::test]
#[serial]
async fn test_clear_success() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    // Mock clear request
    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/clear/.*").unwrap());
        then.status(200).body("Clear successful");
    });

    let mut cli = metta_kg::cli::Cli::parse();
    cli.mork_server_url = Some(server.base_url());
    cli.mettakg_api_url = Some("http://127.0.0.1:8000".to_string());
    // Create client
    let client = Client::tracked(rocket(&cli).await)
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/api/spaces/clear/test/space?expr=")
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert_eq!(body, "true");

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_non_existent_namespace() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    let mut cli = metta_kg::cli::Cli::parse();
    cli.mork_server_url = Some(server.base_url());
    cli.mettakg_api_url = Some("http://127.0.0.1:8000".to_string());
    // Create client
    let client = Client::tracked(rocket(&cli).await)
        .await
        .expect("valid rocket instance");

    // Path does not start with /test/
    let response = client
        .post("/api/spaces/clear/other/space?expr=$x")
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Unauthorized);

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_existing_empty_namespace() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    // Mock clear request
    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/clear/.*").unwrap());
        then.status(200).body("Clear successful");
    });

    let mut cli = metta_kg::cli::Cli::parse();
    cli.mork_server_url = Some(server.base_url());
    cli.mettakg_api_url = Some("http://127.0.0.1:8000".to_string());
    // Create client
    let client = Client::tracked(rocket(&cli).await)
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/api/spaces/clear/test/space?expr=$x")
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert_eq!(body, "true");

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_non_empty_namespace() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    // Mock clear request
    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/clear/.*").unwrap());
        then.status(200).body("Clear successful");
    });

    let mut cli = metta_kg::cli::Cli::parse();
    cli.mork_server_url = Some(server.base_url());
    cli.mettakg_api_url = Some("http://127.0.0.1:8000".to_string());
    // Create client
    let client = Client::tracked(rocket(&cli).await)
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/api/spaces/clear/test/space?expr=$x")
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert_eq!(body, "true");

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_different_namespaces() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    common::setup(&server.base_url());

    let token1 = common::create_test_token("/ns1/", true, true);
    let token2 = common::create_test_token("/ns2/", true, true);

    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/clear/.*").unwrap());
        then.status(200).body("Clear successful");
    });

    let mut cli = metta_kg::cli::Cli::parse();
    cli.mork_server_url = Some(server.base_url());
    cli.mettakg_api_url = Some("http://127.0.0.1:8000".to_string());
    // Create client
    let client = Client::tracked(rocket(&cli).await)
        .await
        .expect("valid rocket instance");

    // Clear in ns1
    let response1 = client
        .post("/api/spaces/clear/ns1/space?expr=$x")
        .header(Header::new("authorization", token1.code.clone()))
        .dispatch()
        .await;
    assert_eq!(response1.status(), Status::Ok);

    // Clear in ns2
    let response2 = client
        .post("/api/spaces/clear/ns2/space?expr=$x")
        .header(Header::new("authorization", token2.code.clone()))
        .dispatch()
        .await;
    assert_eq!(response2.status(), Status::Ok);

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_namespace_mismatch() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    let mut cli = metta_kg::cli::Cli::parse();
    cli.mork_server_url = Some(server.base_url());
    cli.mettakg_api_url = Some("http://127.0.0.1:8000".to_string());
    // Create client
    let client = Client::tracked(rocket(&cli).await)
        .await
        .expect("valid rocket instance");

    // Path does not start with /test/
    let response = client
        .post("/api/spaces/clear/other/space?expr=$x")
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Unauthorized);

    common::teardown_database();
}
