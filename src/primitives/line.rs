use super::{LineVertex, Primitive};
use crate::scene::{parse_hex_color, AnimatedValue, ExpressionContext, LineElement};

pub struct LinePrimitive {
    points: Vec<[f32; 3]>,
    closed: bool,
    base_color: [f32; 4],
    opacity: AnimatedValue,
}

impl LinePrimitive {
    pub fn from_element(element: &LineElement) -> Self {
        let base_color = parse_hex_color(&element.color).unwrap_or([0.0, 1.0, 0.25, 1.0]);

        Self {
            points: element.points.clone(),
            closed: element.closed,
            base_color,
            opacity: element.opacity.clone(),
        }
    }
}

impl Primitive for LinePrimitive {
    fn vertices(&self, ctx: &ExpressionContext) -> Vec<LineVertex> {
        let mut vertices = Vec::new();

        if self.points.len() < 2 {
            return vertices;
        }

        // Evaluate opacity at render time and clamp to valid range
        let opacity = self.opacity.evaluate(ctx).clamp(0.0, 1.0);
        let color = [
            self.base_color[0],
            self.base_color[1],
            self.base_color[2],
            opacity,
        ];

        for i in 0..self.points.len() - 1 {
            vertices.push(LineVertex::new(self.points[i], color));
            vertices.push(LineVertex::new(self.points[i + 1], color));
        }

        if self.closed && self.points.len() > 2 {
            // Safe: points.len() > 2 guarantees last() returns Some
            if let Some(&last) = self.points.last() {
                vertices.push(LineVertex::new(last, color));
                vertices.push(LineVertex::new(self.points[0], color));
            }
        }

        vertices
    }
}
