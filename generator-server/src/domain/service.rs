/*!
   Module `service` provides the canonical implementation of the [MapService] port.
*/

use generator_core::create_map;

use super::{
    models::{CreateMapError, CreateMapRequest, Map},
    ports::{CreatorNotifier, MapMetrics, MapRepository, MapService},
};

/// Canonical implementation of the [MapService] port, through which the map domain API is
/// consumed.
#[derive(Debug, Clone)]
pub struct Service<R, M, N>
where
    R: MapRepository,
    M: MapMetrics,
    N: CreatorNotifier,
{
    repository: R,
    metrics: M,
    notifier: N,
}

impl<R, M, N> Service<R, M, N>
where
    R: MapRepository,
    M: MapMetrics,
    N: CreatorNotifier,
{
    pub fn new(repo: R, metrics: M, notifier: N) -> Self {
        Self {
            repository: repo,
            metrics,
            notifier,
        }
    }
}

impl<R, M, N> MapService for Service<R, M, N>
where
    R: MapRepository,
    M: MapMetrics,
    N: CreatorNotifier,
{
    /// Create the [Map] specified in `req` and trigger notifications.
    ///
    /// # Errors
    ///
    /// - Propagates any [CreateMapError] returned by the [MapRepository].
    async fn create_map(&self, req: &CreateMapRequest) -> Result<Map, CreateMapError> {
        let map_data = create_map(req.raw_columns(), req.raw_rows(), req.style());

        let result = self.repository.persist_map(req, map_data).await;

        match result {
            Ok(ref created_map) => {
                self.metrics.record_map_creation_success().await;
                self.notifier.map_created(created_map).await;
            }
            Err(_) => self.metrics.record_map_creation_failure().await,
        }

        result
    }
}
