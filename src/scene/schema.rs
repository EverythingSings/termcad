use serde::{Deserialize, Serialize};

use super::validate::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub canvas: Canvas,
    #[serde(default)]
    pub camera: Camera,
    #[serde(default = "default_duration")]
    pub duration: f32,
    #[serde(default = "default_fps")]
    pub fps: u32,
    #[serde(default = "default_loop")]
    pub r#loop: bool,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub post: PostProcessing,
}

fn default_duration() -> f32 {
    2.0
}
fn default_fps() -> u32 {
    30
}
fn default_loop() -> bool {
    true
}

impl Scene {
    pub fn total_frames(&self) -> u32 {
        (self.duration * self.fps as f32).ceil() as u32
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        super::validate::validate_scene(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_background")]
    pub background: String,
}

fn default_width() -> u32 {
    800
}
fn default_height() -> u32 {
    600
}
fn default_background() -> String {
    "#0a0a0a".to_string()
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            background: default_background(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    #[serde(default = "default_camera_position")]
    pub position: [f32; 3],
    #[serde(default = "default_camera_target")]
    pub target: [f32; 3],
    #[serde(default = "default_fov")]
    pub fov: f32,
}

fn default_camera_position() -> [f32; 3] {
    [5.0, 5.0, 5.0]
}
fn default_camera_target() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}
fn default_fov() -> f32 {
    45.0
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: default_camera_position(),
            target: default_camera_target(),
            fov: default_fov(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    Grid(GridElement),
    Wireframe(WireframeElement),
    Glyph(GlyphElement),
    Line(LineElement),
    Particles(ParticlesElement),
    Axes(AxesElement),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridElement {
    #[serde(default = "default_grid_divisions")]
    pub divisions: u32,
    #[serde(default = "default_fade_distance")]
    pub fade_distance: f32,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_opacity")]
    pub opacity: AnimatedValue,
}

fn default_grid_divisions() -> u32 {
    20
}
fn default_fade_distance() -> f32 {
    50.0
}
fn default_color() -> String {
    "#00ff41".to_string()
}
fn default_opacity() -> AnimatedValue {
    AnimatedValue::Static(0.5)
}

impl Default for GridElement {
    fn default() -> Self {
        Self {
            divisions: default_grid_divisions(),
            fade_distance: default_fade_distance(),
            color: default_color(),
            opacity: AnimatedValue::Static(0.5),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireframeElement {
    #[serde(default = "default_geometry")]
    pub geometry: GeometryType,
    #[serde(default)]
    pub position: [f32; 3],
    #[serde(default)]
    pub rotation: AnimatedRotation,
    #[serde(default = "default_scale")]
    pub scale: Scale,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_thickness")]
    pub thickness: f32,
    #[serde(default = "default_full_opacity")]
    pub opacity: AnimatedValue,
}

fn default_geometry() -> GeometryType {
    GeometryType::Cube
}
fn default_scale() -> Scale {
    Scale::Uniform(1.0)
}
fn default_thickness() -> f32 {
    2.0
}
fn default_full_opacity() -> AnimatedValue {
    AnimatedValue::Static(1.0)
}

impl Default for WireframeElement {
    fn default() -> Self {
        Self {
            geometry: default_geometry(),
            position: [0.0, 0.0, 0.0],
            rotation: AnimatedRotation::default(),
            scale: default_scale(),
            color: default_color(),
            thickness: default_thickness(),
            opacity: AnimatedValue::Static(1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GeometryType {
    #[default]
    Cube,
    Sphere,
    Torus,
    Ico,
    Cylinder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Scale {
    Uniform(f32),
    NonUniform([f32; 3]),
}

impl Default for Scale {
    fn default() -> Self {
        Scale::Uniform(1.0)
    }
}

impl Scale {
    pub fn to_vec3(&self) -> [f32; 3] {
        match self {
            Scale::Uniform(s) => [*s, *s, *s],
            Scale::NonUniform(v) => *v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimatedRotation {
    #[serde(default)]
    pub x: AnimatedValue,
    #[serde(default)]
    pub y: AnimatedValue,
    #[serde(default)]
    pub z: AnimatedValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimatedValue {
    Static(f32),
    Expression(String),
}

impl Default for AnimatedValue {
    fn default() -> Self {
        AnimatedValue::Static(0.0)
    }
}

impl AnimatedValue {
    pub fn evaluate(&self, ctx: &super::ExpressionContext) -> f32 {
        match self {
            AnimatedValue::Static(v) => *v,
            AnimatedValue::Expression(expr) => super::evaluate_expression(expr, ctx).unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphElement {
    pub text: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default)]
    pub position: [f32; 3],
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub animation: GlyphAnimation,
    #[serde(default = "default_full_opacity")]
    pub opacity: AnimatedValue,
}

fn default_font_size() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GlyphAnimation {
    #[default]
    None,
    Type,
    Flicker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineElement {
    pub points: Vec<[f32; 3]>,
    #[serde(default)]
    pub closed: bool,
    #[serde(default = "default_thickness")]
    pub thickness: f32,
    #[serde(default = "default_glow")]
    pub glow: f32,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_full_opacity")]
    pub opacity: AnimatedValue,
}

fn default_glow() -> f32 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticlesElement {
    #[serde(default = "default_particle_count")]
    pub count: u32,
    #[serde(default = "default_bounds")]
    pub bounds: [f32; 3],
    #[serde(default = "default_particle_size")]
    pub size: f32,
    #[serde(default = "default_depth_fade")]
    pub depth_fade: bool,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_full_opacity")]
    pub opacity: AnimatedValue,
    #[serde(default)]
    pub seed: u64,
}

fn default_particle_count() -> u32 {
    100
}
fn default_bounds() -> [f32; 3] {
    [10.0, 10.0, 10.0]
}
fn default_particle_size() -> f32 {
    2.0
}
fn default_depth_fade() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxesElement {
    #[serde(default = "default_axis_length")]
    pub length: f32,
    #[serde(default)]
    pub colors: AxisColors,
    #[serde(default)]
    pub position: [f32; 3],
    #[serde(default = "default_thickness")]
    pub thickness: f32,
    #[serde(default = "default_full_opacity")]
    pub opacity: AnimatedValue,
}

fn default_axis_length() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisColors {
    #[serde(default = "default_x_color")]
    pub x: String,
    #[serde(default = "default_y_color")]
    pub y: String,
    #[serde(default = "default_z_color")]
    pub z: String,
}

fn default_x_color() -> String {
    "#ff0000".to_string()
}
fn default_y_color() -> String {
    "#00ff00".to_string()
}
fn default_z_color() -> String {
    "#0000ff".to_string()
}

impl Default for AxisColors {
    fn default() -> Self {
        Self {
            x: default_x_color(),
            y: default_y_color(),
            z: default_z_color(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostProcessing {
    #[serde(default)]
    pub bloom: f32,
    #[serde(default)]
    pub scanlines: Option<Scanlines>,
    #[serde(default)]
    pub chromatic_aberration: f32,
    #[serde(default)]
    pub noise: f32,
    #[serde(default)]
    pub vignette: f32,
    #[serde(default)]
    pub crt_curvature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scanlines {
    #[serde(default = "default_scanline_intensity")]
    pub intensity: f32,
    #[serde(default = "default_scanline_count")]
    pub count: u32,
}

fn default_scanline_intensity() -> f32 {
    0.1
}
fn default_scanline_count() -> u32 {
    300
}

pub fn parse_hex_color(hex: &str) -> Option<[f32; 4]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
}
