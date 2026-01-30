use httpmock::prelude::*;
use metta_kg::cli::AppConfig;
use metta_kg::mork_api::{Mm2Cell, Namespace};
use metta_kg::rocket;
use metta_kg::routes::spaces::Mm2InputMultiWithNamespace;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
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

    server.mock(|when, then| {
        when.method(POST).path("/transform");
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

    let mm2_input = Mm2InputMultiWithNamespace {
        patterns: vec![Mm2Cell::new_pattern(
            "($h $t)".to_string(),
            Namespace::from_path_string("test/tlu"),
        )],
        templates: vec![Mm2Cell::new_template(
            "($t)".to_string(),
            Namespace::from_path_string("/test/out"),
        )],
    };

    let transform_resp = client
        .post("/api/spaces/transform")
        .header(Header::new("authorization", token.code.clone()))
        .json(&mm2_input)
        .dispatch()
        .await;
    assert_eq!(transform_resp.status(), Status::Ok);
    let body = transform_resp.into_string().await.expect("response body");
    assert_eq!(body, "true");

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

    server.mock(|when, then| {
        when.method(POST).path("/transform");
        then.status(200).body("Transform successful");
    });

    let client = Client::tracked(rocket(&config).await)
        .await
        .expect("valid rocket instance");

    let mm2_input = Mm2InputMultiWithNamespace {
        patterns: vec![Mm2Cell::new_pattern(
            "($h $t)".to_string(),
            Namespace::from_path_string("/test/tlu"),
        )],
        templates: vec![Mm2Cell::new_template(
            "($t)".to_string(),
            Namespace::from_path_string("/test/out"),
        )],
    };

    let resp = client
        .post("/api/spaces/transform")
        .header(Header::new("authorization", token.code.clone()))
        .json(&mm2_input)
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Unauthorized);

    common::teardown_database();
}
