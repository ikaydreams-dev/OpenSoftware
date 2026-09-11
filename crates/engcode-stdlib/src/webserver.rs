    use axum::{
    body::Bytes,
    extract::Query,
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, Request},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Router, Json,
    http::StatusCode,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use crate::database::Database;

// A script-level route: body is passed in as parsed JSON (or None), along with the
// request headers (for cookies/sessions). The function returns the JSON response,
// the HTTP status code, and any response headers to set (e.g. Set-Cookie).
pub type RouteFunc = dyn Fn(Option<Value>, Vec<(String, String)>) -> Result<(Value, u16, Vec<(String, String)>), String> + Send + Sync + 'static;

pub struct WebServer {
    router: Router,
    port: u16,
    data_cell: Arc<Mutex<Option<Database>>>,
    middlewares: Vec<String>,
}

impl WebServer {
    pub fn new(port: u16) -> Self {
        let router = Router::new();
        Self {
            router,
            port,
            data_cell: Arc::new(Mutex::new(None)),
            middlewares: Vec::new(),
        }
    }

    pub fn set_data_database(&mut self, db: Database) {
        let mut guard = self.data_cell.lock().unwrap();
        if guard.is_none() {
            *guard = Some(db);
        }
    }

    pub fn has_data_database(&self) -> bool {
        self.data_cell.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    // Adds a middleware to the server. Supported types: "cors", "logging".
    // Applied lazily in start() so it wraps every route, including ones added
    // after this call (axum layers only affect routes present at layer time).
    pub fn add_middleware(&mut self, middleware_type: &str) {
        match middleware_type.to_lowercase().as_str() {
            "cors" | "logging" => self.middlewares.push(middleware_type.to_lowercase()),
            _ => {}
        }
    }

    // Adds a multipart file upload route that saves uploaded files to a directory.
    pub fn add_upload_route(&mut self, path: &str, directory: &str) {
        let save_dir = directory.to_string();
        let route_path = path.to_string();
        let handler = move |mut multipart: axum::extract::Multipart| {
            let save_dir = save_dir.clone();
            async move {
                let _ = std::fs::create_dir_all(&save_dir);
                let mut saved = Vec::new();
                while let Some(field) = multipart
                    .next_field()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string(), "status": 400 }))))?
                {
                    let file_name = field
                        .file_name()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("upload-{}", saved.len() + 1));
                    let data = field
                        .bytes()
                        .await
                        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string(), "status": 400 }))))?;
                    let dest = std::path::Path::new(&save_dir).join(&file_name);
                    if let Err(e) = std::fs::write(&dest, &data) {
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string(), "status": 500 }))));
                    }
                    saved.push(json!({
                        "filename": file_name,
                        "size": data.len(),
                        "path": dest.to_string_lossy().to_string(),
                    }));
                }
                Ok(Json(json!({ "uploaded": saved, "status": 201 })))
            }
        };
        self.router = self.router.clone().route(&route_path, post(handler));
    }

    // Adds a script-level route handled by a closure installed by the interpreter.
    pub fn add_script_route(&mut self, method: &str, path: &str, status: Option<u16>, func: Arc<RouteFunc>) {
        let default_status = status.unwrap_or(200);
        let route_path = path.to_string();

        let build_handler = || {
            let func = func.clone();
            let default_status = default_status;
            move |headers: HeaderMap, body: Bytes| {
                let func = func.clone();
                async move {
                    // Convert request headers into (name, value) pairs.
                    let mut header_vec: Vec<(String, String)> = Vec::new();
                    for (name, value) in headers.iter() {
                        if let Ok(v) = value.to_str() {
                            header_vec.push((name.as_str().to_string(), v.to_string()));
                        }
                    }

                    let body_value: Option<Value> = if body.is_empty() {
                        None
                    } else {
                        serde_json::from_slice(&body).ok()
                    };

                    match func(body_value, header_vec) {
                        Ok((value, code, response_headers)) => {
                            let code = if code == 0 { default_status } else { code };
                            let mut response = (
                                StatusCode::from_u16(code).unwrap_or(StatusCode::OK),
                                Json(value),
                            ).into_response();
                            for (name, val) in response_headers {
                                response.headers_mut().append(
                                    HeaderName::from_bytes(name.as_bytes()).unwrap_or(header::SET_COOKIE),
                                    val.parse().unwrap_or_else(|_| HeaderValue::from_static("")),
                                );
                            }
                            response
                        }
                        Err(e) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": e, "status": 500 })),
                        ).into_response(),
                    }
                }
            }
        };

        match method.to_lowercase().as_str() {
            "get" => {
                let handler = build_handler();
                self.router = self.router.clone().route(&route_path, get(handler));
            }
            "post" => {
                let handler = build_handler();
                self.router = self.router.clone().route(&route_path, post(handler));
            }
            "put" => {
                let handler = build_handler();
                self.router = self.router.clone().route(&route_path, put(handler));
            }
            "delete" => {
                let handler = build_handler();
                self.router = self.router.clone().route(&route_path, delete(handler));
            }
            _ => {}
        }
    }

    // Adds a data route backed by the configured database.
    // GET:    returns rows (supports ?page=,&limit=,&sort=,&order=,&filter.<field>=)
    // POST:   inserts the JSON body into the collection (returns 201 + created row)
    // PUT:    updates the row identified by "id" in the JSON body
    // DELETE: deletes the row identified by "id" in the JSON body or ?id=
    pub fn add_data_route(&mut self, method: &str, path: &str, collection: &str) {
        let data_cell = self.data_cell.clone();
        let collection = collection.to_string();
        let route_path = path.to_string();

        match method.to_lowercase().as_str() {
            "get" => {
                let handler = {
                    let data_cell = data_cell.clone();
                    let collection = collection.clone();
                    move |_method: Method, Query(params): Query<HashMap<String, String>>| {
                        let data_cell = data_cell.clone();
                        let collection = collection.clone();
                        async move {
                            handle_data_get(&data_cell, &collection, &params).await
                        }
                    }
                };
                self.router = self.router.clone().route(&route_path, get(handler));
            }
            "post" => {
                let handler = {
                    let data_cell = data_cell.clone();
                    let collection = collection.clone();
                    move |body: Bytes| {
                        let data_cell = data_cell.clone();
                        let collection = collection.clone();
                        async move {
                            handle_data_post(&data_cell, &collection, &body).await
                        }
                    }
                };
                self.router = self.router.clone().route(&route_path, post(handler));
            }
            "put" => {
                let handler = {
                    let data_cell = data_cell.clone();
                    let collection = collection.clone();
                    move |body: Bytes| {
                        let data_cell = data_cell.clone();
                        let collection = collection.clone();
                        async move {
                            handle_data_put(&data_cell, &collection, &body).await
                        }
                    }
                };
                self.router = self.router.clone().route(&route_path, put(handler));
            }
            "delete" => {
                let handler = {
                    let data_cell = data_cell.clone();
                    let collection = collection.clone();
                    move |_method: Method, Query(params): Query<HashMap<String, String>>, body: Bytes| {
                        let data_cell = data_cell.clone();
                        let collection = collection.clone();
                        async move {
                            handle_data_delete(&data_cell, &collection, &params, &body).await
                        }
                    }
                };
                self.router = self.router.clone().route(&route_path, delete(handler));
            }
            _ => {}
        }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: String, status: Option<u16>) {
        let code = StatusCode::from_u16(status.unwrap_or(200)).unwrap_or(StatusCode::OK);
        let route_path = path.to_string();

        // MethodRouter with a handler that returns the static message and status.
        // Build one handler closure reused across methods.
        let make_router = |method: &str| -> axum::routing::MethodRouter {
            match method.to_lowercase().as_str() {
                "get" => get(move || {
                    let handler = handler.clone();
                    async move {
                        (code, Json(json!({ "message": handler }))).into_response()
                    }
                }),
                "post" => post(move || {
                    let handler = handler.clone();
                    async move {
                        (code, Json(json!({ "message": handler }))).into_response()
                    }
                }),
                "put" => put(move || {
                    let handler = handler.clone();
                    async move {
                        (code, Json(json!({ "message": handler }))).into_response()
                    }
                }),
                "delete" => delete(move || {
                    let handler = handler.clone();
                    async move {
                        (code, Json(json!({ "message": handler }))).into_response()
                    }
                }),
                _ => axum::routing::MethodRouter::new(),
            }
        };

        let router = make_router(method);
        self.router = self.router.clone().route(&route_path, router);
    }

    // Serves static files from public/ (CSS, JS, images, uploads).
    pub fn serve_static_files(mut self) -> Self {
        let assets_handler = |axum::extract::Path(path): axum::extract::Path<String>| {
            async move { serve_file(format!("public/assets/{}", path)).await }
        };
        self.router = self.router.clone().route("/assets/{*path}", get(assets_handler));

        let uploads_handler = |axum::extract::Path(path): axum::extract::Path<String>| {
            async move { serve_file(format!("public/uploads/{}", path)).await }
        };
        self.router = self.router.clone().route("/uploads/{*path}", get(uploads_handler));

        self
    }

    pub fn serve_html_pages(mut self) -> Self {
        // Add routes to serve generated HTML pages from the public/ directory,
        // rendering {{collection}} template markers with the live database.
        let data_cell = self.data_cell.clone();

        let page_handler = {
            let data_cell = data_cell.clone();
            move |axum::extract::Path(page): axum::extract::Path<String>| {
                let data_cell = data_cell.clone();
                async move {
                    render_page_from_disk(format!("public/{}.html", page), data_cell).await
                }
            }
        };
        self.router = self.router.clone().route("/{page}", get(page_handler));

        let index_handler = {
            let data_cell = data_cell.clone();
            move || {
                let data_cell = data_cell.clone();
                async move { serve_index_impl(data_cell).await }
            }
        };
        self.router = self.router.clone().route("/", get(index_handler));

        self
    }

    pub fn start(&self, duration: Option<Duration>) -> Result<(), String> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let mut router = self.router.clone();
        for mw in &self.middlewares {
            router = match mw.as_str() {
                "cors" => router.layer(middleware::from_fn(cors_middleware)),
                "logging" => router.layer(middleware::from_fn(logging_middleware)),
                _ => router,
            };
        }

        // Create a new runtime for the web server
        let rt = Runtime::new().map_err(|e| e.to_string())?;

        println!("Web server starting on http://127.0.0.1:{}", self.port);

        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| e.to_string())?;

            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown_signal(duration))
                .await
                .map_err(|e| e.to_string())?;

            Ok::<(), String>(())
        })?;

        println!("Web server stopped");

        Ok(())
    }
}

