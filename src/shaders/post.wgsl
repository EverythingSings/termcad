struct PostUniforms {
    resolution: vec2<f32>,
    time: f32,
    bloom: f32,
    scanline_intensity: f32,
    scanline_count: f32,
    chromatic_aberration: f32,
    noise: f32,
    vignette: f32,
    crt_curvature: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@group(0) @binding(2)
var<uniform> uniforms: PostUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Generate fullscreen triangle
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);

    return out;
}

// Simple hash function for noise
fn hash(p: vec2<f32>) -> f32 {
    let p2 = vec2<f32>(
        dot(p, vec2<f32>(127.1, 311.7)),
        dot(p, vec2<f32>(269.5, 183.3))
    );
    return fract(sin(p2.x) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // Apply CRT curvature
    if uniforms.crt_curvature > 0.0 {
        let center = uv - 0.5;
        let dist = dot(center, center) * uniforms.crt_curvature;
        uv = uv + center * dist;
    }

    // Check bounds after curvature
    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    var color: vec3<f32>;

    // Apply chromatic aberration
    if uniforms.chromatic_aberration > 0.0 {
        let offset = uniforms.chromatic_aberration;
        let r = textureSample(input_texture, input_sampler, uv + vec2<f32>(offset, 0.0)).r;
        let g = textureSample(input_texture, input_sampler, uv).g;
        let b = textureSample(input_texture, input_sampler, uv - vec2<f32>(offset, 0.0)).b;
        color = vec3<f32>(r, g, b);
    } else {
        color = textureSample(input_texture, input_sampler, uv).rgb;
    }

    // Apply bloom (simple glow)
    if uniforms.bloom > 0.0 {
        var bloom_color = vec3<f32>(0.0);
        let bloom_samples = 8;
        let bloom_radius = 0.003;

        for (var i = 0; i < bloom_samples; i++) {
            let angle = f32(i) * 3.14159 * 2.0 / f32(bloom_samples);
            let offset = vec2<f32>(cos(angle), sin(angle)) * bloom_radius;
            bloom_color += textureSample(input_texture, input_sampler, uv + offset).rgb;
        }
        bloom_color /= f32(bloom_samples);

        // Add bloom
        color = mix(color, color + bloom_color * 0.5, uniforms.bloom);
    }

    // Apply scanlines
    if uniforms.scanline_intensity > 0.0 && uniforms.scanline_count > 0.0 {
        let scanline = sin(uv.y * uniforms.scanline_count * 3.14159) * 0.5 + 0.5;
        let scanline_factor = 1.0 - uniforms.scanline_intensity * (1.0 - scanline);
        color *= scanline_factor;
    }

    // Apply noise
    if uniforms.noise > 0.0 {
        let noise_value = hash(uv * uniforms.resolution + vec2<f32>(uniforms.time * 1000.0, 0.0));
        color = mix(color, vec3<f32>(noise_value), uniforms.noise * 0.5);
    }

    // Apply vignette
    if uniforms.vignette > 0.0 {
        let center = uv - 0.5;
        let vignette_factor = 1.0 - dot(center, center) * uniforms.vignette * 2.0;
        color *= max(vignette_factor, 0.0);
    }

    return vec4<f32>(color, 1.0);
}
