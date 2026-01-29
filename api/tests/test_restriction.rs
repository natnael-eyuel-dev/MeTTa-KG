use api::rocket;
use httpmock::prelude::*;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use rocket::serde::json::{json, serde_json};
use serial_test::serial;

#[path = "common.rs"]
mod common;

#[tokio::test]
#[serial]
async fn test_restriction_success() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    // Setup mock server
    let server = MockServer::start();
    common::setup(&server.base_url());

    // Create test token
    let token = common::create_test_token("/test/", true, true);
    let _ = common::create_test_token("/test/paths/", true, false);
    let _ = common::create_test_token("/test/prefixes/", true, false);
    let _ = common::create_test_token("/test/target/", true, false);

    // Mock backend transform call
    server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("Transform successful");
    });

    let client = Client::tracked(rocket())
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&json!({
        "patterns": [
            { "namespace": ["test", "paths"], "pattern": "(path $a $b $v)" },
            { "namespace": ["test", "prefixes"], "pattern": "(prefix $a $b)" }
        ],
        "templates": [
            { "namespace": ["test", "target"], "template": "(output $a $b $v)" }
        ]
    }))
    .unwrap();

    let response = client
        .post("/spaces/restriction")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await;
    assert_eq!(body.expect("response body"), "true");
    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_restriction_unauthorized_namespace() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);
    let client = Client::tracked(rocket())
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&json!({
        "patterns": [
            { "namespace": ["other", "paths"], "pattern": "(path $a $b $v)" },
            { "namespace": ["other", "prefixes"], "pattern": "(prefix $a $b)" }
        ],
        "templates": [
            { "namespace": ["other", "target"], "template": "(output $a $b $v)" }
        ]
    }))
    .unwrap();

    let response = client
        .post("/spaces/restriction")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Unauthorized);
    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_restriction_bad_request_wrong_sources() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }
    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);
    let client = Client::tracked(rocket())
        .await
        .expect("valid rocket instance");

    // Only one source -> BadRequest
    let body = serde_json::to_string(&json!({
        "patterns": [
            { "namespace": ["test", "paths"], "pattern": "(path $a $b $v)" }
        ],
        "templates": [
            { "namespace": ["test", "target"], "template": "(output $a $b $v)" }
        ]
    }))
    .unwrap();

    let response = client
        .post("/spaces/restriction")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::BadRequest);
    common::teardown_database();
}
