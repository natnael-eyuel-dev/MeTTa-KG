use httpmock::prelude::*;
use metta_kg::cli::AppConfig;
use metta_kg::rocket;
use metta_kg::routes::spaces::SetOperationInput;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use rocket::serde::json::serde_json;
use serial_test::serial;
use std::env;

#[path = "common.rs"]
mod common;
//
#[tokio::test]
#[serial]
async fn test_subspace_transform() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    // Expect the fixed-prefix transform to be sent to MORK.
    let transform_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/transform")
            .body_contains("(__animaldata__ (path Foo Baz $x))")
            .body_contains("(__outdata__ $x)");
        then.status(200).body("ok");
    });
    common::setup(&server.base_url());
    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };
    let token = common::create_test_token("/test/subspace/", true, true);
    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&SetOperationInput {
        source: vec![
            "test/subspace/animal/".to_string(),
            "path Foo Baz".to_string(),
        ],
        target: vec!["test/subspace/out/".to_string()],
    })
    .unwrap();

    let response = client
        .post("/api/spaces/subspace")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert_eq!(body, "true");
    assert_eq!(transform_mock.hits(), 1);

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_subspace_endpoint_happy_path() {
    if !common::is_database_running() {
        return;
    }
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("ok");
    });
    common::setup(&server.base_url());
    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };
    let token = common::create_test_token("/test/subspace/", true, true);
    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("rocket instance");
    let body = serde_json::to_string(&SetOperationInput {
        source: vec![
            "test/subspace/animal/".to_string(),
            "path Foo Baz".to_string(),
        ],
        target: vec!["test/subspace/out/".to_string()],
    })
    .unwrap();
    let response = client
        .post("/api/spaces/subspace")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_subspace_endpoint_leading_slash_normalization() {
    if !common::is_database_running() {
        return;
    }
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("ok");
    });
    common::setup(&server.base_url());
    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };
    let token = common::create_test_token("/test/subspace/", true, true);
    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("rocket instance");
    let body = serde_json::to_string(&SetOperationInput {
        source: vec![
            "/test/subspace/animal/".to_string(),
            "path Foo Baz".to_string(),
        ],
        target: vec!["/test/subspace/out/".to_string()],
    })
    .unwrap();
    let response = client
        .post("/api/spaces/subspace")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_subspace_endpoint_empty_prefix() {
    if !common::is_database_running() {
        return;
    }
    let server = MockServer::start();
    let transform_mock = server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("ok");
    });
    common::setup(&server.base_url());
    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };
    let token = common::create_test_token("/test/subspace/", true, true);
    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("rocket instance");
    let body = serde_json::to_string(&SetOperationInput {
        source: vec!["test/subspace/animal/".to_string(), "".to_string()],
        target: vec!["test/subspace/out/".to_string()],
    })
    .unwrap();
    let response = client
        .post("/api/spaces/subspace")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(transform_mock.hits(), 0);
    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_subspace_endpoint_unauthorized_namespace() {
    if !common::is_database_running() {
        return;
    }
    let server = MockServer::start();
    let transform_mock = server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("ok");
    });
    common::setup(&server.base_url());
    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };
    let token = common::create_test_token("/test/subspace/", true, true);
    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("rocket instance");
    let body = serde_json::to_string(&SetOperationInput {
        source: vec![
            "other/space/animal/".to_string(),
            "path Foo Baz".to_string(),
        ],
        target: vec!["test/subspace/out/".to_string()],
    })
    .unwrap();
    let response = client
        .post("/api/spaces/subspace")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Unauthorized);
    assert_eq!(transform_mock.hits(), 0);
    common::teardown_database();
}
