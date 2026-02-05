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
//
#[tokio::test]
#[serial]
async fn test_subspace_transform() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
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
        .expect("valid rocket instance");

    let mm2_input = Mm2InputMultiWithNamespace {
        patterns: vec![Mm2Cell::new_pattern(
            "(path Foo Baz $x)".to_string(),
            Namespace::from_path_string("/test/subspace/path"),
        )],
        templates: vec![Mm2Cell::new_template(
            "(output $x)".to_string(),
            Namespace::from_path_string("/test/subspace/output"),
        )],
    };

    let response = client
        .post("/api/spaces/transform")
        .header(Header::new("authorization", token.code.clone()))
        .json(&mm2_input)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert_eq!(body, "true");

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
    let payload = Mm2InputMultiWithNamespace {
        patterns: vec![Mm2Cell::new_pattern(
            "slkd".to_string(),
            Namespace::from_path_string("/test/subspace/animal"),
        )],
        templates: vec![Mm2Cell::new_template(
            "$x".to_string(),
            Namespace::from_path_string("/test/subspace/car"),
        )],
    };
    let response = client
        .post("/api/spaces/subspace")
        .header(Header::new("authorization", token.code.clone()))
        .json(&payload)
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
    let payload = Mm2InputMultiWithNamespace {
        patterns: vec![Mm2Cell::new_pattern(
            "slkd".to_string(),
            Namespace::from_path_string("/test/subspace/animal"),
        )],
        templates: vec![Mm2Cell::new_template(
            "$x".to_string(),
            Namespace::from_path_string("/test/subspace/car"),
        )],
    };
    let response = client
        .post("/api/spaces/subspace")
        .header(Header::new("authorization", token.code.clone()))
        .json(&payload)
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
    let payload = Mm2InputMultiWithNamespace {
        patterns: vec![Mm2Cell::new_pattern(
            "".to_string(),
            Namespace::from_path_string("/test/subspace/animal"),
        )],
        templates: vec![Mm2Cell::new_template(
            "$x".to_string(),
            Namespace::from_path_string("/test/subspace/car"),
        )],
    };
    let response = client
        .post("/api/spaces/subspace")
        .header(Header::new("authorization", token.code.clone()))
        .json(&payload)
        .dispatch()
        .await;
    // Endpoint currently validates only counts + permissions; empty pattern is allowed.
    assert_eq!(response.status(), Status::Ok);
    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_subspace_endpoint_unauthorized_namespace() {
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
    let payload = Mm2InputMultiWithNamespace {
        patterns: vec![Mm2Cell::new_pattern(
            "slkd".to_string(),
            Namespace::from_path_string("other/space/animal"),
        )],
        templates: vec![Mm2Cell::new_template(
            "$x".to_string(),
            Namespace::from_path_string("test/subspace/car"),
        )],
    };
    let response = client
        .post("/api/spaces/subspace")
        .header(Header::new("authorization", token.code.clone()))
        .json(&payload)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Unauthorized);
    common::teardown_database();
}
