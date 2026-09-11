use engcode_parser::{Program, Statement};
use crate::symbol_table::{SymbolTable, SymbolType};
use crate::error::AnalyzerError;

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), AnalyzerError> {
        for statement in &program.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), AnalyzerError> {
        match stmt {
            Statement::CreateDatabase { name } => {
                // Check if database already exists
                if self.symbol_table.exists(name) {
                    return Err(AnalyzerError::DatabaseAlreadyExists(name.clone()));
                }
                self.symbol_table.define(name.clone(), SymbolType::Database);
                Ok(())
            }
            Statement::CreateCollections { database, names } => {
                // Verify database exists if specified
                if let Some(db_name) = database {
                    if !self.symbol_table.exists(db_name) {
                        return Err(AnalyzerError::DatabaseNotFound(db_name.clone()));
                    }
                }

                // Register collections
                for name in names {
                    self.symbol_table.define(name.clone(), SymbolType::Collection);
                }
                Ok(())
            }
            Statement::Show { .. } => {
                // No semantic checks needed for show
                Ok(())
            }
            Statement::Assignment { variable, .. } => {
                // Register variable in symbol table
                self.symbol_table.define(variable.clone(), SymbolType::Variable);
                Ok(())
            }
            Statement::SetCookie { .. } => Ok(()),
            Statement::Insert { .. } => Ok(()),
            Statement::InsertRaw { .. } => Ok(()),
            Statement::NavigateTo { .. } => Ok(()),
            Statement::GoBack => Ok(()),
            Statement::FetchData { variable, .. } => {
                self.symbol_table.define(variable.clone(), SymbolType::Variable);
                Ok(())
            }
            Statement::Select { .. } => Ok(()),
            Statement::Update { .. } => Ok(()),
            Statement::Delete { .. } => Ok(()),
            Statement::CreateServer { .. } => Ok(()),
            Statement::AddRoute { .. } => Ok(()),
            Statement::AddDataRoute { .. } => Ok(()),
            Statement::StartServer { .. } => Ok(()),
            Statement::AddHandler { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
                Ok(())
            }
            Statement::AddMiddleware { .. } => Ok(()),
            // HTML statements
            Statement::CreatePage { .. } => Ok(()),
            Statement::AddCss { .. } => Ok(()),
            Statement::CreateLayout { .. } => Ok(()),
            Statement::RenderLayout { .. } => Ok(()),
            Statement::AddUploadRoute { .. } => Ok(()),
            Statement::AddUIComponent { .. } => Ok(()),
            Statement::AddElement { .. } => Ok(()),
            Statement::AddButton { .. } => Ok(()),
            Statement::AddForm { .. } => Ok(()),
            Statement::AddInput { .. } => Ok(()),
            Statement::AddHeading { .. } => Ok(()),
            Statement::AddParagraph { .. } => Ok(()),
            Statement::AddLink { .. } => Ok(()),
            Statement::AddImage { .. } => Ok(()),
            Statement::SetStyle { .. } => Ok(()),
            Statement::RenderPage { .. } => Ok(()),
            // Control flow statements
            Statement::If { then_block, else_block, .. } => {
                for stmt in then_block {
                    self.analyze_statement(stmt)?;
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.analyze_statement(stmt)?;
                    }
                }
                Ok(())
            }
            Statement::While { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
                Ok(())
            }
            Statement::For { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
                Ok(())
            }
            Statement::ForEach { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
                Ok(())
            }
            Statement::Break => Ok(()),
            Statement::Continue => Ok(()),
            // Function statements
            Statement::FunctionDef { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
                Ok(())
            }
            Statement::FunctionCall { .. } => Ok(()),
            Statement::Return { .. } => Ok(()),
            // Error handling statements
            Statement::TryCatch { try_block, catch_block, finally_block } => {
                for stmt in try_block {
                    self.analyze_statement(stmt)?;
                }
                for stmt in catch_block {
                    self.analyze_statement(stmt)?;
                }
                if let Some(finally) = finally_block {
                    for stmt in finally {
                        self.analyze_statement(stmt)?;
                    }
                }
                Ok(())
            }
            Statement::Throw { .. } => Ok(()),
            Statement::Validate { .. } => Ok(()),
            // Authentication statements
            Statement::Signup { .. } => Ok(()),
            Statement::Login { .. } => Ok(()),
            Statement::Logout => Ok(()),
            // Testing statements
            Statement::TestBlock { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
                Ok(())
            }
            Statement::Assert { .. } => Ok(()),
            // File I/O statements
            Statement::ReadFile { .. } => Ok(()),
            Statement::WriteFile { .. } => Ok(()),
            Statement::AppendFile { .. } => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_database_detection() {
        let mut analyzer = SemanticAnalyzer::new();

        let mut program = Program::new();
        program.add_statement(Statement::CreateDatabase {
            name: "MyDB".to_string(),
        });
        program.add_statement(Statement::CreateDatabase {
            name: "MyDB".to_string(),
        });

        let result = analyzer.analyze(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_program() {
        let mut analyzer = SemanticAnalyzer::new();

        let mut program = Program::new();
        program.add_statement(Statement::CreateDatabase {
            name: "MyDB".to_string(),
        });
        program.add_statement(Statement::CreateCollections {
            database: Some("MyDB".to_string()),
            names: vec!["users".to_string()],
        });

        assert!(analyzer.analyze(&program).is_ok());
    }
}
