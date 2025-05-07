use crate::domain::{
    models::{CreateMapError, CreateMapRequest, Map},
    ports::MapRepository,
};

#[derive(Debug, Clone)]
pub struct NullRepository;

impl MapRepository for NullRepository {
    async fn persist_map(
        &self,
        req: &CreateMapRequest,
        svg: svg::Document,
    ) -> Result<Map, CreateMapError> {
        let id = uuid::Uuid::new_v4();
        let svg_bytes = svg.to_string().into_bytes();

        let map = Map::new(id, req.columns(), req.rows(), req.style(), svg_bytes);

        Ok(map)
    }
}
