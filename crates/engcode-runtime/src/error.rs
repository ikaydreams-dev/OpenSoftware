use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("No database context found. Create a database first.")]
    NoDatabaseContext,

    #[error("Database '{0}' not found")]
    DatabaseNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Variable '{0}' not found")]
    VariableNotFound(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Break statement outside of loop")]
    BreakOutsideLoop,

    #[error("Continue statement outside of loop")]
    ContinueOutsideLoop,

    #[error("Return from function")]
    ReturnValue(crate::value::Value),

    #[error("Function '{0}' not found")]
    FunctionNotFound(String),

    #[error("Error: {0}")]
    UserError(String),

    #[error("Assertion failed: {0}")]
    AssertionFailed(String),

    #[error("Validation failed: {0:?}")]
    ValidationFailed(Vec<String>),

    #[error("Test failed: {0}")]
    TestFailed(String),
}

impl From<engcode_stdlib::database::DatabaseError> for RuntimeError {
    fn from(err: engcode_stdlib::database::DatabaseError) -> Self {
        RuntimeError::DatabaseError(err.to_string())
    }
}
