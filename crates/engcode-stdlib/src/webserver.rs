use axum::{
    routing::{get, post},
    Router, Json,
    response::{Html, IntoResponse},
    http::StatusCode,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use crate::database::Database;

pub struct WebServer {
    router: Router,
    port: u16,
    data_cell: Arc<Mutex<Option<Database>>>,
}

impl WebServer {
    pub fn new(port: u16) -> Self {
        let router = Router::new();
        Self {
            router,
            port,
            data_cell: Arc::new(Mutex::new(None)),
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

    pub fn add_data_route(&mut self, method: &str, path: &str, collection: &str) {
        // Queries the configured database live, on every request
        let data_cell = self.data_cell.clone();
        let collection = collection.to_string();
        let route_path = path.to_string();

        let handler = move || {
            let data_cell = data_cell.clone();
            async move {
                let guard = data_cell.lock().unwrap();
                match guard.as_ref().map(|db| db.select_all(&collection)) {
                    Some(Ok(rows)) => {
                        let vals: Vec<Value> = rows
                            .iter()
                            .filter_map(|r| serde_json::from_str::<Value>(r).ok())
                            .collect();
                        (StatusCode::OK, Json(json!({ "data": vals }))).into_response()
                    }
                    _ => (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({ "error": "No database configured" })),
                    ).into_response(),
                }
            }
        };

        match method.to_lowercase().as_str() {
            "get" => {
                self.router = self.router.clone().route(&route_path, get(handler));
            }
            "post" => {
                self.router = self.router.clone().route(&route_path, post(handler));
            }
            _ => {}
        }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: String) {
        // Adds a route that returns the given static handler message
        let route_path = path.to_string();

        match method.to_lowercase().as_str() {
            "get" => {
                self.router = self.router.clone().route(&route_path, get(move || {
                    async move {
                        Json(json!({ "message": handler }))
                    }
                }));
            }
            "post" => {
                self.router = self.router.clone().route(&route_path, post(move || {
                    async move {
                        Json(json!({ "message": handler }))
                    }
                }));
            }
            _ => {}
        }
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
        let router = self.router.clone();

        // Create a new runtime for the web server
        let rt = Runtime::new().map_err(|e| e.to_string())?;

        println!("🌐 Web server starting on http://127.0.0.1:{}", self.port);

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
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = tokio::time::sleep(d) => {}
        }
    } else {
        let _ = tokio::signal::ctrl_c().await;
    }
}

pub fn create_server(port: u16) -> WebServer {
    WebServer::new(port)
}

// Renders a page from disk, expanding {{collection}} template markers
async fn render_page_from_disk(file_path: String, data_cell: Arc<Mutex<Option<Database>>>) -> impl IntoResponse {
    match tokio::fs::read_to_string(&file_path).await {
        Ok(contents) => {
            let guard = data_cell.lock().unwrap();
            let html = if let Some(db) = guard.as_ref() {
                render_templates(&contents, db)
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
                render_templates(&contents, db)
            } else {
                contents
            };
            (StatusCode::OK, Html(html)).into_response()
        }
        Err(_) => match tokio::fs::read_to_string("public/home.html").await {
            Ok(contents) => {
                let guard = data_cell.lock().unwrap();
                let html = if let Some(db) = guard.as_ref() {
                    render_templates(&contents, db)
                } else {
                    contents
                };
                (StatusCode::OK, Html(html)).into_response()
            }
            Err(_) => Html("<h1>Welcome to EnglishCode</h1><p>No pages found</p>".to_string()).into_response(),
        },
    }
}

// Replaces every {{collection_name}} marker with an HTML table of that collection's rows
fn render_templates(contents: &str, db: &Database) -> String {
    let mut result = contents.to_string();
    while let Some(start) = result.find("{{") {
        let rest = &result[start + 2..];
        let Some(rel_end) = rest.find("}}") else { break };
        let end = start + 2 + rel_end; // index of the first '}' of '}}'
        let name = rest[..rel_end].trim().to_string();
        let replacement = match db.select_all(&name) {
            Ok(rows) => render_collection_table(&rows),
            Err(_) => format!("<!-- collection '{}' not found -->", name),
        };
        result.replace_range(start..=end + 1, &replacement);
    }
    result
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
