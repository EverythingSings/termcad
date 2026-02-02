use super::camera::Camera;
use super::post::PostProcessor;
use crate::primitives::{
    AxesPrimitive, GlyphPrimitive, GridPrimitive, LinePrimitive, LineVertex, ParticlesPrimitive,
    Primitive, WireframePrimitive,
};
use crate::scene::{parse_hex_color, Element, ExpressionContext, Scene};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum RenderError {
    #[error("GPU initialization failed: {0}")]
    GpuInitFailed(String),

    #[error("Shader compilation failed: {0}")]
    ShaderError(String),

    #[error("Buffer creation failed: {0}")]
    BufferError(String),

    #[error("Frame capture failed: {0}")]
    CaptureFailed(String),
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    resolution: [f32; 2],
    _padding: [f32; 2],
}

pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    output_buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    background_color: [f32; 4],
    camera: Camera,
    elements: Vec<Element>,
    total_frames: u32,
    post_processor: PostProcessor,
}

impl Renderer {
    pub fn new(scene: &Scene) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| RenderError::GpuInitFailed("No suitable GPU adapter found".to_string()))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("termcad device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .map_err(|e| RenderError::GpuInitFailed(e.to_string()))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let width = scene.canvas.width;
        let height = scene.canvas.height;

        // Create texture for rendering
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create output buffer for reading pixels
        let bytes_per_row = (width * 4 + 255) & !255; // Align to 256 bytes
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output buffer"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("line shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/line.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("line render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<LineVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 12,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let camera = Camera::from_scene(&scene.camera, width, height);
        let background_color =
            parse_hex_color(&scene.canvas.background).unwrap_or([0.04, 0.04, 0.04, 1.0]);

        let post_processor =
            PostProcessor::new(Arc::clone(&device), Arc::clone(&queue), width, height, &scene.post);

        Ok(Self {
            device,
            queue,
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture,
            texture_view,
            output_buffer,
            width,
            height,
            background_color,
            camera,
            elements: scene.elements.clone(),
            total_frames: scene.total_frames(),
            post_processor,
        })
    }

    pub fn render_all(&self, json_output: bool) -> Result<Vec<image::RgbaImage>, RenderError> {
        let mut frames = Vec::with_capacity(self.total_frames as usize);

        for frame in 0..self.total_frames {
            let ctx = ExpressionContext::new(frame, self.total_frames);

            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "rendering",
                        "frame": frame + 1,
                        "total": self.total_frames
                    })
                );
            }

            let image = self.render_frame(&ctx)?;
            frames.push(image);
        }

        Ok(frames)
    }

    fn render_frame(&self, ctx: &ExpressionContext) -> Result<image::RgbaImage, RenderError> {
        // Collect vertices from all elements
        let mut all_vertices: Vec<LineVertex> = Vec::new();

        for element in &self.elements {
            let vertices = match element {
                Element::Grid(g) => GridPrimitive::from_element(g).vertices(ctx),
                Element::Wireframe(w) => WireframePrimitive::from_element(w).vertices(ctx),
                Element::Glyph(g) => GlyphPrimitive::from_element(g).vertices(ctx),
                Element::Line(l) => LinePrimitive::from_element(l).vertices(ctx),
                Element::Particles(p) => ParticlesPrimitive::from_element(p).vertices(ctx),
                Element::Axes(a) => AxesPrimitive::from_element(a).vertices(ctx),
            };
            all_vertices.extend(vertices);
        }

        // Create vertex buffer
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&all_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Update uniforms
        let uniforms = Uniforms {
            view_proj: self.camera.view_projection_matrix(),
            resolution: [self.width as f32, self.height as f32],
            _padding: [0.0, 0.0],
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.background_color[0] as f64,
                            g: self.background_color[1] as f64,
                            b: self.background_color[2] as f64,
                            a: self.background_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..all_vertices.len() as u32, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        // Apply post-processing
        let final_texture = self.post_processor.process(&self.texture_view, &self.texture, ctx);

        // Copy texture to buffer
        let bytes_per_row = (self.width * 4 + 255) & !255;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("copy encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: final_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        // Read pixels back
        let buffer_slice = self.output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            // Use ok() instead of unwrap() - if receiver is dropped, recv() will handle the error
            let _ = tx.send(result);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .map_err(|e| RenderError::CaptureFailed(e.to_string()))?
            .map_err(|e| RenderError::CaptureFailed(e.to_string()))?;

        let data = buffer_slice.get_mapped_range();

        // Convert to image, handling row padding
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        for y in 0..self.height {
            let start = (y * bytes_per_row) as usize;
            let end = start + (self.width * 4) as usize;
            pixels.extend_from_slice(&data[start..end]);
        }

        drop(data);
        self.output_buffer.unmap();

        image::RgbaImage::from_raw(self.width, self.height, pixels)
            .ok_or_else(|| RenderError::CaptureFailed("Failed to create image".to_string()))
    }
}

// Helper trait for buffer initialization
trait DeviceExt {
    fn create_buffer_init(&self, desc: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer;
}

impl DeviceExt for wgpu::Device {
    fn create_buffer_init(&self, desc: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer {
        let unpadded_size = desc.contents.len() as u64;
        let padding = (4 - (unpadded_size % 4)) % 4;
        let padded_size = unpadded_size + padding;

        let buffer = self.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label,
            size: padded_size,
            usage: desc.usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });

        buffer.slice(..).get_mapped_range_mut()[..desc.contents.len()]
            .copy_from_slice(desc.contents);
        buffer.unmap();

        buffer
    }
}
