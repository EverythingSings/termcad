use super::{LineVertex, Primitive};
use crate::scene::{parse_hex_color, AxesElement, ExpressionContext};

pub struct AxesPrimitive {
    position: [f32; 3],
    length: f32,
    color_x: [f32; 4],
    color_y: [f32; 4],
    color_z: [f32; 4],
    opacity: f32,
}

impl AxesPrimitive {
    pub fn from_element(element: &AxesElement) -> Self {
        let color_x = parse_hex_color(&element.colors.x).unwrap_or([1.0, 0.0, 0.0, 1.0]);
        let color_y = parse_hex_color(&element.colors.y).unwrap_or([0.0, 1.0, 0.0, 1.0]);
        let color_z = parse_hex_color(&element.colors.z).unwrap_or([0.0, 0.0, 1.0, 1.0]);

        Self {
            position: element.position,
            length: element.length,
            color_x,
            color_y,
            color_z,
            opacity: element.opacity,
        }
    }
}

impl Primitive for AxesPrimitive {
    fn vertices(&self, _ctx: &ExpressionContext) -> Vec<LineVertex> {
        let mut vertices = Vec::new();

        let [ox, oy, oz] = self.position;
        let l = self.length;

        // X axis (red)
        let mut cx = self.color_x;
        cx[3] *= self.opacity;
        vertices.push(LineVertex::new([ox, oy, oz], cx));
        vertices.push(LineVertex::new([ox + l, oy, oz], cx));

        // Y axis (green)
        let mut cy = self.color_y;
        cy[3] *= self.opacity;
        vertices.push(LineVertex::new([ox, oy, oz], cy));
        vertices.push(LineVertex::new([ox, oy + l, oz], cy));

        // Z axis (blue)
        let mut cz = self.color_z;
        cz[3] *= self.opacity;
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
