use crate::domain::{
    models::{CreateMapError, CreateMapRequest, Map},
    ports::MapRepository,
};

use std::{fs::create_dir as create_generated_dir, path::Path};
use svg::save as save_as_svg;
use tracing::event;

#[derive(Debug, Clone)]
pub struct FileSystemRepository;

impl MapRepository for FileSystemRepository {
    async fn persist_map(
        &self,
        req: &CreateMapRequest,
        svg: svg::Document,
    ) -> Result<Map, CreateMapError> {
        let id = uuid::Uuid::new_v4();

        let svg_bytes = svg.to_string().into_bytes();
        let map = Map::new(id, req.columns(), req.rows(), req.style(), svg_bytes);

        let map_filename = {
            use std::time::SystemTime;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            format!("generated/{}-{:?}-map.svg", id, now)
        };

        match Path::new("generated").try_exists() {
            Ok(false) => match create_generated_dir("generated") {
                Ok(_) => {
                    event!(tracing::Level::INFO, "Directory 'generated' created.");
                    Ok(())
                }
                Err(e) => {
                    event!(
                        tracing::Level::ERROR,
                        "Error creating 'generated' directory: {}",
                        e
                    );
                    Err(CreateMapError::FileSystemError(e))
                }
            },
            Err(e) => Err(CreateMapError::FileSystemError(e)),
            _ => Ok(()),
        }?;

        event!(
            tracing::Level::INFO,
            "Saving map as SVG to: {}",
            map_filename
        );

        match save_as_svg(map_filename, &svg) {
            Ok(_) => {
                event!(tracing::Level::INFO, "Map saved successfully!");
            }
            Err(e) => {
                event!(tracing::Level::ERROR, "Error saving SVG file: {}", e);
                return Err(CreateMapError::FileSystemError(e));
            }
        }

        Ok(map)
    }
}
