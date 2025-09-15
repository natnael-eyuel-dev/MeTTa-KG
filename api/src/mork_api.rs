use reqwest::{Client, Method};
use rocket::http::Status;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::any::Any;
use url::form_urlencoded::byte_serialize;

#[derive(Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub enum ExportFormat {
    Metta,
    Json,
    Csv,
    Raw,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransformDetails {
    /// the sub space as per playground convetions. ie. (/ ...)
    pub patterns: Vec<String>, // A sub space
    pub templates: Vec<String>,
}

impl Default for TransformDetails {
    fn default() -> Self {
        TransformDetails {
            patterns: vec![String::from("$x")],
            templates: vec![String::from("$x")],
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]  
pub struct Namespace {  
    pub ns: PathBuf,  
}  
  
impl Namespace {  
    pub fn new() -> Self {  
        Namespace::default()  
    }  
  
    pub fn ns(mut self, ns: PathBuf) -> Self {  
        self.ns = ns;  
        self  
    }  
  
    pub fn to_mork_namespace(&self) -> String {  
        let path_string = self.ns.to_string_lossy().to_string();  
        let clean_path = path_string  
            .trim_start_matches('/')  
            .trim_end_matches('/');  
          
        if clean_path.is_empty() {  
            // Root namespace  
            "{}".to_string()  
        } else {  
            // Convert hierarchical path to nested S-expressions  
            let parts: Vec<&str> = clean_path.split('/').collect();  
            parts.iter().rev().fold("{}".to_string(), |acc, part| {  
                format!("({} {})", part, acc)  
            })  
        }  
    }  
  
    pub fn with_namespace(&self, value: &str) -> String {  
        self.to_mork_namespace().replace("{}", value)  
    }  
  
    pub fn is_valid(&self) -> bool {  
        self.ns.starts_with("/") && self.ns.ends_with("/")  
    }  
  
    // Legacy method for backward compatibility (deprecated)  
    #[deprecated(note = "Use to_mork_namespace() instead")]  
    pub fn encoded(&self) -> String {  
        let path_string = self.ns.to_string_lossy().to_string();  
        let trimmed = path_string.trim_matches('/');  
        if trimmed.is_empty() {  
            String::new()  
        } else {  
            trimmed.replace('/', "|")  
        }  
    }  
}  
  
impl From<PathBuf> for Namespace {  
    fn from(ns: PathBuf) -> Self {  
        Namespace::new().ns(ns)  
    }  
}  
  
impl Default for Namespace {  
    fn default() -> Self {  
        Namespace {  
            ns: PathBuf::from("/"),  
        }  
    }  
} 

#[allow(dead_code)]
impl TransformDetails {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn patterns(mut self, patterns: Vec<String>) -> Self {
        self.patterns = patterns;
        self
    }

    pub fn templates(mut self, templates: Vec<String>) -> Self {
        self.templates = templates;
        self
    }
}

pub struct MorkApiClient {
    base_url: String,
    client: Client,
}

impl MorkApiClient {
    pub fn new() -> Self {
        let mork_url = env::var("METTA_KG_MORK_URL").expect("METTA_KG_MORK_URL must be set");
        Self {
            base_url: mork_url,
            client: Client::new(),
        }
    }

    pub async fn dispatch<R: Request>(&self, request: R) -> Result<String, Status> {
        let url = format!("{}{}", self.base_url, request.path());
        let mut http_request = self.client.request(request.method(), &url);

        if request.path().starts_with("/upload/") {
            if let Some(body) = request.body() {
                if let Some(body_str) = (&body as &dyn Any).downcast_ref::<String>() {
                    http_request = http_request
                        .header("Content-Type", "text/plain")
                        .body(body_str.clone());
                } else {
                    eprintln!("Upload endpoint called with non-string body type");
                    return Err(Status::InternalServerError);
                }
            }
        } else if let Some(body) = request.body() {
            http_request = http_request.json(&body);
        }

        match http_request.send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => Ok(text),
                Err(e) => {
                    eprintln!("Error reading Mork API response text: {}", e);
                    Err(Status::InternalServerError)
                }
            },
            Err(e) => {
                eprintln!("Error sending request to Mork API: {}", e);
                Err(Status::InternalServerError)
            }
        }
    }
}

