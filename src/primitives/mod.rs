mod axes;
mod geometry;
mod glyph;
mod grid;
mod line;
mod particles;
mod wireframe;

pub use axes::AxesPrimitive;
pub use geometry::generate_geometry;
pub use glyph::GlyphPrimitive;
pub use grid::GridPrimitive;
pub use line::LinePrimitive;
pub use particles::ParticlesPrimitive;
pub use wireframe::WireframePrimitive;

use crate::scene::ExpressionContext;

pub trait Primitive {
    fn vertices(&self, ctx: &ExpressionContext) -> Vec<LineVertex>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl LineVertex {
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self { position, color }
    }
}
