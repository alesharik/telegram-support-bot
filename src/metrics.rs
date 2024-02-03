use std::net::SocketAddr;
use std::time::Duration;
use metrics_exporter_prometheus::PrometheusBuilder;
use anyhow::Result;
use metrics_util::MetricKindMask;
use serde::Deserialize;
use tracing::{debug, info};

#[derive(Deserialize, Debug)]
pub struct MetricsConfig {
    pub address: SocketAddr,
}

pub fn install(config: &Option<MetricsConfig>) -> Result<()> {
    let mut builder = PrometheusBuilder::new()
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(Duration::from_secs(10))
        );
    if let Some(ref metrics) = config {
        info!("Will create prometheus metrics endpoint at {}", &metrics.address);
        builder = builder.with_http_listener(metrics.address);
    }
    builder.install()?;

    debug!("Metrics set up");
    Ok(())
}