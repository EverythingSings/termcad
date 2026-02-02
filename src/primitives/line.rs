use super::{LineVertex, Primitive};
use crate::scene::{parse_hex_color, ExpressionContext, LineElement};

pub struct LinePrimitive {
    points: Vec<[f32; 3]>,
    closed: bool,
    color: [f32; 4],
}

impl LinePrimitive {
    pub fn from_element(element: &LineElement) -> Self {
        let mut color = parse_hex_color(&element.color).unwrap_or([0.0, 1.0, 0.25, 1.0]);
        color[3] = element.opacity;

        Self {
            points: element.points.clone(),
            closed: element.closed,
            color,
        }
    }
}

impl Primitive for LinePrimitive {
    fn vertices(&self, _ctx: &ExpressionContext) -> Vec<LineVertex> {
        let mut vertices = Vec::new();

        if self.points.len() < 2 {
            return vertices;
        }

        for i in 0..self.points.len() - 1 {
            vertices.push(LineVertex::new(self.points[i], self.color));
            vertices.push(LineVertex::new(self.points[i + 1], self.color));
        }

        if self.closed && self.points.len() > 2 {
            vertices.push(LineVertex::new(*self.points.last().unwrap(), self.color));
            vertices.push(LineVertex::new(self.points[0], self.color));
        }

        vertices
    }
}
