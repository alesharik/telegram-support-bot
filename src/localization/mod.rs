mod file;
mod bundle;
mod common;
mod config;

use tracing::warn;
pub use bundle::LocalizationBundle;
pub use file::{ParseError, FileContents, Entry};
pub use common::CommonMessages;
pub use config::{LocalizationConfig, from_config};

pub trait LocKey {
    fn key(&self) -> String;

    fn default_message(&self) -> String;

    fn args(self) -> Option<Vec<(String, String)>>;
}

pub fn sanitize(s: String) -> String {
    match sanitize_html::sanitize_str(&sanitize_html::rules::predefined::DEFAULT, &s) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to sanitize string {}: {:?}", s, e);
            String::new()
        }
    }
}