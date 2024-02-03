use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    WrongExtension,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "IO Error: {}", e),
            ParseError::Serde(e) => write!(f, "Serde error: {}", e),
            ParseError::WrongExtension => write!(f, "File extension should be .json"),
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        ParseError::Io(value)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(value: serde_json::Error) -> Self {
        ParseError::Serde(value)
    }
}

impl std::error::Error for ParseError {}

pub type FileContents = HashMap<String, Entry>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    #[serde(rename = "defaultMessage")]
    pub default_message: String,
    #[serde(rename = "description")]
    pub description: Option<String>,
}

async fn read(path: &Path) -> Result<FileContents, ParseError> {
    let contents = {
        let mut reader = File::open(path).await?;
        let mut str = String::new();
        reader.read_to_string(&mut str).await?;
        str
    };
    Ok(serde_json::from_str(&contents)?)
}

pub async fn parse(path: &Path) -> Result<(String, FileContents), ParseError> {
    let Some(ext) = path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .filter(|e| e == "json") else {
        return Err(ParseError::WrongExtension)
    };
    let name = path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_default()
        .trim_end_matches(&ext)
        .trim_end_matches(".")
        .to_string();
    let contents = read(path).await?;
    return Ok((name, contents))
}