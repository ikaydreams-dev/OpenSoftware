use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyzerError {
    #[error("Database '{0}' already exists")]
    DatabaseAlreadyExists(String),

    #[error("Database '{0}' not found")]
    DatabaseNotFound(String),

    #[error("Collection '{0}' already exists in database '{1}'")]
    CollectionAlreadyExists(String, String),

    #[error("Variable '{0}' already defined in this scope")]
    VariableAlreadyDefined(String),

    #[error("Variable '{0}' not defined")]
    VariableNotDefined(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
}
