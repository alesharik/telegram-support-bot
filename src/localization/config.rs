use serde::Deserialize;
use tracing::{info, instrument};
use crate::localization::{LocalizationBundle, ParseError};

#[derive(Deserialize, Debug, Default)]
pub struct LocalizationConfig {
    #[serde(default)]
    default_language: Option<String>,
    #[serde(default)]
    paths: Vec<String>,
}

#[instrument]
pub async fn from_config(cfg: Option<LocalizationConfig>) -> Result<LocalizationBundle, ParseError> {
    let mut bundle = LocalizationBundle::new();
    if let Some(cfg) = cfg {
        if let Some(dlang) = cfg.default_language {
            info!("Using default language {}", &dlang);
            bundle.set_default_lang(dlang);
        }
        for path in cfg.paths {
            info!("Scanning dir {} for localizations", &path);
            bundle.scan_dir(&path).await?;
        }
    }
    info!("Created localization bundle with languages: [{:?}]", bundle.languages());
    Ok(bundle)
}