use reqwest::{Client, Method};
use rocket::http::Status;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::env;
use std::path::PathBuf;

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

    pub fn encoded(&self) -> String {
        let path_str = self.ns.to_string_lossy();
        let trimmed = path_str.trim_matches('/');
        if trimmed.is_empty() {
            "|".to_string()
        } else {
            trimmed.replace('/', "|")
        }
    }

    pub fn with_namespace(&self, value: &str) -> String {
        let encoded_ns = self.encoded();
        if encoded_ns.is_empty() {
            value.to_string()
        } else {
            format!("({encoded_ns} {value})")
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

impl Default for MorkApiClient {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8001".to_string(), // According to Dockerfile.mork
            client: Client::new(),
        }
    }
}

impl MorkApiClient {
    pub fn new() -> Self {
        if let Ok(mork_url) = env::var("METTA_KG_MORK_URL") {
            Self {
                base_url: mork_url,
                client: Client::new(),
            }
        } else {
            Self::default()
        }
    }

    pub async fn dispatch<R: Request>(&self, request: R) -> Result<String, Status> {
        let url = format!("{}{}", self.base_url, request.path());
        let mut http_request = self.client.request(request.method(), &url);

        if request.path().starts_with("/upload/") || request.path() == "/transform" {
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
                    eprintln!("Error reading Mork API response text: {e}");
                    Err(Status::InternalServerError)
                }
            },
            Err(e) => {
                eprintln!("Error sending request to Mork API: {e}");
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
    type Body = String;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> String {
        "/transform".to_string()
    }

    fn body(&self) -> Option<Self::Body> {
        Some(self.transform_code())
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
                self.transform_input
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
            urlencoding::encode(&self.template)
        )
    }

    fn body(&self) -> Option<Self::Body> {
        Some(self.data.clone())
    }
}

#[derive(Default)]
pub struct ExportRequest {
    namespace: Namespace,
    pattern: String,
    template: String,
    format: Option<ExportFormat>,
    max_write: Option<usize>,
}

impl ExportRequest {
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

    pub fn format(mut self, format: ExportFormat) -> Self {
        self.format = Some(format);
        self
    }
}

impl Request for ExportRequest {
    type Body = ();

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        let mut path = format!(
            "/export/{}/{}",
            urlencoding::encode(&self.namespace.with_namespace(&self.pattern)),
            urlencoding::encode(&self.template)
        );

        let mut query_params = Vec::new();

        if let Some(format) = &self.format {
            let format_str = match format {
                ExportFormat::Metta => "metta",
                ExportFormat::Json => "json",
                ExportFormat::Csv => "csv",
                ExportFormat::Raw => "raw",
            };
            query_params.push(format!("format={format_str}"));
        }

        if let Some(max_write) = self.max_write {
            query_params.push(format!("max_write={max_write}"));
        }

        if !query_params.is_empty() {
            path.push_str("/?");
            path.push_str(&query_params.join("&"));
        }

        path
    }

    fn body(&self) -> Option<Self::Body> {
        None
    }
}

#[derive(Default)]
pub struct ClearRequest {
    namespace: Namespace,
    expr: String,
}

impl ClearRequest {
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

    pub fn expr(mut self, expr: String) -> Self {
        self.expr = expr;
        self
    }
}

impl Request for ClearRequest {
    type Body = ();

    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> String {
        let expr_to_use = self.namespace.with_namespace(&self.expr);

        format!("/clear/{}", urlencoding::encode(&expr_to_use))
    }

    fn body(&self) -> Option<Self::Body> {
        None
    }
}
