use super::{LineVertex, Primitive};
use crate::scene::{parse_hex_color, AnimatedValue, AxesElement, ExpressionContext};

pub struct AxesPrimitive {
    position: [f32; 3],
    length: f32,
    base_color_x: [f32; 4],
    base_color_y: [f32; 4],
    base_color_z: [f32; 4],
    opacity: AnimatedValue,
}

impl AxesPrimitive {
    pub fn from_element(element: &AxesElement) -> Self {
        let base_color_x = parse_hex_color(&element.colors.x).unwrap_or([1.0, 0.0, 0.0, 1.0]);
        let base_color_y = parse_hex_color(&element.colors.y).unwrap_or([0.0, 1.0, 0.0, 1.0]);
        let base_color_z = parse_hex_color(&element.colors.z).unwrap_or([0.0, 0.0, 1.0, 1.0]);

        Self {
            position: element.position,
            length: element.length,
            base_color_x,
            base_color_y,
            base_color_z,
            opacity: element.opacity.clone(),
        }
    }
}

impl Primitive for AxesPrimitive {
    fn vertices(&self, ctx: &ExpressionContext) -> Vec<LineVertex> {
        let mut vertices = Vec::new();

        // Evaluate opacity at render time and clamp to valid range
        let opacity = self.opacity.evaluate(ctx).clamp(0.0, 1.0);

        let [ox, oy, oz] = self.position;
        let l = self.length;

        // X axis (red)
        let cx = [
            self.base_color_x[0],
            self.base_color_x[1],
            self.base_color_x[2],
            self.base_color_x[3] * opacity,
        ];
        vertices.push(LineVertex::new([ox, oy, oz], cx));
        vertices.push(LineVertex::new([ox + l, oy, oz], cx));

        // Y axis (green)
        let cy = [
            self.base_color_y[0],
            self.base_color_y[1],
            self.base_color_y[2],
            self.base_color_y[3] * opacity,
        ];
        vertices.push(LineVertex::new([ox, oy, oz], cy));
        vertices.push(LineVertex::new([ox, oy + l, oz], cy));

        // Z axis (blue)
        let cz = [
            self.base_color_z[0],
            self.base_color_z[1],
            self.base_color_z[2],
            self.base_color_z[3] * opacity,
        ];
        vertices.push(LineVertex::new([ox, oy, oz], cz));
        vertices.push(LineVertex::new([ox, oy, oz + l], cz));

        // Arrow heads (small lines at the end of each axis)
        let arrow_size = l * 0.15;

        // X arrow
        vertices.push(LineVertex::new([ox + l, oy, oz], cx));
        vertices.push(LineVertex::new([ox + l - arrow_size, oy + arrow_size * 0.5, oz], cx));
        vertices.push(LineVertex::new([ox + l, oy, oz], cx));
        vertices.push(LineVertex::new([ox + l - arrow_size, oy - arrow_size * 0.5, oz], cx));

        // Y arrow
        vertices.push(LineVertex::new([ox, oy + l, oz], cy));
        vertices.push(LineVertex::new([ox + arrow_size * 0.5, oy + l - arrow_size, oz], cy));
        vertices.push(LineVertex::new([ox, oy + l, oz], cy));
        vertices.push(LineVertex::new([ox - arrow_size * 0.5, oy + l - arrow_size, oz], cy));

        // Z arrow
        vertices.push(LineVertex::new([ox, oy, oz + l], cz));
        vertices.push(LineVertex::new([ox, oy + arrow_size * 0.5, oz + l - arrow_size], cz));
        vertices.push(LineVertex::new([ox, oy, oz + l], cz));
        vertices.push(LineVertex::new([ox, oy - arrow_size * 0.5, oz + l - arrow_size], cz));

        vertices
    }
}
