use colored::*;

pub struct ErrorDisplay {
    source_code: String,
    file_path: String,
}

impl ErrorDisplay {
    pub fn new(source_code: String, file_path: String) -> Self {
        Self {
            source_code,
            file_path,
        }
    }

    pub fn display_parse_error(&self, error: &str, line: Option<usize>) {
        eprintln!("\n{} {}", "✗".red().bold(), "Parse Error".red().bold());
        eprintln!("{} {}", "File:".bright_black(), self.file_path.yellow());

        if let Some(line_num) = line {
            eprintln!("{} {}", "Line:".bright_black(), line_num.to_string().cyan());
            self.show_code_context(line_num);
        }

        eprintln!("\n{} {}", "Error:".red().bold(), error);
        self.show_suggestion(error);
    }

    pub fn display_runtime_error(&self, error: &str, context: &str) {
        eprintln!("\n{} {}", "✗".red().bold(), "Runtime Error".red().bold());
        eprintln!("{} {}", "File:".bright_black(), self.file_path.yellow());
        eprintln!("\n{} {}", "Error:".red().bold(), error);

        if !context.is_empty() {
            eprintln!("\n{}", context.bright_black());
        }
    }

    pub fn display_semantic_error(&self, error: &str, line: Option<usize>) {
        eprintln!("\n{} {}", "✗".red().bold(), "Semantic Error".red().bold());
        eprintln!("{} {}", "File:".bright_black(), self.file_path.yellow());

        if let Some(line_num) = line {
            eprintln!("{} {}", "Line:".bright_black(), line_num.to_string().cyan());
            self.show_code_context(line_num);
        }

        eprintln!("\n{} {}", "Error:".red().bold(), error);
    }

    fn show_code_context(&self, line_num: usize) {
        let lines: Vec<&str> = self.source_code.lines().collect();

        if line_num == 0 || line_num > lines.len() {
            return;
        }

        let start = if line_num > 2 { line_num - 2 } else { 1 };
        let end = std::cmp::min(line_num + 2, lines.len());

        eprintln!("\n{}", "Code:".bright_black());

        for i in start..=end {
            let line_content = lines.get(i - 1).unwrap_or(&"");
            let line_indicator = format!("{:>4} │", i);

            if i == line_num {
                // Highlight the error line
                eprintln!(
                    "{} {}",
                    line_indicator.red().bold(),
                    line_content.white().bold()
                );
                // Add error pointer
                let spaces = " ".repeat(6 + line_content.len().saturating_sub(line_content.trim_start().len()));
                eprintln!("{}{}", spaces, "^".repeat(line_content.trim().len()).red().bold());
            } else {
                eprintln!(
                    "{} {}",
                    line_indicator.bright_black(),
                    line_content.bright_black()
                );
            }
        }
    }

    fn show_suggestion(&self, error: &str) {
        let suggestion = match error {
            e if e.contains("Expected database name") => {
                Some("Try: create a database called \"MyDB\"")
            }
            e if e.contains("Expected collection name") => {
                Some("Try: create these collections in it\n      users\n      posts")
            }
            e if e.contains("Unexpected token") => {
                Some("Make sure you're using valid English keywords like:\n      create, show, set, insert, select, update, delete")
            }
            e if e.contains("Expected variable name") => {
                Some("Try: set name to \"value\" or let age = 25")
            }
            e if e.contains("No database context") => {
                Some("You need to create a database first:\n      create a database called \"MyDB\"")
            }
            _ => None,
        };

        if let Some(hint) = suggestion {
            eprintln!("\n{}", "Suggestion:".green().bold());
            eprintln!("{}", hint.bright_white());
        }
    }
}

pub fn format_friendly_error(error: &str) -> String {
    match error {
        e if e.contains("Database") && e.contains("already exists") => {
            format!("That database already exists. Try using a different name or delete the existing one.")
        }
        e if e.contains("not found") => {
            format!("I couldn't find that. Make sure you've created it first.")
        }
        e if e.contains("Variable") && e.contains("not found") => {
            format!("That variable doesn't exist. Did you forget to set it?")
        }
        _ => error.to_string(),
    }
}
