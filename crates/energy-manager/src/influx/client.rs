use anyhow::Result;
use futures::stream;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::config::InfluxConfig;
use crate::types::{FieldValue, InfluxPoint};

pub struct InfluxWriter {
    client: Client,
    bucket: String,
    batch_size: usize,
    flush_interval: Duration,
    rx: mpsc::Receiver<InfluxPoint>,
    buffer: Vec<DataPoint>,
}

impl InfluxWriter {
    pub fn new(cfg: &InfluxConfig, rx: mpsc::Receiver<InfluxPoint>) -> Self {
        let client = Client::new(&cfg.url, &cfg.org, &cfg.token);
        Self {
            client,
            bucket: cfg.bucket.clone(),
            batch_size: cfg.batch_size,
            flush_interval: Duration::from_secs_f64(cfg.flush_interval_sec),
            rx,
            buffer: Vec::with_capacity(cfg.batch_size * 2),
        }
    }

    pub async fn run(mut self) {
        info!("InfluxDB writer started (bucket={}, batch={}, flush={:?})",
            self.bucket, self.batch_size, self.flush_interval);
        let mut ticker = interval(self.flush_interval);
        loop {
            tokio::select! {
                Some(pt) = self.rx.recv() => {
                    if let Some(dp) = to_data_point(&pt) {
                        self.buffer.push(dp);
                    }
                    if self.buffer.len() >= self.batch_size {
                        self.flush().await;
                    }
                }
                _ = ticker.tick() => {
                    if !self.buffer.is_empty() {
                        self.flush().await;
                    }
                }
                else => break,
            }
        }
        // Final flush
        if !self.buffer.is_empty() {
            self.flush().await;
        }
    }

    async fn flush(&mut self) {
        let points = std::mem::take(&mut self.buffer);
        debug!("InfluxDB flush: {} points", points.len());
        match self.client.write(&self.bucket, stream::iter(points)).await {
            Ok(_) => {}
            Err(e) => {
                warn!("InfluxDB write error: {e}");
                // Points are lost — acceptable for non-critical data.
                // A real implementation could buffer and retry.
            }
        }
    }
}

fn to_data_point(pt: &InfluxPoint) -> Option<DataPoint> {
    let ts_nanos = pt.timestamp.timestamp_nanos_opt()?;

    let mut builder = DataPoint::builder(&pt.measurement);
    for (k, v) in &pt.tags {
        builder = builder.tag(k, v);
    }
    for (k, v) in &pt.fields {
        builder = match v {
            FieldValue::Float(f) => builder.field(k.as_str(), *f),
            FieldValue::Int(i)   => builder.field(k.as_str(), *i),
            FieldValue::Str(s)   => builder.field(k.as_str(), s.as_str()),
            FieldValue::Bool(b)  => builder.field(k.as_str(), *b),
        };
    }
    builder.timestamp(ts_nanos).build().ok()
}

pub async fn spawn(cfg: InfluxConfig, rx: mpsc::Receiver<InfluxPoint>) -> Result<()> {
    if !cfg.enabled {
        info!("InfluxDB writer disabled — points will be dropped");
        tokio::spawn(async move {
            let mut rx = rx;
            while rx.recv().await.is_some() {}
        });
        return Ok(());
    }

    let writer = InfluxWriter::new(&cfg, rx);

    // Probe connectivity
    match writer.client.health().await {
        Ok(h) => info!("InfluxDB reachable: status={:?}", h.status),
        Err(e) => error!("InfluxDB not reachable: {e} — continuing anyway"),
    }

    tokio::spawn(async move { writer.run().await });
    Ok(())
}
