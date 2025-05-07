/*
   Module `ports` specifies the API by which external modules interact with the map domain.

   All traits are bounded by `Send + Sync + 'static`, since their implementations must be shareable
   between request-handling threads.

   Trait methods are explicitly asynchronous, including `Send` bounds on response types,
   since the application is expected to always run in a multithreaded environment.
*/

use std::future::Future;

use crate::domain::models::*;

/// `MapService` is the public API for the map domain.
///
/// External modules must conform to this contract – the domain is not concerned with the
/// implementation details or underlying technology of any external code.
pub trait MapService: Clone + Send + Sync + 'static {
    /// Asynchronously create a new [Map].
    ///
    /// # Errors
    ///
    /// - [CreateMapError::InvalidColumnSize] if a [Map] has an invalid column size.
    /// - [CreateMapError::InvalidRowSize] if a [Map] has an invalid row size.
    fn create_map(
        &self,
        req: &CreateMapRequest,
    ) -> impl Future<Output = Result<Map, CreateMapError>> + Send;
}

/// `MapRepository` represents a store of the created maps.
///
/// External modules must conform to this contract – the domain is not concerned with the
/// implementation details or underlying technology of any external code.
pub trait MapRepository: Send + Sync + Clone + 'static {
    /// Asynchronously persist a new [Map].
    fn persist_map(
        &self,
        req: &CreateMapRequest,
        data: svg::Document,
    ) -> impl Future<Output = Result<Map, CreateMapError>> + Send;
}

/// `MapMetrics` describes an aggregator of map creation related metrics, such as a time-series
/// database.
pub trait MapMetrics: Send + Sync + Clone + 'static {
    /// Record a successful author creation.
    fn record_map_creation_success(&self) -> impl Future<Output = ()> + Send;

    /// Record an author creation failure.
    fn record_map_creation_failure(&self) -> impl Future<Output = ()> + Send;
}

/// `CreatorNotifier` triggers notifications to map creators.
pub trait CreatorNotifier: Send + Sync + Clone + 'static {
    fn map_created(&self, map: &Map) -> impl Future<Output = ()> + Send;
}
