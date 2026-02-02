use super::{LineVertex, Primitive};
use crate::scene::{parse_hex_color, AnimatedValue, ExpressionContext, ParticlesElement};

pub struct ParticlesPrimitive {
    positions: Vec<[f32; 3]>,
    base_color: [f32; 4],
    opacity: AnimatedValue,
    size: f32,
    depth_fade: bool,
    bounds: [f32; 3],
}

impl ParticlesPrimitive {
    pub fn from_element(element: &ParticlesElement) -> Self {
        let base_color = parse_hex_color(&element.color).unwrap_or([0.0, 1.0, 0.25, 1.0]);

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
            base_color,
            opacity: element.opacity.clone(),
            size: element.size,
            depth_fade: element.depth_fade,
            bounds: element.bounds,
        }
    }
}

impl Primitive for ParticlesPrimitive {
    fn vertices(&self, ctx: &ExpressionContext) -> Vec<LineVertex> {
        let mut vertices = Vec::new();

        // Evaluate opacity at render time and clamp to valid range
        let base_opacity = self.opacity.evaluate(ctx).clamp(0.0, 1.0);

        // Draw particles as small crosses
        let half_size = self.size * 0.02; // Scale down for world space

        for pos in &self.positions {
            let mut opacity = base_opacity;

            // Apply depth fade based on Z position
            if self.depth_fade {
                let max_z = self.bounds[2] / 2.0;
                let fade = 1.0 - (pos[2].abs() / max_z).min(1.0) * 0.7;
                opacity *= fade;
            }

            let color = [
                self.base_color[0],
                self.base_color[1],
                self.base_color[2],
                opacity,
            ];

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