pub trait Request {
    type Body: Serialize + Any;
    fn method(&self) -> Method;
    fn path(&self) -> String;
    fn body(&self) -> Option<Self::Body> {
        None
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TransformRequest {
    namespace: Namespace,
    transform_input: TransformDetails,
}

impl TransformRequest {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn namespace(mut self, ns: PathBuf) -> Self {
        self.namespace = Namespace::from(if ns.to_string_lossy().is_empty() {
            PathBuf::from("/")
        } else {
            ns.to_path_buf()
        });
        self
    }

    pub fn transform_input(mut self, inp: TransformDetails) -> Self {
        self.transform_input = inp;
        self
    }

    fn multi_patterns(&self) -> String {
        format!(
            "(, {})",
            self.transform_input
                .patterns
                .iter()
                .map(|pattern| { self.namespace.with_namespace(pattern) })
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    fn multi_templates(&self) -> String {
        format!(
            "(, {})",
            self.transform_input
                .templates
                .iter()
                .map(|pattern| { self.namespace.with_namespace(pattern) })
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    pub fn transform_code(&self) -> String {
        format!(
            "(transform {} {})",
            self.multi_patterns(),
            self.multi_templates()
        )
    }
}

impl Request for TransformRequest {
    type Body = ();

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        format!("/transform/{}", &self.transform_code())
    }

    fn body(&self) -> Option<Self::Body> {
        Some(())
    }
}

#[derive(Default)]
pub struct ImportRequest {
    namespace: Namespace,
    transform_input: TransformDetails,
    uri: String,
}

impl ImportRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn namespace(mut self, ns: PathBuf) -> Self {
        self.namespace = Namespace::from(if ns.to_string_lossy().is_empty() {
            PathBuf::from("/")
        } else {
            ns.to_path_buf()
        });
        self
    }

    pub fn uri(mut self, uri: String) -> Self {
        self.uri = uri;
        self
    }
}

impl Request for ImportRequest {
    type Body = ();

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!(
            "/import/{}/{}/?uri={}",
            "$x",
            self.namespace.with_namespace(
                &self
                    .transform_input
                    .templates
                    .first()
                    .unwrap_or(&"$x".to_string())
            ),
            self.uri
        )
    }

    fn body(&self) -> Option<Self::Body> {
        None
    }
}

#[derive(Default)]
#[allow(dead_code)]
pub struct ReadRequest {
    namespace: Namespace,
    transform_input: TransformDetails,
    export_url: Option<String>,
    format: Option<ExportFormat>,
}

impl ReadRequest {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn namespace(mut self, namespace: PathBuf) -> Self {
        self.namespace = Namespace::from(if namespace.to_string_lossy().is_empty() {
            PathBuf::from("/")
        } else {
            namespace.to_path_buf()
        });
        self
    }
}

impl Request for ReadRequest {
    type Body = ();

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!(
            "/export/{0}/{1}/",
            // Match everything under self.namespace
            self.namespace.with_namespace(
                self.transform_input
                    .patterns
                    .first()
                    .unwrap_or(&String::from("&x"))
            ),
            // Exporting everything seems valid here
            self.transform_input
                .templates
                .first()
                .unwrap_or(&String::from("&x")),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ExploreRequest {
    namespace: Namespace,
    pattern: String,
    token: String,
}

impl ExploreRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn namespace(mut self, namespace: PathBuf) -> Self {
        self.namespace = Namespace::from(if namespace.to_string_lossy().is_empty() {
            PathBuf::from("/")
        } else {
            namespace.to_path_buf()
        });
        self
    }

    pub fn pattern(mut self, pattern: String) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn token(mut self, token: String) -> Self {
        self.token = token;
        self
    }
}

impl Request for ExploreRequest {
    type Body = ();

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        format!(
            "/explore/{}/{}/",
            self.namespace.with_namespace(&self.pattern),
            self.token
        )
    }
}

#[derive(Default)]  
pub struct UploadRequest {  
    namespace: Namespace,  
    pattern: String,  
    template: String,  
    data: String,  
}  
  
impl UploadRequest {  
    pub fn new() -> Self {  
        Self::default()  
    }  
  
    pub fn namespace(mut self, ns: PathBuf) -> Self {  
        self.namespace = Namespace::from(if ns.to_string_lossy().is_empty() {  
            PathBuf::from("/")  
        } else {  
            ns.to_path_buf()  
        });  
        self  
    }  
  
    pub fn pattern(mut self, pattern: String) -> Self {  
        self.pattern = pattern;  
        self  
    }  
  
    pub fn template(mut self, template: String) -> Self {  
        self.template = template;  
        self  
    }  
  
    pub fn data(mut self, data: String) -> Self {  
        self.data = data;  
        self  
    }  
  
    // New method to generate namespace-formatted template  
    fn get_formatted_template(&self) -> String {  
        if self.template == "$x" {  
            // Use namespace formatting for the default pattern  
            self.namespace.with_namespace("$x")  
        } else {  
            // Use custom template as-is  
            self.template.clone()  
        }  
    }  
}  
  
impl Request for UploadRequest {  
    type Body = String;  
  
    fn method(&self) -> Method {  
        Method::POST  
    }  
  
    fn path(&self) -> String {  
        format!(  
            "/upload/{}/{}",  
            urlencoding::encode(&self.pattern),  
            urlencoding::encode(&self.get_formatted_template())  
        )  
    }  
  
    fn body(&self) -> Option<Self::Body> {  
        Some(self.data.clone())  
    }  
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use std::path::PathBuf;

    #[test]
    fn test_encoded_space() {
        let ns = Namespace::new();
        assert_eq!(ns.encoded(), "|");

        let ns_with_value = Namespace::from(PathBuf::from("/foo/bar/"));
        assert_eq!(ns_with_value.encoded(), "|foo|bar|");

        let ns_with_no_slashes = Namespace::from(PathBuf::from("foo"));
        assert_eq!(ns_with_no_slashes.encoded(), "foo");
    }
}