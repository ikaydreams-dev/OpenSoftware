use std::collections::HashMap;
use engcode_stdlib::database::Database;
use engcode_stdlib::{WebServer, HtmlPage, AuthSystem};
use crate::value::Value;
use engcode_parser::Statement;

pub struct ExecutionContext {
    databases: HashMap<String, Database>,
    current_database: Option<String>,
    pub variables: HashMap<String, Value>,
    web_server: Option<WebServer>,
    html_page: Option<HtmlPage>,
    layouts: HashMap<String, String>,
    css_framework: Option<String>,
    functions: HashMap<String, (Vec<String>, Vec<Statement>)>, // name -> (params, body)
    auth_system: Option<AuthSystem>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            databases: HashMap::new(),
            current_database: None,
            variables: HashMap::new(),
            web_server: None,
            html_page: None,
            layouts: HashMap::new(),
            css_framework: None,
            functions: HashMap::new(),
            auth_system: None,
        }
    }

    pub fn add_database(&mut self, name: String, db: Database) {
        self.databases.insert(name, db);
    }

    pub fn get_database_mut(&mut self, name: &str) -> Option<&mut Database> {
        self.databases.get_mut(name)
    }

    pub fn get_database(&self, name: &str) -> Option<&Database> {
        self.databases.get(name)
    }

    pub fn set_current_database(&mut self, name: Option<String>) {
        self.current_database = name;
    }

    pub fn current_database(&self) -> Option<String> {
        self.current_database.clone()
    }

    // Variable management
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    // Web server management
    pub fn set_web_server(&mut self, server: WebServer) {
        self.web_server = Some(server);
    }

    pub fn get_web_server_mut(&mut self) -> Option<&mut WebServer> {
        self.web_server.as_mut()
    }

    pub fn take_web_server(&mut self) -> Option<WebServer> {
        self.web_server.take()
    }

    // HTML page management
    pub fn set_html_page(&mut self, page: HtmlPage) {
        self.html_page = Some(page);
    }

    pub fn get_html_page_mut(&mut self) -> Option<&mut HtmlPage> {
        self.html_page.as_mut()
    }

    pub fn take_html_page(&mut self) -> Option<HtmlPage> {
        self.html_page.take()
    }

    // Layout management
    pub fn store_layout(&mut self, name: String, html: String) {
        self.layouts.insert(name, html);
    }

    pub fn get_layout(&self, name: &str) -> Option<&String> {
        self.layouts.get(name)
    }

    // CSS framework management
    pub fn set_css_framework(&mut self, framework: Option<String>) {
        self.css_framework = framework;
    }

    pub fn css_framework(&self) -> Option<String> {
        self.css_framework.clone()
    }

    // Function management
    pub fn define_function(&mut self, name: String, parameters: Vec<String>, body: Vec<Statement>) {
        self.functions.insert(name, (parameters, body));
    }

    pub fn get_function(&self, name: &str) -> Option<&(Vec<String>, Vec<Statement>)> {
        self.functions.get(name)
    }

    // Auth management
    pub fn ensure_auth_system(&mut self, db: Database, secret_key: String) {
        if self.auth_system.is_none() {
            self.auth_system = Some(AuthSystem::new(db, secret_key));
        }
    }

    pub fn get_auth_system(&self) -> Option<&AuthSystem> {
        self.auth_system.as_ref()
    }

    pub fn get_auth_system_mut(&mut self) -> Option<&mut AuthSystem> {
        self.auth_system.as_mut()
    }

    // Snapshots used by script-level route handlers
    pub fn snapshot_functions(&self) -> HashMap<String, (Vec<String>, Vec<Statement>)> {
        self.functions.clone()
    }

    pub fn snapshot_auth(&self) -> Option<(String, String)> {
        self.auth_system.as_ref().map(|a| (a.database_name(), a.secret_key_name()))
    }
}