async fn shutdown_signal(duration: Option<Duration>) {
    if let Some(d) = duration {
        tokio::time::sleep(d).await;
    } else {
        let _ = tokio::signal::ctrl_c().await;
    }
}

pub fn create_server(port: u16) -> WebServer {
    WebServer::new(port)
}

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

async fn cors_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
    if req.method() == Method::OPTIONS {
        let mut response = Response::new(axum::body::Body::empty());
        let headers = response.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, header::HeaderValue::from_static("*"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, header::HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, header::HeaderValue::from_static("Content-Type, Authorization"));
        *response.status_mut() = StatusCode::NO_CONTENT;
        return response;
    }

    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, header::HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, header::HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, header::HeaderValue::from_static("Content-Type, Authorization"));
    response
}

async fn logging_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();
    let response = next.run(req).await;
    let status = response.status();
    println!("[engcode] {} {} -> {} ({}ms)", method, uri, status, start.elapsed().as_millis());
    response
}

// ---------------------------------------------------------------------------
// Data route handlers
// ---------------------------------------------------------------------------

async fn handle_data_get(
    data_cell: &Arc<Mutex<Option<Database>>>,
    collection: &str,
    params: &HashMap<String, String>,
) -> Response {
    let page = params.get("page").and_then(|p| p.parse::<usize>().ok()).unwrap_or(1).max(1);
    let limit = params
        .get("limit")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(100)
        .clamp(1, 1000);
    let sort = params.get("sort").map(|s| s.to_string());
    let desc = params.get("order").map(|o| o.to_lowercase() == "desc").unwrap_or(false);

    // Filter params: filter.<field>=<value>
    let mut filter = serde_json::Map::new();
    for (k, v) in params {
        if let Some(field) = k.strip_prefix("filter.") {
            if let Ok(n) = v.parse::<i64>() {
                filter.insert(field.to_string(), json!(n));
            } else if let Ok(f) = v.parse::<f64>() {
                filter.insert(field.to_string(), json!(f));
            } else if v.eq_ignore_ascii_case("true") {
                filter.insert(field.to_string(), json!(true));
            } else if v.eq_ignore_ascii_case("false") {
                filter.insert(field.to_string(), json!(false));
            } else {
                filter.insert(field.to_string(), json!(v));
            }
        }
    }

    let guard = data_cell.lock().unwrap();
    match guard.as_ref().map(|db| {
        db.select_filtered(collection, &Value::Object(filter), sort.as_deref(), desc)
    }) {
        Some(Ok(rows)) => {
            let total = rows.len();
            let start = (page - 1) * limit;
            let end = (start + limit).min(total);
            let page_rows: Vec<Value> = rows[start..end].iter().map(|(_, v)| v.clone()).collect();
            let pages = if limit == 0 { 0 } else { (total + limit - 1) / limit };

            (StatusCode::OK, Json(json!({
                "data": page_rows,
                "meta": {
                    "total": total,
                    "page": page,
                    "limit": limit,
                    "pages": pages
                }
            }))).into_response()
        }
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "No database configured", "status": 503 })),
        ).into_response(),
    }
}

