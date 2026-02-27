// Error types for Axiom core
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AxiomError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Invalid atom index: {0}")]
    InvalidIndex(usize),

    #[error("Empty structure")]
    EmptyStructure,

    #[error("Bond computation error: {0}")]
    BondComputationError(String),

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Selection error: {0}")]
    SelectionError(String),
}

pub type Result<T> = std::result::Result<T, AxiomError>;
