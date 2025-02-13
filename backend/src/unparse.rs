use crate::core::*;

pub fn exp(e: &Exp) -> Result<String, String> {
    serde_json::to_string(e).map_err(|e| e.to_string())
}
