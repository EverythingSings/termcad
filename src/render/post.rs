use crate::scene::{ExpressionContext, PostProcessing};
use std::sync::Arc;

pub struct PostProcessor {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    width: u32,
    height: u32,
    output_texture: wgpu::Texture,
    post_pipeline: Option<wgpu::RenderPipeline>,
    post_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    settings: PostProcessing,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniforms {
    resolution: [f32; 2],
    time: f32,
    bloom: f32,
    scanline_intensity: f32,
    scanline_count: f32,
    chromatic_aberration: f32,
    noise: f32,
    vignette: f32,
    crt_curvature: f32,
    _padding: [f32; 2],
}

impl PostProcessor {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        width: u32,
        height: u32,
        settings: &PostProcessing,
    ) -> Self {
        // Create output texture
        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("post output texture"),
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

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("post uniform buffer"),
            size: std::mem::size_of::<PostUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let post_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("post bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Check if we need post-processing
        let needs_post = settings.bloom > 0.0
            || settings.scanlines.is_some()
            || settings.chromatic_aberration > 0.0
            || settings.noise > 0.0
            || settings.vignette > 0.0
            || settings.crt_curvature > 0.0;

        let post_pipeline = if needs_post {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("post shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/post.wgsl").into()),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("post pipeline layout"),
                bind_group_layouts: &[&post_bind_group_layout],
                push_constant_ranges: &[],
            });

            Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("post pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            }))
        } else {
            None
        };

        Self {
            device,
            queue,
            width,
            height,
            output_texture,
            post_pipeline,
            post_bind_group_layout,
            sampler,
            uniform_buffer,
            settings: settings.clone(),
        }
    }

    pub fn process<'a>(
        &'a self,
        input_view: &wgpu::TextureView,
        input_texture: &'a wgpu::Texture,
        ctx: &ExpressionContext,
    ) -> &'a wgpu::Texture {
        // No post-processing needed, return input directly
        let Some(pipeline) = &self.post_pipeline else {
            return input_texture;
        };

        // Update uniforms
        let (scanline_intensity, scanline_count) = self
            .settings
            .scanlines
            .as_ref()
            .map(|s| (s.intensity, s.count as f32))
            .unwrap_or((0.0, 0.0));

        let uniforms = PostUniforms {
            resolution: [self.width as f32, self.height as f32],
            time: ctx.t,
            bloom: self.settings.bloom,
            scanline_intensity,
            scanline_count,
            chromatic_aberration: self.settings.chromatic_aberration,
            noise: self.settings.noise,
            vignette: self.settings.vignette,
            crt_curvature: self.settings.crt_curvature,
            _padding: [0.0, 0.0],
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post bind group"),
            layout: &self.post_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let output_view = self
            .output_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("post encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        &self.output_texture
    }
}
