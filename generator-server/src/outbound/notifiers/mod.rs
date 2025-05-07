use crate::domain::{models::Map, ports::CreatorNotifier};

#[derive(Debug, Clone)]
pub struct NullNotifier;

impl CreatorNotifier for NullNotifier {
    async fn map_created(&self, _: &Map) {}
}
