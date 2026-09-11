use std::collections::HashMap;

pub struct HtmlPage {
    pub name: String,
    pub title: String,
    pub elements: Vec<HtmlElement>,
    pub styles: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub enum HtmlElement {
    Button {
        text: String,
        id: Option<String>,
        class: Option<String>,
        onclick: Option<String>,
    },
    Input {
        input_type: String,
        id: Option<String>,
        name: Option<String>,
        placeholder: Option<String>,
        value: Option<String>,
        required: bool,
    },
    Heading {
        level: u8, // 1-6
        text: String,
    },
    Paragraph {
        text: String,
    },
    Link {
        text: String,
        href: String,
    },
    Image {
        src: String,
        alt: String,
    },
    Div {
        id: Option<String>,
        class: Option<String>,
        children: Vec<HtmlElement>,
    },
    Form {
        action: Option<String>,
        method: Option<String>,
        children: Vec<HtmlElement>,
    },
}

impl HtmlPage {
    pub fn new(name: String, title: Option<String>) -> Self {
        Self {
            name: name.clone(),
            title: title.unwrap_or(name),
            elements: Vec::new(),
            styles: HashMap::new(),
        }
    }

    pub fn add_element(&mut self, element: HtmlElement) {
        self.elements.push(element);
    }

    pub fn set_style(&mut self, selector: String, property: String, value: String) {
        self.styles
            .entry(selector)
            .or_insert_with(HashMap::new)
            .insert(property, value);
    }

    pub fn render(&self) -> String {
        let mut html = String::new();

        // HTML boilerplate
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>{}</title>\n", self.title));

        // Styles
        if !self.styles.is_empty() {
            html.push_str("    <style>\n");
            for (selector, properties) in &self.styles {
                html.push_str(&format!("        {} {{\n", selector));
                for (prop, val) in properties {
                    html.push_str(&format!("            {}: {};\n", prop, val));
                }
                html.push_str("        }\n");
            }
            html.push_str("    </style>\n");
        }

        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Render all elements
        for element in &self.elements {
            html.push_str(&self.render_element(element, 1));
        }

        html.push_str("</body>\n");
        html.push_str("</html>");

        html
    }

    fn render_element(&self, element: &HtmlElement, indent_level: usize) -> String {
        let indent = "    ".repeat(indent_level);
        let mut html = String::new();

        match element {
            HtmlElement::Button { text, id, class, onclick } => {
                html.push_str(&indent);
                html.push_str("<button");
                if let Some(i) = id {
                    html.push_str(&format!(" id=\"{}\"", i));
                }
                if let Some(c) = class {
                    html.push_str(&format!(" class=\"{}\"", c));
                }
                if let Some(o) = onclick {
                    html.push_str(&format!(" onclick=\"{}\"", o));
                }
                html.push_str(&format!(">{}</button>\n", text));
            }
            HtmlElement::Input {
                input_type,
                id,
                name,
                placeholder,
                value,
                required,
            } => {
                html.push_str(&indent);
                html.push_str(&format!("<input type=\"{}\"", input_type));
                if let Some(i) = id {
                    html.push_str(&format!(" id=\"{}\"", i));
                }
                if let Some(n) = name {
                    html.push_str(&format!(" name=\"{}\"", n));
                }
                if let Some(p) = placeholder {
                    html.push_str(&format!(" placeholder=\"{}\"", p));
                }
                if let Some(v) = value {
                    html.push_str(&format!(" value=\"{}\"", v));
                }
                if *required {
                    html.push_str(" required");
                }
                html.push_str(">\n");
            }
            HtmlElement::Heading { level, text } => {
                html.push_str(&indent);
                html.push_str(&format!("<h{}>{}</h{}>\n", level, text, level));
            }
            HtmlElement::Paragraph { text } => {
                html.push_str(&indent);
                html.push_str(&format!("<p>{}</p>\n", text));
            }
            HtmlElement::Link { text, href } => {
                html.push_str(&indent);
                html.push_str(&format!("<a href=\"{}\">{}</a>\n", href, text));
            }
            HtmlElement::Image { src, alt } => {
                html.push_str(&indent);
                html.push_str(&format!("<img src=\"{}\" alt=\"{}\">\n", src, alt));
            }
            HtmlElement::Div { id, class, children } => {
                html.push_str(&indent);
                html.push_str("<div");
                if let Some(i) = id {
                    html.push_str(&format!(" id=\"{}\"", i));
                }
                if let Some(c) = class {
                    html.push_str(&format!(" class=\"{}\"", c));
                }
                html.push_str(">\n");
                for child in children {
                    html.push_str(&self.render_element(child, indent_level + 1));
                }
                html.push_str(&indent);
                html.push_str("</div>\n");
            }
            HtmlElement::Form { action, method, children } => {
                html.push_str(&indent);
                html.push_str("<form");
                if let Some(a) = action {
                    html.push_str(&format!(" action=\"{}\"", a));
                }
                if let Some(m) = method {
                    html.push_str(&format!(" method=\"{}\"", m));
                }
                html.push_str(">\n");
                for child in children {
                    html.push_str(&self.render_element(child, indent_level + 1));
                }
                html.push_str(&indent);
                html.push_str("</form>\n");
            }
        }

        html
    }