async fn handle_data_post(
    data_cell: &Arc<Mutex<Option<Database>>>,
    collection: &str,
    body: &Bytes,
) -> Response {
    let Ok(payload) = serde_json::from_slice::<Value>(body) else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid JSON body", "status": 400 }))).into_response();
    };

    let mut guard = data_cell.lock().unwrap();
    match guard.as_mut() {
        Some(db) => {
            let data_str = payload.to_string();
            match db.insert_with_id(collection, &data_str) {
                Ok(id) => (StatusCode::CREATED, Json(json!({
                    "data": payload,
                    "id": id
                }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "error": e.to_string(), "status": 500
                }))).into_response(),
            }
        }
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "error": "No database configured", "status": 503 }))).into_response(),
    }
}

async fn handle_data_put(
    data_cell: &Arc<Mutex<Option<Database>>>,
    collection: &str,
    body: &Bytes,
) -> Response {
    let Ok(mut payload) = serde_json::from_slice::<Value>(body) else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid JSON body", "status": 400 }))).into_response();
    };

    let id = match payload.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Missing 'id' in body", "status": 400 }))).into_response();
        }
    };

    let mut guard = data_cell.lock().unwrap();
    match guard.as_mut() {
        Some(db) => {
            let Some(row) = db.get_row_by_id(collection, id).ok().flatten() else {
                return (StatusCode::NOT_FOUND, Json(json!({ "error": "Row not found", "status": 404 }))).into_response();
            };
            let mut merged = row;
            if let (Some(src), Some(dst)) = (payload.as_object_mut(), merged.as_object_mut()) {
                src.remove("id");
                for (k, v) in src.iter() {
                    dst.insert(k.clone(), v.clone());
                }
            }
            match db.update_row(collection, id, &merged.to_string()) {
                Ok(()) => (StatusCode::OK, Json(json!({ "data": merged }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string(), "status": 500 }))).into_response(),
            }
        }
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "error": "No database configured", "status": 503 }))).into_response(),
    }
}

