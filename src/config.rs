use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use crate::database::DatabaseConfig;
use crate::localization::LocalizationConfig;
use crate::metrics::MetricsConfig;
use crate::telegram::TelegramConfig;


#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub metrics: Option<MetricsConfig>,
    pub telegram: TelegramConfig,
    pub localization: Option<LocalizationConfig>,
    pub database: DatabaseConfig,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        Ok(Config::builder()
            .add_source(File::with_name("config.toml").required(false))
            .add_source(File::with_name("/config/config.toml").required(false))
            .add_source(Environment::with_prefix("app").separator("_"))
            .build()?
            .try_deserialize()?)
    }
}