    pub fn save_to_file(&self, directory: &str) -> std::io::Result<String> {
        use std::fs;
        use std::path::Path;

        // Create directory if it doesn't exist
        fs::create_dir_all(directory)?;

        let filename = format!("{}/{}.html", directory, self.name);
        let html = self.render();

        fs::write(&filename, html)?;

        Ok(filename)
    }
}

// Helper function to create default styles
pub fn create_default_styles() -> HashMap<String, HashMap<String, String>> {
    let mut styles = HashMap::new();

    // Body styles
    let mut body_styles = HashMap::new();
    body_styles.insert("font-family".to_string(), "Arial, sans-serif".to_string());
    body_styles.insert("margin".to_string(), "20px".to_string());
    body_styles.insert("padding".to_string(), "0".to_string());
    styles.insert("body".to_string(), body_styles);

    // Button styles
    let mut button_styles = HashMap::new();
    button_styles.insert("background-color".to_string(), "#007bff".to_string());
    button_styles.insert("color".to_string(), "white".to_string());
    button_styles.insert("padding".to_string(), "10px 20px".to_string());
    button_styles.insert("border".to_string(), "none".to_string());
    button_styles.insert("border-radius".to_string(), "4px".to_string());
    button_styles.insert("cursor".to_string(), "pointer".to_string());
    button_styles.insert("font-size".to_string(), "16px".to_string());
    styles.insert("button".to_string(), button_styles);

    // Input styles
    let mut input_styles = HashMap::new();
    input_styles.insert("padding".to_string(), "10px".to_string());
    input_styles.insert("border".to_string(), "1px solid #ddd".to_string());
    input_styles.insert("border-radius".to_string(), "4px".to_string());
    input_styles.insert("font-size".to_string(), "14px".to_string());
    input_styles.insert("width".to_string(), "100%".to_string());
    input_styles.insert("box-sizing".to_string(), "border-box".to_string());
    input_styles.insert("margin-bottom".to_string(), "10px".to_string());
    styles.insert("input".to_string(), input_styles);

    styles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_page() {
        let page = HtmlPage::new("home".to_string(), Some("Home Page".to_string()));
        assert_eq!(page.name, "home");
        assert_eq!(page.title, "Home Page");
    }

    #[test]
    fn test_add_button() {
        let mut page = HtmlPage::new("test".to_string(), None);
        page.add_element(HtmlElement::Button {
            text: "Click Me".to_string(),
            id: None,
            class: None,
            onclick: None,
        });
        assert_eq!(page.elements.len(), 1);
    }

    #[test]
    fn test_render_simple_page() {
        let mut page = HtmlPage::new("test".to_string(), Some("Test Page".to_string()));
        page.add_element(HtmlElement::Heading {
            level: 1,
            text: "Welcome".to_string(),
        });
        page.add_element(HtmlElement::Paragraph {
            text: "Hello, World!".to_string(),
        });

        let html = page.render();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Page</title>"));
        assert!(html.contains("<h1>Welcome</h1>"));
        assert!(html.contains("<p>Hello, World!</p>"));
    }

    #[test]
    fn test_render_button() {
        let mut page = HtmlPage::new("test".to_string(), None);
        page.add_element(HtmlElement::Button {
            text: "Submit".to_string(),
            id: Some("submit-btn".to_string()),
            class: Some("btn-primary".to_string()),
            onclick: Some("handleSubmit()".to_string()),
        });

        let html = page.render();
        assert!(html.contains("<button"));
        assert!(html.contains("id=\"submit-btn\""));
        assert!(html.contains("class=\"btn-primary\""));
        assert!(html.contains("onclick=\"handleSubmit()\""));
        assert!(html.contains(">Submit</button>"));
    }
}
