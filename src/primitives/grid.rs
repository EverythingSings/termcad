use super::{LineVertex, Primitive};
use crate::scene::{parse_hex_color, AnimatedValue, ExpressionContext, GridElement};

pub struct GridPrimitive {
    pub divisions: u32,
    pub fade_distance: f32,
    pub base_color: [f32; 4],
    pub opacity: AnimatedValue,
}

impl GridPrimitive {
    pub fn from_element(element: &GridElement) -> Self {
        let base_color = parse_hex_color(&element.color).unwrap_or([0.0, 1.0, 0.25, 1.0]);

        Self {
            divisions: element.divisions,
            fade_distance: element.fade_distance,
            base_color,
            opacity: element.opacity.clone(),
        }
    }
}

impl Primitive for GridPrimitive {
    fn vertices(&self, ctx: &ExpressionContext) -> Vec<LineVertex> {
        let mut vertices = Vec::new();

        // Evaluate opacity at render time and clamp to valid range
        let base_opacity = self.opacity.evaluate(ctx).clamp(0.0, 1.0);

        let half_size = self.fade_distance / 2.0;
        let step = half_size * 2.0 / self.divisions as f32;

        // Generate grid lines along X axis
        for i in 0..=self.divisions {
            let z = -half_size + i as f32 * step;
            let fade_factor = 1.0 - (z.abs() / half_size).powf(2.0);
            let color = [
                self.base_color[0],
                self.base_color[1],
                self.base_color[2],
                base_opacity * fade_factor.max(0.0),
            ];

            vertices.push(LineVertex::new([-half_size, 0.0, z], color));
            vertices.push(LineVertex::new([half_size, 0.0, z], color));
        }

        // Generate grid lines along Z axis
        for i in 0..=self.divisions {
            let x = -half_size + i as f32 * step;
            let fade_factor = 1.0 - (x.abs() / half_size).powf(2.0);
            let color = [
                self.base_color[0],
                self.base_color[1],
                self.base_color[2],
                base_opacity * fade_factor.max(0.0),
            ];

            vertices.push(LineVertex::new([x, 0.0, -half_size], color));
            vertices.push(LineVertex::new([x, 0.0, half_size], color));
        }

        vertices
    }
}
