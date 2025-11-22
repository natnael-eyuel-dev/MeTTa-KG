use std::path::PathBuf;

use api::mork_api::{Pattern, Template};
use api::rocket;
use api::routes::spaces::Mm2InputMultiWithNamespace;
use httpmock::prelude::*;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use serial_test::serial;

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

    let token = common::create_test_token("/test/", true, true);

    server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("Transform successful");
    });

    let client = Client::tracked(rocket()).await.expect("valid rocket instance");

    let upload_body = "(mammal (human (male 1)))\n(mammal (human (male 1)))\n(reptile (snake (male 1)))";
    let upload_resp = client
        .post("/spaces/upload/test/tlu")
        .header(Header::new("authorization", token.code.clone()))
        .body(upload_body)
        .dispatch()
        .await;
    assert_eq!(upload_resp.status(), Status::Ok);

    let mm2_input = Mm2InputMultiWithNamespace {
        patterns: vec![Pattern::default()
            .pattern("($h $t)".to_string())
            .namespace(PathBuf::from("/test/tlu"))],
        templates: vec![Template::default()
            .template("($t)".to_string())
            .namespace(PathBuf::from("/test/out"))],
    };

    let transform_resp = client
        .post("/spaces/transform")
        .header(Header::new("authorization", token.code.clone()))
        .json(&mm2_input)
        .dispatch()
        .await;
    assert_eq!(transform_resp.status(), Status::Ok);
    let body = transform_resp.into_string().await.expect("response body");
    assert_eq!(body, "true");

    server.mock(|when, then| {
        when.method(POST).path("/read");
        then.status(200).body(
            "(test/out (test/outXXXX $x))\n(human (male 1))\n(human (male 1))\n(snake (male 1))",
        );
    });

    let read_resp = client
        .get("/spaces/test/out")
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

    let token = common::create_test_token("/test/", true, false);

    server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("Transform successful");
    });

    let client = Client::tracked(rocket()).await.expect("valid rocket instance");

    let mm2_input = Mm2InputMultiWithNamespace {
        patterns: vec![Pattern::default()
            .pattern("($h $t)".to_string())
            .namespace(PathBuf::from("/test/tlu"))],
        templates: vec![Template::default()
            .template("($t)".to_string())
            .namespace(PathBuf::from("/test/out"))],
    };

    let resp = client
        .post("/spaces/transform")
        .header(Header::new("authorization", token.code.clone()))
        .json(&mm2_input)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Unauthorized);

    common::teardown_database();
}