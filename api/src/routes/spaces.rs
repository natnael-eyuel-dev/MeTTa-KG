use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use url::Url;
use uuid::Uuid;
use rocket::tokio::io::AsyncReadExt;
use rocket::response::stream::{EventStream, Event};
use rocket::tokio::time::{sleep, Duration};
use rocket::tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use rocket::response::Responder;
use futures::stream::{Stream, StreamExt};

use rocket::{get, post, Data};
use rocket::response::status::Custom;
use std::path::PathBuf;

use crate::model::Token;
use crate::mork_api::{
    ExploreRequest, ImportRequest, MorkApiClient, ReadRequest, Request, TransformDetails, UploadRequest,
    TransformRequest,
};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Transformation {
    pub space: PathBuf,
    pub patterns: Vec<String>,
    pub templates: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ExploreInput {
    pub pattern: String,
    pub token: String,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ExploreOutput {
    pub expr: String,
    pub token: String
}

// sse endpoint for real-time transformation updates
#[post("/spaces/transform-sse/<path..>", data = "<transformation>")]
pub async fn transform_sse(
    token: Token,
    path: PathBuf,
    transformation: Json<Transformation>,
) -> Result<EventStream![Event + 'static], Status> {
    let token_namespace = token.namespace.strip_prefix("/").unwrap();

    if !transformation.space.starts_with(token_namespace)
        || !token.permission_read
        || !token.permission_write
    {
        return Err(Status::Unauthorized);
    }

    let mork_api_client = MorkApiClient::new();
    let request = TransformRequest::new()
        .namespace(transformation.space.to_path_buf())
        .transform_input(
            TransformDetails::new()
                .patterns(transformation.patterns.clone())
                .templates(transformation.templates.clone()),
        );

    // start the transformation
    match mork_api_client.dispatch(request).await {
        Ok(_) => {
            // create the SSE stream
            let space_path = transformation.space.clone();
            let token_clone = token.clone();
            
            let stream = EventStream! {
                yield Event::data("Transform initiated successfully").event("status");
                
                // poll for completion
                let mut interval = interval(Duration::from_secs(3));
                loop {
                    interval.tick().await;
                    
                    let explore_request = ExploreRequest::new()
                        .namespace(space_path.clone())
                        .pattern("$x".to_string())
                        .token("".to_string());
                    
                    match mork_api_client.dispatch(explore_request).await {
                        Ok(_) => {
                            yield Event::data("Transform completed successfully").event("complete");
                            break;
                        },
                        Err(err_status) if err_status == Status::ServiceUnavailable => {
                            yield Event::data("Transform in progress...").event("progress");
                        },
                        Err(e) => {
                            yield Event::data(format!("Transform failed: {:?}", e)).event("error");
                            break;
                        }
                    }
                }
            };
            
            Ok(stream)
        },
        Err(e) => Err(e),
    }
}

// sse endpoint for monitoring transformation status
#[get("/spaces/transform-status/<path..>?<token>")]
pub async fn transform_status(
    path: PathBuf,
    token: String,
) -> Result<EventStream![Event + 'static], Status> {

    let mork_api_client = MorkApiClient::new();
    
    let stream = EventStream! {
        yield Event::data("Monitoring transformation status").event("status");
        
        let mut interval = interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            
            let explore_request = ExploreRequest::new()
                .namespace(path.clone())
                .pattern("$x".to_string())
                .token("".to_string());
            
            match mork_api_client.dispatch(explore_request).await {
                Ok(_) => {
                    yield Event::data("Transform completed successfully").event("complete");
                    break;
                },
                Err(err_status) if err_status == Status::ServiceUnavailable => {
                    yield Event::data("Transform in progress...").event("progress");
                },
                Err(e) => {
                    yield Event::data(format!("Transform failed: {:?}", e)).event("error");
                    break;
                }
            }
        }
    };
    
    Ok(stream)
}

#[post("/spaces/upload/<path..>", data = "<data>", rank=1)]
pub async fn upload(
    token: Token,
    path: PathBuf,
    data: Data<'_>,
) -> Result<Json<String>, Custom<String>> {
    let token_namespace = token.namespace.strip_prefix("/").unwrap();
    if !path.starts_with(token_namespace) || !token.permission_write {
        return Err(Custom(Status::Unauthorized, "Unauthorized".to_string()));
    }
    
    let mut body = String::new();
    if let Err(e) = data.open(rocket::data::ByteUnit::Mebibyte(20)).read_to_string(&mut body).await {
        eprintln!("Failed to read body: {e}");
        return Err(Custom(Status::BadRequest, format!("Failed to read body: {e}")));
    }

    let pattern = "$x";
    let namespace_str = path.to_str().unwrap_or("").trim_matches('/');
    let template = if namespace_str.is_empty() {
        "$x".to_string()
    } else {
        format!("({} $x)", namespace_str)
    };

    let mork_api_client = MorkApiClient::new();
    let request = UploadRequest::new()
        .namespace(path)
        .pattern(pattern.to_string())
        .template(template)
        .data(body);

    match mork_api_client.dispatch(request).await {
        Ok(text) => Ok(Json(text)),
        Err(e) => Err(Custom(Status::InternalServerError, format!("Failed to contact backend: {e}"))),
    }
}

#[post("/spaces/<path..>?<uri>")]
pub async fn import(token: Token, path: PathBuf, uri: String) -> Result<Json<bool>, Status> {
    if !path.starts_with(token.namespace.strip_prefix("/").unwrap()) || !token.permission_write {
        return Err(Status::Unauthorized);
    }

    // validate uri
    if Url::parse(&uri).is_err() {
        return Err(Status::BadRequest);
    }

    let mork_api_client = MorkApiClient::new();
    let request = ImportRequest::new().namespace(path).uri(uri);

    match mork_api_client.dispatch(request).await {
        Ok(_) => Ok(Json(true)),
        Err(e) => Err(e),
    }
}

#[get("/spaces/<path..>")]
pub async fn read(token: Token, path: PathBuf) -> Result<Json<String>, Status> {
    if !path.starts_with(token.namespace.strip_prefix("/").unwrap()) || !token.permission_read {
        return Err(Status::Unauthorized);
    }

    let mork_api_client = MorkApiClient::new();
    let request = ReadRequest::new().namespace(path);

    let response = mork_api_client.dispatch(request).await.map(Json);
    response
}

#[post("/explore/spaces/<path..>", data = "<data>")]
pub async fn explore(
    token: Token,
    path: PathBuf,
    data: Json<ExploreInput>,
) -> Result<Json<String>, Status> {
    if !path.starts_with(token.namespace.strip_prefix("/").unwrap()) || !token.permission_read {
        return Err(Status::Unauthorized);
    }

    let mork_api_client = MorkApiClient::new();
    let request = ExploreRequest::new()
        .namespace(path)
        .pattern(data.pattern.clone())
        .token(data.token.clone());

    println!("explore path: {:?}", request.path());

    let response = mork_api_client.dispatch(request).await.map(Json);
    println!("explore response: {:?}", response);
    response
}