pub mod react_native;
pub mod ios;
pub mod android;
pub mod transpiler;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MobileError {
    #[error("Transpilation error: {0}")]
    TranspilationError(String),

    #[error("iOS build error: {0}")]
    IosBuildError(String),

    #[error("Android build error: {0}")]
    AndroidBuildError(String),

    #[error("React Native error: {0}")]
    ReactNativeError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, MobileError>;
