use colored::*;
use engcode_parser::{Program, Statement, Expression};
use crate::context::ExecutionContext;
use crate::error::RuntimeError;
use crate::value::Value;

pub struct Interpreter {
    context: ExecutionContext,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
        }
    }

    pub fn execute(&mut self, program: Program) -> Result<(), RuntimeError> {
        for statement in program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: Statement) -> Result<(), RuntimeError> {
        match stmt {
            Statement::CreateDatabase { name } => {
                self.execute_create_database(name)
            }
            Statement::CreateCollections { database, names } => {
                self.execute_create_collections(database, names)
            }
            Statement::Show { message } => {
                let value = self.evaluate_expression(message)?;
                println!("{}", value);
                Ok(())
            }
            Statement::Assignment { variable, value } => {
                self.execute_assignment(variable, value)
            }
            Statement::Insert { collection, data } => {
                self.execute_insert(collection, data)
            }
            Statement::Select { collection, fields, condition } => {
                self.execute_select(collection, fields, condition)
            }
            Statement::Update { collection, data, condition } => {
                self.execute_update(collection, data, condition)
            }
            Statement::Delete { collection, condition } => {
                self.execute_delete(collection, condition)
            }
            Statement::CreateServer { port } => {
                self.execute_create_server(port)
            }
            Statement::AddRoute { method, path, response } => {
                self.execute_add_route(method, path, response)
            }
            Statement::AddDataRoute { method, path, collection } => {
                self.execute_add_data_route(method, path, collection)
            }
            Statement::StartServer { duration_seconds } => {
                self.execute_start_server(duration_seconds)
            }
            // HTML statements
            Statement::CreatePage { name, title } => {
                self.execute_create_page(name, title)
            }
            Statement::AddButton { text, properties } => {
                self.execute_add_button(text, properties)
            }
            Statement::AddInput { input_type, properties } => {
                self.execute_add_input(input_type, properties)
            }
            Statement::AddHeading { level, text } => {
                self.execute_add_heading(level, text)
            }
            Statement::AddParagraph { text } => {
                self.execute_add_paragraph(text)
            }
            Statement::AddElement { .. } => Ok(()), // TODO
            Statement::AddForm { .. } => Ok(()), // TODO
            Statement::AddLink { .. } => Ok(()), // TODO
            Statement::AddImage { src, alt } => {
                self.execute_add_image(src, alt)
            }
            Statement::SetStyle { .. } => Ok(()), // TODO
            Statement::RenderPage { page_name } => {
                self.execute_render_page(page_name)
            }
            // Control flow statements
            Statement::If { condition, then_block, else_block } => {
                self.execute_if(condition, then_block, else_block)
            }
            Statement::While { condition, body } => {
                self.execute_while(condition, body)
            }
            Statement::For { variable, start, end, body } => {
                self.execute_for(variable, start, end, body)
            }
            Statement::ForEach { variable, collection, body } => {
                self.execute_for_each(variable, collection, body)
            }
            Statement::Break => {
                Err(RuntimeError::BreakOutsideLoop)
            }
            Statement::Continue => {
                Err(RuntimeError::ContinueOutsideLoop)
            }
            // Function statements
            Statement::FunctionDef { name, parameters, body } => {
                self.execute_function_def(name, parameters, body)
            }
            Statement::FunctionCall { name, arguments } => {
                self.execute_function_call(name, arguments)?;
                Ok(())
            }
            Statement::Return { value } => {
                let return_val = if let Some(expr) = value {
                    self.evaluate_expression(expr)?
                } else {
                    Value::Null
                };
                Err(RuntimeError::ReturnValue(return_val))
            }
            // Error handling
            Statement::TryCatch { try_block, catch_block, finally_block } => {
                self.execute_try_catch(try_block, catch_block, finally_block)
            }
            Statement::Throw { message } => {
                let msg = self.evaluate_expression(message)?;
                let error_msg = match msg {
                    Value::String(s) => s,
                    _ => format!("{}", msg),
                };
                Err(RuntimeError::UserError(error_msg))
            }
            Statement::Signup { username, email, password } => {
                self.execute_signup(username, email, password)
            }
            Statement::Login { username, password } => {
                self.execute_login(username, password)
            }
            Statement::Logout => {
                self.execute_logout()
            }
            Statement::TestBlock { name, body } => {
                self.execute_test_block(name, body)
            }
            Statement::Assert { condition, message } => {
                self.execute_assert(condition, message)
            }
            Statement::ReadFile { path, into } => {
                self.execute_read_file(path, into)
            }
            Statement::WriteFile { path, content } => {
                self.execute_write_file(path, content)
            }
            Statement::AppendFile { path, content } => {
                self.execute_append_file(path, content)
            }
        }
    }

    fn execute_create_database(&mut self, name: String) -> Result<(), RuntimeError> {
        // Create database using stdlib
        let db = engcode_stdlib::database::create_database(&name)?;

        println!("{} Created database {}", "✓".green().bold(), name.cyan().bold());

        self.context.add_database(name.clone(), db);
        self.context.set_current_database(Some(name));

        Ok(())
    }

    fn execute_create_collections(
        &mut self,
        database: Option<String>,
        names: Vec<String>,
    ) -> Result<(), RuntimeError> {
        let db_name = database
            .or_else(|| self.context.current_database())
            .ok_or(RuntimeError::NoDatabaseContext)?;

        let db = self.context.get_database_mut(&db_name)
            .ok_or_else(|| RuntimeError::DatabaseNotFound(db_name.clone()))?;

        for name in names {
            engcode_stdlib::database::create_collection(db, &name)?;
            // Reset the collection so repeated runs don't accumulate duplicate rows.
            // "create collections in it" declares the schema, so each run starts fresh.
            engcode_stdlib::database::reset_collection(db, &name)?;
            println!("  {} Created collection {}", "✓".green(), name.bright_white());
        }

        Ok(())
    }

    fn execute_assignment(&mut self, variable: String, expr: Expression) -> Result<(), RuntimeError> {
        let value = self.evaluate_expression(expr)?;
        self.context.set_variable(variable.clone(), value.clone());

        println!("  {} Set {} = {}", "→".bright_black(), variable.cyan(), value);
        Ok(())
    }

    fn evaluate_expression(&mut self, expr: Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(n)),
            Expression::String(s) => {
                // Handle string interpolation
                Ok(Value::String(self.interpolate_string(&s)?))
            }
            Expression::Boolean(b) => Ok(Value::Boolean(b)),
            Expression::Null => Ok(Value::Null),
            Expression::Identifier(name) => {
                self.context
                    .get_variable(&name)
                    .cloned()
                    .ok_or_else(|| RuntimeError::VariableNotFound(name))
            }
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate_expression(elem)?);
                }
                Ok(Value::Array(values))
            }
            Expression::Object(pairs) => {
                use std::collections::HashMap;
                let mut map = HashMap::new();
                for (key, value_expr) in pairs {
                    let value = self.evaluate_expression(value_expr)?;
                    map.insert(key, value);
                }
                Ok(Value::Object(map))
            }
            Expression::BinaryOp { left, operator, right } => {
                let left_val = self.evaluate_expression(*left.clone())?;
                let right_val = self.evaluate_expression(*right.clone())?;
                self.evaluate_binary_op(left_val, &operator, right_val)
            }
            Expression::UnaryOp { operator, operand } => {
                let val = self.evaluate_expression(*operand.clone())?;
                match operator {
                    engcode_parser::ast::UnaryOperator::Not => {
                        Ok(Value::Boolean(!val.is_truthy()))
                    }
                    engcode_parser::ast::UnaryOperator::Negative => {
                        let num = val.as_number().ok_or_else(|| {
                            RuntimeError::TypeError("Operand must be a number".to_string())
                        })?;
                        Ok(Value::Number(-num))
                    }
                }
            }
            Expression::FunctionCall { name, arguments } => {
                self.execute_function_call(name.clone(), arguments.clone())
            }
            Expression::MethodCall { object, method, arguments } => {
                self.execute_method_call(*object.clone(), method.clone(), arguments.clone())
            }
            Expression::IndexAccess { object, index } => {
                let obj_value = self.evaluate_expression(*object.clone())?;
                let index_value = self.evaluate_expression(*index.clone())?;

                match obj_value {
                    Value::Array(arr) => {
                        let idx = index_value.as_number()
                            .ok_or_else(|| RuntimeError::TypeError("Array index must be a number".to_string()))? as usize;

                        arr.get(idx)
                            .cloned()
                            .ok_or_else(|| RuntimeError::TypeError(format!("Index {} out of bounds", idx)))
                    }
                    Value::String(s) => {
                        let idx = index_value.as_number()
                            .ok_or_else(|| RuntimeError::TypeError("String index must be a number".to_string()))? as usize;

                        s.chars().nth(idx)
                            .map(|c| Value::String(c.to_string()))
                            .ok_or_else(|| RuntimeError::TypeError(format!("Index {} out of bounds", idx)))
                    }
                    _ => Err(RuntimeError::TypeError("Can only index arrays and strings".to_string()))
                }
            }
            Expression::PropertyAccess { object, property } => {
                let obj_value = self.evaluate_expression(*object.clone())?;

                match obj_value {
                    Value::Object(map) => {
                        map.get(&property)
                            .cloned()
                            .ok_or_else(|| RuntimeError::TypeError(format!("Property '{}' not found", property)))
                    }
                    _ => Err(RuntimeError::TypeError(format!("Cannot access property '{}' on non-object", property)))
                }
            }
        }
    }

    fn execute_insert(
        &mut self,
        collection: String,
        data: Vec<(String, Expression)>,
    ) -> Result<(), RuntimeError> {
        // Evaluate all expressions to values and build simple JSON string
        let mut json_pairs = Vec::new();
        for (key, expr) in data {
            let value = self.evaluate_expression(expr)?;
            let value_str = match value {
                Value::String(s) => format!("\"{}\"", s),
                Value::Number(n) => n.to_string(),
                Value::Boolean(b) => b.to_string(),
                Value::Null => "null".to_string(),
                _ => format!("{}", value),
            };
            json_pairs.push(format!("\"{}\": {}", key, value_str));
        }

        let json_data = format!("{{{}}}", json_pairs.join(", "));

        // Now get database and insert
        let db_name = self.context.current_database()
            .ok_or(RuntimeError::NoDatabaseContext)?;

        let db = self.context.get_database_mut(&db_name)
            .ok_or_else(|| RuntimeError::DatabaseNotFound(db_name.clone()))?;

        engcode_stdlib::database::insert_data(db, &collection, &json_data)?;

        println!("  {} Inserted into {}", "✓".green(), collection.cyan());
        Ok(())
    }

    fn execute_select(
        &mut self,
        collection: String,
        _fields: Vec<String>,
        condition: Option<Expression>,
    ) -> Result<(), RuntimeError> {
        let db_name = self.context.current_database()
            .ok_or(RuntimeError::NoDatabaseContext)?;

        let db = self.context.get_database_mut(&db_name)
            .ok_or_else(|| RuntimeError::DatabaseNotFound(db_name.clone()))?;

        let mut rows = engcode_stdlib::database::select_all(db, &collection)?;

        // Filter by WHERE condition if present
        if let Some(cond) = condition {
            rows = self.filter_rows(rows, &cond)?;
        }

        println!("\n{} Found {} rows in {}:", "→".cyan(), rows.len(), collection.cyan());
        for (i, row) in rows.iter().enumerate() {
            println!("  {}. {}", i + 1, row);
        }
        println!();

        Ok(())
    }

    fn filter_rows(&mut self, rows: Vec<String>, condition: &Expression) -> Result<Vec<String>, RuntimeError> {
        use serde_json::Value as JsonValue;

        let mut filtered = Vec::new();

        for row in rows {
            // Parse JSON row
            let json: JsonValue = serde_json::from_str(&row)
                .map_err(|e| RuntimeError::TypeError(format!("Failed to parse row: {}", e)))?;

            // Evaluate condition with row data
            if self.evaluate_condition_with_data(&json, condition)? {
                filtered.push(row);
            }
        }

        Ok(filtered)
    }

    fn evaluate_condition_with_data(&mut self, data: &serde_json::Value, condition: &Expression) -> Result<bool, RuntimeError> {
        use serde_json::Value as JsonValue;

        match condition {
            Expression::BinaryOp { left, operator, right } => {
                // Get left value (usually a field name)
                let left_val = match &**left {
                    Expression::Identifier(field) => {
                        // Get field from JSON data
                        match data.get(field) {
                            Some(JsonValue::String(s)) => Value::String(s.clone()),
                            Some(JsonValue::Number(n)) => Value::Number(n.as_f64().unwrap_or(0.0)),
                            Some(JsonValue::Bool(b)) => Value::Boolean(*b),
                            Some(JsonValue::Null) => Value::Null,
                            _ => Value::Null,
                        }
                    }
                    _ => self.evaluate_expression((**left).clone())?,
                };

                // Get right value
                let right_val = self.evaluate_expression((**right).clone())?;

                // Compare
                use engcode_parser::ast::BinaryOperator;
                let result = match operator {
                    BinaryOperator::EqualTo => {
                        match (&left_val, &right_val) {
                            (Value::Number(l), Value::Number(r)) => l == r,
                            (Value::String(l), Value::String(r)) => l == r,
                            (Value::Boolean(l), Value::Boolean(r)) => l == r,
                            _ => false,
                        }
                    }
                    BinaryOperator::GreaterThan => {
                        match (&left_val, &right_val) {
                            (Value::Number(l), Value::Number(r)) => l > r,
                            _ => false,
                        }
                    }
                    BinaryOperator::LessThan => {
                        match (&left_val, &right_val) {
                            (Value::Number(l), Value::Number(r)) => l < r,
                            _ => false,
                        }
                    }
                    BinaryOperator::NotEqualTo => {
                        match (&left_val, &right_val) {
                            (Value::Number(l), Value::Number(r)) => l != r,
                            (Value::String(l), Value::String(r)) => l != r,
                            (Value::Boolean(l), Value::Boolean(r)) => l != r,
                            _ => true,
                        }
                    }
                    BinaryOperator::And => {
                        left_val.is_truthy() && right_val.is_truthy()
                    }
                    BinaryOperator::Or => {
                        left_val.is_truthy() || right_val.is_truthy()
                    }
                    _ => false,
                };

                Ok(result)
            }
            _ => {
                // For non-binary expressions, just evaluate as boolean
                let val = self.evaluate_expression(condition.clone())?;
                Ok(val.is_truthy())
            }
        }
    }

    fn execute_update(
        &mut self,
        collection: String,
        data: Vec<(String, Expression)>,
        condition: Option<Expression>,
    ) -> Result<(), RuntimeError> {
        use serde_json::Value as JsonValue;

        // Evaluate all update expressions first
        let mut updates = Vec::new();
        for (key, expr) in data {
            let value = self.evaluate_expression(expr)?;
            updates.push((key, value));
        }

        let db_name = self.context.current_database()
            .ok_or(RuntimeError::NoDatabaseContext)?;

        // Fetch all rows with their ids
        let rows = {
            let db = self.context.get_database_mut(&db_name)
                .ok_or_else(|| RuntimeError::DatabaseNotFound(db_name.clone()))?;
            engcode_stdlib::database::select_rows(db, &collection)?
        };

        // Filter matching rows and merge updates into their JSON
        let mut pending: Vec<(i64, String)> = Vec::new();
        for (id, row_json) in rows {
            let mut json: JsonValue = serde_json::from_str(&row_json)
                .map_err(|e| RuntimeError::TypeError(format!("Failed to parse row: {}", e)))?;

            if let Some(cond) = &condition {
                if !self.evaluate_condition_with_data(&json, cond)? {
                    continue;
                }
            }

            if let Some(map) = json.as_object_mut() {
                for (key, value) in &updates {
                    map.insert(key.clone(), Self::value_to_json_value(value));
                }
            }

            pending.push((id, Self::format_json_object(&json)));
        }

        // Apply updates to database
        for (id, new_data) in &pending {
            let db = self.context.get_database_mut(&db_name)
                .ok_or_else(|| RuntimeError::DatabaseNotFound(db_name.clone()))?;
            engcode_stdlib::database::update_row(db, &collection, *id, new_data)?;
        }

        println!("  {} Updated {} ({} row{})", "✓".green(), collection.cyan(), pending.len(), if pending.len() == 1 { "" } else { "s" });
        Ok(())
    }

    fn execute_delete(
        &mut self,
        collection: String,
        condition: Option<Expression>,
    ) -> Result<(), RuntimeError> {
        use serde_json::Value as JsonValue;

        let db_name = self.context.current_database()
            .ok_or(RuntimeError::NoDatabaseContext)?;

        let rows = {
            let db = self.context.get_database_mut(&db_name)
                .ok_or_else(|| RuntimeError::DatabaseNotFound(db_name.clone()))?;
            engcode_stdlib::database::select_rows(db, &collection)?
        };

        let mut to_delete: Vec<i64> = Vec::new();
        for (id, row_json) in rows {
            let json: JsonValue = serde_json::from_str(&row_json)
                .map_err(|e| RuntimeError::TypeError(format!("Failed to parse row: {}", e)))?;

            if let Some(cond) = &condition {
                if !self.evaluate_condition_with_data(&json, cond)? {
                    continue;
                }
            }

            to_delete.push(id);
        }

        let db = self.context.get_database_mut(&db_name)
            .ok_or_else(|| RuntimeError::DatabaseNotFound(db_name.clone()))?;
        engcode_stdlib::database::delete_rows_by_ids(db, &collection, &to_delete)?;

        println!("  {} Deleted {} row{} from {}", "✓".green(), to_delete.len(), if to_delete.len() == 1 { "" } else { "s" }, collection.cyan());
        Ok(())
    }

    fn value_to_json_value(value: &Value) -> serde_json::Value {
        use serde_json::Value as JsonValue;
        match value {
            Value::String(s) => JsonValue::String(s.clone()),
            Value::Number(n) => serde_json::Number::from_f64(*n).map(JsonValue::Number).unwrap_or(JsonValue::Null),
            Value::Boolean(b) => JsonValue::Bool(*b),
            Value::Null => JsonValue::Null,
            Value::Array(arr) => JsonValue::Array(arr.iter().map(Self::value_to_json_value).collect()),
            Value::Object(map) => {
                let obj = map.iter().map(|(k, v)| (k.clone(), Self::value_to_json_value(v))).collect();
                JsonValue::Object(obj)
            }
        }
    }

    fn format_json_object(value: &serde_json::Value) -> String {
        use serde_json::Value as JsonValue;
        match value {
            JsonValue::Object(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, Self::format_json_value(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            other => Self::format_json_value(other),
        }
    }

    fn format_json_value(value: &serde_json::Value) -> String {
        use serde_json::Value as JsonValue;
        match value {
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Number(n) => {
                if let Some(f) = n.as_f64() {
                    if f.fract() == 0.0 {
                        format!("{:.0}", f)
                    } else {
                        f.to_string()
                    }
                } else {
                    n.to_string()
                }
            }
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Null => "null".to_string(),
            JsonValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(Self::format_json_value).collect();
                format!("[{}]", items.join(", "))
            }
            JsonValue::Object(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, Self::format_json_value(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
        }
    }

    fn execute_create_server(&mut self, port: u16) -> Result<(), RuntimeError> {
        let server = engcode_stdlib::webserver::create_server(port).serve_html_pages();
        self.context.set_web_server(server);

        println!("{} Created web server on port {}", "✓".green().bold(), port.to_string().cyan());
        println!("  {} HTML pages will be served from /public directory", "→".bright_black());
        Ok(())
    }

    fn execute_add_route(
        &mut self,
        method: String,
        path: String,
        response_expr: Expression,
    ) -> Result<(), RuntimeError> {
        let response_value = self.evaluate_expression(response_expr)?;
        let response_str = format!("{}", response_value);

        if let Some(server) = self.context.get_web_server_mut() {
            server.add_route(&method, &path, response_str);
            println!("  {} Added route {} {}", "→".cyan(), method.yellow(), path.cyan());
        } else {
            return Err(RuntimeError::TypeError(
                "No web server created. Create a server first.".to_string(),
            ));
        }

        Ok(())
    }

    fn execute_start_server(&mut self, duration_seconds: Option<f64>) -> Result<(), RuntimeError> {
        if let Some(server) = self.context.take_web_server() {
            println!("\n{}", "Starting web server...".green().bold());
            let duration = duration_seconds.map(std::time::Duration::from_secs_f64);
            server.start(duration).map_err(|e| RuntimeError::TypeError(e))?;
            Ok(())
        } else {
            Err(RuntimeError::TypeError(
                "No web server to start. Create a server first.".to_string(),
            ))
        }
    }

    // HTML execution methods
    fn execute_create_page(&mut self, name: String, title: Option<String>) -> Result<(), RuntimeError> {
        use engcode_stdlib::HtmlPage;

        let page = HtmlPage::new(name.clone(), title.clone());
        self.context.set_html_page(page);

        let title_display = title.unwrap_or_else(|| name.clone());
        println!("{} Created page \"{}\" with title \"{}\"", "✓".green().bold(), name.cyan(), title_display);
        Ok(())
    }

    fn execute_add_button(&mut self, text: String, _properties: Vec<(String, Expression)>) -> Result<(), RuntimeError> {
        use engcode_stdlib::HtmlElement;

        if let Some(page) = self.context.get_html_page_mut() {
            page.add_element(HtmlElement::Button {
                text: text.clone(),
                id: None,
                class: None,
                onclick: None,
            });
            println!("  {} Added button: \"{}\"", "→".cyan(), text);
            Ok(())
        } else {
            Err(RuntimeError::TypeError(
                "No page created. Use 'create a page called X' first.".to_string(),
            ))
        }
    }

    fn execute_add_input(&mut self, input_type: String, _properties: Vec<(String, Expression)>) -> Result<(), RuntimeError> {
        use engcode_stdlib::HtmlElement;

        if let Some(page) = self.context.get_html_page_mut() {
            page.add_element(HtmlElement::Input {
                input_type: input_type.clone(),
                id: None,
                name: None,
                placeholder: None,
                value: None,
                required: false,
            });
            println!("  {} Added input: type=\"{}\"", "→".cyan(), input_type);
            Ok(())
        } else {
            Err(RuntimeError::TypeError(
                "No page created. Use 'create a page called X' first.".to_string(),
            ))
        }
    }

    fn execute_add_heading(&mut self, level: u8, text: String) -> Result<(), RuntimeError> {
        use engcode_stdlib::HtmlElement;

        if let Some(page) = self.context.get_html_page_mut() {
            page.add_element(HtmlElement::Heading {
                level,
                text: text.clone(),
            });
            println!("  {} Added heading h{}: \"{}\"", "→".cyan(), level, text);
            Ok(())
        } else {
            Err(RuntimeError::TypeError(
                "No page created. Use 'create a page called X' first.".to_string(),
            ))
        }
    }

    fn execute_add_paragraph(&mut self, text: String) -> Result<(), RuntimeError> {
        use engcode_stdlib::HtmlElement;

        if let Some(page) = self.context.get_html_page_mut() {
            page.add_element(HtmlElement::Paragraph {
                text: text.clone(),
            });
            println!("  {} Added paragraph: \"{}\"", "→".cyan(), text);
            Ok(())
        } else {
            Err(RuntimeError::TypeError(
                "No page created. Use 'create a page called X' first.".to_string(),
            ))
        }
    }

    fn execute_add_image(&mut self, src: String, alt: String) -> Result<(), RuntimeError> {
        use engcode_stdlib::HtmlElement;

        if let Some(page) = self.context.get_html_page_mut() {
            page.add_element(HtmlElement::Image {
                src: src.clone(),
                alt: alt.clone(),
            });
            println!("  {} Added image: src=\"{}\" alt=\"{}\"", "→".cyan(), src, alt);
            Ok(())
        } else {
            Err(RuntimeError::TypeError(
                "No page created. Use 'create a page called X' first.".to_string(),
            ))
        }
    }

    fn execute_render_page(&mut self, page_name: String) -> Result<(), RuntimeError> {
        if let Some(page) = self.context.take_html_page() {
            // Apply default styles
            let default_styles = engcode_stdlib::htmlbuilder::create_default_styles();
            let mut page_with_styles = page;
            for (selector, props) in default_styles {
                for (prop, val) in props {
                    page_with_styles.set_style(selector.clone(), prop, val);
                }
            }

            // Save to file in public directory
            match page_with_styles.save_to_file("public") {
                Ok(filename) => {
                    println!("\n{} Rendered page to {}", "✓".green().bold(), filename.cyan());
                    println!("  {} Open in browser: file://{}/{}", "→".bright_black(), std::env::current_dir().unwrap().display(), filename);
                    Ok(())
                }
                Err(e) => Err(RuntimeError::TypeError(format!("Failed to save HTML file: {}", e))),
            }
        } else {
            Err(RuntimeError::TypeError(
                format!("No page named '{}' found", page_name),
            ))
        }
    }

    // Control flow execution
    fn execute_if(
        &mut self,
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    ) -> Result<(), RuntimeError> {
        let condition_value = self.evaluate_expression(condition)?;

        if condition_value.is_truthy() {
            for stmt in then_block {
                self.execute_statement(stmt)?;
            }
        } else if let Some(else_stmts) = else_block {
            for stmt in else_stmts {
                self.execute_statement(stmt)?;
            }
        }

        Ok(())
    }

    fn execute_while(&mut self, condition: Expression, body: Vec<Statement>) -> Result<(), RuntimeError> {
        loop {
            let condition_value = self.evaluate_expression(condition.clone())?;

            if !condition_value.is_truthy() {
                break;
            }

            for stmt in &body {
                match self.execute_statement(stmt.clone()) {
                    Err(RuntimeError::BreakOutsideLoop) => return Ok(()), // Break from loop
                    Err(RuntimeError::ContinueOutsideLoop) => break,       // Continue to next iteration
                    Err(e) => return Err(e),
                    Ok(_) => {}
                }
            }
        }

        Ok(())
    }

    fn execute_for(
        &mut self,
        variable: String,
        start: Expression,
        end: Expression,
        body: Vec<Statement>,
    ) -> Result<(), RuntimeError> {
        let start_val = self.evaluate_expression(start)?;
        let end_val = self.evaluate_expression(end)?;

        let start_num = start_val.as_number().ok_or_else(|| {
            RuntimeError::TypeError("Start value must be a number".to_string())
        })?;

        let end_num = end_val.as_number().ok_or_else(|| {
            RuntimeError::TypeError("End value must be a number".to_string())
        })?;

        for i in (start_num as i64)..=(end_num as i64) {
            self.context.set_variable(variable.clone(), Value::Number(i as f64));

            for stmt in &body {
                match self.execute_statement(stmt.clone()) {
                    Err(RuntimeError::BreakOutsideLoop) => return Ok(()),
                    Err(RuntimeError::ContinueOutsideLoop) => break,
                    Err(e) => return Err(e),
                    Ok(_) => {}
                }
            }
        }

        Ok(())
    }

    fn execute_for_each(
        &mut self,
        variable: String,
        collection: Expression,
        body: Vec<Statement>,
    ) -> Result<(), RuntimeError> {
        let collection_value = self.evaluate_expression(collection)?;

        let items = collection_value.as_array().ok_or_else(|| {
            RuntimeError::TypeError("For each requires an array".to_string())
        })?;

        for item in items {
            self.context.set_variable(variable.clone(), item.clone());

            for stmt in &body {
                match self.execute_statement(stmt.clone()) {
                    Err(RuntimeError::BreakOutsideLoop) => return Ok(()),
                    Err(RuntimeError::ContinueOutsideLoop) => break,
                    Err(e) => return Err(e),
                    Ok(_) => {}
                }
            }
        }

        Ok(())
    }

    fn evaluate_binary_op(
        &self,
        left: Value,
        operator: &engcode_parser::ast::BinaryOperator,
        right: Value,
    ) -> Result<Value, RuntimeError> {
        use engcode_parser::ast::BinaryOperator;

        match operator {
            BinaryOperator::EqualTo => Ok(Value::Boolean(left == right)),
            BinaryOperator::NotEqualTo => Ok(Value::Boolean(left != right)),
            BinaryOperator::GreaterThan => {
                let l = left.as_number().ok_or_else(|| RuntimeError::TypeError("Left operand must be a number".to_string()))?;
                let r = right.as_number().ok_or_else(|| RuntimeError::TypeError("Right operand must be a number".to_string()))?;
                Ok(Value::Boolean(l > r))
            }
            BinaryOperator::LessThan => {
                let l = left.as_number().ok_or_else(|| RuntimeError::TypeError("Left operand must be a number".to_string()))?;
                let r = right.as_number().ok_or_else(|| RuntimeError::TypeError("Right operand must be a number".to_string()))?;
                Ok(Value::Boolean(l < r))
            }
            BinaryOperator::GreaterThanOrEqual => {
                let l = left.as_number().ok_or_else(|| RuntimeError::TypeError("Left operand must be a number".to_string()))?;
                let r = right.as_number().ok_or_else(|| RuntimeError::TypeError("Right operand must be a number".to_string()))?;
                Ok(Value::Boolean(l >= r))
            }
            BinaryOperator::LessThanOrEqual => {
                let l = left.as_number().ok_or_else(|| RuntimeError::TypeError("Left operand must be a number".to_string()))?;
                let r = right.as_number().ok_or_else(|| RuntimeError::TypeError("Right operand must be a number".to_string()))?;
                Ok(Value::Boolean(l <= r))
            }
            BinaryOperator::And => {
                Ok(Value::Boolean(left.is_truthy() && right.is_truthy()))
            }
            BinaryOperator::Or => {
                Ok(Value::Boolean(left.is_truthy() || right.is_truthy()))
            }
            BinaryOperator::Add => {
                let l = left.as_number().ok_or_else(|| RuntimeError::TypeError("Left operand must be a number".to_string()))?;
                let r = right.as_number().ok_or_else(|| RuntimeError::TypeError("Right operand must be a number".to_string()))?;
                Ok(Value::Number(l + r))
            }
            BinaryOperator::Subtract => {
                let l = left.as_number().ok_or_else(|| RuntimeError::TypeError("Left operand must be a number".to_string()))?;
                let r = right.as_number().ok_or_else(|| RuntimeError::TypeError("Right operand must be a number".to_string()))?;
                Ok(Value::Number(l - r))
            }
            BinaryOperator::Multiply => {
                let l = left.as_number().ok_or_else(|| RuntimeError::TypeError("Left operand must be a number".to_string()))?;
                let r = right.as_number().ok_or_else(|| RuntimeError::TypeError("Right operand must be a number".to_string()))?;
                Ok(Value::Number(l * r))
            }
            BinaryOperator::Divide => {
                let l = left.as_number().ok_or_else(|| RuntimeError::TypeError("Left operand must be a number".to_string()))?;
                let r = right.as_number().ok_or_else(|| RuntimeError::TypeError("Right operand must be a number".to_string()))?;
                if r == 0.0 {
                    return Err(RuntimeError::TypeError("Division by zero".to_string()));
                }
                Ok(Value::Number(l / r))
            }
        }
    }

    // Function execution
    fn execute_function_def(
        &mut self,
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    ) -> Result<(), RuntimeError> {
        self.context.define_function(name.clone(), parameters, body);
        println!("{} Defined function {}", "✓".green().bold(), name.cyan());
        Ok(())
    }

    fn execute_function_call(
        &mut self,
        name: String,
        arguments: Vec<Expression>,
    ) -> Result<Value, RuntimeError> {
        // Get function definition
        let (parameters, body) = self
            .context
            .get_function(&name)
            .ok_or_else(|| RuntimeError::FunctionNotFound(name.clone()))?
            .clone();

        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.evaluate_expression(arg)?);
        }

        // Check argument count
        if arg_values.len() != parameters.len() {
            return Err(RuntimeError::TypeError(format!(
                "Function {} expects {} arguments, got {}",
                name,
                parameters.len(),
                arg_values.len()
            )));
        }

        // Save current variables (for scope isolation)
        let saved_vars = self.context.variables.clone();

        // Set parameters as variables
        for (param, value) in parameters.iter().zip(arg_values.iter()) {
            self.context.set_variable(param.clone(), value.clone());
        }

        // Execute function body
        let mut return_value = Value::Null;
        for stmt in body {
            match self.execute_statement(stmt) {
                Err(RuntimeError::ReturnValue(val)) => {
                    return_value = val;
                    break;
                }
                Err(e) => {
                    // Restore variables before returning error
                    self.context.variables = saved_vars;
                    return Err(e);
                }
                Ok(_) => {}
            }
        }

        // Restore variables
        self.context.variables = saved_vars;

        Ok(return_value)
    }

    fn execute_method_call(
        &mut self,
        object: Expression,
        method: String,
        arguments: Vec<Expression>,
    ) -> Result<Value, RuntimeError> {
        let mut obj_value = self.evaluate_expression(object)?;

        match method.as_str() {
            // Array methods
            "push" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("push requires 1 argument".to_string()));
                }
                let value = self.evaluate_expression(arguments[0].clone())?;
                obj_value.push(value).map_err(|e| RuntimeError::TypeError(e))?;
                Ok(Value::Null)
            }
            "pop" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("pop takes no arguments".to_string()));
                }
                obj_value.pop().map_err(|e| RuntimeError::TypeError(e))
            }
            "length" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("length takes no arguments".to_string()));
                }
                obj_value.get_length().map(Value::Number).map_err(|e| RuntimeError::TypeError(e))
            }
            "contains" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("contains requires 1 argument".to_string()));
                }
                let value = self.evaluate_expression(arguments[0].clone())?;
                obj_value.contains(&value).map(Value::Boolean).map_err(|e| RuntimeError::TypeError(e))
            }
            // String methods
            "uppercase" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("uppercase takes no arguments".to_string()));
                }
                obj_value.to_uppercase().map_err(|e| RuntimeError::TypeError(e))
            }
            "lowercase" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("lowercase takes no arguments".to_string()));
                }
                obj_value.to_lowercase().map_err(|e| RuntimeError::TypeError(e))
            }
            "trim" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("trim takes no arguments".to_string()));
                }
                obj_value.trim().map_err(|e| RuntimeError::TypeError(e))
            }
            "split" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("split requires 1 argument".to_string()));
                }
                let delimiter = self.evaluate_expression(arguments[0].clone())?;
                let delim_str = delimiter.as_string().ok_or_else(|| {
                    RuntimeError::TypeError("split delimiter must be a string".to_string())
                })?;
                obj_value.split_string(delim_str).map_err(|e| RuntimeError::TypeError(e))
            }
            "replace" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::TypeError("replace requires 2 arguments".to_string()));
                }
                let from = self.evaluate_expression(arguments[0].clone())?;
                let to = self.evaluate_expression(arguments[1].clone())?;
                let from_str = from.as_string().ok_or_else(|| {
                    RuntimeError::TypeError("replace 'from' must be a string".to_string())
                })?;
                let to_str = to.as_string().ok_or_else(|| {
                    RuntimeError::TypeError("replace 'to' must be a string".to_string())
                })?;
                obj_value.replace_string(from_str, to_str).map_err(|e| RuntimeError::TypeError(e))
            }
            "join" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("join requires 1 argument".to_string()));
                }
                let separator = self.evaluate_expression(arguments[0].clone())?;
                let sep_str = separator.as_string().ok_or_else(|| {
                    RuntimeError::TypeError("join separator must be a string".to_string())
                })?;
                obj_value.join(sep_str).map_err(|e| RuntimeError::TypeError(e))
            }
            "reverse" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("reverse takes no arguments".to_string()));
                }
                obj_value.reverse().map_err(|e| RuntimeError::TypeError(e))?;
                Ok(obj_value)
            }
            "sort" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("sort takes no arguments".to_string()));
                }
                obj_value.sort_array().map_err(|e| RuntimeError::TypeError(e))?;
                Ok(obj_value)
            }
            "first" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("first takes no arguments".to_string()));
                }
                obj_value.first().map_err(|e| RuntimeError::TypeError(e))
            }
            "last" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::TypeError("last takes no arguments".to_string()));
                }
                obj_value.last().map_err(|e| RuntimeError::TypeError(e))
            }
            "substring" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(RuntimeError::TypeError("substring requires 1 or 2 arguments".to_string()));
                }
                let start = self.evaluate_expression(arguments[0].clone())?;
                let start_idx = start.as_number().ok_or_else(|| {
                    RuntimeError::TypeError("substring start must be a number".to_string())
                })? as usize;

                let end_idx = if arguments.len() == 2 {
                    let end = self.evaluate_expression(arguments[1].clone())?;
                    Some(end.as_number().ok_or_else(|| {
                        RuntimeError::TypeError("substring end must be a number".to_string())
                    })? as usize)
                } else {
                    None
                };

                obj_value.substring(start_idx, end_idx).map_err(|e| RuntimeError::TypeError(e))
            }
            "indexof" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("indexOf requires 1 argument".to_string()));
                }
                let search = self.evaluate_expression(arguments[0].clone())?;
                let search_str = search.as_string().ok_or_else(|| {
                    RuntimeError::TypeError("indexOf argument must be a string".to_string())
                })?;
                obj_value.index_of(search_str).map_err(|e| RuntimeError::TypeError(e))
            }
            "startswith" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("startsWith requires 1 argument".to_string()));
                }
                let prefix = self.evaluate_expression(arguments[0].clone())?;
                let prefix_str = prefix.as_string().ok_or_else(|| {
                    RuntimeError::TypeError("startsWith argument must be a string".to_string())
                })?;
                obj_value.starts_with(prefix_str).map_err(|e| RuntimeError::TypeError(e))
            }
            "endswith" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("endsWith requires 1 argument".to_string()));
                }
                let suffix = self.evaluate_expression(arguments[0].clone())?;
                let suffix_str = suffix.as_string().ok_or_else(|| {
                    RuntimeError::TypeError("endsWith argument must be a string".to_string())
                })?;
                obj_value.ends_with(suffix_str).map_err(|e| RuntimeError::TypeError(e))
            }
            "charat" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("charAt requires 1 argument".to_string()));
                }
                let idx = self.evaluate_expression(arguments[0].clone())?;
                let index = idx.as_number().ok_or_else(|| {
                    RuntimeError::TypeError("charAt index must be a number".to_string())
                })? as usize;
                obj_value.char_at(index).map_err(|e| RuntimeError::TypeError(e))
            }
            "concat" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::TypeError("concat requires 1 argument".to_string()));
                }
                let other = self.evaluate_expression(arguments[0].clone())?;
                let other_str = other.as_string().ok_or_else(|| {
                    RuntimeError::TypeError("concat argument must be a string".to_string())
                })?;
                obj_value.concat_string(other_str).map_err(|e| RuntimeError::TypeError(e))
            }
            _ => Err(RuntimeError::TypeError(format!("Unknown method: {}", method))),
        }
    }

    fn execute_try_catch(
        &mut self,
        try_block: Vec<Statement>,
        catch_block: Vec<Statement>,
        finally_block: Option<Vec<Statement>>,
    ) -> Result<(), RuntimeError> {
        // Execute try block
        let try_result: Result<(), RuntimeError> = (|| {
            for stmt in try_block {
                self.execute_statement(stmt)?;
            }
            Ok(())
        })();

        // If error occurred, execute catch block
        if try_result.is_err() {
            for stmt in catch_block {
                self.execute_statement(stmt)?;
            }
        }

        // Always execute finally block if present
        if let Some(finally) = finally_block {
            for stmt in finally {
                self.execute_statement(stmt)?;
            }
        }

        Ok(())
    }

    fn execute_signup(&mut self, username: String, email: String, password: String) -> Result<(), RuntimeError> {
        // Get or create the auth system bound to the current database
        self.ensure_auth_system()?;

        let auth = self.context.get_auth_system_mut()
            .ok_or_else(|| RuntimeError::TypeError("Auth system not initialized".to_string()))?;

        let user = auth.signup(&username, &email, &password)
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        println!("{} Signed up user: {} ({})", "✓".green(), user.username.cyan(), user.email.yellow());

        // Store in variables
        self.context.set_variable("current_user".to_string(), Value::String(user.username.clone()));
        self.context.set_variable("user_email".to_string(), Value::String(user.email));

        Ok(())
    }

    fn execute_login(&mut self, username: String, password: String) -> Result<(), RuntimeError> {
        // Get or create the auth system bound to the current database
        self.ensure_auth_system()?;

        let auth = self.context.get_auth_system_mut()
            .ok_or_else(|| RuntimeError::TypeError("Auth system not initialized".to_string()))?;

        let session = auth.login(&username, &password)
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        println!("{} Logged in user: {}", "✓".green(), username.cyan());

        // Store in variables
        self.context.set_variable("current_user".to_string(), Value::String(username));
        self.context.set_variable("logged_in".to_string(), Value::Boolean(true));
        self.context.set_variable("session_token".to_string(), Value::String(session.token));

        Ok(())
    }

    fn execute_logout(&mut self) -> Result<(), RuntimeError> {
        let token = match self.context.get_variable("session_token") {
            Some(Value::String(t)) => t.clone(),
            _ => {
                println!("  {} Not logged in", "→".bright_black());
                return Ok(());
            }
        };

        if let Some(auth) = self.context.get_auth_system_mut() {
            auth.logout(&token).map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
        }

        println!("✓ Logged out");

        // Clear user variables
        self.context.set_variable("current_user".to_string(), Value::Null);
        self.context.set_variable("logged_in".to_string(), Value::Boolean(false));
        self.context.set_variable("session_token".to_string(), Value::Null);

        Ok(())
    }

    fn ensure_auth_system(&mut self) -> Result<(), RuntimeError> {
        if self.context.get_auth_system().is_some() {
            return Ok(());
        }

        // Auth is bound to the current database for persistence; fall back to
        // a default "users" database when no database has been selected yet.
        let db_name = self.context.current_database()
            .unwrap_or_else(|| "users".to_string());

        // Open a dedicated connection to the same database file so the auth
        // system can coordinate with interpreter CRUD operations.
        let db = engcode_stdlib::database::create_database(&db_name)?;

        // Ensure a "users" collection exists
        let mut db = db;
        db.create_collection("users")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        let secret = std::env::var("ENGCODE_SECRET")
            .unwrap_or_else(|_| "engcode-default-secret".to_string());

        self.context.ensure_auth_system(db, secret);
        Ok(())
    }

    fn execute_add_data_route(
        &mut self,
        method: String,
        path: String,
        collection: String,
    ) -> Result<(), RuntimeError> {
        let db_name = self.context.current_database()
            .ok_or(RuntimeError::NoDatabaseContext)?;

        if let Some(server) = self.context.get_web_server_mut() {
            // Only open a connection once; route handlers query it per request.
            if !server.has_data_database() {
                let db = engcode_stdlib::database::create_database(&db_name)?;
                server.set_data_database(db);
            }
            server.add_data_route(&method, &path, &collection);
            println!("  {} Added data route {} {} (collection {})", "→".cyan(), method.yellow(), path.cyan(), collection.yellow());
        } else {
            return Err(RuntimeError::TypeError(
                "No web server created. Create a server first.".to_string(),
            ));
        }

        Ok(())
    }

    fn execute_test_block(&mut self, name: String, body: Vec<Statement>) -> Result<(), RuntimeError> {
        println!("\n{} {}", "Testing:".bold().cyan(), name.bold());

        let mut passed = 0;
        let mut failed = 0;

        for stmt in body {
            match self.execute_statement(stmt) {
                Ok(_) => passed += 1,
                Err(e) => {
                    failed += 1;
                    println!("  {} {}", "✗".red(), e);
                }
            }
        }

        if failed == 0 {
            println!("{} {} tests passed", "✓".green().bold(), passed);
        } else {
            println!("{} {} passed, {} failed", "!".yellow().bold(), passed, failed);
        }

        Ok(())
    }

    fn execute_assert(&mut self, condition: Expression, message: Option<String>) -> Result<(), RuntimeError> {
        let result = self.evaluate_expression(condition)?;

        match result {
            Value::Boolean(true) => {
                println!("  {} Assertion passed", "✓".green());
                Ok(())
            }
            Value::Boolean(false) => {
                let msg = message.unwrap_or_else(|| "Assertion failed".to_string());
                println!("  {} {}", "✗".red(), msg);
                Err(RuntimeError::AssertionFailed(msg))
            }
            _ => {
                Err(RuntimeError::TypeError("Assert condition must be boolean".to_string()))
            }
        }
    }

    fn execute_read_file(&mut self, path: Expression, into: String) -> Result<(), RuntimeError> {
        let path_val = self.evaluate_expression(path)?;
        let file_path = path_val.as_string().ok_or_else(|| {
            RuntimeError::TypeError("File path must be a string".to_string())
        })?;

        let content = std::fs::read_to_string(file_path).map_err(|e| {
            RuntimeError::UserError(format!("Failed to read file: {}", e))
        })?;

        self.context.set_variable(into.clone(), Value::String(content));
        println!("  {} Read file into {}", "→".bright_black(), into.cyan());
        Ok(())
    }

    fn execute_write_file(&mut self, path: Expression, content: Expression) -> Result<(), RuntimeError> {
        let path_val = self.evaluate_expression(path)?;
        let file_path = path_val.as_string().ok_or_else(|| {
            RuntimeError::TypeError("File path must be a string".to_string())
        })?;

        let content_val = self.evaluate_expression(content)?;
        let content_str = content_val.to_string();

        std::fs::write(file_path, content_str).map_err(|e| {
            RuntimeError::UserError(format!("Failed to write file: {}", e))
        })?;

        println!("  {} Wrote to file {}", "✓".green(), file_path.cyan());
        Ok(())
    }

    fn execute_append_file(&mut self, path: Expression, content: Expression) -> Result<(), RuntimeError> {
        let path_val = self.evaluate_expression(path)?;
        let file_path = path_val.as_string().ok_or_else(|| {
            RuntimeError::TypeError("File path must be a string".to_string())
        })?;

        let content_val = self.evaluate_expression(content)?;
        let content_str = content_val.to_string();

        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .map_err(|e| RuntimeError::UserError(format!("Failed to open file: {}", e)))?;

        file.write_all(content_str.as_bytes())
            .map_err(|e| RuntimeError::UserError(format!("Failed to append to file: {}", e)))?;

        println!("  {} Appended to file {}", "✓".green(), file_path.cyan());
        Ok(())
    }

    fn interpolate_string(&mut self, s: &str) -> Result<String, RuntimeError> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Check if next char exists and is not '{'
                if chars.peek() == Some(&'{') {
                    // Escaped brace: {{
                    result.push('{');
                    chars.next(); // consume second '{'
                } else {
                    // Start of interpolation
                    let mut var_name = String::new();
                    let mut found_close = false;

                    while let Some(inner_ch) = chars.next() {
                        if inner_ch == '}' {
                            found_close = true;
                            break;
                        }
                        var_name.push(inner_ch);
                    }

                    if !found_close {
                        return Err(RuntimeError::TypeError(
                            "Unclosed interpolation brace".to_string()
                        ));
                    }

                    // Get variable value
                    let value = self.context
                        .get_variable(&var_name.trim())
                        .cloned()
                        .ok_or_else(|| RuntimeError::VariableNotFound(var_name.trim().to_string()))?;

                    result.push_str(&value.to_string());
                }
            } else if ch == '}' && chars.peek() == Some(&'}') {
                // Escaped closing brace: }}
                result.push('}');
                chars.next(); // consume second '}'
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engcode_parser::{Statement, Program};

    #[test]
    fn test_show_statement() {
        let mut program = Program::new();
        program.add_statement(Statement::Show {
            message: Expression::String("Hello!".to_string()),
        });

        let mut interpreter = Interpreter::new();
        assert!(interpreter.execute(program).is_ok());
    }
}

