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

#[tokio::test]
#[serial]
async fn test_tails_union_transform_success() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };

    let token = common::create_test_token("/test/", true, true);

    server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(r"^/upload/.*").unwrap());
        then.status(200).body("Upload successful");
    });

    // Ensure tails_union sends the expected transform shape to MORK:
    // pattern is ($h $t) under the source namespace, template emits $t under the target namespace.
    let transform_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/transform")
            .body_contains("(__tludata__ ($h $t))")
            .body_contains("(__outdata__ $t)");
        then.status(200).body("Transform successful");
    });

    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("valid rocket instance");

    let upload_body =
        "(mammal (human (male 1)))\n(mammal (human (male 1)))\n(reptile (snake (male 1)))";
    let upload_resp = client
        .post("/api/spaces/upload/test/tlu")
        .header(Header::new("authorization", token.code.clone()))
        .body(upload_body)
        .dispatch()
        .await;
    assert_eq!(upload_resp.status(), Status::Ok);

    let body = serde_json::to_string(&SetOperationInput {
        source: vec!["test/tlu/".to_string()],
        target: vec!["test/out/".to_string()],
    })
    .unwrap();

    let resp = client
        .post("/api/spaces/tails_union")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().await.expect("response body");
    assert_eq!(body, "true");
    assert_eq!(transform_mock.hits(), 1);

    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"^/export/.*").unwrap());
        then.status(200).body(
            "(test/out (test/outXXXX $x))\n(human (male 1))\n(human (male 1))\n(snake (male 1))",
        );
    });

    let read_resp = client
        .get("/api/spaces/test/out")
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;
    assert_eq!(read_resp.status(), Status::Ok);
    let read_body = read_resp.into_string().await.unwrap_or_default();
    assert!(read_body.contains("human (male 1)"));
    assert!(read_body.contains("snake (male 1)"));

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_tails_union_unauthorized() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };

    let token = common::create_test_token("/test/", true, false);

    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&SetOperationInput {
        source: vec!["test/tlu/".to_string()],
        target: vec!["test/out/".to_string()],
    })
    .unwrap();

    let resp = client
        .post("/api/spaces/tails_union")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Unauthorized);

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_tails_union_bad_request_wrong_counts() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };

    let token = common::create_test_token("/test/", true, true);

    // If counts are wrong, the endpoint should reject before contacting MORK.
    let transform_mock = server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("Transform successful");
    });

    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&SetOperationInput {
        source: vec!["test/tlu/".to_string(), "test/extra/".to_string()],
        target: vec!["test/out/".to_string()],
    })
    .unwrap();

    let resp = client
        .post("/api/spaces/tails_union")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::BadRequest);
    assert_eq!(transform_mock.hits(), 0);

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_tails_union_unauthorized_namespace_rejected() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    let config = AppConfig {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        mork_server_url: server.base_url(),
        mettakg_api_url: "http://localhost:8000".to_string(),
    };

    // Token only allows /test/
    let token = common::create_test_token("/test/", true, true);

    // No MORK calls should happen when unauthorized.
    let transform_mock = server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("Transform successful");
    });

    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&SetOperationInput {
        source: vec!["other/tlu/".to_string()],
        target: vec!["test/out/".to_string()],
    })
    .unwrap();

    let resp = client
        .post("/api/spaces/tails_union")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Unauthorized);
    assert_eq!(transform_mock.hits(), 0);

    common::teardown_database();
}
