use generator_core::MapStyle;

use derive_more::From;
use thiserror::Error;
use uuid::Uuid;

/// A uniquely identifiable blog of blog blog.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Map {
    id: Uuid,
    columns: MapDimension,
    rows: MapDimension,
    style: MapStyle,
    data: Vec<u8>,
}

impl Map {
    pub fn new(
        id: Uuid,
        columns: MapDimension,
        rows: MapDimension,
        style: MapStyle,
        data: Vec<u8>,
    ) -> Self {
        Self {
            id,
            columns,
            rows,
            style,
            data,
        }
    }

    pub fn id(&self) -> &uuid::Uuid {
        &self.id
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapDimension(u32);

#[derive(Clone, Debug, Error)]
#[error("map dimension cannot be zero")]
pub struct MapDimensionCannotBeZeroError;

impl MapDimension {
    pub fn new(raw: u32) -> Result<Self, MapDimensionCannotBeZeroError> {
        if raw == 0 {
            return Err(MapDimensionCannotBeZeroError);
        }

        Ok(Self(raw))
    }
}

#[derive(Clone, Debug, Error)]
#[error("map style is not known")]
pub struct MapStyleNotKnownError;

/// The fields required by the domain to create an [Author].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct CreateMapRequest {
    columns: MapDimension,
    rows: MapDimension,
    style: MapStyle,
}

impl CreateMapRequest {
    pub fn new(columns: MapDimension, rows: MapDimension, style: MapStyle) -> Self {
        Self {
            columns,
            rows,
            style,
        }
    }

    pub fn columns(&self) -> MapDimension {
        self.columns
    }

    pub fn raw_columns(&self) -> u32 {
        self.columns.0
    }

    pub fn rows(&self) -> MapDimension {
        self.rows
    }

    pub fn raw_rows(&self) -> u32 {
        self.rows.0
    }

    pub fn style(&self) -> MapStyle {
        self.style
    }
}

#[derive(Debug, Error)]
pub enum CreateMapError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
    #[error("Failed to persist map: {0}")]
    FileSystemError(#[from] std::io::Error),
}
