//! CMSIS-SVD v1.3.9: structures (serde) + custom XML parser.
//!
//! Architecture:
//! - `xml` — minimal XML parser (no external XML crates).
//! - `svd` — CMSIS-SVD model (derive Serialize/Deserialize).
//! - `convert` — strict conversion/validation from XML AST to the SVD model.

pub mod convert;
pub mod error;
pub mod hal;
pub mod pac;
pub mod svd;
pub mod xml;

pub use crate::error::{Error, Result};

/// Parse SVD XML (string) into a strictly validated model.
pub fn parse_svd(xml: &str) -> Result<svd::Device> {
    let doc = crate::xml::Document::parse(xml)?;
    crate::convert::device_from_document(&doc)
}

/// Parse SVD file into a strictly validated model.
pub fn parse_svd_file(path: &std::path::Path) -> Result<svd::Device> {
    let xml = std::fs::read_to_string(path)?;
    parse_svd(&xml)
}
