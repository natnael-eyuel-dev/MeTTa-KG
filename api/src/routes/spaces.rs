use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::io::AsyncReadExt;
use serde::{Deserialize, Serialize};
use url::Url;

use rocket::response::status::Custom;
use rocket::serde::json;
use rocket::{get, post, Data};
use std::path::PathBuf;
use tokio::time::{sleep, Duration, Instant};

use crate::model::Token;
use crate::mork_api::{
    ClearRequest, ExploreRequest, ExportFormat, ExportRequest, ImportRequest, Mm2Cell,
    MorkApiClient, Namespace, ReadRequest, StatusRequest, StatusResponse, TransformDetails,
    TransformRequest, UploadRequest,
};

trait SourceTargetPermissions {
    type Ns: ToString + Clone;

    fn source(&self) -> Vec<Self::Ns>;
    fn target(&self) -> Vec<Self::Ns>;

    fn source_target_permissions(&self, token: Token) -> bool {
        let token_namespace = token.namespace.strip_prefix("/").unwrap();

        // check `permission read`
        let has_read_permission = self
            .source()
            .iter()
            .all(|pattern| pattern.to_string().starts_with(token_namespace))
            && token.permission_read;

        let has_write_permission = self
            .target()
            .iter()
            .all(|template| template.to_string().starts_with(token_namespace))
            && token.permission_write;

        has_read_permission && has_write_permission
    }
}

