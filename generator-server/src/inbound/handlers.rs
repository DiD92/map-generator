use super::api::{ApiError, ApiSuccess};
use crate::domain::models::{
    CreateMapRequest, Map, MapDimension, MapDimensionCannotBeZeroError, MapStyleNotKnownError,
};
use crate::domain::ports::MapService;
use crate::inbound::AppState;

use generator_core::MapStyle;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The response body data field for successful [Map] creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateMapResponseData {
    id: String,
}

impl From<&Map> for CreateMapResponseData {
    fn from(map: &Map) -> Self {
        Self {
            id: map.id().to_string(),
        }
    }
}

#[derive(Debug, Clone, Error)]
pub(super) enum ParseCreateMapHttpRequestError {
    #[error(transparent)]
    Dimensions(#[from] MapDimensionCannotBeZeroError),
    #[error(transparent)]
    Style(#[from] MapStyleNotKnownError),
}

/// The body of an [Map] creation request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateMapHttpRequestBody {
    columns: u32,
    rows: u32,
    style: String,
}

impl CreateMapHttpRequestBody {
    /// Converts the HTTP request body into a domain request.
    fn try_into_domain(self) -> Result<CreateMapRequest, ParseCreateMapHttpRequestError> {
        if self.columns == 0 || self.rows == 0 {
            return Err(ParseCreateMapHttpRequestError::Dimensions(
                MapDimensionCannotBeZeroError,
            ));
        }

        let style = MapStyle::try_from_str(&self.style).map_err(|_| MapStyleNotKnownError)?;

        let cols = MapDimension::new(self.columns)?;
        let rows = MapDimension::new(self.rows)?;

        Ok(CreateMapRequest::new(cols, rows, style))
    }
}

/// Create a new [Map].
///
/// # Responses
///
/// - 201 Created: the [Map] was successfully created.
/// - 422 Unprocessable entity: The [Map] creation request had invalid parameters.
pub(super) async fn create_map_handler<MS: MapService>(
    State(state): State<AppState<MS>>,
    Json(body): Json<CreateMapHttpRequestBody>,
) -> Result<ApiSuccess<CreateMapResponseData>, ApiError> {
    let domain_req = body.try_into_domain()?;
    state
        .map_service
        .create_map(&domain_req)
        .await
        .map_err(ApiError::from)
        .map(|ref map| ApiSuccess::new(StatusCode::CREATED, map.into()))
}