async fn handle_data_delete(
    data_cell: &Arc<Mutex<Option<Database>>>,
    collection: &str,
    params: &HashMap<String, String>,
    body: &Bytes,
) -> Response {
    let id_from_query = params.get("id").and_then(|v| v.parse::<i64>().ok());
    let id_from_body = serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|v| v.get("id").and_then(|i| i.as_i64()).or_else(|| v.as_i64()));
    let id = id_from_query.or(id_from_body);

    let Some(id) = id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Missing 'id'", "status": 400 }))).into_response();
    };

    let mut guard = data_cell.lock().unwrap();
    match guard.as_mut() {
        Some(db) => match db.delete_by_id(collection, id) {
            Ok(true) => (StatusCode::OK, Json(json!({ "deleted": id }))).into_response(),
            Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Row not found", "status": 404 }))).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string(), "status": 500 }))).into_response(),
        },
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "error": "No database configured", "status": 503 }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Page rendering with templates
// ---------------------------------------------------------------------------

async fn render_page_from_disk(file_path: String, data_cell: Arc<Mutex<Option<Database>>>) -> impl IntoResponse {
    match tokio::fs::read_to_string(&file_path).await {
        Ok(contents) => {
            let guard = data_cell.lock().unwrap();
            let html = if let Some(db) = guard.as_ref() {
                render_full_template(&contents, db)
            } else {
                contents
            };
            (StatusCode::OK, Html(html)).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Html("<h1>404 Not Found</h1><p>Page not found</p>".to_string()),
        ).into_response(),
    }
}

