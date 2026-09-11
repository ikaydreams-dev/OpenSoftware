use crate::{MobileError, Result};
use engcode_parser::ast::{Statement, Expression};
use std::collections::HashMap;

pub struct MobileTranspiler {
    components: Vec<String>,
    imports: Vec<String>,
    state_variables: HashMap<String, String>,
    styles: Vec<String>,
}

impl MobileTranspiler {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            imports: vec![
                "import React, { useState, useEffect } from 'react';".to_string(),
                "import { View, Text, Button, TextInput, Image, StyleSheet, ScrollView, TouchableOpacity } from 'react-native';".to_string(),
            ],
            state_variables: HashMap::new(),
            styles: Vec::new(),
        }
    }

    pub fn transpile_to_react_native(&mut self, statements: &[Statement]) -> Result<String> {
        for stmt in statements {
            self.transpile_statement(stmt)?;
        }

        Ok(self.generate_react_native_code())
    }

    fn transpile_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::CreatePage { name, title, .. } => {
                self.transpile_page(name, title.as_deref())
            }
            Statement::AddButton { text, properties } => {
                self.add_button(text, properties)
            }
            Statement::AddHeading { level, text } => {
                self.add_heading(*level, text)
            }
            Statement::AddParagraph { text } => {
                self.add_paragraph(text)
            }
            Statement::AddImage { src, alt } => {
                self.add_image(src, alt)
            }
            Statement::AddInput { input_type, properties } => {
                self.add_input(input_type, properties)
            }
            Statement::Assignment { variable, value } => {
                self.add_state_variable(variable, value)
            }
            Statement::If { condition, then_block, else_block } => {
                self.transpile_if(condition, then_block, else_block.as_ref().map(|v| v.as_slice()))
            }
            Statement::While { condition, body } => {
                self.transpile_while(condition, body)
            }
            Statement::For { variable, start, end, body } => {
                self.transpile_for(variable, start, end, body)
            }
            Statement::ForEach { variable, collection, body } => {
                self.transpile_foreach(variable, collection, body)
            }
            _ => Ok(()), // Handle other statements as needed
        }
    }

    fn transpile_page(&mut self, _name: &str, title: Option<&str>) -> Result<()> {
        if let Some(t) = title {
            self.components.push(format!(
                "      <Text style={{{{fontSize: 24, fontWeight: 'bold', marginBottom: 20}}}}>{}</Text>",
                t
            ));
        }
        Ok(())
    }

    fn add_button(&mut self, text: &str, _properties: &[(String, Expression)]) -> Result<()> {
        let button = format!(
            "      <TouchableOpacity style={{{{backgroundColor: '#007bff', padding: 15, borderRadius: 8, marginVertical: 10}}}}>\n        <Text style={{{{color: 'white', fontSize: 16, textAlign: 'center'}}}}{}</Text>\n      </TouchableOpacity>",
            text
        );
        self.components.push(button);
        Ok(())
    }

    fn add_heading(&mut self, level: u8, text: &str) -> Result<()> {
        let size = match level {
            1 => 32,
            2 => 28,
            3 => 24,
            4 => 20,
            5 => 18,
            _ => 16,
        };
        self.components.push(format!(
            "      <Text style={{{{fontSize: {}, fontWeight: 'bold', marginVertical: 10}}}}>{}</Text>",
            size, text
        ));
        Ok(())
    }

    fn add_paragraph(&mut self, text: &str) -> Result<()> {
        self.components.push(format!(
            "      <Text style={{{{fontSize: 16, marginVertical: 5}}}}>{}</Text>",
            text
        ));
        Ok(())
    }

    fn add_image(&mut self, src: &str, _alt: &str) -> Result<()> {
        self.components.push(format!(
            "      <Image source={{{{uri: '{}'}}}} style={{{{width: '100%', height: 200, resizeMode: 'cover', marginVertical: 10}}}} />",
            src
        ));
        Ok(())
    }

    fn add_input(&mut self, input_type: &str, properties: &[(String, Expression)]) -> Result<()> {
        let mut placeholder = String::from("Enter text");
        for (key, val) in properties {
            if key == "placeholder" {
                if let Expression::String(s) = val {
                    placeholder = s.clone();
                }
            }
        }

        let secure = input_type == "password";
        self.components.push(format!(
            "      <TextInput\n        placeholder=\"{}\"\n        secureTextEntry={{{}}}\n        style={{{{borderWidth: 1, borderColor: '#ddd', padding: 12, borderRadius: 8, marginVertical: 10, fontSize: 16}}}}\n      />",
            placeholder, secure
        ));
        Ok(())
    }

    fn add_state_variable(&mut self, variable: &str, _value: &Expression) -> Result<()> {
        self.state_variables.insert(variable.to_string(), "null".to_string());
        Ok(())
    }

    fn transpile_if(&mut self, _condition: &Expression, then_block: &[Statement], _else_block: Option<&[Statement]>) -> Result<()> {
        for stmt in then_block {
            self.transpile_statement(stmt)?;
        }
        Ok(())
    }

    fn transpile_while(&mut self, _condition: &Expression, body: &[Statement]) -> Result<()> {
        for stmt in body {
            self.transpile_statement(stmt)?;
        }
        Ok(())
    }

    fn transpile_for(&mut self, _variable: &str, _start: &Expression, _end: &Expression, body: &[Statement]) -> Result<()> {
        for stmt in body {
            self.transpile_statement(stmt)?;
        }
        Ok(())
    }

    fn transpile_foreach(&mut self, _variable: &str, _collection: &Expression, body: &[Statement]) -> Result<()> {
        for stmt in body {
            self.transpile_statement(stmt)?;
        }
        Ok(())
    }

    fn generate_react_native_code(&self) -> String {
        let mut code = String::new();

        // Add imports
        for import in &self.imports {
            code.push_str(import);
            code.push('\n');
        }
        code.push('\n');

        // Add component
        code.push_str("export default function App() {\n");

        // Add state hooks
        for (var, _) in &self.state_variables {
            code.push_str(&format!("  const [{}, set{}] = useState(null);\n",
                var,
                var.chars().next().unwrap().to_uppercase().to_string() + &var[1..]
            ));
        }

        code.push_str("\n  return (\n");
        code.push_str("    <ScrollView style={styles.container}>\n");

        // Add components
        for component in &self.components {
            code.push_str(component);
            code.push('\n');
        }

        code.push_str("    </ScrollView>\n");
        code.push_str("  );\n");
        code.push_str("}\n\n");

        // Add styles
        code.push_str("const styles = StyleSheet.create({\n");
        code.push_str("  container: {\n");
        code.push_str("    flex: 1,\n");
        code.push_str("    backgroundColor: '#fff',\n");
        code.push_str("    padding: 20,\n");
        code.push_str("  },\n");
        code.push_str("});\n");

        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engcode_parser::ast::*;

    #[test]
    fn test_transpile_simple_page() {
        let mut transpiler = MobileTranspiler::new();
        let statements = vec![
            Statement::CreatePage {
                name: "home".to_string(),
                title: Some("Welcome".to_string()),
                layout: None,
            },
            Statement::AddHeading {
                level: 1,
                text: "Hello World".to_string(),
            },
            Statement::AddButton {
                text: "Click Me".to_string(),
                properties: vec![],
            },
        ];

        let result = transpiler.transpile_to_react_native(&statements).unwrap();
        assert!(result.contains("import React"));
        assert!(result.contains("TouchableOpacity"));
        assert!(result.contains("Hello World"));
        assert!(result.contains("Click Me"));
    }
}
