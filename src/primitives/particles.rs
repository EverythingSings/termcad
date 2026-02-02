use super::{LineVertex, Primitive};
use crate::scene::{parse_hex_color, ExpressionContext, ParticlesElement};

pub struct ParticlesPrimitive {
    positions: Vec<[f32; 3]>,
    color: [f32; 4],
    size: f32,
    depth_fade: bool,
    bounds: [f32; 3],
}

impl ParticlesPrimitive {
    pub fn from_element(element: &ParticlesElement) -> Self {
        let mut color = parse_hex_color(&element.color).unwrap_or([0.0, 1.0, 0.25, 1.0]);
        color[3] = element.opacity;

        // Generate particle positions using a simple PRNG
        let mut positions = Vec::with_capacity(element.count as usize);
        let mut seed = if element.seed == 0 {
            12345u64
        } else {
            element.seed
        };

        for _ in 0..element.count {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let x = ((seed >> 16) as f32 / 65535.0 - 0.5) * element.bounds[0];

            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let y = ((seed >> 16) as f32 / 65535.0 - 0.5) * element.bounds[1];

            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let z = ((seed >> 16) as f32 / 65535.0 - 0.5) * element.bounds[2];

            positions.push([x, y, z]);
        }

        Self {
            positions,
            color,
            size: element.size,
            depth_fade: element.depth_fade,
            bounds: element.bounds,
        }
    }
}

impl Primitive for ParticlesPrimitive {
    fn vertices(&self, _ctx: &ExpressionContext) -> Vec<LineVertex> {
        let mut vertices = Vec::new();

        // Draw particles as small crosses
        let half_size = self.size * 0.02; // Scale down for world space

        for pos in &self.positions {
            let mut color = self.color;

            // Apply depth fade based on Z position
            if self.depth_fade {
                let max_z = self.bounds[2] / 2.0;
                let fade = 1.0 - (pos[2].abs() / max_z).min(1.0) * 0.7;
                color[3] *= fade;
            }

            // Horizontal line
            vertices.push(LineVertex::new(
                [pos[0] - half_size, pos[1], pos[2]],
                color,
            ));
            vertices.push(LineVertex::new(
                [pos[0] + half_size, pos[1], pos[2]],
                color,
            ));

            // Vertical line
            vertices.push(LineVertex::new(
                [pos[0], pos[1] - half_size, pos[2]],
                color,
            ));
            vertices.push(LineVertex::new(
                [pos[0], pos[1] + half_size, pos[2]],
                color,
            ));
        }

        vertices
    }
}