async fn serve_index_impl(data_cell: Arc<Mutex<Option<Database>>>) -> impl IntoResponse {
    match tokio::fs::read_to_string("public/index.html").await {
        Ok(contents) => {
            let guard = data_cell.lock().unwrap();
            let html = if let Some(db) = guard.as_ref() {
                render_full_template(&contents, db)
            } else {
                contents
            };
            (StatusCode::OK, Html(html)).into_response()
        }
        Err(_) => match tokio::fs::read_to_string("public/home.html").await {
            Ok(contents) => {
                let guard = data_cell.lock().unwrap();
                let html = if let Some(db) = guard.as_ref() {
                    render_full_template(&contents, db)
                } else {
                    contents
                };
                (StatusCode::OK, Html(html)).into_response()
            }
            Err(_) => Html("<h1>Welcome to EnglishCode</h1><p>No pages found</p>".to_string()).into_response(),
        },
    }
}

// Main template entry point. Supports:
//   {{collection}}                  -> table of a collection
//   {{each <var> in <collection>}}  -> repeat the body for every row; {{<var>.<field>}}
//   {{if <collection>}} / {{if not <collection>}} -> conditional block
async fn serve_file(path: String) -> impl IntoResponse {
    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let content_type = match path.rsplit('.').next().unwrap_or("") {
                "css" => "text/css",
                "js" => "application/javascript",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "svg" => "image/svg+xml",
                "ico" => "image/x-icon",
                _ => "application/octet-stream",
            };
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                bytes,
            ).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

fn render_full_template(contents: &str, db: &Database) -> String {
    render_block_with_vars(contents, db, &[])
}

type ScopeVar = (String, Value);

fn render_block_with_vars(block: &str, db: &Database, vars: &[ScopeVar]) -> String {
    let mut out = String::new();
    let mut i = 0;
    let bytes = block.as_bytes();

    while i < bytes.len() {
        if block[i..].starts_with("{{") {
            let Some(rel_end) = block[i + 2..].find("}}") else {
                out.push_str(&block[i..]);
                break;
            };
            let marker = &block[i + 2..i + 2 + rel_end];
            let marker = marker.trim();

            if let Some(spec) = marker.strip_prefix("each ") {
                let body_after_marker = i + 2 + rel_end + 2;
                let (inner, next_i) = extract_block(block, body_after_marker, "{{endeach}}");
                out.push_str(&render_each_block(spec, &inner, db, vars));
                i = next_i;
            } else if let Some(cond) = marker.strip_prefix("if ") {
                let body_after_marker = i + 2 + rel_end + 2;
                let (inner, next_i) = extract_block(block, body_after_marker, "{{endif}}");
                out.push_str(&render_if_block(cond, &inner, db, vars));
                i = next_i;
            } else {
                out.push_str(&render_reference(marker, db, vars));
                i += 2 + rel_end + 2;
            }
        } else {
            match block[i..].find("{{") {
                Some(next) => {
                    out.push_str(&block[i..i + next]);
                    i += next;
                }
                None => {
                    out.push_str(&block[i..]);
                    break;
                }
            }
        }
    }

    out
}

