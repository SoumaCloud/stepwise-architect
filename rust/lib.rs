pub mod advice;
pub mod costs;
pub mod store;

use serde_json::Value;
use std::io::Read;
use std::path::Path;

pub type Result<T> = std::result::Result<T, String>;
pub const MAX_JSON_BYTES: u64 = 8 * 1024 * 1024;

pub fn read_json(path: &Path) -> Result<Value> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_reader(file)
}

pub fn parse_reader(reader: impl Read) -> Result<Value> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_JSON_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_JSON_BYTES {
        return Err("JSON exceeds the 8 MiB input limit.".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| format!("Invalid JSON: {e}"))
}

pub fn text<'a>(value: &'a Value, key: &str, max: usize) -> Result<&'a str> {
    let s = value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string."))?;
    if s.trim().is_empty() || s.chars().count() > max {
        return Err(format!("{key} must contain 1 to {max} characters."));
    }
    Ok(s)
}
