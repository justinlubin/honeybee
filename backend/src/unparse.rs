//! # Unparsing Honeybee core syntax
//!
//! This module provides serialization for expressions to JSON.

use crate::core::*;

/// Serialize an expression to JSON
pub fn exp(e: &Exp) -> Result<String, String> {
    serde_json::to_string(e).map_err(|e| e.to_string())
}
