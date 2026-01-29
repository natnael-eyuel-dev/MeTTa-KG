use api::rocket;
use httpmock::prelude::*;
use httpmock::Regex;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use rocket::serde::json::{json, serde_json};
use serial_test::serial;

#[path = "common.rs"]
mod common;

#[tokio::test]
#[serial]
async fn test_restriction_filters_paths_by_prefix() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    // Export prefixes (wrapped in namespace tags as returned by MORK export)
    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*prefixes.*").unwrap());
        then.status(200).body(
            "(__root__ (test (prefixes (__prefixesdata__ (prefix Foo Bar)))))\n(__root__ (test (prefixes (__prefixesdata__ (prefix Foo Baz)))))\n",
        );
    });

    // Export paths from the left space
    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*paths.*").unwrap());
        then.status(200).body(
            "(__root__ (test (paths (__pathsdata__ (path Foo Bar 1)))))\n(__root__ (test (paths (__pathsdata__ (path Foo Bar 2)))))\n(__root__ (test (paths (__pathsdata__ (path Foo Bar 3)))))\n(__root__ (test (paths (__pathsdata__ (path Foo Baz A)))))\n(__root__ (test (paths (__pathsdata__ (path Foo Baz B)))))\n(__root__ (test (paths (__pathsdata__ (path Foo Baz C)))))\n(__root__ (test (paths (__pathsdata__ (path Foo Cux Red)))))\n(__root__ (test (paths (__pathsdata__ (path Foo Cux Blue)))))\n",
        );
    });

    // Upload should contain only matching path facts
    server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(r"/upload/.*").unwrap())
            .body(
                "(path Foo Bar 1)\n(path Foo Bar 2)\n(path Foo Bar 3)\n(path Foo Baz A)\n(path Foo Baz B)\n(path Foo Baz C)\n",
            );
        then.status(200).body("Upload successful");
    });

    let client = Client::tracked(rocket())
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&json!({
        "source": ["/test/paths/", "/test/prefixes/"],
        "target": ["/test/target/"]
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
async fn test_restriction_keeps_exact_match_path_when_prefix_equals_path() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*prefixes.*").unwrap());
        then.status(200).body("(prefix Foo Bar)\n");
    });

    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*paths.*").unwrap());
        then.status(200)
            .body("(path Foo Bar)\n(path Foo Bar Baz)\n");
    });

    server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(r"/upload/.*").unwrap())
            .body("(path Foo Bar)\n(path Foo Bar Baz)\n");
        then.status(200).body("Upload successful");
    });

    let client = Client::tracked(rocket())
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&json!({
        "source": ["/test/paths/", "/test/prefixes/"],
        "target": ["/test/target/"]
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
async fn test_restriction_empty_prefixes_writes_nothing() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*prefixes.*").unwrap());
        then.status(200).body("");
    });

    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*paths.*").unwrap());
        then.status(200).body("(path Foo Bar 1)\n");
    });

    // If there are no matching prefixes, endpoint returns true without uploading.
    let upload_mock = server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(r"/upload/.*").unwrap());
        then.status(200).body("Upload successful");
    });

    let client = Client::tracked(rocket())
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&json!({
        "source": ["/test/paths/", "/test/prefixes/"],
        "target": ["/test/target/"]
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

    assert_eq!(upload_mock.hits(), 0);

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_restriction_non_commutative_swapped_sources_no_upload() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    let token = common::create_test_token("/test/", true, true);

    // Swapping source order changes the meaning; this verifies order matters.
    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*paths.*").unwrap());
        then.status(200).body("(path Foo Bar 1)\n");
    });
    server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*prefixes.*").unwrap());
        then.status(200).body("(prefix Foo Bar)\n");
    });

    let upload_mock = server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(r"/upload/.*").unwrap());
        then.status(200).body("Upload successful");
    });

    let client = Client::tracked(rocket())
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&json!({
        "source": ["/test/prefixes/", "/test/paths/"],
        "target": ["/test/target/"]
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

    assert_eq!(upload_mock.hits(), 0);

    common::teardown_database();
}

#[tokio::test]
#[serial]
async fn test_restriction_bad_request_wrong_counts() {
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
        "source": ["/test/paths/"],
        "target": ["/test/target/"]
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

#[tokio::test]
#[serial]
async fn test_restriction_unauthorized_namespace_rejected() {
    if !common::is_database_running() {
        eprintln!("Warning: Database not running, skipping test");
        return;
    }

    let server = MockServer::start();
    common::setup(&server.base_url());

    // Token only allows /test/
    let token = common::create_test_token("/test/", true, true);

    // No MORK calls should happen when unauthorized.
    let export_mock = server.mock(|when, then| {
        when.method(GET)
            .path_matches(Regex::new(r"/export/.*").unwrap());
        then.status(200).body("(ignored)");
    });

    let client = Client::tracked(rocket())
        .await
        .expect("valid rocket instance");

    let body = serde_json::to_string(&json!({
        "source": ["/other/paths/", "/test/prefixes/"],
        "target": ["/test/target/"]
    }))
    .unwrap();

    let response = client
        .post("/spaces/restriction")
        .body(body)
        .header(Header::new("authorization", token.code.clone()))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Unauthorized);

    assert_eq!(export_mock.hits(), 0);
    common::teardown_database();
}
