use crate::domain::ports::MapMetrics;

#[derive(Debug, Clone)]
pub struct NullMetrics;

impl MapMetrics for NullMetrics {
    async fn record_map_creation_success(&self) {}

    async fn record_map_creation_failure(&self) {}
}
