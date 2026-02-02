use super::{generate_geometry, LineVertex, Primitive};
use crate::scene::{parse_hex_color, ExpressionContext, WireframeElement};

pub struct WireframePrimitive {
    element: WireframeElement,
    base_color: [f32; 4],
}

impl WireframePrimitive {
    pub fn from_element(element: &WireframeElement) -> Self {
        let base_color = parse_hex_color(&element.color).unwrap_or([0.0, 1.0, 0.25, 1.0]);

        Self {
            element: element.clone(),
            base_color,
        }
    }

    fn apply_transform(&self, point: [f32; 3], ctx: &ExpressionContext) -> [f32; 3] {
        let scale = self.element.scale.to_vec3();

        // Apply scale
        let mut p = [point[0] * scale[0], point[1] * scale[1], point[2] * scale[2]];

        // Evaluate rotation
        let rx = self.element.rotation.x.evaluate(ctx).to_radians();
        let ry = self.element.rotation.y.evaluate(ctx).to_radians();
        let rz = self.element.rotation.z.evaluate(ctx).to_radians();

        // Apply rotation (Y * X * Z order)
        p = rotate_y(p, ry);
        p = rotate_x(p, rx);
        p = rotate_z(p, rz);

        // Apply translation
        p[0] += self.element.position[0];
        p[1] += self.element.position[1];
        p[2] += self.element.position[2];

        p
    }
}

impl Primitive for WireframePrimitive {
    fn vertices(&self, ctx: &ExpressionContext) -> Vec<LineVertex> {
        let geometry = generate_geometry(&self.element.geometry);

        // Evaluate opacity at render time and clamp to valid range
        let opacity = self.element.opacity.evaluate(ctx).clamp(0.0, 1.0);
        let color = [
            self.base_color[0],
            self.base_color[1],
            self.base_color[2],
            opacity,
        ];

        let mut vertices = Vec::new();

        for (start_idx, end_idx) in geometry.edges {
            let start = self.apply_transform(geometry.vertices[start_idx], ctx);
            let end = self.apply_transform(geometry.vertices[end_idx], ctx);

            vertices.push(LineVertex::new(start, color));
            vertices.push(LineVertex::new(end, color));
        }

        vertices
    }
}

fn rotate_x(p: [f32; 3], angle: f32) -> [f32; 3] {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    [p[0], p[1] * cos_a - p[2] * sin_a, p[1] * sin_a + p[2] * cos_a]
}

fn rotate_y(p: [f32; 3], angle: f32) -> [f32; 3] {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    [p[0] * cos_a + p[2] * sin_a, p[1], -p[0] * sin_a + p[2] * cos_a]
}

fn rotate_z(p: [f32; 3], angle: f32) -> [f32; 3] {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    [p[0] * cos_a - p[1] * sin_a, p[0] * sin_a + p[1] * cos_a, p[2]]
}
