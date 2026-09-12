#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    CreateDatabase {
        name: String,
    },
    CreateCollections {
        database: Option<String>,
        names: Vec<String>,
    },
    Show {
        message: Expression,
    },
    Assignment {
        variable: String,
        value: Expression,
    },
    SetCookie {
        name: String,
        value: Expression,
    },
    Insert {
        collection: String,
        data: Vec<(String, Expression)>, // key-value pairs
    },
    InsertRaw {
        collection: String,
        value: Expression,
    },
    Select {
        collection: String,
        fields: Vec<String>, // empty means all
        condition: Option<Expression>,
        join: Option<JoinClause>,
    },
    Update {
        collection: String,
        data: Vec<(String, Expression)>,
        condition: Option<Expression>,
    },
    Delete {
        collection: String,
        condition: Option<Expression>,
    },
    CreateServer {
        port: u16,
    },
    AddRoute {
        method: String, // "get", "post", "put", "delete"
        path: String,
        response: Expression,
        status_code: Option<u16>,
    },
    AddDataRoute {
        method: String,
        path: String,
        collection: String,
    },
    StartServer {
        duration_seconds: Option<f64>,
    },
    // Web Enhancement Statements
    AddHandler {
        method: String,
        path: String,
        body_var: String,
        body: Vec<Statement>,
        status_code: Option<u16>,
    },
    AddMiddleware {
        middleware_type: String,
    },
    // HTML/UI Statements
    CreatePage {
        name: String,
        title: Option<String>,
        layout: Option<String>,
    },
    AddCss {
        framework: String,
    },
    CreateLayout {
        name: String,
    },
    RenderLayout {
        name: String,
    },
    AddUploadRoute {
        path: String,
        directory: String,
    },
    AddWebSocketRoute {
        path: String,
    },
    AddRateLimit {
        limit: u64,
        window_secs: u64,
    },
    BeginTransaction,
    CommitTransaction,
    RollbackTransaction,
    AddUIComponent {
        component: String,  // "toast", "alert", "spinner", "modal", "tabs", "accordion", "container", "grid"
        text: String,
        title: Option<String>,
        items: Vec<(String, String)>, // (label, content) pairs for tabs/accordion
    },
    AddElement {
        element_type: String, // "div", "button", "form", "input", etc
        properties: Vec<(String, Expression)>, // id, class, text, etc
    },
    AddButton {
        text: String,
        properties: Vec<(String, Expression)>,
    },
    AddForm {
        properties: Vec<(String, Expression)>,
    },
    AddInput {
        input_type: String, // "text", "email", "password", etc
        properties: Vec<(String, Expression)>,
    },
    AddHeading {
        level: u8, // 1-6 for h1-h6
        text: String,
    },
    AddParagraph {
        text: String,
    },
    AddLink {
        text: String,
        url: String,
    },
    AddImage {
        src: String,
        alt: String,
    },
    SetStyle {
        selector: String,
        styles: Vec<(String, String)>, // property-value pairs
    },
    RenderPage {
        page_name: String,
    },
    // Control Flow Statements
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        variable: String,
        start: Expression,
        end: Expression,
        body: Vec<Statement>,
    },
    ForEach {
        variable: String,
        collection: Expression,
        body: Vec<Statement>,
    },
    Break,
    Continue,
    // Mobile navigation
    NavigateTo {
        page: Expression,
    },
    GoBack,
    FetchData {
        url: Expression,
        variable: String,
    },
    // Function Statements
    FunctionDef {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    Return {
        value: Option<Expression>,
    },
    // Error Handling
    TryCatch {
        try_block: Vec<Statement>,
        catch_block: Vec<Statement>,
        finally_block: Option<Vec<Statement>>,
    },
    Throw {
        message: Expression,
    },
    // Request validation
    Validate {
        rules: Vec<ValidationRule>,
    },
    // Authentication
    Signup {
        username: String,
        email: String,
        password: String,
    },
    Login {
        username: String,
        password: String,
    },
    Logout,
    // Testing
    TestBlock {
        name: String,
        body: Vec<Statement>,
    },
    Assert {
        condition: Expression,
        message: Option<String>,
    },
    // File I/O
    ReadFile {
        path: Expression,
        into: String,
    },
    WriteFile {
        path: Expression,
        content: Expression,
    },
    AppendFile {
        path: Expression,
        content: Expression,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Identifier(String),
    Array(Vec<Expression>),
    Object(Vec<(String, Expression)>),
    // Comparison expressions
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
    IndexAccess {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },
}

// An inner JOIN between two collections on a boolean condition.
// e.g. select all from orders join customers on orders.customerid is customers.id
#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub collection: String,
    pub condition: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    EqualTo,
    NotEqualTo,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    And,
    Or,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negative,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationRule {
    Required(String),
    Type { field: String, expected: String },
    Min { field: String, value: f64 },
    Max { field: String, value: f64 },
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}
