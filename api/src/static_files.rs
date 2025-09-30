use include_dir::{include_dir, Dir};
use rocket::http::ContentType;
use rocket::response::content;
use rocket::{get, routes, Route};
use std::path::PathBuf;

static ASSETS: Dir<'_> = include_dir!("../frontend/dist");

#[get("/")]
pub fn index() -> Option<content::RawHtml<&'static [u8]>> {
    ASSETS
        .get_file("index.html")
        .map(|file| content::RawHtml(file.contents()))
}

#[get("/<file..>")]
pub fn files(file: PathBuf) -> Option<(ContentType, &'static [u8])> {
    let filename = file.to_string_lossy();
    ASSETS.get_file(&*filename).map(|file| {
        let content_type = mime_guess::from_path(&*filename).first_or_octet_stream();
        let media_type = format!("{}/{}", content_type.type_(), content_type.subtype());
        (
            ContentType::parse_flexible(&media_type).unwrap_or(ContentType::Binary),
            file.contents(),
        )
    })
}
#[get("/<_..>", rank = 10)]
pub fn spa_fallback() -> Option<content::RawHtml<&'static [u8]>> {
    // Serve index.html for SPA routing
    ASSETS
        .get_file("index.html")
        .map(|file| content::RawHtml(file.contents()))
}

pub fn routes() -> Vec<Route> {
    routes![index, files, spa_fallback]
}
