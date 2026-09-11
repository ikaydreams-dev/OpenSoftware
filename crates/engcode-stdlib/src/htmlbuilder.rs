use std::collections::HashMap;

pub struct HtmlPage {
    pub name: String,
    pub title: String,
    pub layout: Option<String>,
    pub css_framework: Option<String>,
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
    Placeholder {
        marker: String,
    },
    Toast {
        text: String,
    },
    Alert {
        text: String,
    },
    Spinner,
    Modal {
        title: String,
        content: String,
    },
    TabSet {
        tabs: Vec<(String, String)>, // (label, content)
    },
    Accordion {
        items: Vec<(String, String)>, // (title, content)
    },
    Container {
        children: Vec<HtmlElement>,
    },
    Grid {
        children: Vec<HtmlElement>,
    },
}

impl HtmlPage {
    pub fn new(name: String, title: Option<String>) -> Self {
        Self {
            name: name.clone(),
            title: title.unwrap_or(name),
            layout: None,
            css_framework: None,
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
        html.push_str(&format!("    <title>{}</title>\n", escape_html(&self.title)));

        // CSS framework CDN link
        if let Some(framework) = &self.css_framework {
            if let Some(cdn) = css_framework_cdn(framework) {
                html.push_str(&format!("    {}\n", cdn));
            }
        }

        // Styles
        html.push_str("    <style>\n");
        let base_css = [
            ".engc-container { max-width: 1200px; margin: 0 auto; padding: 0 16px; }",
            ".engc-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 16px; }",
            ".engc-row { display: flex; flex-wrap: wrap; gap: 16px; }",
            "@media (max-width: 768px) { .engc-row > * { flex: 1 1 100%; } }",
            ".engc-tabs { margin-bottom: 16px; }",
            ".engc-tab-buttons { display: flex; flex-wrap: wrap; gap: 4px; border-bottom: 2px solid #eee; }",
            ".engc-tab-button { background: #f5f5f5; border: 1px solid #ddd; border-bottom: none; padding: 8px 16px; cursor: pointer; border-radius: 4px 4px 0 0; }",
            ".engc-tab-button-active { background: #007bff; color: #fff; }",
            ".engc-tab-panel { padding: 16px; border: 1px solid #ddd; border-top: none; }",
            ".engc-accordion-item { border: 1px solid #eee; margin-bottom: 6px; border-radius: 4px; }",
            ".engc-accordion-title { width: 100%; text-align: left; background: #f9f9f9; padding: 12px 16px; border: none; cursor: pointer; font-size: 15px; }",
            ".engc-accordion-content { display: none; padding: 12px 16px; }",
            ".engc-accordion-item.engc-open .engc-accordion-content { display: block; }",
        ];
        for rule in base_css {
            html.push_str(&format!("        {}\n", rule));
        }
        for (selector, properties) in &self.styles {
            html.push_str(&format!("        {} {{\n", selector));
            for (prop, val) in properties {
                html.push_str(&format!("            {}: {};\n", prop, val));
            }
            html.push_str("        }\n");
        }
        html.push_str("    </style>\n");

        html.push_str("    <script>\n");
        html.push_str("      function showToast(text) { var t = document.getElementById('engc-toast'); if (t) { t.innerHTML = text; t.style.display = 'block'; setTimeout(function(){ t.style.display = 'none'; }, 3000); } }\n");
        html.push_str("      function showModal(id) { var m = document.getElementById(id); if (m) { m.style.display = 'flex'; } }\n");
        html.push_str("      function hideModal(id) { var m = document.getElementById(id); if (m) { m.style.display = 'none'; } }\n");
        html.push_str("      function showTab(containerId, idx) { var c = document.getElementById(containerId); if (!c) return; var ps = c.querySelectorAll('.engc-tab-panel'); var bs = c.querySelectorAll('.engc-tab-button'); for (var i = 0; i < ps.length; i++) { ps[i].style.display = (i === idx) ? 'block' : 'none'; } for (var j = 0; j < bs.length; j++) { if (j === idx) { bs[j].classList.add('engc-tab-button-active'); } else { bs[j].classList.remove('engc-tab-button-active'); } } }\n");
        html.push_str("      function toggleAccordion(btn) { var item = btn.parentElement; item.classList.toggle('engc-open'); }\n");
        html.push_str("    </script>\n");

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
            HtmlElement::Placeholder { marker } => {
                html.push_str(&indent);
                html.push_str(marker);
                html.push('\n');
            }
            HtmlElement::Toast { text } => {
                html.push_str(&indent);
                html.push_str("<div id=\"engc-toast\" style=\"display:none;position:fixed;bottom:20px;right:20px;background:#333;color:#fff;padding:12px 20px;border-radius:6px;z-index:9999;box-shadow:0 2px 10px rgba(0,0,0,0.3);\">");
                html.push_str(&escape_html(text));
                html.push_str("</div>\n");
            }
            HtmlElement::Alert { text } => {
                html.push_str(&indent);
                html.push_str("<div class=\"engc-alert\" style=\"background:#fff3cd;border:1px solid #ffe08a;color:#664d03;padding:12px 16px;border-radius:6px;margin-bottom:12px;\">");
                html.push_str(&escape_html(text));
                html.push_str("</div>\n");
            }
            HtmlElement::Spinner => {
                html.push_str(&indent);
                html.push_str("<div class=\"engc-spinner\" style=\"display:inline-block;width:24px;height:24px;border:3px solid rgba(0,0,0,0.1);border-top-color:#007bff;border-radius:50%;animation:spin 0.8s linear infinite;\"></div>\n");
            }
            HtmlElement::Modal { title, content } => {
                let id = format!("engc-modal-{}", html.len());
                html.push_str(&indent);
                html.push_str(&format!(
                    "<div id=\"{}\" style=\"display:none;position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.5);justify-content:center;align-items:center;z-index:10000;\">\n",
                    id
                ));
                html.push_str(&indent);
                html.push_str("  <div style=\"background:#fff;padding:24px;border-radius:8px;max-width:480px;width:90%;box-shadow:0 4px 20px rgba(0,0,0,0.2);\">\n");
                html.push_str(&indent);
                html.push_str(&format!("    <h3 style=\"margin-top:0;\">{}</h3>\n", escape_html(title)));
                html.push_str(&indent);
                html.push_str(&format!("    <p>{}</p>\n", escape_html(content)));
                html.push_str(&indent);
                html.push_str(&format!("    <button onclick=\"hideModal('{}')\" style=\"margin-top:12px;\">Close</button>\n", id));
                html.push_str(&indent);
                html.push_str("  </div>\n");
                html.push_str(&indent);
                html.push_str("</div>\n");
            }
            HtmlElement::TabSet { tabs } => {
                let id = format!("engc-tabs-{}", html.len());
                html.push_str(&indent);
                html.push_str(&format!("<div class=\"engc-tabs\" id=\"{}\">\n", id));
                html.push_str(&indent);
                html.push_str("  <div class=\"engc-tab-buttons\">\n");
                for (i, (label, _)) in tabs.iter().enumerate() {
                    let active = if i == 0 { " engc-tab-button-active" } else { "" };
                    html.push_str(&format!("    <button class=\"engc-tab-button{}\" onclick=\"showTab('{}', {})\">{}</button>\n", active, id, i, escape_html(label)));
                }
                html.push_str(&indent);
                html.push_str("  </div>\n");
                for (i, (_, content)) in tabs.iter().enumerate() {
                    let display = if i == 0 { "block" } else { "none" };
                    html.push_str(&format!("  <div class=\"engc-tab-panel\" style=\"display:{};\">{}</div>\n", display, escape_html(content)));
                }
                html.push_str(&indent);
                html.push_str("</div>\n");
            }
            HtmlElement::Accordion { items } => {
                html.push_str(&indent);
                html.push_str("<div class=\"engc-accordion\">\n");
                for (title, content) in items {
                    html.push_str(&indent);
                    html.push_str("  <div class=\"engc-accordion-item\">\n");
                    html.push_str(&format!("    <button class=\"engc-accordion-title\" onclick=\"toggleAccordion(this)\">{}</button>\n", escape_html(title)));
                    html.push_str(&format!("    <div class=\"engc-accordion-content\">{}</div>\n", escape_html(content)));
                    html.push_str(&indent);
                    html.push_str("  </div>\n");
                }
                html.push_str(&indent);
                html.push_str("</div>\n");
            }
            HtmlElement::Container { children } => {
                html.push_str(&indent);
                html.push_str("<div class=\"engc-container\">\n");
                for child in children {
                    html.push_str(&self.render_element(child, indent_level + 1));
                }
                html.push_str(&indent);
                html.push_str("</div>\n");
            }
            HtmlElement::Grid { children } => {
                html.push_str(&indent);
                html.push_str("<div class=\"engc-grid\">\n");
                for child in children {
                    html.push_str(&self.render_element(child, indent_level + 1));
                }
                html.push_str(&indent);
                html.push_str("</div>\n");
            }
        }

        html
    }

    pub fn save_to_file(&self, directory: &str) -> std::io::Result<String> {
        use std::fs;

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

    // Spinner animation
    let mut spinner_styles = HashMap::new();
    spinner_styles.insert("animation".to_string(), "engc-spin 0.8s linear infinite".to_string());
    styles.insert("@keyframes engc-spin".to_string(), spinner_styles.clone());
    styles.insert("@keyframes spin".to_string(), spinner_styles);

    styles
}

pub fn css_framework_cdn(framework: &str) -> Option<String> {
    match framework.to_lowercase().as_str() {
        "tailwind" | "tailwindcss" => Some("<script src=\"https://cdn.tailwindcss.com\"></script>".to_string()),
        "bootstrap" => Some("<link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css\">".to_string()),
        "bulma" => Some("<link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/bulma@1.0.2/css/bulma.min.css\">".to_string()),
        "foundation" => Some("<link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/foundation-sites@6.9.0/dist/css/foundation.min.css\">".to_string()),
        "watercss" | "water" => Some("<link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/water.css@2/out/water.css\">".to_string()),
        "sakura" | "sakuracss" => Some("<link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/sakura.css@1.5.0/css/sakura.css\">".to_string()),
        _ => None,
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
