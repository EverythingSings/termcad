use super::schema::*;

pub fn spinning_cube() -> Scene {
    Scene {
        canvas: Canvas {
            width: 800,
            height: 600,
            background: "#0a0a0a".to_string(),
        },
        camera: Camera {
            position: [5.0, 5.0, 5.0],
            target: [0.0, 0.0, 0.0],
            fov: 45.0,
        },
        duration: 2.0,
        fps: 30,
        r#loop: true,
        elements: vec![
            Element::Grid(GridElement {
                divisions: 20,
                fade_distance: 50.0,
                color: "#00ff41".to_string(),
                opacity: 0.3,
            }),
            Element::Wireframe(WireframeElement {
                geometry: GeometryType::Cube,
                position: [0.0, 0.5, 0.0],
                rotation: AnimatedRotation {
                    x: AnimatedValue::Static(0.0),
                    y: AnimatedValue::Expression("t * 360".to_string()),
                    z: AnimatedValue::Static(0.0),
                },
                scale: Scale::Uniform(1.0),
                color: "#00ff41".to_string(),
                thickness: 2.0,
                opacity: 1.0,
            }),
        ],
        post: PostProcessing {
            bloom: 0.3,
            scanlines: Some(Scanlines {
                intensity: 0.1,
                count: 300,
            }),
            chromatic_aberration: 0.002,
            noise: 0.02,
            vignette: 0.3,
            crt_curvature: 0.0,
        },
    }
}

pub fn grid_flythrough() -> Scene {
    Scene {
        canvas: Canvas {
            width: 800,
            height: 600,
            background: "#0a0a0a".to_string(),
        },
        camera: Camera {
            position: [0.0, 2.0, 10.0],
            target: [0.0, 0.0, 0.0],
            fov: 60.0,
        },
        duration: 3.0,
        fps: 30,
        r#loop: true,
        elements: vec![
            Element::Grid(GridElement {
                divisions: 40,
                fade_distance: 100.0,
                color: "#00ff41".to_string(),
                opacity: 0.5,
            }),
            Element::Axes(AxesElement {
                length: 2.0,
                colors: AxisColors::default(),
                position: [0.0, 0.0, 0.0],
                thickness: 3.0,
                opacity: 1.0,
            }),
        ],
        post: PostProcessing {
            bloom: 0.4,
            scanlines: Some(Scanlines {
                intensity: 0.15,
                count: 400,
            }),
            chromatic_aberration: 0.003,
            noise: 0.03,
            vignette: 0.4,
            crt_curvature: 0.0,
        },
    }
}

pub fn text_terminal() -> Scene {
    Scene {
        canvas: Canvas {
            width: 800,
            height: 600,
            background: "#0a0a0a".to_string(),
        },
        camera: Camera {
            position: [0.0, 0.0, 5.0],
            target: [0.0, 0.0, 0.0],
            fov: 45.0,
        },
        duration: 2.0,
        fps: 30,
        r#loop: true,
        elements: vec![
            Element::Glyph(GlyphElement {
                text: "SYSTEM ONLINE".to_string(),
                font_size: 0.5,
                position: [0.0, 1.0, 0.0],
                color: "#00ff41".to_string(),
                animation: GlyphAnimation::Type,
                opacity: 1.0,
            }),
            Element::Glyph(GlyphElement {
                text: "> READY".to_string(),
                font_size: 0.3,
                position: [0.0, 0.0, 0.0],
                color: "#00ff41".to_string(),
                animation: GlyphAnimation::Flicker,
                opacity: 0.8,
            }),
            Element::Line(LineElement {
                points: vec![[-2.0, -1.0, 0.0], [2.0, -1.0, 0.0]],
                closed: false,
                thickness: 1.0,
                glow: 0.5,
                color: "#00ff41".to_string(),
                opacity: 0.5,
            }),
        ],
        post: PostProcessing {
            bloom: 0.5,
            scanlines: Some(Scanlines {
                intensity: 0.2,
                count: 300,
            }),
            chromatic_aberration: 0.004,
            noise: 0.05,
            vignette: 0.5,
            crt_curvature: 0.0,
        },
    }
}
