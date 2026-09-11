use crate::{MobileError, Result};
use engcode_parser::ast::{Expression, Statement};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Page {
    name: String,
    title: String,
    elements: Vec<String>,
    state_variables: Vec<String>,
    fetches: Vec<(String, String)>, // (url, variable)
    last_button_index: Option<usize>,
}

impl Page {
    fn new(name: &str, title: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            title: title.unwrap_or(name).to_string(),
            elements: Vec::new(),
            state_variables: Vec::new(),
            fetches: Vec::new(),
            last_button_index: None,
        }
    }
}

pub struct MobileTranspiler {
    pages: Vec<Page>,
    current_page: usize,
    imports: Vec<String>,
    styles: Vec<String>,
}

impl MobileTranspiler {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            current_page: 0,
            imports: vec![
                "import React, { useState, useEffect } from 'react';".to_string(),
                "import { View, Text, TextInput, Image, StyleSheet, ScrollView, TouchableOpacity, ActivityIndicator, Linking } from 'react-native';".to_string(),
                "import { NavigationContainer } from '@react-navigation/native';".to_string(),
                "import { createNativeStackNavigator } from '@react-navigation/native-stack';".to_string(),
            ],
            styles: Vec::new(),
        }
    }

    pub fn transpile_to_react_native(&mut self, statements: &[Statement]) -> Result<String> {
        for stmt in statements {
            self.transpile_statement(stmt)?;
        }

        Ok(self.generate_react_native_code())
    }

    fn current_page_mut(&mut self) -> Result<&mut Page> {
        if self.pages.is_empty() {
            return Err(MobileError::TranspilationError(
                "No page created. Use 'create a page called X' first.".to_string(),
            ));
        }
        Ok(&mut self.pages[self.current_page])
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
            Statement::AddLink { text, url } => {
                self.add_link(text, url)
            }
            Statement::Assignment { variable, value } => {
                self.add_state_variable(variable, value)
            }
            Statement::NavigateTo { page } => {
                self.attach_navigation(page)?;
                Ok(())
            }
            Statement::GoBack => {
                self.attach_go_back()?;
                Ok(())
            }
            Statement::FetchData { url, variable } => {
                self.add_fetch(url, variable)
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
            _ => Ok(()),
        }
    }

    fn transpile_page(&mut self, name: &str, title: Option<&str>) -> Result<()> {
        self.pages.push(Page::new(name, title));
        self.current_page = self.pages.len() - 1;
        Ok(())
    }

    fn attach_navigation(&mut self, page: &Expression) -> Result<()> {
        let target = match self.expression_to_string(page) {
            Some(name) => name,
            None => return Ok(()),
        };
        let page = self.current_page_mut()?;
        if let Some(idx) = page.last_button_index {
            self.replace_button_onpress(idx, &format!("navigation.navigate('{}')", target));
        }
        Ok(())
    }

    fn attach_go_back(&mut self) -> Result<()> {
        let page = self.current_page_mut()?;
        if let Some(idx) = page.last_button_index {
            self.replace_button_onpress(idx, "navigation.goBack()");
        }
        Ok(())
    }

    fn replace_button_onpress(&mut self, index: usize, handler: &str) {
        let original = self.pages[self.current_page].elements.get(index).cloned();
        if let Some(mut element) = original {
            if let Some(end) = element.find('>') {
                let end = end + 1;
                let (open, rest) = element.split_at(end);
                let inner = &open[..open.len() - 1];
                element = inner.to_string()
                    + " onPress={"
                    + &format!("() => {}", handler)
                    + "}>"
                    + rest;
                self.pages[self.current_page].elements[index] = element;
            }
        }
    }

    fn expression_to_string(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::String(s) => Some(s.clone()),
            Expression::Identifier(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn add_button(&mut self, text: &str, _properties: &[(String, Expression)]) -> Result<()> {
        let button = format!(
            "<TouchableOpacity style={{{{backgroundColor: '#007bff', padding: 15, borderRadius: 8, marginVertical: 10}}}}>\n        <Text style={{{{color: 'white', fontSize: 16, textAlign: 'center'}}}}>{}</Text>\n      </TouchableOpacity>",
            text
        );
        let page = self.current_page_mut()?;
        page.elements.push(button);
        page.last_button_index = Some(page.elements.len() - 1);
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
        let element = format!(
            "<Text style={{{{fontSize: {}, fontWeight: 'bold', marginVertical: 10}}}}>{}</Text>",
            size, text
        );
        let page = self.current_page_mut()?;
        page.elements.push(element);
        Ok(())
    }

    fn add_paragraph(&mut self, text: &str) -> Result<()> {
        let element = format!(
            "<Text style={{{{fontSize: 16, marginVertical: 5}}}}>{}</Text>",
            text
        );
        let page = self.current_page_mut()?;
        page.elements.push(element);
        Ok(())
    }

    fn add_link(&mut self, text: &str, href: &str) -> Result<()> {
        let element = format!(
            "<Text style={{{{color: '#007bff', marginVertical: 5, textDecorationLine: 'underline'}}}} onPress={{{{() => Linking.openURL('{}')}}}}>{}</Text>",
            href, text
        );
        let page = self.current_page_mut()?;
        page.elements.push(element);
        Ok(())
    }

    fn add_image(&mut self, src: &str, _alt: &str) -> Result<()> {
        let element = format!(
            "<Image source={{{{uri: '{}'}}}} style={{{{width: '100%', height: 200, resizeMode: 'cover', marginVertical: 10}}}} />",
            src
        );
        let page = self.current_page_mut()?;
        page.elements.push(element);
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
        let element = format!(
            "<TextInput\n        placeholder=\"{}\"\n        secureTextEntry={{{}}}\n        style={{{{borderWidth: 1, borderColor: '#ddd', padding: 12, borderRadius: 8, marginVertical: 10, fontSize: 16}}}}\n      />",
            placeholder, secure
        );
        let page = self.current_page_mut()?;
        page.elements.push(element);
        Ok(())
    }

    fn add_state_variable(&mut self, variable: &str, _value: &Expression) -> Result<()> {
        let page = self.current_page_mut()?;
        if !page.state_variables.iter().any(|v| v == variable) {
            page.state_variables.push(variable.to_string());
        }
        Ok(())
    }

    fn add_fetch(&mut self, url: &Expression, variable: &str) -> Result<()> {
        let url_str = match self.expression_to_string(url) {
            Some(url) => url,
            None => return Ok(()),
        };
        let page = self.current_page_mut()?;
        page.fetches.push((url_str, variable.to_string()));
        if !variable.is_empty() && !page.state_variables.iter().any(|v| v == variable) {
            page.state_variables.push(variable.to_string());
        }
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

        for import in &self.imports {
            code.push_str(import);
            code.push('\n');
        }
        code.push_str("\nconst Stack = createNativeStackNavigator();\n\n");

        // App shell with navigation container
        code.push_str("export default function App() {\n");
        code.push_str("  return (\n");
        code.push_str("    <NavigationContainer>\n");
        code.push_str("      <Stack.Navigator>\n");
        for page in &self.pages {
            let screen_name = screen_component(&page.name);
            code.push_str(&format!(
                "        <Stack.Screen name=\"{}\" component={{{}}} options={{{{ title: '{}' }}}} />\n",
                page.name, screen_name, escaped(&page.title)
            ));
        }
        code.push_str("      </Stack.Navigator>\n");
        code.push_str("    </NavigationContainer>\n");
        code.push_str("  );\n");
        code.push_str("}\n\n");

        // One screen component per page
        for page in &self.pages {
            let screen_name = screen_component(&page.name);
            code.push_str(&format!("function {}({{ navigation }}) {{\n", screen_name));

            // State hooks
            for var in &page.state_variables {
                code.push_str(&format!("  const [{}, set{}] = useState(null);\n", var, cap(&var)));
            }

            // Fetch effects
            for (url, var_name) in &page.fetches {
                code.push_str("  useEffect(() => {\n");
code.push_str(&format!(
                "    fetch('{}').then(r => r.json()).then(data => set{}(data)).catch(() => {{}});\n",
                url, cap(var_name)
            ));
                code.push_str("  }, []);\n");
            }

            code.push_str("\n  return (\n");
            code.push_str("    <ScrollView style={styles.container}>\n");
            for element in &page.elements {
                code.push_str("      ");
                code.push_str(element);
                code.push('\n');
            }
            code.push_str("    </ScrollView>\n");
            code.push_str("  );\n");
            code.push_str("}\n\n");
        }

        // Shared styles
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

fn screen_component(name: &str) -> String {
    let base = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    format!("{}Screen", cap(&base))
}

fn cap(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn escaped(text: &str) -> String {
    text.replace('\'', "\\'")
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
            Statement::NavigateTo {
                page: Expression::String("profile".to_string()),
            },
            Statement::CreatePage {
                name: "profile".to_string(),
                title: Some("Profile".to_string()),
                layout: None,
            },
            Statement::AddParagraph {
                text: "My profile".to_string(),
            },
        ];

        let result = transpiler.transpile_to_react_native(&statements).unwrap();
        assert!(result.contains("import React"));
        assert!(result.contains("NavigationContainer"));
        assert!(result.contains("createNativeStackNavigator"));
        assert!(result.contains("Hello World"));
        assert!(result.contains("navigation.navigate('profile')"));
        assert!(result.contains("<Stack.Screen name=\"home\""));
        assert!(result.contains("<Stack.Screen name=\"profile\""));
    }

    #[test]
    fn test_transpile_demo_file() {
        let source = std::fs::read_to_string(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/mobile-demo.eng"),
        )
        .expect("open examples/mobile-demo.eng");
        let tokens = engcode_lexer::Lexer::new(source).tokenize();
        let mut parser = engcode_parser::Parser::new(tokens);
        let program = parser.parse().expect("parse demo");
        let mut transpiler = MobileTranspiler::new();
        let code = transpiler
            .transpile_to_react_native(&program.statements)
            .expect("transpile demo");
        std::fs::write(
            std::env::temp_dir().join("engc-mobile-demo.txt"),
            &code,
        )
        .expect("write output");
        assert!(code.contains("Stack.Screen name=\"home\""));
        assert!(code.contains("Stack.Screen name=\"profile\""));
        assert!(code.contains("Stack.Screen name=\"settings\""));
        assert!(code.contains("navigation.navigate('home')"));
        assert!(code.contains("navigation.goBack()"));
        assert!(code.contains("fetch('https://api.example.com/user')"));
        assert!(code.contains("onPress={() => navigation.navigate('profile')}>"));
        assert!(code.contains("onPress={() => navigation.goBack()}>"));
    }
}