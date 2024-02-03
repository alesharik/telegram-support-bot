mod file;
mod bundle;
mod common;
mod config;

pub use bundle::LocalizationBundle;
pub use file::{ParseError, FileContents, Entry};
pub use common::CommonMessages;
pub use config::{LocalizationConfig, from_config};

pub trait LocKey {
    fn key(&self) -> String;

    fn default_message(&self) -> String;

    fn args(self) -> Option<Vec<(String, String)>>;
}