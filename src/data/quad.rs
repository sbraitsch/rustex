use super::vertex::Vertex;

pub fn expand_from(vertex: Vertex, rel_idx: u16) -> ([Vertex; 4], [u16; 6]) {
    let x = vertex.position[0];
    let y = vertex.position[1];
    let dim = 0.01;

    let vertices = [
        Vertex::new(x + dim / 2.0, y + dim / 2.0), // top right
        Vertex::new(x - dim / 2.0, y + dim / 2.0), // top left
        Vertex::new(x + dim / 2.0, y - dim / 2.0), // bottom right
        Vertex::new(x - dim / 2.0, y - dim / 2.0), // bottom left
    ];

    let indices: [u16; 6] = [
        rel_idx,
        rel_idx + 1,
        rel_idx + 3,
        rel_idx,
        rel_idx + 3,
        rel_idx + 2,
    ];

    (vertices, indices)
}
