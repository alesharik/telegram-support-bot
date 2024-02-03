use tracing_subscriber::EnvFilter;
use crate::config::Configuration;

mod metrics;
mod schema;
mod config;
mod telegram;
mod database;
mod localization;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().with_env_filter(EnvFilter::from_default_env()).init();
    let config = Configuration::new()?;
    metrics::install(&config.metrics)?;
    let bundle = localization::from_config(config.localization).await?;
    let db = database::connect(config.database).await?;
    telegram::run(config.telegram, db, bundle).await?;
    Ok(())
}