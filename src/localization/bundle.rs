use std::collections::HashMap;
use std::path::Path;
use crate::localization::file::{FileContents, ParseError};
use crate::localization::LocKey;

pub struct LocalizationBundle {
    langs: HashMap<String, FileContents>,
    default_lang: Option<String>,
}

impl LocalizationBundle {
    pub fn new() -> Self {
        LocalizationBundle { langs: HashMap::new(), default_lang: None }
    }

    pub fn set_default_lang(&mut self, lang: String) {
        self.default_lang = Some(lang);
    }

    pub fn languages(&self) -> Vec<String> {
        self.langs.keys().map(|k| k.to_string()).collect()
    }

    pub fn add(&mut self, lang: impl Into<String>, contents: FileContents) {
        self.langs.insert(lang.into(), contents);
    }

    pub async fn add_file(&mut self, file: &Path) -> Result<(), ParseError> {
        let (lang, contents) = super::file::parse(file).await?;
        self.add(lang, contents);
        Ok(())
    }

    pub async fn scan_dir(&mut self, dir: impl AsRef<Path>) -> Result<(), ParseError> {
        let mut paths = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = paths.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                self.add_file(&entry.path()).await?;
            }
        }
        Ok(())
    }

    pub fn localize(&self, lang: Option<String>, key: impl LocKey) -> String {
        let k = key.key();
        let mut msg = lang
            .and_then(|lang| self.langs.get(&lang))
            .or_else(|| self.default_lang.clone().and_then(|d| self.langs.get(&d) ))
            .and_then(|l| l.get(&k))
            .map(|e| e.default_message.to_string())
            .unwrap_or_else(|| key.default_message());
        if let Some(args) = key.args() {
            for (k, v) in args {
                msg = msg.replace(&format!("{{{k}}}"), &v)
            }
        }
        msg
    }
}