/// The input for a transformation operation.
/// see mm2 operations for more    // TODO: Add links
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Mm2InputMulti {
    pub patterns: Vec<String>,
    pub templates: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Mm2InputMultiWithNamespace {
    pub patterns: Vec<Mm2Cell>,
    pub templates: Vec<Mm2Cell>,
}

impl SourceTargetPermissions for Mm2InputMultiWithNamespace {
    type Ns = Namespace;

    fn source(&self) -> Vec<Self::Ns> {
        self.patterns
            .iter()
            .map(|p| p.namespace().clone())
            .collect()
    }

    fn target(&self) -> Vec<Self::Ns> {
        self.templates
            .iter()
            .map(|t| t.namespace().clone())
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mm2Input {
    pub pattern: String,
    pub template: String,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ExploreInput {
    pub pattern: String,
    pub token: String,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SetOperationInput {
    pub source: Vec<String>,
    pub target: Vec<String>,
}

impl SourceTargetPermissions for SetOperationInput {
    type Ns = String;

    fn source(&self) -> Vec<Self::Ns> {
        self.source.clone()
    }

    fn target(&self) -> Vec<Self::Ns> {
        self.target.clone()
    }
}

/// Fetches the `<path..>` space content. Use cautously as it will load everything.
/// It is recommended to use the `/spaces/<path..>?op=explore` instead for large queries
#[get("/spaces/<path..>", rank = 1, data = "<mm2>")]
pub async fn read(
    token: Token,
    path: PathBuf,
    mm2: Option<Json<Mm2InputMulti>>,
) -> Result<Json<String>, Status> {
    if !path.starts_with(token.namespace.strip_prefix("/").unwrap()) || !token.permission_read {
        return Err(Status::Unauthorized);
    }

    // shadowing for backwards compatibility
    // TODO: remove `Option` once all clients are updated
    let mm2 = mm2.unwrap_or(Json(Mm2InputMulti::default()));

    let mork_api_client = MorkApiClient::new();
    let transform_input = TransformDetails::new()
        .patterns(vec![Mm2Cell::new_pattern(
            mm2.patterns.first().cloned().unwrap_or("$x".to_string()),
            Namespace::from(path.to_path_buf()),
        )])
        .templates(vec![Mm2Cell::new_template(
            mm2.templates.first().cloned().unwrap_or("$x".to_string()),
            Namespace::from(path.to_path_buf()),
        )]);
    let request = ReadRequest::new().transform_input(transform_input);

    let response = mork_api_client.dispatch(request).await.map(Json);
    response
}

/// Upload to the `<path..>` space. Exectes mm2 on the imported data.
#[post("/spaces/upload/<path..>", data = "<data>")]
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
    if let Err(e) = data
        .open(rocket::data::ByteUnit::Mebibyte(20))
        .read_to_string(&mut body)
        .await
    {
        return Err(Custom(
            Status::BadRequest,
            format!("Failed to read body: {e}"),
        ));
    }

    let pattern = "$x";
    let template = "$x";

    let mork_api_client = MorkApiClient::new();
    let request = UploadRequest::new()
        .namespace(path)
        .pattern(pattern.to_string())
        .template(template.to_string())
        .data(body);

    match mork_api_client.dispatch(request).await {
        Ok(text) => Ok(Json(text)),
        Err(e) => Err(Custom(
            Status::InternalServerError,
            format!("Failed to contact backend: {e}"),
        )),
    }
}

/// Imports data from `<uri>` into the `<path..>` space. Exectes mm2 on the imported data.
#[post("/spaces/import/<path..>?<uri>")]
pub async fn import(token: Token, path: PathBuf, uri: String) -> Result<Json<bool>, Status> {
    if !path.starts_with(token.namespace.strip_prefix("/").unwrap()) || !token.permission_write {
        return Err(Status::Unauthorized);
    }

    // validate uri
    if Url::parse(&uri).is_err() {
        return Err(Status::BadRequest);
    }

    let mork_api_client = MorkApiClient::new();
    let template = Mm2Cell::new_template("$x".to_string(), Namespace::from(path));
    let request = ImportRequest::new().to(template).uri(uri);

    match mork_api_client.dispatch(request).await {
        Ok(_) => Ok(Json(true)),
        Err(e) => Err(e),
    }
}

/// Performs an explore operation on the `<path..>` space. Get the result that
/// matches the `<pattern>` by incrementally traversing the resulting space.
#[post("/spaces/explore/<path..>", data = "<explore_input>")]
pub async fn explore(
    token: Token,
    path: PathBuf,
    explore_input: Json<ExploreInput>,
) -> Result<Json<String>, Status> {
    if !path.starts_with(token.namespace.strip_prefix("/").unwrap()) || !token.permission_read {
        return Err(Status::Unauthorized);
    }

    let mork_api_client = MorkApiClient::new();
    let request = ExploreRequest::new()
        .namespace(path)
        .pattern(explore_input.pattern.clone())
        .token(explore_input.token.clone());

    let response = mork_api_client.dispatch(request).await.map(Json);
    response
}

/// Performs an export operation on the `<path..>` space. Get the result that
/// matches the `<pattern>` by incrementally traversing the resulting space.
#[post("/spaces/export/<path..>", data = "<export_input>")]
pub async fn export(
    token: Token,
    path: PathBuf,
    export_input: Json<Mm2Input>,
) -> Result<Json<String>, Status> {
    if !path.starts_with(token.namespace.strip_prefix("/").unwrap()) || !token.permission_read {
        return Err(Status::Unauthorized);
    }

    let mork_api_client = MorkApiClient::new();
    let request = ExportRequest::new()
        .namespace(path)
        .pattern(export_input.pattern.clone())
        .template(export_input.template.clone())
        .format(ExportFormat::Metta);

    match mork_api_client.dispatch(request).await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err(e),
    }
}

#[post("/spaces/clear/<path..>?<expr>")]
pub async fn clear(token: Token, path: PathBuf, expr: String) -> Result<Json<bool>, Status> {
    let token_namespace = token.namespace.strip_prefix("/").unwrap();
    if !path.starts_with(token_namespace) || !token.permission_write {
        return Err(Status::Unauthorized);
    }

    let mork_api_client = MorkApiClient::new();
    let request = ClearRequest::new().namespace(path).expr(expr);

    match mork_api_client.dispatch(request).await {
        Ok(_) => Ok(Json(true)),
        Err(e) => Err(e),
    }
}

/// Performs a transformation operation on the `<path..>` space
#[post("/spaces/transform", data = "<mm2>")]
pub async fn transform(
    token: Token,
    mm2: Json<Mm2InputMultiWithNamespace>,
) -> Result<Json<bool>, Status> {
    let mm2 = mm2.into_inner();
    if !mm2.clone().source_target_permissions(token) {
        return Err(Status::Unauthorized);
    }

    let mork_api_client = MorkApiClient::new();
    let request = TransformRequest::new().transform_input(
        TransformDetails::new()
            .patterns(mm2.clone().patterns)
            .templates(mm2.templates),
    );

    // TODO: use server sent events instead
    match mork_api_client.dispatch(request).await {
        Ok(_) => Ok(Json(true)),
        Err(e) => Err(e),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////// SET OPERATIONS //////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Performs a composition operation on provided namespaces. `token` must have `permission_write`
/// on the target namespace and `permission_read` on all source namespaces.
/// # Composition Transformation
/// ```lisp
/// (transform
///     (, (namespace1 $a) (namespace2 $b))  ; (namespace $c) (namespace $d) etc ...
///     (, (output-namespace $a $b))  ; $c $d etc ...
/// )
/// ```
///
/// Currently it only handles a single target namespace
#[post("/spaces/composition", data = "<operation_input>")]
pub async fn composition(
    token: Token,
    operation_input: Json<SetOperationInput>,
) -> Result<Json<bool>, Status> {
    if !operation_input.source_target_permissions(token) {
        return Err(Status::Unauthorized);
    }

    let transform_input = composition_transform(operation_input.into_inner())?;

    let request = TransformRequest::new().transform_input(transform_input.clone());
    let mork_api_client = MorkApiClient::new();

    match mork_api_client.dispatch(request).await {
        Ok(_) => Ok(Json(true)),
        Err(e) => Err(e),
    }
}

/// Performs a general restriction operation (x <| y) using an API-side trie.
/// Keeps each `(path ...)` fact from source[0] if there exists a `(prefix ...)` fact
/// in source[1] that is a prefix of the path tokens.
///
/// Payload:
/// `{ "source": ["/ns/paths", "/ns/prefixes"], "target": ["/ns/out"] }`
#[post("/spaces/restriction", data = "<input>")]
pub async fn restriction(
    token: Token,
    input: Json<SetOperationInput>,
) -> Result<Json<bool>, Status> {
    mod restriction_impl {
        use std::collections::HashMap;

        fn normalize_ns(ns: &str) -> String {
            let trimmed = ns.trim();
            let trimmed = trimmed.strip_prefix('/').unwrap_or(trimmed);
            let mut s = trimmed.to_string();
            if !s.ends_with('/') {
                s.push('/');
            }
            s
        }

        pub(super) fn ns_is_within(token_ns: &str, requested: &str) -> bool {
            let token_norm = normalize_ns(token_ns);
            let req_norm = normalize_ns(requested);
            req_norm.starts_with(&token_norm)
        }

        pub(super) fn normalize_ns_for_mork(ns: &str) -> String {
            normalize_ns(ns)
        }

        #[derive(Default)]
        pub(super) struct TrieNode {
            children: HashMap<String, TrieNode>,
            terminal: bool,
        }

        impl TrieNode {
            pub(super) fn insert(&mut self, prefix: &[String]) {
                let mut node = self;
                for part in prefix {
                    node = node.children.entry(part.clone()).or_default();
                }
                node.terminal = true;
            }

            pub(super) fn has_any_prefix(&self, path: &[String]) -> bool {
                let mut node = self;
                for part in path {
                    if node.terminal {
                        return true;
                    }
                    match node.children.get(part) {
                        Some(n) => node = n,
                        None => return false,
                    }
                }
                node.terminal
            }
        }

        pub(super) fn extract_balanced_sexprs(text: &str) -> Vec<String> {
            let mut out = Vec::new();
            let mut stack: Vec<usize> = Vec::new();

            for (idx, ch) in text.char_indices() {
                match ch {
                    '(' => stack.push(idx),
                    ')' => {
                        if let Some(start) = stack.pop() {
                            out.push(text[start..=idx].to_string());
                        }
                    }
                    _ => {}
                }
            }

            out
        }

        pub(super) fn parse_flat_list(expr: &str) -> Option<Vec<String>> {
            let s = expr.trim();
            if !s.starts_with('(') || !s.ends_with(')') {
                return None;
            }
            let inner = &s[1..s.len() - 1];
            if inner.contains('(') || inner.contains(')') {
                return None;
            }
            let parts: Vec<String> = inner
                .split_whitespace()
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            if parts.is_empty() {
                None
            } else {
                Some(parts)
            }
        }
    }

    let input = input.into_inner();

    if input.source.len() != 2 || input.target.len() != 1 {
        return Err(Status::BadRequest);
    }

    let src_paths = input.source[0].clone();
    let src_prefixes = input.source[1].clone();
    let dst = input.target[0].clone();

    if !token.permission_read || !token.permission_write {
        return Err(Status::Unauthorized);
    }
    if !restriction_impl::ns_is_within(&token.namespace, &src_paths)
        || !restriction_impl::ns_is_within(&token.namespace, &src_prefixes)
        || !restriction_impl::ns_is_within(&token.namespace, &dst)
    {
        return Err(Status::Unauthorized);
    }

    let mork_api_client = MorkApiClient::new();

    let export_all = |ns: &str| {
        ExportRequest::new()
            .namespace(PathBuf::from(restriction_impl::normalize_ns_for_mork(ns)))
            .pattern("$x".to_string())
            .template("$x".to_string())
            .format(ExportFormat::Metta)
    };

    let prefixes_text = mork_api_client.dispatch(export_all(&src_prefixes)).await?;
    let paths_text = mork_api_client.dispatch(export_all(&src_paths)).await?;

    let mut trie = restriction_impl::TrieNode::default();
    for sexpr in restriction_impl::extract_balanced_sexprs(&prefixes_text) {
        if let Some(parts) = restriction_impl::parse_flat_list(&sexpr) {
            if parts.first().map(|s| s.as_str()) == Some("prefix") && parts.len() >= 2 {
                trie.insert(&parts[1..]);
            }
        }
    }

    let mut survivors: Vec<String> = Vec::new();
    for sexpr in restriction_impl::extract_balanced_sexprs(&paths_text) {
        if let Some(parts) = restriction_impl::parse_flat_list(&sexpr) {
            if parts.first().map(|s| s.as_str()) == Some("path") && parts.len() >= 2 {
                let tokens = parts[1..].to_vec();
                if trie.has_any_prefix(&tokens) {
                    survivors.push(sexpr);
                }
            }
        }
    }

    if survivors.is_empty() {
        return Ok(Json(true));
    }

    let data = format!("{}\n", survivors.join("\n"));
    let upload = UploadRequest::new()
        .namespace(PathBuf::from(restriction_impl::normalize_ns_for_mork(&dst)))
        .pattern("$x".to_string())
        .template("$x".to_string())
        .data(data);

    let _ = mork_api_client.dispatch(upload).await?;
    Ok(Json(true))
}

/// Performs an intersection operation on provided namespaces. `token` must have `permission_write`
/// on the target namespace and `permission_read` on all source namespaces.
/// Intersection is implemented as a positive join on a shared variable across all sources.
/// # Intersection Transformation (conceptual)
/// ```lisp
/// (transform
///     (, (namespace1 $x) (namespace2 $x) ...)  ; all sources share the same variable $x
///     (, (target $x))
/// )
/// ```
#[post("/spaces/intersection", data = "<operation_input>")]
pub async fn intersection(
    token: Token,
    operation_input: Json<SetOperationInput>,
) -> Result<Json<bool>, Status> {
    // check `permission read` for all sources
    if !operation_input.source_target_permissions(token) {
        return Err(Status::Unauthorized);
    }

    let transform_input = intersection_transform(operation_input.into_inner())?;

    let request = TransformRequest::new().transform_input(transform_input);
    let mork_api_client = MorkApiClient::new();

    match mork_api_client.dispatch(request).await {
        Ok(_) => Ok(Json(true)),
        Err(e) => Err(e),
    }
}

#[post("/spaces/union", data = "<operation_input>")]
pub async fn union(
    token: Token,
    operation_input: Json<SetOperationInput>,
) -> Result<Json<bool>, Status> {
    if !operation_input.source_target_permissions(token) {
        return Err(Status::Unauthorized);
    }

    // path to be used for polling
    let request_path = match operation_input.clone().into_inner().target.first() {
        Some(value) => value.clone(),
        None => return Err(Status::BadRequest),
    };

    // create a vector of queries
    let transform_inputs = union_transform(operation_input.into_inner())?;
    let mork_api_client = MorkApiClient::new();

    for transform_input in transform_inputs {
        let request = TransformRequest::new().transform_input(transform_input);

        match mork_api_client.dispatch(request).await {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        // poll status endpoint
        poll(PathBuf::from(&request_path), &mork_api_client).await?;
    }

    Ok(Json(true))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////// HELPER FUNCTIONS ////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn poll(path: PathBuf, mork_api_client: &MorkApiClient) -> Result<bool, Status> {
    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(40);

    // Check if space is clear by using status endpoint
    let check_request = StatusRequest::new()
        .namespace(path.clone())
        .pattern("$x".to_string());

    loop {
        // exit condition stop polling after some second
        if start_time.elapsed() > timeout_duration {
            return Err(Status::RequestTimeout);
        }

        // wait 1 second between each status request
        sleep(Duration::from_millis(1000)).await;

        // destructure status endpoint json response
        let status_response: StatusResponse =
            match mork_api_client.dispatch(check_request.clone()).await {
                Ok(result) => match json::from_str::<StatusResponse>(&result) {
                    Ok(c) => c,
                    Err(_) => return Err(Status::RequestTimeout),
                },
                Err(_) => return Err(Status::RequestTimeout),
            };

        if status_response.status == "pathClear" {
            break;
        }
    }
    Ok(true)
}

fn composition_transform(input: SetOperationInput) -> Result<TransformDetails, Status> {
    let mut template = String::new();

    let patterns = input
        .source
        .iter()
        .enumerate()
        .map(|(index, source_ns)| {
            let c = index.to_string();
            template.push('$');
            template.push_str(&c);
            template.push(' ');

            Mm2Cell::new_pattern(format!("${}", c), Namespace::from(PathBuf::from(source_ns)))
        })
        .collect::<Vec<Mm2Cell>>();

    let transform_input =
        TransformDetails::new()
            .patterns(patterns)
            .templates(vec![Mm2Cell::new_template(
                template,
                Namespace::from(PathBuf::from(input.target.first().cloned().unwrap())),
            )]);

    Ok(transform_input)
}

fn intersection_transform(input: SetOperationInput) -> Result<TransformDetails, Status> {
    // Require at least 2 sources and exactly 1 target
    if input.source.len() < 2 || input.target.len() != 1 {
        return Err(Status::BadRequest);
    }

    let patterns = input
        .source
        .iter()
        .map(|source_ns| {
            Mm2Cell::new_pattern("$x".to_string(), Namespace::from(PathBuf::from(source_ns)))
        })
        .collect::<Vec<Mm2Cell>>();

    let transform_input =
        TransformDetails::new()
            .patterns(patterns)
            .templates(vec![Mm2Cell::new_template(
                "$x".to_string(),
                Namespace::from(PathBuf::from(input.target.first().cloned().unwrap())),
            )]);

    Ok(transform_input)
}

fn union_transform(input: SetOperationInput) -> Result<Vec<TransformDetails>, Status> {
    // Exceed the maximum number of source namespaces for composition, 26
    // and
    // Only one target namespace is allowed
    if input.source.len() > 26 && input.target.len() != 1 {
        return Err(Status::BadRequest);
    }

    let mut union_query: Vec<TransformDetails> = Vec::new();

    for source_ns in input.source.iter() {
        union_query.push(
            TransformDetails::new()
                .patterns(vec![Mm2Cell::new_pattern(
                    "$x".to_string(),
                    Namespace::from_path_string(source_ns),
                )])
                .templates(vec![Mm2Cell::new_template(
                    "$x".to_string(),
                    Namespace::from_path_string(input.target.first().unwrap()),
                )]),
        );
    }

    Ok(union_query)
}

// unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_transform() {
        let input = SetOperationInput {
            source: vec!["ns1".to_string(), "ns2".to_string()],
            target: vec!["ns3".to_string()],
        };

        let transform_input = composition_transform(input).unwrap();

        assert_eq!(
            transform_input.patterns[0].build(),
            "(__root__ (ns1 (__ns1data__ $0)))".to_string()
        );
        assert_eq!(
            transform_input.patterns[1].build(),
            "(__root__ (ns2 (__ns2data__ $1)))".to_string()
        );
        assert_eq!(
            transform_input.templates[0].build(),
            "(__root__ (ns3 (__ns3data__ $0 $1 )))".to_string()
        );
    }

    #[test]
    fn test_union_transform() {
        let input = SetOperationInput {
            source: vec!["ns1".to_string(), "ns2".to_string()],
            target: vec!["ns3".to_string()],
        };

        let transform_inputs = union_transform(input).unwrap();

        assert_eq!(transform_inputs.len(), 2);

        assert_eq!(
            transform_inputs[0].patterns[0].build(),
            "(__root__ (ns1 (__ns1data__ $x)))".to_string()
        );
        assert_eq!(
            transform_inputs[1].patterns[0].build(),
            "(__root__ (ns2 (__ns2data__ $x)))".to_string()
        );
        assert_eq!(
            transform_inputs[0].templates[0].build(),
            "(__root__ (ns3 (__ns3data__ $x)))".to_string()
        );
        assert_eq!(
            transform_inputs[1].templates[0].build(),
            "(__root__ (ns3 (__ns3data__ $x)))".to_string()
        );
    }

    #[test]
    fn test_intersection_transform() {
        let input = SetOperationInput {
            source: vec!["ns1".to_string(), "ns2".to_string()],
            target: vec!["ns3".to_string()],
        };

        let transform_input = intersection_transform(input).unwrap();

        assert_eq!(transform_input.patterns.len(), 2);
        assert_eq!(transform_input.templates.len(), 1);

        assert_eq!(
            transform_input.patterns[0].build(),
            "(__root__ (ns1 (__ns1data__ $x)))".to_string()
        );
        assert_eq!(
            transform_input.patterns[1].build(),
            "(__root__ (ns2 (__ns2data__ $x)))".to_string()
        );
        assert_eq!(
            transform_input.templates[0].build(),
            "(__root__ (ns3 (__ns3data__ $x)))".to_string()
        );
    }
}
