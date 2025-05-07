use crate::types::{Cell, Edge, Room};

use std::collections::HashSet;

pub(crate) struct PolygonBuilder;

impl PolygonBuilder {
    pub fn build_for(room: &Room) -> (HashSet<Cell>, HashSet<Edge>) {
        let mut valid_vertices = HashSet::new();
        let mut valid_edges = HashSet::new();

        let mut edges_to_remove = HashSet::new();

        for cell in &room.cells {
            for vertex in cell.get_vertices() {
                valid_vertices.insert(vertex);
            }

            for edge in cell.get_edges() {
                if valid_edges.contains(&edge) {
                    edges_to_remove.insert(edge);
                }
                valid_edges.insert(edge);
            }
        }

        for edge in edges_to_remove.iter() {
            valid_edges.remove(edge);
        }

        // To know if a point should be kept, we need to check if it has 2 edges connecting to it
        let mut vertices_to_remove = Vec::new();

        for vertex in valid_vertices.iter() {
            let mut neighbour_count = 0;
            for neighbour in vertex.neighbours() {
                let neighbour_edge = Edge::new(*vertex, neighbour);

                if valid_edges.contains(&neighbour_edge) {
                    neighbour_count += 1;
                }
            }

            if neighbour_count != 2 {
                vertices_to_remove.push(*vertex);
            }
        }

        for vertex in vertices_to_remove.iter() {
            valid_vertices.remove(vertex);
        }

        (valid_vertices, valid_edges)
    }
}

#[cfg(test)]
mod test {
    use crate::types::Rect;

    use super::*;

    #[test]
    fn polygon_builder_works_for_simple_room() {
        let rect = Rect {
            origin: Cell { col: 0, row: 0 },
            width: 2,
            height: 2,
        };

        let room = Room::new_from_rect(rect);

        let (vertices, edges) = PolygonBuilder::build_for(&room);

        assert_eq!(vertices.len(), 8);
        assert_eq!(edges.len(), 8);

        let expected_vertices: HashSet<Cell> = vec![
            Cell { col: 0, row: 0 },
            Cell { col: 1, row: 0 },
            Cell { col: 2, row: 0 },
            Cell { col: 0, row: 1 },
            Cell { col: 0, row: 2 },
            Cell { col: 1, row: 2 },
            Cell { col: 2, row: 2 },
            Cell { col: 2, row: 1 },
        ]
        .into_iter()
        .collect();

        assert_eq!(vertices, expected_vertices);

        let expected_edges: HashSet<Edge> = vec![
            Edge::new(Cell { col: 0, row: 0 }, Cell { col: 1, row: 0 }),
            Edge::new(Cell { col: 1, row: 0 }, Cell { col: 2, row: 0 }),
            Edge::new(Cell { col: 0, row: 0 }, Cell { col: 0, row: 1 }),
            Edge::new(Cell { col: 0, row: 1 }, Cell { col: 0, row: 2 }),
            Edge::new(Cell { col: 0, row: 2 }, Cell { col: 1, row: 2 }),
            Edge::new(Cell { col: 1, row: 2 }, Cell { col: 2, row: 2 }),
            Edge::new(Cell { col: 2, row: 0 }, Cell { col: 2, row: 1 }),
            Edge::new(Cell { col: 2, row: 1 }, Cell { col: 2, row: 2 }),
        ]
        .into_iter()
        .collect();

        assert_eq!(edges, expected_edges);
    }

    #[test]
    fn polygon_builder_works_for_complex_room() {
        let rect_1 = Rect {
            origin: Cell { col: 0, row: 0 },
            width: 2,
            height: 1,
        };
        let rect_2 = Rect {
            origin: Cell { col: 2, row: 0 },
            width: 2,
            height: 2,
        };

        let room_1 = Room::new_from_rect(rect_1);
        let room_2 = Room::new_from_rect(rect_2);
        let room = room_1.merged_with(room_2);

        assert_eq!(room.cells.len(), 6);

        let (vertices, edges) = PolygonBuilder::build_for(&room);

        assert_eq!(vertices.len(), 12);
        assert_eq!(edges.len(), 12);

        let expected_vertices: HashSet<Cell> = vec![
            Cell { col: 0, row: 0 },
            Cell { col: 1, row: 0 },
            Cell { col: 2, row: 0 },
            Cell { col: 3, row: 0 },
            Cell { col: 4, row: 0 },
            Cell { col: 4, row: 1 },
            Cell { col: 4, row: 2 },
            Cell { col: 3, row: 2 },
            Cell { col: 2, row: 2 },
            Cell { col: 2, row: 1 },
            Cell { col: 1, row: 1 },
            Cell { col: 0, row: 1 },
        ]
        .into_iter()
        .collect();

        assert_eq!(vertices, expected_vertices);

        let expected_edges: HashSet<Edge> = vec![
            Edge::new(Cell { col: 0, row: 0 }, Cell { col: 1, row: 0 }),
            Edge::new(Cell { col: 1, row: 0 }, Cell { col: 2, row: 0 }),
            Edge::new(Cell { col: 2, row: 0 }, Cell { col: 3, row: 0 }),
            Edge::new(Cell { col: 3, row: 0 }, Cell { col: 4, row: 0 }),
            Edge::new(Cell { col: 4, row: 0 }, Cell { col: 4, row: 1 }),
            Edge::new(Cell { col: 4, row: 1 }, Cell { col: 4, row: 2 }),
            Edge::new(Cell { col: 4, row: 2 }, Cell { col: 3, row: 2 }),
            Edge::new(Cell { col: 3, row: 2 }, Cell { col: 2, row: 2 }),
            Edge::new(Cell { col: 2, row: 2 }, Cell { col: 2, row: 1 }),
            Edge::new(Cell { col: 2, row: 1 }, Cell { col: 1, row: 1 }),
            Edge::new(Cell { col: 1, row: 1 }, Cell { col: 0, row: 1 }),
            Edge::new(Cell { col: 0, row: 1 }, Cell { col: 0, row: 0 }),
        ]
        .into_iter()
        .collect();

        assert_eq!(edges, expected_edges);
    }
}
