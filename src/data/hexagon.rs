use super::vertex::Vertex;

pub fn generate_hexagon_vertices() -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::PI / 3.0;
        let x = 0.5 * angle.cos();
        let y = 0.5 * angle.sin();
        vertices.push(Vertex {
            position: [x, y, 0.0],
            color: [0.0, 0.0, 0.0],
        });
    }
    vertices.push(vertices[0]);
    //vertices.push(vertices[1]);

    vertices
}

pub const INDICES: &[u16] = &[0, 1, 2, 3, 4, 5, 0];