// Extracts the text between the current position and the closing marker.
fn extract_block(block: &str, start: usize, closer: &str) -> (String, usize) {
    match block[start..].find(closer) {
        Some(pos) => (block[start..start + pos].to_string(), start + pos + closer.len()),
        None => (block[start..].to_string(), block.len()),
    }
}

fn render_each_block(spec: &str, inner: &str, db: &Database, vars: &[ScopeVar]) -> String {
    // spec is "<var> in <collection>"
    let parts: Vec<&str> = spec.splitn(3, " in ").collect();
    if parts.len() != 3 {
        return String::new();
    }
    let var_name = parts[0].trim();
    let collection = parts[2].trim();

    let Ok(rows) = db.select_all(collection) else {
        return format!("<!-- collection '{}' not found -->", collection);
    };

    let mut out = String::new();
    for row in rows {
        if let Ok(value) = serde_json::from_str::<Value>(&row) {
            let mut scope = vars.to_vec();
            scope.push((var_name.to_string(), value));
            out.push_str(&render_block_with_vars(inner, db, &scope));
        }
    }
    out
}

fn render_if_block(cond: &str, inner: &str, db: &Database, vars: &[ScopeVar]) -> String {
    let negate = cond.trim_start().starts_with("not ");
    let name = cond.trim_start().trim_start_matches("not ").trim();
    let has_rows = db.select_all(name).map(|r| !r.is_empty()).unwrap_or(false);
    if has_rows && !negate || !has_rows && negate {
        render_block_with_vars(inner, db, vars)
    } else {
        String::new()
    }
}

fn render_reference(marker: &str, db: &Database, vars: &[ScopeVar]) -> String {
    // Field reference: <var>.<field>
    if let Some(dot) = marker.find('.') {
        let var_name = &marker[..dot];
        let field = &marker[dot + 1..];
        if let Some((_, value)) = vars.iter().rev().find(|(name, _)| name == var_name) {
            return field_value_to_text(value, field);
        }
    }

    // Plain variable reference
    if let Some((_, value)) = vars.iter().rev().find(|(name, _)| name == marker) {
        return json_value_to_text(value);
    }

    // Collection table
    if let Ok(rows) = db.select_all(marker) {
        return render_collection_table(&rows);
    }

    format!("<!-- '{}' not found -->", marker)
}

fn field_value_to_text(value: &Value, field: &str) -> String {
    match value.get(field) {
        Some(v) => json_value_to_text(v),
        None => String::new(),
    }
}

// Renders rows (JSON strings) as an HTML table with unioned columns
fn render_collection_table(rows: &[String]) -> String {
    let parsed: Vec<Value> = rows
        .iter()
        .filter_map(|r| serde_json::from_str::<Value>(r).ok())
        .collect();

    if parsed.is_empty() {
        return "<p><em>No records found</em></p>".to_string();
    }

    let mut columns: Vec<String> = Vec::new();
    for row in &parsed {
        if let Some(obj) = row.as_object() {
            for k in obj.keys() {
                if !columns.contains(k) {
                    columns.push(k.clone());
                }
            }
        }
    }

    let mut html = String::from("<table border=\"1\" cellpadding=\"6\" cellspacing=\"0\"><thead><tr>");
    for col in &columns {
        html.push_str(&format!("<th>{}</th>", escape_html(col)));
    }
    html.push_str("</tr></thead><tbody>");

    for row in &parsed {
        if let Some(obj) = row.as_object() {
            html.push_str("<tr>");
            for col in &columns {
                let val = obj
                    .get(col)
                    .map(json_value_to_text)
                    .unwrap_or_default();
                html.push_str(&format!("<td>{}</td>", escape_html(&val)));
            }
            html.push_str("</tr>");
        }
    }

    html.push_str("</tbody></table>");
    html
}

fn json_value_to_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                return i.to_string();
            }
            if let Some(u) = n.as_u64() {
                return u.to_string();
            }
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 {
                    return format!("{}", f as i64);
                }
                return f.to_string();
            }
            n.to_string()
        }
        other => other.to_string(),
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}