use crate::ast::{Program, Statement, Expression, UnaryOperator, BinaryOperator, JoinClause};
use engcode_lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    current_database: Option<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            current_database: None,
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut program = Program::new();

        while !self.is_at_end() {
            // Skip newlines
            while self.current_token() == &Token::Newline {
                self.advance();
            }

            if self.is_at_end() {
                break;
            }

            let statement = self.parse_statement()?;
            program.add_statement(statement);
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token() {
            Token::Create => self.parse_create_statement(),
            Token::Show => self.parse_show_statement(),
            Token::Set | Token::Let | Token::Make | Token::Store | Token::Save => {
                self.parse_assignment()
            }
            Token::Add => {
                // Check what follows "add" to route to correct handler
                match self.peek_token() {
                    Some(Token::Route) | Some(Token::Endpoint) => self.parse_add_route(),
                    Some(Token::Middleware) => self.parse_add_middleware(),
                    Some(Token::Button) => self.parse_add_button(),
                    Some(Token::Input) => self.parse_add_input(),
                    Some(Token::Form) => self.parse_add_form(),
                    Some(Token::Element) => self.parse_add_element(),
                    Some(Token::Heading) => self.parse_add_heading(),
                    Some(Token::Paragraph) => self.parse_add_paragraph(),
                    Some(Token::Image) | Some(Token::Icon) => self.parse_add_image(),
                    Some(Token::Link) => self.parse_add_link(),
                    Some(Token::Style) => self.parse_add_css(),
                    Some(Token::Identifier(name)) if name == "css" => self.parse_add_css(),
                    Some(Token::Upload) => self.parse_add_upload_route(),
                    Some(Token::WebSocket) => self.parse_add_websocket_route(),
                    Some(Token::Identifier(name)) if name == "websocket" || name == "socket" => {
                        self.parse_add_websocket_route()
                    }
                    Some(Token::Identifier(name)) if name == "rate" => self.parse_add_rate_limit(),
                    Some(Token::Toast) | Some(Token::Alert) | Some(Token::Spinner)
                    | Some(Token::Loading) | Some(Token::Modal)
                    | Some(Token::Tabs) | Some(Token::Accordion)
                    | Some(Token::Container) | Some(Token::Grid) => {
                        self.parse_add_ui_component()
                    }
                    _ => self.parse_insert(), // Default to insert for data operations
                }
            }
            Token::Handle => self.parse_add_handler(),
            Token::Insert | Token::Put => self.parse_insert(),
            Token::Select | Token::Get | Token::Find | Token::Fetch => self.parse_select(),
            Token::Update | Token::Change | Token::Modify => self.parse_update(),
            Token::Delete | Token::Remove => self.parse_delete(),
            Token::Start => self.parse_start_server(),
            Token::Render => self.parse_render_page(),
            Token::If | Token::When => self.parse_if_statement(),
            Token::While => self.parse_while_loop(),
            Token::For => self.parse_for_loop(),
            Token::Break => {
                self.advance();
                Ok(Statement::Break)
            }
            Token::Continue => {
                self.advance();
                Ok(Statement::Continue)
            }
            Token::Define => self.parse_function_def(),
            Token::Call | Token::Invoke | Token::Run => self.parse_function_call_statement(),
            Token::Return => self.parse_return(),
            Token::Try => self.parse_try_catch(),
            Token::Throw | Token::Raise => self.parse_throw(),
            Token::Identifier(name) if name == "validate" => self.parse_validate(),
            Token::Identifier(name) if name == "navigate" => self.parse_navigate(),
            Token::Identifier(name) if name == "go" => self.parse_go_back(),
            Token::Signup => self.parse_signup(),
            Token::Login => self.parse_login(),
            Token::Logout => self.parse_logout(),
            Token::Test => self.parse_test_block(),
            Token::Assert | Token::Expect => self.parse_assert(),
            Token::Begin => self.parse_begin_transaction(),
            Token::Commit => {
                self.advance();
                Ok(Statement::CommitTransaction)
            }
            Token::Rollback => {
                self.advance();
                Ok(Statement::RollbackTransaction)
            }
            Token::Read => self.parse_read_file(),
            Token::Write => self.parse_write_file(),
            Token::Append => self.parse_append_file(),
            Token::Identifier(_) | Token::Text | Token::Item | Token::Value | Token::Image | Token::Link => {
                // Could be method call statement like: items.push(5)
                self.parse_expression_statement()
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token())),
        }
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expression()?;

        // For now, we only support method calls as statements
        // Wrap in a dummy assignment or just execute the expression
        match expr {
            Expression::MethodCall { .. } => {
                // Create a temporary variable to hold the result
                Ok(Statement::Assignment {
                    variable: "_".to_string(),
                    value: expr,
                })
            }
            _ => Err("Only method calls can be used as statements".to_string()),
        }
    }

    fn parse_create_statement(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Create)?;

        // Skip optional articles
        if matches!(self.current_token(), Token::A | Token::An | Token::The) {
            self.advance();
        }

        match self.current_token() {
            Token::Web | Token::Server => return self.parse_create_server_inner(),
            Token::Page => return self.parse_create_page(),
            Token::Database | Token::DB => self.parse_create_database(),
            Token::Collections | Token::Collection => self.parse_create_collections(),
            Token::Identifier(name) if name == "layout" || name == "template" => {
                return self.parse_create_layout()
            }
            Token::These => {
                self.advance();
                self.parse_create_collections()
            }
            _ => Err("Expected 'database', 'collections', or 'web server' after 'create'".to_string()),
        }
    }

    fn parse_create_server_inner(&mut self) -> Result<Statement, String> {
        // Already consumed "create" and optional "a"
        // Now at "web" or "server"

        // Consume "web" or "server"
        if matches!(self.current_token(), Token::Web | Token::Server) {
            self.advance();
        }

        // Skip "server" if we had "web"
        if matches!(self.current_token(), Token::Server) {
            self.advance();
        }

        // Consume "on"
        if matches!(self.current_token(), Token::On) {
            self.advance();
        }

        // Consume "port"
        if matches!(self.current_token(), Token::Port) {
            self.advance();
        }

        // Get port number
        let port = match self.current_token() {
            Token::Number(n) => *n as u16,
            _ => return Err("Expected port number".to_string()),
        };
        self.advance();

        Ok(Statement::CreateServer { port })
    }

    fn parse_create_database(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'database' or 'db'

        self.consume(&Token::Called)?;

        let name = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected database name in quotes".to_string()),
        };

        self.advance();

        // Track current database
        self.current_database = Some(name.clone());

        Ok(Statement::CreateDatabase { name })
    }

    fn parse_create_collections(&mut self) -> Result<Statement, String> {
        // consume 'collections' or 'collection'
        if matches!(self.current_token(), Token::Collections | Token::Collection) {
            self.advance();
        }

        // Form A: create collections called "users" "files" in it
        let names = if matches!(self.current_token(), Token::Called | Token::Named) {
            self.advance();
            let n = self.parse_identifier_list()?;
            if matches!(self.current_token(), Token::In) {
                self.advance();
                if matches!(self.current_token(), Token::It | Token::These) {
                    self.advance();
                }
            }
            n
        } else {
            // Form B: create collections in it / then names on following lines
            if matches!(self.current_token(), Token::In) {
                self.advance();
            }
            if matches!(self.current_token(), Token::It | Token::These) {
                self.advance();
            }
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
            self.parse_identifier_list()?
        };

        Ok(Statement::CreateCollections {
            database: self.current_database.clone(),
            names,
        })
    }

    fn parse_show_statement(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Show)?;

        let message = self.parse_expression()?;

        Ok(Statement::Show { message })
    }

    fn parse_identifier_list(&mut self) -> Result<Vec<String>, String> {
        let mut names = Vec::new();

        loop {
            match self.current_token() {
                Token::Identifier(name) => {
                    names.push(name.clone());
                    self.advance();
                }
                Token::String(name) => {
                    names.push(name.clone());
                    self.advance();
                }
                _ => break,
            }

            // Skip comma if present
            if self.current_token() == &Token::Comma {
                self.advance();
            }

            // Skip newline if present
            if self.current_token() == &Token::Newline {
                self.advance();
            }
        }

        if names.is_empty() {
            return Err("Expected at least one collection name".to_string());
        }

        Ok(names)
    }

    fn consume(&mut self, expected: &Token) -> Result<(), String> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, got {:?}",
                expected,
                self.current_token()
            ))
        }
    }

    fn current_token(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else {
            &Token::EOF
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current_token() == &Token::EOF
    }

    fn peek_token(&self) -> Option<&Token> {
        if self.position + 1 < self.tokens.len() {
            Some(&self.tokens[self.position + 1])
        } else {
            None
        }
    }

    // Helper to extract a string from a token that could be an identifier or keyword
    fn token_as_identifier(&self, token: &Token) -> Option<String> {
        match token {
            Token::Identifier(s) => Some(s.clone()),
            // Single letters (articles, etc.)
            Token::A => Some("a".to_string()),
            Token::An => Some("an".to_string()),
            // Allow certain keywords as field names / method names / variable names
            Token::Title => Some("title".to_string()),
            Token::Type => Some("type".to_string()),
            Token::Size => Some("size".to_string()),
            Token::Text => Some("text".to_string()),
            Token::Value => Some("value".to_string()),
            Token::Color => Some("color".to_string()),
            Token::Style => Some("style".to_string()),
            Token::Content => Some("content".to_string()),
            Token::Body => Some("body".to_string()),
            Token::Image => Some("image".to_string()),
            Token::Link => Some("link".to_string()),
            Token::Item => Some("item".to_string()),
            Token::Label => Some("label".to_string()),
            // Method names
            Token::Uppercase => Some("uppercase".to_string()),
            Token::Lowercase => Some("lowercase".to_string()),
            Token::Trim => Some("trim".to_string()),
            Token::Split => Some("split".to_string()),
            Token::Replace => Some("replace".to_string()),
            Token::Push => Some("push".to_string()),
            Token::Pop => Some("pop".to_string()),
            Token::Length => Some("length".to_string()),
            Token::Contains => Some("contains".to_string()),
            Token::First => Some("first".to_string()),
            Token::Last => Some("last".to_string()),
            Token::Sort => Some("sort".to_string()),
            Token::Reverse => Some("reverse".to_string()),
            Token::First => Some("first".to_string()),
            Token::Last => Some("last".to_string()),
            Token::Join => Some("join".to_string()),
            Token::Substring => Some("substring".to_string()),
            Token::Concat => Some("concat".to_string()),
            Token::IndexOf => Some("indexof".to_string()),
            // More common words
            Token::Named => Some("name".to_string()),
            Token::Called => Some("called".to_string()),
            Token::Port => Some("port".to_string()),
            Token::Server => Some("server".to_string()),
            Token::Message => Some("message".to_string()),
            Token::Database => Some("database".to_string()),
            Token::User => Some("user".to_string()),
            Token::Email => Some("email".to_string()),
            Token::Password => Some("password".to_string()),
            Token::Status => Some("status".to_string()),
            Token::Order => Some("order".to_string()),
            Token::Limit => Some("limit".to_string()),
            Token::Query => Some("query".to_string()),
            Token::Search => Some("search".to_string()),
            Token::Log => Some("log".to_string()),
            Token::Token => Some("token".to_string()),
            Token::Header => Some("header".to_string()),
            Token::Session => Some("session".to_string()),
            Token::File => Some("file".to_string()),
            Token::Path => Some("path".to_string()),
            Token::Type => Some("type".to_string()),
            Token::Src => Some("src".to_string()),
            Token::Icon => Some("icon".to_string()),
            // CSS-relevant tokens for style property names
            Token::Font => Some("font".to_string()),
            Token::Align => Some("align".to_string()),
            Token::Display => Some("display".to_string()),
            Token::Padding => Some("padding".to_string()),
            Token::Margin => Some("margin".to_string()),
            Token::Border => Some("border".to_string()),
            Token::Background => Some("background".to_string()),
            Token::Width => Some("width".to_string()),
            Token::Height => Some("height".to_string()),
            Token::Left => Some("left".to_string()),
            Token::Right => Some("right".to_string()),
            Token::Flex => Some("flex".to_string()),
            Token::Grid => Some("grid".to_string()),
            Token::Position => Some("position".to_string()),
            Token::Overflow => Some("overflow".to_string()),
            Token::Top => Some("top".to_string()),
            Token::Bottom => Some("bottom".to_string()),
            Token::Center => Some("center".to_string()),
            Token::Hidden => Some("hidden".to_string()),
            Token::Visible => Some("visible".to_string()),
            Token::Row => Some("row".to_string()),
            Token::Start => Some("start".to_string()),
            Token::End => Some("end".to_string()),
            _ => None,
        }
    }

    fn parse_create_server(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Create)?;

        // Skip "a" or "web"
        if matches!(self.current_token(), Token::A | Token::Web) {
            self.advance();
        }

        // Consume "web" or "server"
        if matches!(self.current_token(), Token::Web | Token::Server) {
            self.advance();
        }

        // Skip "server" if we had "web"
        if matches!(self.current_token(), Token::Server) {
            self.advance();
        }

        // Consume "on"
        if matches!(self.current_token(), Token::On) {
            self.advance();
        }

        // Consume "port"
        if matches!(self.current_token(), Token::Port) {
            self.advance();
        }

        // Get port number
        let port = match self.current_token() {
            Token::Number(n) => *n as u16,
            _ => return Err("Expected port number".to_string()),
        };
        self.advance();

        Ok(Statement::CreateServer { port })
    }

    // Parses: add route get "/health" returning "ok" [returning] status 200
    fn parse_add_route(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Add)?;

        // Consume "route" or "endpoint"
        if matches!(self.current_token(), Token::Route | Token::Endpoint) {
            self.advance();
        }

        // Get HTTP method (get, post, put, delete)
        let method = match self.current_token() {
            Token::Get => "GET",
            Token::Post => "POST",
            Token::Put => "PUT",
            Token::Delete => "DELETE",
            Token::Identifier(s) if s.to_lowercase() == "post" => "POST",
            _ => return Err("Expected HTTP method (get, post, put, delete)".to_string()),
        };
        self.advance();

        // Get path
        let path = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected route path in quotes".to_string()),
        };
        self.advance();

        // Skip "returning" or "return"
        if matches!(self.current_token(), Token::Identifier(s) if s == "returning") {
            self.advance();
        } else if matches!(self.current_token(), Token::Return) {
            self.advance();
        }

        // Data route: returning rows from <collection>
        if matches!(self.current_token(), Token::Identifier(s) if s == "rows") {
            self.advance();

            // Skip "from"
            if !matches!(self.current_token(), Token::From) {
                return Err("Expected 'from' after 'rows'".to_string());
            }
            self.advance();

            let collection = match self.current_token() {
                Token::Identifier(name) => name.clone(),
                Token::String(name) => name.clone(),
                _ => return Err("Expected collection name after 'from'".to_string()),
            };
            self.advance();

            return Ok(Statement::AddDataRoute {
                method: method.to_string(),
                path,
                collection,
            });
        }

        // Optional leading status: "returning status <number>"
        let mut status_code = None;
        if matches!(self.current_token(), Token::Status) {
            self.advance();
            if let Token::Number(n) = self.current_token() {
                status_code = Some(*n as u16);
                self.advance();
            }
        }

        // Get response (or a sensible default when only a status was given)
        let response = if matches!(self.current_token(), Token::Newline | Token::EOF) {
            Expression::String(format!("{} {} endpoint", method, path))
        } else {
            self.parse_expression()?
        };

        // Optional trailing status: "... <response> [returning] status <number>"
        if status_code.is_none() {
            if matches!(self.current_token(), Token::Identifier(s) if s == "returning") {
                self.advance();
            }
            if matches!(self.current_token(), Token::Status) {
                self.advance();
                if let Token::Number(n) = self.current_token() {
                    status_code = Some(*n as u16);
                    self.advance();
                }
            }
        }

        Ok(Statement::AddRoute {
            method: method.to_string(),
            path,
            response,
            status_code,
        })
    }

    fn parse_start_server(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Start)?;

        // Skip "the"
        if matches!(self.current_token(), Token::The) {
            self.advance();
        }

        // Skip "server"
        if matches!(self.current_token(), Token::Server) {
            self.advance();
        }

        // Optional: "for <number> seconds" => auto-stop after N seconds
        let mut duration_seconds = None;
        if matches!(self.current_token(), Token::For) {
            self.advance();

            match self.current_token().clone() {
                Token::Number(n) => {
                    duration_seconds = Some(n);
                    self.advance();
                }
                _ => return Err("Expected a number after 'for'".to_string()),
            }

            // Skip optional "seconds"
            if matches!(self.current_token(), Token::Identifier(s) if s == "seconds") {
                self.advance();
            }
        }

        Ok(Statement::StartServer { duration_seconds })
    }

    // Parses: handle post "/api/users" with body as data [end]
    //        handle get "/api/health" returning status 200 [end]
    fn parse_add_handler(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Handle)?;

        let method = match self.current_token() {
            Token::Get => "GET",
            Token::Post => "POST",
            Token::Put => "PUT",
            Token::Delete => "DELETE",
            Token::Identifier(s) if s.to_lowercase() == "post" => "POST",
            _ => return Err("Expected HTTP method (get, post, put, delete)".to_string()),
        };
        self.advance();

        let path = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected route path in quotes".to_string()),
        };
        self.advance();

        // Parse the header line: optional "with body as <var>" and optional "returning status <code>"
        let mut body_var = String::new();
        let mut status_code = None;
        if matches!(self.current_token(), Token::With) {
            self.advance();
            if matches!(self.current_token(), Token::Body) {
                self.advance();
                if matches!(self.current_token(), Token::Identifier(s) if s == "as") {
                    self.advance();
                }
                if let Some(name) = self.token_as_identifier(self.current_token()) {
                    body_var = name.to_string();
                    self.advance();
                } else {
                    return Err("Expected variable name after 'body as'".to_string());
                }
            }
        }
        if matches!(self.current_token(), Token::Return) {
            self.advance();
            if matches!(self.current_token(), Token::Status) {
                self.advance();
                if let Token::Number(n) = self.current_token() {
                    status_code = Some(*n as u16);
                    self.advance();
                }
            }
        }

        // Skip newline before the handler body
        while matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        // Parse the handler body until "end"
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::End | Token::EOF) {
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            body.push(self.parse_statement()?);
        }

        // Skip "end"
        if matches!(self.current_token(), Token::End) {
            self.advance();
        }

        Ok(Statement::AddHandler {
            method: method.to_string(),
            path,
            body_var,
            body,
            status_code,
        })
    }

    // Parses: add middleware cors | add middleware logging
    fn parse_add_middleware(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Add)?;
        self.consume(&Token::Middleware)?;

        let middleware_type = match self.current_token() {
            Token::Cors => "cors",
            Token::Log | Token::Logging => "logging",
            Token::Identifier(s) if s.to_lowercase() == "cors" => "cors",
            Token::Identifier(s) if s.to_lowercase() == "logging" || s.to_lowercase() == "log" => "logging",
            _ => return Err("Expected 'cors' or 'logging' after 'add middleware'".to_string()),
        }.to_string();
        self.advance();

        Ok(Statement::AddMiddleware { middleware_type })
    }

    fn parse_assignment(&mut self) -> Result<Statement, String> {
        // Consume: set/let/make/store/save
        self.advance();

        // `set cookie "name" to "value"` is a dedicated statement.
        if matches!(&self.current_token(), Token::Identifier(name) if name == "cookie") {
            return self.parse_set_cookie();
        }

        // `set style "selector" ...` is a dedicated statement.
        if matches!(&self.current_token(), Token::Style)
            || matches!(&self.current_token(), Token::Identifier(name) if name == "style")
        {
            return self.parse_set_style();
        }

        // Get variable name (allow keywords as variable names)
        let variable = self.token_as_identifier(self.current_token())
            .ok_or_else(|| "Expected variable name".to_string())?;
        self.advance();

        // Consume: to/is/=
        if matches!(self.current_token(), Token::To | Token::Is | Token::Equals) {
            self.advance();
        }

        // Parse value expression
        let value = self.parse_expression()?;

        Ok(Statement::Assignment { variable, value })
    }

    fn parse_set_cookie(&mut self) -> Result<Statement, String> {
        // At start, current token is "cookie".
        self.advance();

        let name = match self.current_token() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected cookie name after 'set cookie'".to_string()),
        };
        self.advance();

        // Consume optional: to/is/=
        if matches!(self.current_token(), Token::To | Token::Is | Token::Equals) {
            self.advance();
        }

        let value = self.parse_expression()?;

        Ok(Statement::SetCookie { name, value })
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_arithmetic_expression()
    }

    fn parse_arithmetic_expression(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_term()?;

        // Handle + and -
        loop {
            let operator = match self.current_token() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_term()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_postfix_expression()?;

        // Handle *, /, times, divided by
        loop {
            let operator = match self.current_token() {
                Token::Star | Token::Times => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                _ => break,
            };

            self.advance();
            // "divided by N" — skip the optional "by"
            if matches!(self.current_token(), Token::By) {
                self.advance();
            }
            let right = self.parse_postfix_expression()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary_expression()?;

        // Check for method calls (dot syntax) or array indexing
        loop {
            match self.current_token() {
                Token::Dot => {
            self.advance(); // consume dot

            // Get property/method name (allow keywords)
            let name = self.token_as_identifier(self.current_token())
                .ok_or_else(|| "Expected property or method name after dot".to_string())?;
            self.advance();

            // Check if it's a method call (has parentheses) or property access
            if matches!(self.current_token(), Token::LeftParen) {
                // Method call with parentheses
                self.advance(); // consume (
                let mut args = Vec::new();

                while !matches!(self.current_token(), Token::RightParen | Token::EOF) {
                    args.push(self.parse_primary_expression()?);

                    if matches!(self.current_token(), Token::Comma) {
                        self.advance();
                    }
                }

                if matches!(self.current_token(), Token::RightParen) {
                    self.advance(); // consume )
                }

                expr = Expression::MethodCall {
                    object: Box::new(expr),
                    method: name,
                    arguments: args,
                };
            } else {
                // Check if it's a known method that works without parens
                let is_method = matches!(name.as_str(),
                    "push" | "pop" | "uppercase" | "lowercase" | "trim" |
                    "length" | "reverse" | "sort" | "first" | "last" | "join"
                );

                if is_method {
                    // Treat as method call with no arguments
                    expr = Expression::MethodCall {
                        object: Box::new(expr),
                        method: name,
                        arguments: Vec::new(),
                    };
                } else {
                    // Property access
                    expr = Expression::PropertyAccess {
                        object: Box::new(expr),
                        property: name,
                    };
                }
            }
                }
                Token::LeftBracket => {
                    self.advance(); // consume [

                    // Parse index expression
                    let index = self.parse_primary_expression()?;

                    // Consume ]
                    if matches!(self.current_token(), Token::RightBracket) {
                        self.advance();
                    } else {
                        return Err("Expected ] after array index".to_string());
                    }

                    expr = Expression::IndexAccess {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        // Handle unary minus for negative numbers
        if matches!(self.current_token(), Token::Minus) {
            self.advance(); // consume -
            let operand = self.parse_primary_expression()?;
            return Ok(Expression::UnaryOp {
                operator: UnaryOperator::Negative,
                operand: Box::new(operand),
            });
        }

        match self.current_token() {
            Token::Number(n) => {
                let val = *n;
                self.advance();
                Ok(Expression::Number(val))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expression::String(val))
            }
            Token::True => {
                self.advance();
                Ok(Expression::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Boolean(false))
            }
            Token::Null => {
                self.advance();
                Ok(Expression::Null)
            }
            Token::Identifier(name) => {
                let val = name.clone();
                self.advance();
                Ok(Expression::Identifier(val))
            }
            Token::LeftBracket => self.parse_array(),
            Token::LeftBrace => self.parse_object(),
            _ => {
                // Try to treat as identifier (for keywords used as variable names)
                if let Some(name) = self.token_as_identifier(self.current_token()) {
                    self.advance();
                    Ok(Expression::Identifier(name))
                } else {
                    Err(format!("Expected expression, got {:?}", self.current_token()))
                }
            }
        }
    }

    fn parse_array(&mut self) -> Result<Expression, String> {
        self.advance(); // consume '['

        let mut elements = Vec::new();

        while !matches!(self.current_token(), Token::RightBracket | Token::EOF) {
            elements.push(self.parse_expression()?);

            // Skip comma if present
            if matches!(self.current_token(), Token::Comma) {
                self.advance();
            }
        }

        if matches!(self.current_token(), Token::RightBracket) {
            self.advance(); // consume ']'
        }

        Ok(Expression::Array(elements))
    }

    fn parse_object(&mut self) -> Result<Expression, String> {
        self.advance(); // consume '{'

        let mut pairs = Vec::new();

        while !matches!(self.current_token(), Token::RightBrace | Token::EOF) {
            // Get key (identifier or string)
            let key = match self.current_token() {
                Token::Identifier(s) => s.clone(),
                Token::String(s) => s.clone(),
                _ => {
                    if let Some(k) = self.token_as_identifier(self.current_token()) {
                        k
                    } else {
                        return Err("Expected object key".to_string());
                    }
                }
            };
            self.advance();

            // Expect colon
            if !matches!(self.current_token(), Token::Colon) {
                return Err("Expected : after object key".to_string());
            }
            self.advance();

            // Parse value
            let value = self.parse_primary_expression()?;

            pairs.push((key, value));

            // Skip comma if present
            if matches!(self.current_token(), Token::Comma) {
                self.advance();
            }
        }

        if matches!(self.current_token(), Token::RightBrace) {
            self.advance(); // consume '}'
        }

        Ok(Expression::Object(pairs))
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        // insert/add/put
        self.advance();

        // Skip optional: a/an/the
        if matches!(self.current_token(), Token::A | Token::An | Token::The) {
            self.advance();
        }

        // Raw value insertion: insert <expr> into <collection>
        if !matches!(self.current_token(), Token::Into | Token::In) {
            let saved = self.position;
            if let Ok(value) = self.parse_expression() {
                if matches!(self.current_token(), Token::Into | Token::In) {
                    self.advance();
                    let collection = match self.current_token() {
                        Token::Identifier(name) => name.clone(),
                        Token::String(name) => name.clone(),
                        _ => return Err("Expected collection name after 'into'".to_string()),
                    };
                    self.advance();
                    return Ok(Statement::InsertRaw { collection, value });
                }
            }
            // Not a raw insert; backtrack and treat as standard form
            self.position = saved;
        }

        // Parse data as key-value pairs
        let mut data = Vec::new();

        // Simple format: insert into users with name "John" and age 30
        if matches!(self.current_token(), Token::Into | Token::In) {
            self.advance();
        }

        // Get collection name
        let collection = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected collection name".to_string()),
        };
        self.advance();

        // Skip "with"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }

        // Parse key-value pairs: name "John" and age 30
        while !matches!(self.current_token(), Token::Newline | Token::EOF) {
            // Get key (could be identifier or keyword used as field name)
            let key = match self.token_as_identifier(self.current_token()) {
                Some(k) => k,
                None => break,
            };
            self.advance();

            // Skip optional "is"/"="
            if matches!(self.current_token(), Token::Is | Token::Equals) {
                self.advance();
            }

            // Get value
            let value = self.parse_expression()?;
            data.push((key, value));

            // Skip "and" or comma
            if matches!(self.current_token(), Token::And | Token::Comma) {
                self.advance();
            }
        }

        Ok(Statement::Insert { collection, data })
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        // Detect `fetch data from "url"` / `fetch from "url"` (URL fetch) vs a
        // database select. A URL-in-string is a strong signal; a collection name
        // is an identifier, never a string.
        if matches!(self.current_token(), Token::Fetch) {
            let mut idx = self.position + 1;
            if self.tokens.get(idx).map_or(false, |t| matches!(t, Token::Identifier(name) if name == "data")) {
                idx += 1;
            }
            if matches!(self.tokens.get(idx), Some(Token::From)) {
                if let Some(Token::String(s)) = self.tokens.get(idx + 1) {
                    if s.starts_with("http") || s.starts_with('/') || s.contains("://") {
                        return self.parse_fetch_data();
                    }
                }
            }
        }

        // select/get/find/fetch
        self.advance();

        // Skip "all" if present
        if matches!(self.current_token(), Token::All) {
            self.advance();
        }

        // Skip "from"
        if matches!(self.current_token(), Token::From) {
            self.advance();
        }

        // Get collection name
        let collection = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected collection name".to_string()),
        };
        self.advance();

        // Parse WHERE clause if present
        let condition = if matches!(self.current_token(), Token::Where) {
            self.advance(); // consume 'where'
            Some(self.parse_comparison_expression()?)
        } else {
            None
        };

        // Parse optional JOIN clause: join <collection> on <bool expression>
        let join = if matches!(self.current_token(), Token::Join) {
            self.advance(); // consume 'join'
            let collection = match self.current_token() {
                Token::Identifier(name) => name.clone(),
                _ => return Err("Expected joined collection name after 'join'".to_string()),
            };
            self.advance();
            if !matches!(self.current_token(), Token::On) {
                return Err("Expected 'on' after join collection name".to_string());
            }
            self.advance();
            let join_condition = Some(self.parse_comparison_expression()?);
            Some(JoinClause {
                collection,
                condition: join_condition,
            })
        } else {
            None
        };

        Ok(Statement::Select {
            collection,
            fields: vec![],
            condition,
            join,
        })
    }

    fn parse_update(&mut self) -> Result<Statement, String> {
        // update/change/modify
        self.advance();

        // Get collection name
        let collection = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected collection name".to_string()),
        };
        self.advance();

        // Skip "with" or "set"
        if matches!(self.current_token(), Token::With | Token::Set) {
            self.advance();
        }

        // Parse key-value pairs
        let mut data = Vec::new();
        while !matches!(self.current_token(), Token::Newline | Token::EOF | Token::Where) {
            let key = match self.token_as_identifier(self.current_token()) {
                Some(k) => k,
                None => break,
            };
            self.advance();

            if matches!(self.current_token(), Token::Is | Token::Equals | Token::To) {
                self.advance();
            }

            let value = self.parse_expression()?;
            data.push((key, value));

            if matches!(self.current_token(), Token::And | Token::Comma) {
                self.advance();
            }
        }

        // Parse WHERE clause if present
        let condition = if matches!(self.current_token(), Token::Where) {
            self.advance(); // consume 'where'
            Some(self.parse_comparison_expression()?)
        } else {
            None
        };

        Ok(Statement::Update {
            collection,
            data,
            condition,
        })
    }

    fn parse_delete(&mut self) -> Result<Statement, String> {
        // delete/remove
        self.advance();

        // Skip "all" if present
        if matches!(self.current_token(), Token::All) {
            self.advance();
        }

        // Skip "from"
        if matches!(self.current_token(), Token::From) {
            self.advance();
        }

        // Get collection name
        let collection = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected collection name".to_string()),
        };
        self.advance();

        // Parse WHERE clause if present
        let condition = if matches!(self.current_token(), Token::Where) {
            self.advance(); // consume 'where'
            Some(self.parse_comparison_expression()?)
        } else {
            None
        };

        Ok(Statement::Delete {
            collection,
            condition,
        })
    }

    // HTML/Page parsing methods
    fn parse_create_page(&mut self) -> Result<Statement, String> {
        // Already consumed "create" and optional "a", now at "page"
        self.advance(); // consume "page"

        // Consume "called" or "named"
        if matches!(self.current_token(), Token::Called | Token::Named) {
            self.advance();
        }

        // Get page name
        let name = match self.current_token() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected page name".to_string()),
        };
        self.advance();

        // Optional clauses: with title "..." and/or with layout "..."
        let mut title = None;
        let mut layout = None;
        let mut done = false;
        while !done {
            match self.current_token() {
                Token::With | Token::And => {
                    self.advance();
                    match self.current_token() {
                        Token::Title => {
                            self.advance();
                            if let Token::String(s) = self.current_token() {
                                title = Some(s.clone());
                                self.advance();
                            }
                        }
                        Token::Identifier(name) if name == "layout" => {
                            self.advance();
                            if let Token::String(s) = self.current_token() {
                                layout = Some(s.clone());
                                self.advance();
                            }
                        }
                        _ => done = true,
                    }
                }
                _ => done = true,
            }
        }

        Ok(Statement::CreatePage { name, title, layout })
    }

    fn parse_create_layout(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "layout" or "template"

        if matches!(self.current_token(), Token::Called | Token::Named) {
            self.advance();
        }

        let name = match self.current_token() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected layout name".to_string()),
        };
        self.advance();

        Ok(Statement::CreateLayout { name })
    }

    fn parse_add_button(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "button"

        // Get button text: labeled "Click Me" or "Click Me"
        let text = if matches!(self.current_token(), Token::Labeled) {
            self.advance();
            match self.current_token() {
                Token::String(s) => {
                    let t = s.clone();
                    self.advance();
                    t
                }
                _ => return Err("Expected button text in quotes".to_string()),
            }
        } else {
            match self.current_token() {
                Token::String(s) => {
                    let t = s.clone();
                    self.advance();
                    t
                }
                _ => return Err("Expected button text".to_string()),
            }
        };

        // Parse optional properties: "with id X", "with class X"
        let mut properties: Vec<(String, Expression)> = Vec::new();
        loop {
            if matches!(self.current_token(), Token::And) || matches!(self.current_token(), Token::With) {
                self.advance();
            } else {
                break;
            }
            let key = match self.current_token() {
                Token::Identifier(name) if name == "id" => "id".to_string(),
                Token::Identifier(name) if name == "class" => "class".to_string(),
                _ => break,
            };
            self.advance();
            if let Token::String(s) = self.current_token() {
                properties.push((key, Expression::String(s.clone())));
                self.advance();
            } else {
                break;
            }
        }

        Ok(Statement::AddButton { text, properties })
    }

    fn parse_add_input(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "input"

        // Get input type: "of type X", "with type X", or just default to "text"
        let mut input_type = "text".to_string();
        if matches!(self.current_token(), Token::Of) || matches!(self.current_token(), Token::With) {
            self.advance();
            if matches!(self.current_token(), Token::Type) {
                self.advance();
                if let Token::String(s) = self.current_token() {
                    input_type = s.clone();
                    self.advance();
                }
            }
        }

        let mut properties: Vec<(String, Expression)> = Vec::new();

        // Optional additional properties: "and placeholder X", "and name X",
        // "and required", "and value X"
        loop {
            if matches!(self.current_token(), Token::And) || matches!(self.current_token(), Token::With) {
                self.advance();
            } else {
                break;
            }
            let key = match self.current_token() {
                Token::Placeholder => "placeholder".to_string(),
                Token::Named => "name".to_string(),
                Token::Identifier(name) if name == "name" => "name".to_string(),
                Token::Required => "required".to_string(),
                Token::Value => "value".to_string(),
                _ => break,
            };
            self.advance();
            if key == "required" {
                properties.push((key, Expression::Boolean(true)));
                continue;
            }
            if let Token::String(s) = self.current_token() {
                properties.push((key, Expression::String(s.clone())));
                self.advance();
            } else {
                break;
            }
        }

        Ok(Statement::AddInput {
            input_type,
            properties,
        })
    }

    fn parse_add_form(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "form"

        // Skip optional: a/an/the
        if matches!(self.current_token(), Token::A | Token::An | Token::The) {
            self.advance();
        }

        let mut action = String::new();
        let mut method = String::from("post");

        // "with action '/api/contact' and method 'get'"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }

        if matches!(self.current_token(), Token::Identifier(n) if n == "action") {
            self.advance();
            if let Token::String(s) = self.current_token() {
                action = s.clone();
                self.advance();
            }
        }

        if matches!(self.current_token(), Token::And) || matches!(self.current_token(), Token::With) {
            self.advance();
        }
        if matches!(self.current_token(), Token::Identifier(n) if n == "method") {
            self.advance();
            if let Token::String(s) = self.current_token() {
                method = s.clone().to_lowercase();
                self.advance();
            }
        }

        Ok(Statement::AddForm {
            properties: vec![
                ("action".to_string(), Expression::String(action)),
                ("method".to_string(), Expression::String(method)),
            ],
        })
    }

    fn parse_add_element(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "element"

        // Skip optional: a/an/the
        if matches!(self.current_token(), Token::A | Token::An | Token::The) {
            self.advance();
        }

        let element_type = match self.current_token() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            Token::Div => "div".to_string(),
            Token::Section => "section".to_string(),
            Token::Header => "header".to_string(),
            Token::Footer => "footer".to_string(),
            Token::Span => "span".to_string(),
            _ => return Err("Expected element type".to_string()),
        };
        self.advance();

        let mut properties: Vec<(String, Expression)> = Vec::new();

        // "with text '...'" / "with class '...'" / "with id '...'"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }
        loop {
            let key = match self.current_token() {
                Token::Text => "text".to_string(),
                Token::Identifier(name) if name == "class" => "class".to_string(),
                Token::Identifier(name) if name == "id" => "id".to_string(),
                Token::Style => "style".to_string(),
                _ => break,
            };
            self.advance();
            if let Token::String(s) = self.current_token() {
                properties.push((key, Expression::String(s.clone())));
                self.advance();
            } else {
                break;
            }
            if matches!(self.current_token(), Token::And) {
                self.advance();
            }
        }

        Ok(Statement::AddElement {
            element_type,
            properties,
        })
    }

    fn parse_set_style(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "set"

        // Skip optional: the/style
        if matches!(self.current_token(), Token::The)
            || matches!(self.current_token(), Token::Style) {
            self.advance();
        }

        let selector = match self.current_token() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected style selector".to_string()),
        };
        self.advance();

        // Optional connector: to/with/=
        if matches!(self.current_token(), Token::To | Token::With | Token::Equals) {
            self.advance();
        }

        let mut styles: Vec<(String, String)> = Vec::new();

        // "color 'blue' and font-size '16px'" / "color is 'blue'"
        loop {
            // Build property name, joining identifier + minus + identifier runs.
            let mut prop = String::new();
            loop {
                if let Some(seg) = self.token_as_identifier(self.current_token()) {
                    if !prop.is_empty() {
                        prop.push('-');
                    }
                    prop.push_str(&seg);
                    self.advance();
                } else if prop.is_empty() {
                    break;
                } else {
                    break;
                }
                if matches!(self.current_token(), Token::Minus) {
                    self.advance();
                    if let Some(seg) = self.token_as_identifier(self.current_token()) {
                        prop.push('-');
                        prop.push_str(&seg);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if prop.is_empty() {
                break;
            }

            // Skip optional connectors: is/=/to/with
            if matches!(
                self.current_token(),
                Token::Is | Token::Equals | Token::To | Token::With
            ) {
                self.advance();
            }

            // Read value: string or number
            let value = match self.current_token() {
                Token::String(s) => {
                    let v = s.clone();
                    self.advance();
                    v
                }
                Token::Number(n) => {
                    let v = if n.fract() == 0.0 {
                        format!("{}", *n as i64)
                    } else {
                        format!("{}", n)
                    };
                    self.advance();
                    v
                }
                _ => break,
            };

            styles.push((prop, value));

            if matches!(self.current_token(), Token::And) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(Statement::SetStyle { selector, styles })
    }

    fn parse_add_link(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "link"

        // Skip optional: labeled
        if matches!(self.current_token(), Token::Labeled) {
            self.advance();
        }

        let text = match self.current_token() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected link text".to_string()),
        };
        self.advance();

        // Optional: "from 'url'" / "to 'url'" / "with url '...'"
        let mut url = String::new();
        if matches!(self.current_token(), Token::From | Token::To) {
            self.advance();
            if let Token::String(s) = self.current_token() {
                url = s.clone();
                self.advance();
            }
        } else if matches!(self.current_token(), Token::With) {
            self.advance();
            if matches!(self.current_token(), Token::Url) {
                self.advance();
                if let Token::String(s) = self.current_token() {
                    url = s.clone();
                    self.advance();
                }
            }
        }

        Ok(Statement::AddLink { text, url })
    }

    fn parse_add_heading(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "heading"

        // Get heading level (default h1)
        let mut level = 1;
        if matches!(self.current_token(), Token::Number(n) if *n >= 1.0 && *n <= 6.0) {
            if let Token::Number(n) = self.current_token() {
                level = *n as u8;
            }
            self.advance();
        } else if matches!(self.current_token(), Token::Identifier(s) if s == "level") {
            self.advance();
            if let Token::Number(n) = self.current_token() {
                level = *n as u8;
                self.advance();
            }
        }

        // Skip "with text"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }
        if matches!(self.current_token(), Token::Text) {
            self.advance();
        }

        // Get text
        let text = match self.current_token() {
            Token::String(s) => {
                let t = s.clone();
                self.advance();
                t
            }
            _ => return Err("Expected heading text".to_string()),
        };

        Ok(Statement::AddHeading { level, text })
    }

    fn parse_add_paragraph(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "paragraph"

        // Skip "with text" / "with"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }
        if matches!(self.current_token(), Token::Text) {
            self.advance();
        }

        let text = match self.current_token() {
            Token::String(s) => {
                let t = s.clone();
                self.advance();
                t
            }
            _ => return Err("Expected paragraph text".to_string()),
        };

        Ok(Statement::AddParagraph { text })
    }

    fn parse_add_image(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "image" or "icon"

        let mut src = String::new();
        let mut alt = String::new();

        // Parse: "from path/to/image.png" or "from path/to/image.png with alt text"
        if matches!(self.current_token(), Token::From | Token::Src | Token::Source) {
            self.advance();
        }

        // Get image source/path
        src = match self.current_token() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected image source path".to_string()),
        };
        self.advance();

        // Parse optional "with alt" or "alt"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }

        if matches!(self.current_token(), Token::Alt) {
            self.advance();
            alt = match self.current_token() {
                Token::String(s) => s.clone(),
                _ => return Err("Expected alt text in quotes".to_string()),
            };
            self.advance();
        }

        Ok(Statement::AddImage { src, alt })
    }

    fn parse_add_css(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "css" or "style"

        // Skip optional "framework"
        if matches!(self.current_token(), Token::Identifier(name) if name == "framework")
            || matches!(self.current_token(), Token::Component)
        {
            self.advance();
        }

        let framework = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            Token::String(s) => s.clone(),
            _ => return Err("Expected CSS framework name (tailwind, bootstrap)".to_string()),
        };
        self.advance();

        Ok(Statement::AddCss { framework })
    }

    fn parse_add_upload_route(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "upload"

        // Skip optional "route" / "endpoint"
        if matches!(self.current_token(), Token::Route | Token::Endpoint) {
            self.advance();
        }

        // Get path in quotes
        let path = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected upload route path in quotes".to_string()),
        };
        self.advance();

        // Optional: to "uploads/" directory
        let mut directory = "uploads".to_string();
        if matches!(self.current_token(), Token::To | Token::Into | Token::In) {
            self.advance();
            if let Token::String(s) = self.current_token() {
                directory = s.clone();
                self.advance();
            } else if let Token::Identifier(s) = self.current_token() {
                directory = s.clone();
                self.advance();
            }
        }

        Ok(Statement::AddUploadRoute { path, directory })
    }

    fn parse_add_websocket_route(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        self.advance(); // consume "websocket"

        // Skip optional "route" / "endpoint"
        if matches!(self.current_token(), Token::Route | Token::Endpoint) {
            self.advance();
        }

        // Get path in quotes
        let path = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected websocket route path in quotes".to_string()),
        };
        self.advance();

        Ok(Statement::AddWebSocketRoute { path })
    }

    // Parses: add rate limit [with] N requests per minute|second|hour
    // e.g. "add rate limit 20 requests per minute"
    fn parse_add_rate_limit(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"
        // consume "rate" (Identifier) and "limit" (Token::Limit)
        if !matches!(self.current_token(), Token::Identifier(name) if name == "rate") {
            return Err("Expected 'rate' after 'add'".to_string());
        }
        self.advance();
        if !matches!(self.current_token(), Token::Limit) {
            return Err("Expected 'limit' after 'rate'".to_string());
        }
        self.advance();

        // Optional "with"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }

        let limit = match self.current_token() {
            Token::Number(n) => {
                let n = *n as u64;
                self.advance();
                n
            }
            _ => 100,
        };

        // Consume trailing words like "requests per minute|second|hour" and
        // pick the window unit from them.
        let mut window_secs = 60u64;
        while matches!(self.current_token(), Token::Identifier(_)) {
            if let Token::Identifier(word) = self.current_token() {
                match word.as_str() {
                    "second" => window_secs = 1,
                    "hour" => window_secs = 3600,
                    "minute" => window_secs = 60,
                    _ => {}
                }
            }
            self.advance();
        }

        Ok(Statement::AddRateLimit { limit, window_secs })
    }

    fn parse_add_ui_component(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "add"

        let component = match self.current_token() {
            Token::Toast => "toast".to_string(),
            Token::Alert => "alert".to_string(),
            Token::Spinner | Token::Loading => "spinner".to_string(),
            Token::Modal => "modal".to_string(),
            Token::Tabs => "tabs".to_string(),
            Token::Accordion => "accordion".to_string(),
            Token::Container => "container".to_string(),
            Token::Grid => "grid".to_string(),
            _ => return Err("Expected UI component (toast, alert, spinner, modal, tabs, accordion, container, grid)".to_string()),
        };
        self.advance();

        // For spinner, no text needed
        let mut text = String::new();
        let mut title = None;
        let mut items: Vec<(String, String)> = Vec::new();

        if component == "tabs" || component == "accordion" {
            // add tabs with "Label One: Content one" and "Label Two: Content two"
            if matches!(self.current_token(), Token::With) {
                self.advance();
            }
            if matches!(self.current_token(), Token::Items) {
                self.advance();
            }
            while let Token::String(s) = self.current_token() {
                let pair = s.clone();
                self.advance();
                match pair.split_once(':') {
                    Some((label, content)) => {
                        items.push((label.trim().to_string(), content.trim().to_string()));
                    }
                    None => {
                        // Treat as a bare label with empty content (tab label only).
                        items.push((pair.trim().to_string(), String::new()));
                    }
                }
                if matches!(self.current_token(), Token::And) {
                    self.advance();
                } else {
                    break;
                }
            }
        } else if component == "spinner" || component == "modal" {
            // modal: add modal with title "..." and content "..."
            //       or add modal "message"
            if matches!(self.current_token(), Token::With) {
                self.advance();
                if matches!(self.current_token(), Token::Title) {
                    self.advance();
                    if let Token::String(s) = self.current_token() {
                        title = Some(s.clone());
                        self.advance();
                    }
                }
                if matches!(self.current_token(), Token::And) {
                    self.advance();
                }
                if matches!(self.current_token(), Token::Content | Token::Text) {
                    self.advance();
                    if let Token::String(s) = self.current_token() {
                        text = s.clone();
                        self.advance();
                    }
                }
            } else if let Token::String(s) = self.current_token() {
                text = s.clone();
                self.advance();
            }
        } else {
            // toast / alert: add toast "message"
            if matches!(self.current_token(), Token::Labeled) {
                self.advance();
            }
            if matches!(self.current_token(), Token::With) {
                self.advance();
                if matches!(self.current_token(), Token::Text) {
                    self.advance();
                }
            }
            if let Token::String(s) = self.current_token() {
                text = s.clone();
                self.advance();
            }
        }

        Ok(Statement::AddUIComponent { component, text, title, items })
    }

    fn parse_render_page(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "render"

        // render layout "base"
        if matches!(self.current_token(), Token::Identifier(name) if name == "layout") {
            self.advance();
            let name = match self.current_token() {
                Token::String(s) => s.clone(),
                Token::Identifier(s) => s.clone(),
                _ => return Err("Expected layout name".to_string()),
            };
            self.advance();
            return Ok(Statement::RenderLayout { name });
        }

        // Skip "page"
        if matches!(self.current_token(), Token::Page) {
            self.advance();
        }

        // Get page name
        let page_name = match self.current_token() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected page name".to_string()),
        };
        self.advance();

        Ok(Statement::RenderPage { page_name })
    }

    // Control flow parsing
    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "if" or "when"

        // Parse condition
        let condition = self.parse_comparison_expression()?;

        // Skip optional "then"
        if matches!(self.current_token(), Token::Then) {
            self.advance();
        }

        // Skip newline after condition
        if matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        // Parse then block (statements until "else" or "otherwise" or "end")
        let mut then_block = Vec::new();
        while !matches!(
            self.current_token(),
            Token::Else | Token::Otherwise | Token::End | Token::EOF
        ) {
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            then_block.push(self.parse_statement()?);
        }

        // Parse optional else block (including else-if)
        let else_block = if matches!(self.current_token(), Token::Else | Token::Otherwise) {
            self.advance(); // consume "else" or "otherwise"

            if matches!(self.current_token(), Token::Newline) {
                self.advance();
            }

            // Check for else-if pattern
            if matches!(self.current_token(), Token::If) {
                // Parse the if as a statement and wrap it in else block
                let if_stmt = self.parse_if_statement()?;
                Some(vec![if_stmt])
            } else {
                // Regular else block
                let mut else_stmts = Vec::new();
                while !matches!(self.current_token(), Token::End | Token::EOF) {
                    if matches!(self.current_token(), Token::Newline) {
                        self.advance();
                        continue;
                    }
                    else_stmts.push(self.parse_statement()?);
                }
                Some(else_stmts)
            }
        } else {
            None
        };

        // Skip "end" if present
        if matches!(self.current_token(), Token::End) {
            self.advance();
        }

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while_loop(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "while"

        let condition = self.parse_comparison_expression()?;

        // Skip optional "do"
        if matches!(self.current_token(), Token::Do) {
            self.advance();
        }

        if matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        // Parse loop body
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::End | Token::EOF) {
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            body.push(self.parse_statement()?);
        }

        // Skip "end"
        if matches!(self.current_token(), Token::End) {
            self.advance();
        }

        Ok(Statement::While { condition, body })
    }

    fn parse_for_loop(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "for"

        // Check for "for each" pattern
        if matches!(self.current_token(), Token::Each) {
            return self.parse_for_each_loop();
        }

        // Parse "for i from 1 to 10"
        let variable = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected variable name after 'for'".to_string()),
        };
        self.advance();

        // Skip "from"
        if matches!(self.current_token(), Token::From) {
            self.advance();
        }

        let start = self.parse_expression()?;

        // Consume "to"
        if !matches!(self.current_token(), Token::To) {
            return Err("Expected 'to' in for loop".to_string());
        }
        self.advance();

        let end = self.parse_expression()?;

        // Skip optional "do"
        if matches!(self.current_token(), Token::Do) {
            self.advance();
        }

        if matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        // Parse loop body
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::End | Token::EOF) {
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            body.push(self.parse_statement()?);
        }

        // Skip "end"
        if matches!(self.current_token(), Token::End) {
            self.advance();
        }

        Ok(Statement::For {
            variable,
            start,
            end,
            body,
        })
    }

    fn parse_for_each_loop(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "each"

        let variable = self.token_as_identifier(self.current_token())
            .ok_or_else(|| "Expected variable name after 'each'".to_string())?;
        self.advance();

        // Consume "in"
        if !matches!(self.current_token(), Token::In) {
            return Err("Expected 'in' in for each loop".to_string());
        }
        self.advance();

        let collection = self.parse_expression()?;

        // Skip optional "do"
        if matches!(self.current_token(), Token::Do) {
            self.advance();
        }

        if matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        // Parse loop body
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::End | Token::EOF) {
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            body.push(self.parse_statement()?);
        }

        // Skip "end"
        if matches!(self.current_token(), Token::End) {
            self.advance();
        }

        Ok(Statement::ForEach {
            variable,
            collection,
            body,
        })
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        use crate::ast::BinaryOperator;

        let left = self.parse_expression()?;

        // Check for comparison operator
        let operator = match self.current_token() {
            Token::EqualTo => {
                self.advance();
                BinaryOperator::EqualTo
            }
            Token::Is | Token::Equals => {
                self.advance();
                BinaryOperator::EqualTo
            }
            Token::NotEqualTo => {
                self.advance();
                BinaryOperator::NotEqualTo
            }
            Token::GreaterThan => {
                self.advance();
                BinaryOperator::GreaterThan
            }
            Token::LessThan => {
                self.advance();
                BinaryOperator::LessThan
            }
            Token::GreaterThanOrEqual => {
                self.advance();
                BinaryOperator::GreaterThanOrEqual
            }
            Token::LessThanOrEqual => {
                self.advance();
                BinaryOperator::LessThanOrEqual
            }
            Token::And => {
                self.advance();
                BinaryOperator::And
            }
            Token::Or => {
                self.advance();
                BinaryOperator::Or
            }
            _ => return Ok(left), // No comparison operator, return left as-is
        };

        let right = self.parse_expression()?;

        Ok(Expression::BinaryOp {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    // Function parsing
    fn parse_function_def(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "define"

        // Skip optional "function" or "procedure"
        if matches!(self.current_token(), Token::Function | Token::Procedure) {
            self.advance();
        }

        // Get function name (could be identifier or keyword used as function name)
        let name = match self.token_as_identifier(self.current_token()) {
            Some(n) => n,
            None => match self.current_token() {
                Token::Identifier(n) => n.clone(),
                _ => return Err("Expected function name".to_string()),
            }
        };
        self.advance();

        // Skip optional "that"
        if matches!(self.current_token(), Token::That) {
            self.advance();
        }

        // Parse parameters: "takes x and y" or "takes x, y, z"
        let mut parameters = Vec::new();
        if matches!(self.current_token(), Token::Takes) {
            self.advance();

            while matches!(self.current_token(), Token::Identifier(_)) {
                if let Token::Identifier(param) = self.current_token() {
                    parameters.push(param.clone());
                    self.advance();

                    // Skip "and" or comma
                    if matches!(self.current_token(), Token::And | Token::Comma) {
                        self.advance();
                    }
                }
            }
        }

        // Skip newline
        if matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        // Parse function body
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::End | Token::EOF) {
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            body.push(self.parse_statement()?);
        }

        // Skip "end"
        if matches!(self.current_token(), Token::End) {
            self.advance();
        }

        Ok(Statement::FunctionDef {
            name,
            parameters,
            body,
        })
    }

    fn parse_function_call_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "call", "invoke", or "run"

        // Get function name (could be identifier or keyword used as function name)
        let name = match self.token_as_identifier(self.current_token()) {
            Some(n) => n,
            None => match self.current_token() {
                Token::Identifier(n) => n.clone(),
                _ => return Err("Expected function name".to_string()),
            }
        };
        self.advance();

        // Parse arguments: "with x and y" or "with x, y, z" or just "x, y, z"
        let mut arguments = Vec::new();

        // Skip optional "with"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }

        // Parse arguments until newline or EOF
        while !matches!(self.current_token(), Token::Newline | Token::EOF) {
            arguments.push(self.parse_expression()?);

            // Skip "and" or comma
            if matches!(self.current_token(), Token::And | Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(Statement::FunctionCall { name, arguments })
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "return"

        // Check if there's a return value
        let value = if matches!(self.current_token(), Token::Newline | Token::EOF) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Statement::Return { value })
    }

    fn parse_try_catch(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Try)?;

        // Skip newline after try
        while matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        // Parse try block
        let mut try_block = Vec::new();
        while !matches!(self.current_token(), Token::Catch | Token::Finally | Token::End | Token::EOF) {
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            try_block.push(self.parse_statement()?);
        }

        // Parse catch block
        let mut catch_block = Vec::new();
        if matches!(self.current_token(), Token::Catch) {
            self.advance(); // consume 'catch'

            // Skip newline after catch
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }

            while !matches!(self.current_token(), Token::Finally | Token::End | Token::EOF) {
                if matches!(self.current_token(), Token::Newline) {
                    self.advance();
                    continue;
                }
                catch_block.push(self.parse_statement()?);
            }
        }

        // Parse optional finally block
        let mut finally_block = None;
        if matches!(self.current_token(), Token::Finally) {
            self.advance(); // consume 'finally'

            // Skip newline after finally
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }

            let mut finally_stmts = Vec::new();
            while !matches!(self.current_token(), Token::End | Token::EOF) {
                if matches!(self.current_token(), Token::Newline) {
                    self.advance();
                    continue;
                }
                finally_stmts.push(self.parse_statement()?);
            }
            finally_block = Some(finally_stmts);
        }

        // Consume 'end'
        if matches!(self.current_token(), Token::End) {
            self.advance();
        }

        Ok(Statement::TryCatch {
            try_block,
            catch_block,
            finally_block,
        })
    }

    fn parse_throw(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'throw' or 'raise'

        let message = self.parse_expression()?;

        Ok(Statement::Throw { message })
    }

    fn parse_navigate(&mut self) -> Result<Statement, String> {
        // navigate to page "name"
        // navigate to "name"
        self.advance(); // consume "navigate"

        // Skip optional: to
        if matches!(self.current_token(), Token::To) {
            self.advance();
        }
        // Skip optional: page/screen
        if matches!(self.current_token(), Token::Page)
            || matches!(&self.current_token(), Token::Identifier(name) if name == "screen") {
            self.advance();
        }

        let page = self.parse_expression()?;

        Ok(Statement::NavigateTo { page })
    }

    fn parse_go_back(&mut self) -> Result<Statement, String> {
        // go back
        self.advance(); // consume "go"
        if matches!(self.current_token(), Token::Identifier(name) if name == "back") {
            self.advance();
        }
        Ok(Statement::GoBack)
    }

    fn parse_fetch_data(&mut self) -> Result<Statement, String> {
        // fetch data from "url"
        // fetch from "url"
        self.advance(); // consume "fetch"

        // Skip optional: data/content
        if matches!(&self.current_token(), Token::Identifier(name) if name == "data")
            || matches!(self.current_token(), Token::Content) {
            self.advance();
        }

        // Expect from
        if !matches!(self.current_token(), Token::From) {
            return Err("Expected 'from' in fetch statement".to_string());
        }
        self.advance();

        let url = self.parse_expression()?;

        // Optional: store/save into variable  =>  fetch data from "u" into items
        let mut variable = String::new();
        if matches!(self.current_token(), Token::Into | Token::To | Token::In) {
            self.advance();
            if let Some(name) = self.token_as_identifier(self.current_token()) {
                variable = name;
                self.advance();
            }
        }
        // Optional: fetch data from "u" as items
        if variable.is_empty() && matches!(&self.current_token(), Token::Identifier(name) if name == "as") {
            self.advance();
            if let Some(name) = self.token_as_identifier(self.current_token()) {
                variable = name;
                self.advance();
            }
        }

        Ok(Statement::FetchData { url, variable })
    }

    fn parse_validate(&mut self) -> Result<Statement, String> {
        use crate::ast::ValidationRule;

        self.advance(); // consume 'validate'

        // Skip optional: request / the / with / body
        while self.current_token() == &Token::Request
            || self.current_token() == &Token::Body
            || self.current_token() == &Token::With
            || self.current_token() == &Token::The
            || matches!(self.current_token(), Token::Identifier(name) if name == "request")
        {
            self.advance();
        }

        let mut rules = Vec::new();

        while !matches!(self.current_token(), Token::Newline | Token::EOF | Token::End) {
            // Get field name
            let field = match self.current_token() {
                Token::String(s) => s.clone(),
                Token::Identifier(s) => s.clone(),
                Token::Item | Token::Value | Token::Type => {
                    self.token_as_identifier(self.current_token()).unwrap_or_default()
                }
                _ => break,
            };
            self.advance();

            // Skip optional "is" / "be" / "with" / article
            if matches!(
                self.current_token(),
                Token::Is | Token::Be | Token::With | Token::A | Token::An | Token::The
            ) {
                self.advance();
                if matches!(self.current_token(), Token::A | Token::An | Token::The) {
                    self.advance();
                }
            }

            // Determine rule
            match self.current_token() {
                Token::Required => {
                    self.advance();
                    rules.push(ValidationRule::Required(field));
                }
                Token::Identifier(name) if name == "as" => {
                    self.advance();
                    let expected = match self.current_token() {
                        Token::Identifier(s) => s.clone(),
                        Token::String(s) => s.clone(),
                        Token::Email => "email".to_string(),
                        Token::Text => "string".to_string(),
                        _ => return Err("Expected a type after 'as' (string, number, email, boolean)".to_string()),
                    };
                    self.advance();
                    rules.push(ValidationRule::Type { field, expected });
                }
                Token::Identifier(name) if name == "min" || name == "minimum" => {
                    self.advance();
                    while matches!(
                        self.current_token(),
                        Token::Value | Token::Length | Token::Of | Token::A
                    ) {
                        self.advance();
                    }
                    let value = match self.current_token() {
                        Token::Number(n) => *n,
                        _ => return Err("Expected number after 'min'".to_string()),
                    };
                    self.advance();
                    rules.push(ValidationRule::Min { field, value });
                }
                Token::Identifier(name) if name == "max" || name == "maximum" => {
                    self.advance();
                    while matches!(
                        self.current_token(),
                        Token::Value | Token::Length | Token::Of | Token::A
                    ) {
                        self.advance();
                    }
                    let value = match self.current_token() {
                        Token::Number(n) => *n,
                        _ => return Err("Expected number after 'max'".to_string()),
                    };
                    self.advance();
                    rules.push(ValidationRule::Max { field, value });
                }
                _ => break,
            }

            // Skip "and" or comma
            if matches!(self.current_token(), Token::And | Token::Comma) {
                self.advance();
            }
        }

        Ok(Statement::Validate { rules })
    }

    fn parse_signup(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Signup)?;

        // signup user "alice" with email "alice@test.com" and password "pass123"
        // Skip optional "user"
        if matches!(self.current_token(), Token::User) {
            self.advance();
        }

        // Get username
        let username = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected username string after 'signup'".to_string()),
        };
        self.advance();

        // Expect "with email"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }
        if matches!(self.current_token(), Token::Email) {
            self.advance();
        }

        // Get email
        let email = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected email string".to_string()),
        };
        self.advance();

        // Expect "and password"
        if matches!(self.current_token(), Token::And) {
            self.advance();
        }
        if matches!(self.current_token(), Token::Password) {
            self.advance();
        }

        // Get password
        let password = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected password string".to_string()),
        };
        self.advance();

        Ok(Statement::Signup {
            username,
            email,
            password,
        })
    }

    fn parse_login(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Login)?;

        // login user "alice" with password "pass123"
        // Skip optional "user"
        if matches!(self.current_token(), Token::User) {
            self.advance();
        }

        // Get username
        let username = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected username string after 'login'".to_string()),
        };
        self.advance();

        // Expect "with password"
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }
        if matches!(self.current_token(), Token::Password) {
            self.advance();
        }

        // Get password
        let password = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected password string".to_string()),
        };
        self.advance();

        Ok(Statement::Login { username, password })
    }

    fn parse_logout(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Logout)?;
        Ok(Statement::Logout)
    }

    fn parse_begin_transaction(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Begin)?;
        // Optional "transaction"
        if matches!(self.current_token(), Token::Identifier(name) if name == "transaction") {
            self.advance();
        }
        Ok(Statement::BeginTransaction)
    }

    fn parse_test_block(&mut self) -> Result<Statement, String> {
        self.consume(&Token::Test)?;

        // Get test name
        let name = match self.current_token() {
            Token::String(s) => s.clone(),
            _ => return Err("Expected test name string".to_string()),
        };
        self.advance();

        // Skip newline
        while matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        // Parse test body
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::End | Token::EOF) {
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            body.push(self.parse_statement()?);
        }

        // Consume end
        if matches!(self.current_token(), Token::End) {
            self.advance();
        }

        Ok(Statement::TestBlock { name, body })
    }

    fn parse_assert(&mut self) -> Result<Statement, String> {
        self.advance(); // consume assert/expect

        // Parse condition
        let condition = self.parse_comparison_expression()?;

        // Optional "with message '...'" or "message '...'"
        let mut message = None;
        if matches!(self.current_token(), Token::With) {
            self.advance();
        }
        if matches!(self.current_token(), Token::Message) {
            self.advance();
            if let Token::String(s) = self.current_token() {
                message = Some(s.clone());
                self.advance();
            }
        }

        Ok(Statement::Assert { condition, message })
    }

    fn parse_read_file(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "read"

        // Optional "file"
        if matches!(self.current_token(), Token::File) {
            self.advance();
        }

        // Get file path
        let path = self.parse_expression()?;

        // Expect "into"
        if !matches!(self.current_token(), Token::Into) {
            return Err("Expected 'into' after file path".to_string());
        }
        self.advance();

        // Get variable name
        let into = self.token_as_identifier(self.current_token())
            .ok_or_else(|| "Expected variable name".to_string())?;
        self.advance();

        Ok(Statement::ReadFile { path, into })
    }

    fn parse_write_file(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "write"

        // Get content expression
        let content = self.parse_expression()?;

        // Expect "to"
        if !matches!(self.current_token(), Token::To) {
            return Err("Expected 'to' after content".to_string());
        }
        self.advance();

        // Optional "file"
        if matches!(self.current_token(), Token::File) {
            self.advance();
        }

        // Get file path
        let path = self.parse_expression()?;

        Ok(Statement::WriteFile { path, content })
    }

    fn parse_append_file(&mut self) -> Result<Statement, String> {
        self.advance(); // consume "append"

        // Get content expression
        let content = self.parse_expression()?;

        // Expect "to"
        if !matches!(self.current_token(), Token::To) {
            return Err("Expected 'to' after content".to_string());
        }
        self.advance();

        // Optional "file"
        if matches!(self.current_token(), Token::File) {
            self.advance();
        }

        // Get file path
        let path = self.parse_expression()?;

        Ok(Statement::AppendFile { path, content })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select_with_join() {
        let source = r#"select all from orders join customers on orders.customerid is customers.id"#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Select { collection, join, condition: None, .. } => {
                assert_eq!(collection, "orders");
                let j = join.as_ref().expect("Expected join clause");
                assert_eq!(j.collection, "customers");
                let cond = j.condition.as_ref().expect("Expected join condition");
                match cond {
                    Expression::BinaryOp { left, operator, right } => {
                        assert_eq!(*operator, crate::ast::BinaryOperator::EqualTo);
                        assert!(matches!(**left, Expression::PropertyAccess { .. }));
                        assert!(matches!(**right, Expression::PropertyAccess { .. }));
                    }
                    _ => panic!("Expected BinaryOp join condition"),
                }
            }
            _ => panic!("Expected Select with join"),
        }
    }

    #[test]
    fn test_parse_word_arithmetic() {
        use crate::ast::BinaryOperator;
        let source = r#"set n to 2 plus 3 times 4"#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
match &program.statements[0] {
            Statement::Assignment { value, .. } => {
                if let Expression::BinaryOp { operator, right, .. } = value {
                    // 2 plus (3 times 4): outer Add, right side Multiply
                    assert_eq!(operator, &BinaryOperator::Add);
                    if let Expression::BinaryOp { operator, .. } = &**right {
                        assert_eq!(operator, &BinaryOperator::Multiply);
                    } else {
                        panic!("Expected Multiply as right side");
                    }
                } else {
                    panic!("Expected nested BinaryOp");
                }
            }
            _ => panic!("Expected Assignment"),
        }
    }

    #[test]
    fn test_parse_assert_with_message() {
        let source = r#"assert 2 plus 3 equalto 5 with message "oops""#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        match &program.statements[0] {
            Statement::Assert { message, condition } => {
                assert_eq!(message, &Some("oops".to_string()));
                assert!(matches!(condition, Expression::BinaryOp { .. }));
            }
            _ => panic!("Expected Assert"),
        }
    }

    #[test]
    fn test_parse_set_style() {
        let source = r#"set style "h1" with color "blue" and font-size "32px""#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::SetStyle { selector, styles } => {
                assert_eq!(selector, "h1");
                assert_eq!(styles.len(), 2);
                assert_eq!(styles[0], ("color".to_string(), "blue".to_string()));
                assert_eq!(styles[1], ("font-size".to_string(), "32px".to_string()));
            }
            _ => panic!("Expected SetStyle"),
        }
    }

    #[test]
    fn test_parse_transaction_statements() {
        let source = r#"begin transaction
rollback
commit"#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 3);
        assert!(matches!(program.statements[0], Statement::BeginTransaction));
        assert!(matches!(program.statements[1], Statement::RollbackTransaction));
        assert!(matches!(program.statements[2], Statement::CommitTransaction));
    }

    #[test]
    fn test_parse_add_form() {
        let source = r#"add form with action "/api/contact" and method "post""#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        match &program.statements[0] {
            Statement::AddForm { properties } => {
                assert_eq!(properties[0], ("action".to_string(), Expression::String("/api/contact".to_string())));
                assert_eq!(properties[1], ("method".to_string(), Expression::String("post".to_string())));
            }
            _ => panic!("Expected AddForm"),
        }
    }

    #[test]
    fn test_parse_add_websocket_route() {
        let source = r#"add websocket route "/ws""#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        match &program.statements[0] {
            Statement::AddWebSocketRoute { path } => {
                assert_eq!(path, "/ws");
            }
            _ => panic!("Expected AddWebSocketRoute"),
        }
    }

    #[test]
    fn test_parse_add_input_with_name_and_required() {
        let source = r#"add input with type "email" and name "email" and placeholder "you@example.com" and required"#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        match &program.statements[0] {
            Statement::AddInput { input_type, properties } => {
                assert_eq!(input_type, "email");
                assert!(properties.len() >= 3, "expected placeholder/name/required, got {:?}", properties);
                assert!(properties.iter().any(|(k, _)| k == "name"));
                assert!(properties.iter().any(|(k, _)| k == "required"));
                assert!(properties.iter().any(|(k, _)| k == "placeholder"));
            }
            _ => panic!("Expected AddInput"),
        }
    }

    #[test]
    fn test_parse_add_element() {
        let source = r#"add element "div" with text "Hello" and class "card""#;
        let mut lexer = engcode_lexer::Lexer::new(source.to_string());
        let tokens = lexer.tokenize_with_positions().into_iter().map(|t| t.token).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        match &program.statements[0] {
            Statement::AddElement { element_type, properties } => {
                assert_eq!(element_type, "div");
                assert!(properties.iter().any(|(k, _)| k == "text"));
                assert!(properties.iter().any(|(k, _)| k == "class"));
            }
            _ => panic!("Expected AddElement"),
        }
    }

    #[test]
    fn test_parse_create_database() {
        let tokens = vec![
            Token::Create,
            Token::A,
            Token::Database,
            Token::Called,
            Token::String("Roadmap".to_string()),
            Token::EOF,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::CreateDatabase { name } => {
                assert_eq!(name, "Roadmap");
            }
            _ => panic!("Expected CreateDatabase"),
        }
    }

    #[test]
    fn test_parse_create_collections() {
        let tokens = vec![
            Token::Create,
            Token::These,
            Token::Collections,
            Token::In,
            Token::It,
            Token::Newline,
            Token::Identifier("may".to_string()),
            Token::Newline,
            Token::Identifier("april".to_string()),
            Token::Newline,
            Token::Identifier("march".to_string()),
            Token::EOF,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::CreateCollections { database: _, names } => {
                assert_eq!(names.len(), 3);
                assert_eq!(names[0], "may");
                assert_eq!(names[1], "april");
                assert_eq!(names[2], "march");
            }
            _ => panic!("Expected CreateCollections"),
        }
    }

    #[test]
    fn test_parse_full_example() {
        let tokens = vec![
            Token::Create,
            Token::A,
            Token::Database,
            Token::Called,
            Token::String("Roadmap".to_string()),
            Token::Newline,
            Token::Create,
            Token::These,
            Token::Collections,
            Token::In,
            Token::It,
            Token::Newline,
            Token::Identifier("may".to_string()),
            Token::Newline,
            Token::Identifier("april".to_string()),
            Token::Newline,
            Token::Identifier("march".to_string()),
            Token::EOF,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.statements.len(), 2);
    }
}
