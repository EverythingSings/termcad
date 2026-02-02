use crate::scene::GeometryType;
use std::f32::consts::PI;

pub struct GeometryData {
    pub vertices: Vec<[f32; 3]>,
    pub edges: Vec<(usize, usize)>,
}

pub fn generate_geometry(geometry_type: &GeometryType) -> GeometryData {
    match geometry_type {
        GeometryType::Cube => generate_cube(),
        GeometryType::Sphere => generate_sphere(16, 12),
        GeometryType::Torus => generate_torus(24, 12, 1.0, 0.3),
        GeometryType::Ico => generate_icosahedron(),
        GeometryType::Cylinder => generate_cylinder(16, 1.0, 2.0),
    }
}

fn generate_cube() -> GeometryData {
    let s = 0.5;
    let vertices = vec![
        [-s, -s, -s],
        [s, -s, -s],
        [s, s, -s],
        [-s, s, -s],
        [-s, -s, s],
        [s, -s, s],
        [s, s, s],
        [-s, s, s],
    ];

    let edges = vec![
        // Bottom face
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        // Top face
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        // Vertical edges
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    GeometryData { vertices, edges }
}

fn generate_sphere(segments: usize, rings: usize) -> GeometryData {
    let mut vertices = Vec::new();
    let mut edges = Vec::new();

    // Generate vertices
    for ring in 0..=rings {
        let phi = PI * ring as f32 / rings as f32;
        for seg in 0..segments {
            let theta = 2.0 * PI * seg as f32 / segments as f32;

            let x = phi.sin() * theta.cos();
            let y = phi.cos();
            let z = phi.sin() * theta.sin();

            vertices.push([x * 0.5, y * 0.5, z * 0.5]);
        }
    }

    // Generate edges - horizontal rings
    for ring in 0..=rings {
        let base = ring * segments;
        for seg in 0..segments {
            let next = (seg + 1) % segments;
            edges.push((base + seg, base + next));
        }
    }

    // Generate edges - vertical lines
    for seg in 0..segments {
        for ring in 0..rings {
            let current = ring * segments + seg;
            let next = (ring + 1) * segments + seg;
            edges.push((current, next));
        }
    }

    GeometryData { vertices, edges }
}

fn generate_torus(
    tube_segments: usize,
    radial_segments: usize,
    major_radius: f32,
    minor_radius: f32,
) -> GeometryData {
    let mut vertices = Vec::new();
    let mut edges = Vec::new();

    // Generate vertices
    for radial in 0..radial_segments {
        let phi = 2.0 * PI * radial as f32 / radial_segments as f32;
        for tube in 0..tube_segments {
            let theta = 2.0 * PI * tube as f32 / tube_segments as f32;

            let x = (major_radius + minor_radius * theta.cos()) * phi.cos();
            let y = minor_radius * theta.sin();
            let z = (major_radius + minor_radius * theta.cos()) * phi.sin();

            vertices.push([x * 0.5, y * 0.5, z * 0.5]);
        }
    }

    // Generate edges
    for radial in 0..radial_segments {
        let next_radial = (radial + 1) % radial_segments;
        for tube in 0..tube_segments {
            let next_tube = (tube + 1) % tube_segments;

            let current = radial * tube_segments + tube;
            let tube_next = radial * tube_segments + next_tube;
            let radial_next = next_radial * tube_segments + tube;

            edges.push((current, tube_next));
            edges.push((current, radial_next));
        }
    }

    GeometryData { vertices, edges }
}

fn generate_icosahedron() -> GeometryData {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let s = 0.3; // Scale factor

    let vertices = vec![
        [-1.0 * s, phi * s, 0.0],
        [1.0 * s, phi * s, 0.0],
        [-1.0 * s, -phi * s, 0.0],
        [1.0 * s, -phi * s, 0.0],
        [0.0, -1.0 * s, phi * s],
        [0.0, 1.0 * s, phi * s],
        [0.0, -1.0 * s, -phi * s],
        [0.0, 1.0 * s, -phi * s],
        [phi * s, 0.0, -1.0 * s],
        [phi * s, 0.0, 1.0 * s],
        [-phi * s, 0.0, -1.0 * s],
        [-phi * s, 0.0, 1.0 * s],
    ];

    let edges = vec![
        (0, 1),
        (0, 5),
        (0, 7),
        (0, 10),
        (0, 11),
        (1, 5),
        (1, 7),
        (1, 8),
        (1, 9),
        (2, 3),
        (2, 4),
        (2, 6),
        (2, 10),
        (2, 11),
        (3, 4),
        (3, 6),
        (3, 8),
        (3, 9),
        (4, 5),
        (4, 9),
        (4, 11),
        (5, 9),
        (5, 11),
        (6, 7),
        (6, 8),
        (6, 10),
        (7, 8),
        (7, 10),
        (8, 9),
        (10, 11),
    ];

    GeometryData { vertices, edges }
}

fn generate_cylinder(segments: usize, radius: f32, height: f32) -> GeometryData {
    let mut vertices = Vec::new();
    let mut edges = Vec::new();

    let half_height = height * 0.25;
    let r = radius * 0.5;

    // Bottom circle
    for seg in 0..segments {
        let theta = 2.0 * PI * seg as f32 / segments as f32;
        vertices.push([r * theta.cos(), -half_height, r * theta.sin()]);
    }

    // Top circle
    for seg in 0..segments {
        let theta = 2.0 * PI * seg as f32 / segments as f32;
        vertices.push([r * theta.cos(), half_height, r * theta.sin()]);
    }

    // Bottom circle edges
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        edges.push((seg, next));
    }

    // Top circle edges
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        edges.push((segments + seg, segments + next));
    }

    // Vertical edges
    for seg in 0..segments {
        edges.push((seg, segments + seg));
    }

    GeometryData { vertices, edges }
}
