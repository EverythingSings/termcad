use super::schema::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid color format: {0}")]
    InvalidColor(String),

    #[error("Invalid canvas dimensions: {0}")]
    InvalidDimensions(String),

    #[error("Invalid element configuration: {0}")]
    InvalidElement(String),

    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

pub fn validate_scene(scene: &Scene) -> Result<(), ValidationError> {
    validate_canvas(&scene.canvas)?;
    validate_camera(&scene.camera)?;

    if scene.duration <= 0.0 {
        return Err(ValidationError::InvalidValue(
            "duration must be positive".to_string(),
        ));
    }

    if scene.fps == 0 || scene.fps > 120 {
        return Err(ValidationError::InvalidValue(
            "fps must be between 1 and 120".to_string(),
        ));
    }

    for (i, element) in scene.elements.iter().enumerate() {
        validate_element(element)
            .map_err(|e| ValidationError::InvalidElement(format!("Element {}: {}", i, e)))?;
    }

    validate_post_processing(&scene.post)?;

    Ok(())
}

fn validate_canvas(canvas: &Canvas) -> Result<(), ValidationError> {
    if canvas.width == 0 || canvas.width > 4096 {
        return Err(ValidationError::InvalidDimensions(
            "width must be between 1 and 4096".to_string(),
        ));
    }

    if canvas.height == 0 || canvas.height > 4096 {
        return Err(ValidationError::InvalidDimensions(
            "height must be between 1 and 4096".to_string(),
        ));
    }

    validate_color(&canvas.background)?;

    Ok(())
}

fn validate_camera(camera: &Camera) -> Result<(), ValidationError> {
    if camera.fov <= 0.0 || camera.fov >= 180.0 {
        return Err(ValidationError::InvalidValue(
            "FOV must be between 0 and 180 degrees".to_string(),
        ));
    }

    Ok(())
}

fn validate_element(element: &Element) -> Result<(), ValidationError> {
    match element {
        Element::Grid(grid) => validate_grid(grid),
        Element::Wireframe(wf) => validate_wireframe(wf),
        Element::Glyph(glyph) => validate_glyph(glyph),
        Element::Line(line) => validate_line(line),
        Element::Particles(particles) => validate_particles(particles),
        Element::Axes(axes) => validate_axes(axes),
    }
}

fn validate_grid(grid: &GridElement) -> Result<(), ValidationError> {
    validate_color(&grid.color)?;
    validate_opacity(&grid.opacity)?;

    if grid.divisions == 0 {
        return Err(ValidationError::InvalidValue(
            "divisions must be positive".to_string(),
        ));
    }

    if grid.fade_distance <= 0.0 {
        return Err(ValidationError::InvalidValue(
            "fade_distance must be positive".to_string(),
        ));
    }

    Ok(())
}

fn validate_wireframe(wf: &WireframeElement) -> Result<(), ValidationError> {
    validate_color(&wf.color)?;
    validate_opacity(&wf.opacity)?;
    validate_thickness(wf.thickness)?;
    validate_animated_rotation(&wf.rotation)?;
    validate_scale(&wf.scale)?;

    Ok(())
}

fn validate_scale(scale: &Scale) -> Result<(), ValidationError> {
    match scale {
        Scale::Uniform(s) => {
            if *s <= 0.0 {
                return Err(ValidationError::InvalidValue("scale must be positive".into()));
            }
        }
        Scale::NonUniform(v) => {
            for (i, s) in v.iter().enumerate() {
                if *s <= 0.0 {
                    return Err(ValidationError::InvalidValue(format!(
                        "scale[{}] must be positive",
                        i
                    )));
                }
            }
        }
        Scale::UniformExpression(expr) => {
            let ctx = super::ExpressionContext::new(0, 30);
            super::evaluate_expression(expr, &ctx).map_err(|e| {
                ValidationError::InvalidExpression(format!("scale '{}': {}", expr, e))
            })?;
        }
        Scale::PerAxis(animated) => {
            validate_animated_value(&animated.x, "scale.x")?;
            validate_animated_value(&animated.y, "scale.y")?;
            validate_animated_value(&animated.z, "scale.z")?;
        }
    }
    Ok(())
}

fn validate_glyph(glyph: &GlyphElement) -> Result<(), ValidationError> {
    validate_color(&glyph.color)?;
    validate_opacity(&glyph.opacity)?;

    if glyph.text.is_empty() {
        return Err(ValidationError::InvalidValue(
            "glyph text cannot be empty".to_string(),
        ));
    }

    if glyph.font_size <= 0.0 {
        return Err(ValidationError::InvalidValue(
            "font_size must be positive".to_string(),
        ));
    }

    Ok(())
}

fn validate_line(line: &LineElement) -> Result<(), ValidationError> {
    validate_color(&line.color)?;
    validate_opacity(&line.opacity)?;
    validate_thickness(line.thickness)?;

    if line.points.len() < 2 {
        return Err(ValidationError::InvalidValue(
            "line must have at least 2 points".to_string(),
        ));
    }

    if line.glow < 0.0 || line.glow > 1.0 {
        return Err(ValidationError::InvalidValue(
            "glow must be between 0.0 and 1.0".to_string(),
        ));
    }

    Ok(())
}

fn validate_particles(particles: &ParticlesElement) -> Result<(), ValidationError> {
    validate_color(&particles.color)?;
    validate_opacity(&particles.opacity)?;

    if particles.count == 0 {
        return Err(ValidationError::InvalidValue(
            "particle count must be positive".to_string(),
        ));
    }

    if particles.size <= 0.0 {
        return Err(ValidationError::InvalidValue(
            "particle size must be positive".to_string(),
        ));
    }

    Ok(())
}

fn validate_axes(axes: &AxesElement) -> Result<(), ValidationError> {
    validate_color(&axes.colors.x)?;
    validate_color(&axes.colors.y)?;
    validate_color(&axes.colors.z)?;
    validate_opacity(&axes.opacity)?;
    validate_thickness(axes.thickness)?;

    if axes.length <= 0.0 {
        return Err(ValidationError::InvalidValue(
            "axis length must be positive".to_string(),
        ));
    }

    Ok(())
}

fn validate_post_processing(post: &PostProcessing) -> Result<(), ValidationError> {
    if post.bloom < 0.0 || post.bloom > 1.0 {
        return Err(ValidationError::InvalidValue(
            "bloom must be between 0.0 and 1.0".to_string(),
        ));
    }

    if post.chromatic_aberration < 0.0 || post.chromatic_aberration > 0.1 {
        return Err(ValidationError::InvalidValue(
            "chromatic_aberration must be between 0.0 and 0.1".to_string(),
        ));
    }

    if post.noise < 0.0 || post.noise > 1.0 {
        return Err(ValidationError::InvalidValue(
            "noise must be between 0.0 and 1.0".to_string(),
        ));
    }

    if post.vignette < 0.0 || post.vignette > 1.0 {
        return Err(ValidationError::InvalidValue(
            "vignette must be between 0.0 and 1.0".to_string(),
        ));
    }

    if post.crt_curvature < 0.0 || post.crt_curvature > 1.0 {
        return Err(ValidationError::InvalidValue(
            "crt_curvature must be between 0.0 and 1.0".to_string(),
        ));
    }

    if let Some(ref scanlines) = post.scanlines {
        if scanlines.intensity < 0.0 || scanlines.intensity > 1.0 {
            return Err(ValidationError::InvalidValue(
                "scanline intensity must be between 0.0 and 1.0".to_string(),
            ));
        }
        if scanlines.count == 0 {
            return Err(ValidationError::InvalidValue(
                "scanline count must be positive".to_string(),
            ));
        }
    }

    Ok(())
}

fn validate_color(color: &str) -> Result<(), ValidationError> {
    if parse_hex_color(color).is_none() {
        return Err(ValidationError::InvalidColor(format!(
            "'{}' is not a valid hex color (expected #RRGGBB)",
            color
        )));
    }
    Ok(())
}

fn validate_opacity(opacity: &AnimatedValue) -> Result<(), ValidationError> {
    match opacity {
        AnimatedValue::Static(v) => {
            if *v < 0.0 || *v > 1.0 {
                return Err(ValidationError::InvalidValue(
                    "opacity must be between 0.0 and 1.0".to_string(),
                ));
            }
        }
        AnimatedValue::Expression(expr) => {
            // Validate expression syntax by evaluating at t=0
            let ctx = super::ExpressionContext::new(0, 30);
            super::evaluate_expression(expr, &ctx).map_err(|e| {
                ValidationError::InvalidExpression(format!("opacity '{}': {}", expr, e))
            })?;
            // Note: We cannot validate that runtime values stay in 0-1 range,
            // but expressions are clamped in the primitives anyway
        }
    }
    Ok(())
}

fn validate_thickness(thickness: f32) -> Result<(), ValidationError> {
    if thickness <= 0.0 {
        return Err(ValidationError::InvalidValue(
            "thickness must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_animated_rotation(rotation: &AnimatedRotation) -> Result<(), ValidationError> {
    validate_animated_value(&rotation.x, "rotation.x")?;
    validate_animated_value(&rotation.y, "rotation.y")?;
    validate_animated_value(&rotation.z, "rotation.z")?;
    Ok(())
}

fn validate_animated_value(value: &AnimatedValue, _name: &str) -> Result<(), ValidationError> {
    match value {
        AnimatedValue::Static(_) => Ok(()),
        AnimatedValue::Expression(expr) => {
            // Try to evaluate the expression with t=0 to check validity
            let ctx = super::ExpressionContext::new(0, 30);
            super::evaluate_expression(expr, &ctx).map_err(|e| {
                ValidationError::InvalidExpression(format!("'{}': {}", expr, e))
            })?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================
    // Test Helpers
    // ===========================================

    fn make_canvas(width: u32, height: u32, background: &str) -> Canvas {
        Canvas {
            width,
            height,
            background: background.to_string(),
        }
    }

    fn make_camera(fov: f32) -> Camera {
        Camera {
            position: [5.0, 5.0, 5.0],
            target: [0.0, 0.0, 0.0],
            fov,
        }
    }

    fn make_grid(divisions: u32, fade_distance: f32, color: &str) -> GridElement {
        GridElement {
            divisions,
            fade_distance,
            color: color.to_string(),
            opacity: AnimatedValue::Static(0.5),
        }
    }

    fn make_wireframe(color: &str, thickness: f32) -> WireframeElement {
        WireframeElement {
            color: color.to_string(),
            thickness,
            ..Default::default()
        }
    }

    fn make_glyph(text: &str, font_size: f32, color: &str) -> GlyphElement {
        GlyphElement {
            text: text.to_string(),
            font_size,
            position: [0.0, 0.0, 0.0],
            color: color.to_string(),
            animation: GlyphAnimation::None,
            opacity: AnimatedValue::Static(1.0),
        }
    }

    fn make_line(points: Vec<[f32; 3]>, glow: f32, color: &str, thickness: f32) -> LineElement {
        LineElement {
            points,
            closed: false,
            thickness,
            glow,
            color: color.to_string(),
            opacity: AnimatedValue::Static(1.0),
        }
    }

    fn make_particles(count: u32, size: f32, color: &str) -> ParticlesElement {
        ParticlesElement {
            count,
            bounds: [10.0, 10.0, 10.0],
            size,
            depth_fade: true,
            color: color.to_string(),
            opacity: AnimatedValue::Static(1.0),
            seed: 0,
        }
    }

    fn make_axes(length: f32, thickness: f32, colors: AxisColors) -> AxesElement {
        AxesElement {
            length,
            colors,
            position: [0.0, 0.0, 0.0],
            thickness,
            opacity: AnimatedValue::Static(1.0),
        }
    }

    fn make_post(bloom: f32, chromatic_aberration: f32) -> PostProcessing {
        PostProcessing {
            bloom,
            chromatic_aberration,
            noise: 0.0,
            vignette: 0.0,
            crt_curvature: 0.0,
            scanlines: None,
        }
    }

    fn make_scene(canvas: Canvas, camera: Camera, duration: f32, fps: u32) -> Scene {
        Scene {
            canvas,
            camera,
            duration,
            fps,
            r#loop: true,
            elements: vec![],
            post: PostProcessing::default(),
        }
    }

    // ===========================================
    // Color Validation Tests
    // ===========================================

    #[test]
    fn test_validate_color_valid() {
        assert!(validate_color("#000000").is_ok());
        assert!(validate_color("#FFFFFF").is_ok());
        assert!(validate_color("#00ff41").is_ok());
        assert!(validate_color("#aAbBcC").is_ok());
    }

    #[test]
    fn test_validate_color_invalid_short() {
        let result = validate_color("#FFF");
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    #[test]
    fn test_validate_color_invalid_char() {
        let result = validate_color("#12345G");
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    #[test]
    fn test_validate_color_without_hash_valid() {
        // Implementation is lenient - allows colors without # prefix
        assert!(validate_color("000000").is_ok());
        assert!(validate_color("FFFFFF").is_ok());
    }

    #[test]
    fn test_validate_color_wrong_length_no_hash() {
        // 5 chars without hash = invalid (not 6)
        let result = validate_color("12345");
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    #[test]
    fn test_validate_color_too_long() {
        let result = validate_color("#1234567");
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    // ===========================================
    // Canvas Validation Tests
    // ===========================================

    #[test]
    fn test_validate_canvas_min_dimensions() {
        let canvas = make_canvas(1, 1, "#000000");
        assert!(validate_canvas(&canvas).is_ok());
    }

    #[test]
    fn test_validate_canvas_max_dimensions() {
        let canvas = make_canvas(4096, 4096, "#000000");
        assert!(validate_canvas(&canvas).is_ok());
    }

    #[test]
    fn test_validate_canvas_zero_width() {
        let canvas = make_canvas(0, 600, "#000000");
        let result = validate_canvas(&canvas);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidDimensions(_)) => {}
            _ => panic!("Expected InvalidDimensions error"),
        }
    }

    #[test]
    fn test_validate_canvas_zero_height() {
        let canvas = make_canvas(800, 0, "#000000");
        let result = validate_canvas(&canvas);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidDimensions(_)) => {}
            _ => panic!("Expected InvalidDimensions error"),
        }
    }

    #[test]
    fn test_validate_canvas_exceeds_max() {
        let canvas = make_canvas(4097, 600, "#000000");
        let result = validate_canvas(&canvas);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidDimensions(_)) => {}
            _ => panic!("Expected InvalidDimensions error"),
        }
    }

    #[test]
    fn test_validate_canvas_invalid_color() {
        let canvas = make_canvas(800, 600, "invalid");
        let result = validate_canvas(&canvas);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    // ===========================================
    // Camera Validation Tests
    // ===========================================

    #[test]
    fn test_validate_camera_valid_fov() {
        assert!(validate_camera(&make_camera(45.0)).is_ok());
        assert!(validate_camera(&make_camera(90.0)).is_ok());
    }

    #[test]
    fn test_validate_camera_fov_boundary() {
        assert!(validate_camera(&make_camera(0.01)).is_ok());
        assert!(validate_camera(&make_camera(179.99)).is_ok());
    }

    #[test]
    fn test_validate_camera_fov_zero() {
        let result = validate_camera(&make_camera(0.0));
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(_)) => {}
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_validate_camera_fov_180() {
        let result = validate_camera(&make_camera(180.0));
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(_)) => {}
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_validate_camera_fov_negative() {
        let result = validate_camera(&make_camera(-10.0));
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(_)) => {}
            _ => panic!("Expected InvalidValue error"),
        }
    }

    // ===========================================
    // Scene Timing Validation Tests
    // ===========================================

    #[test]
    fn test_validate_scene_valid_timing() {
        let scene = make_scene(Canvas::default(), Camera::default(), 2.0, 30);
        assert!(validate_scene(&scene).is_ok());
    }

    #[test]
    fn test_validate_scene_fps_boundaries() {
        let scene_min = make_scene(Canvas::default(), Camera::default(), 1.0, 1);
        assert!(validate_scene(&scene_min).is_ok());

        let scene_max = make_scene(Canvas::default(), Camera::default(), 1.0, 120);
        assert!(validate_scene(&scene_max).is_ok());
    }

    #[test]
    fn test_validate_scene_zero_duration() {
        let scene = make_scene(Canvas::default(), Camera::default(), 0.0, 30);
        let result = validate_scene(&scene);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("duration"));
            }
            _ => panic!("Expected InvalidValue error about duration"),
        }
    }

    #[test]
    fn test_validate_scene_negative_duration() {
        let scene = make_scene(Canvas::default(), Camera::default(), -1.0, 30);
        let result = validate_scene(&scene);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("duration"));
            }
            _ => panic!("Expected InvalidValue error about duration"),
        }
    }

    #[test]
    fn test_validate_scene_zero_fps() {
        let scene = make_scene(Canvas::default(), Camera::default(), 2.0, 0);
        let result = validate_scene(&scene);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("fps"));
            }
            _ => panic!("Expected InvalidValue error about fps"),
        }
    }

    #[test]
    fn test_validate_scene_fps_exceeds_max() {
        let scene = make_scene(Canvas::default(), Camera::default(), 2.0, 121);
        let result = validate_scene(&scene);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("fps"));
            }
            _ => panic!("Expected InvalidValue error about fps"),
        }
    }

    // ===========================================
    // Grid Validation Tests
    // ===========================================

    #[test]
    fn test_validate_grid_valid() {
        let grid = make_grid(20, 50.0, "#00ff41");
        assert!(validate_grid(&grid).is_ok());
    }

    #[test]
    fn test_validate_grid_zero_divisions() {
        let grid = make_grid(0, 50.0, "#00ff41");
        let result = validate_grid(&grid);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("divisions"));
            }
            _ => panic!("Expected InvalidValue error about divisions"),
        }
    }

    #[test]
    fn test_validate_grid_zero_fade() {
        let grid = make_grid(20, 0.0, "#00ff41");
        let result = validate_grid(&grid);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("fade_distance"));
            }
            _ => panic!("Expected InvalidValue error about fade_distance"),
        }
    }

    #[test]
    fn test_validate_grid_negative_fade() {
        let grid = make_grid(20, -10.0, "#00ff41");
        let result = validate_grid(&grid);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("fade_distance"));
            }
            _ => panic!("Expected InvalidValue error about fade_distance"),
        }
    }

    #[test]
    fn test_validate_grid_invalid_color() {
        let grid = make_grid(20, 50.0, "bad");
        let result = validate_grid(&grid);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    // ===========================================
    // Wireframe Validation Tests
    // ===========================================

    #[test]
    fn test_validate_wireframe_valid() {
        let wf = make_wireframe("#00ff41", 2.0);
        assert!(validate_wireframe(&wf).is_ok());
    }

    #[test]
    fn test_validate_wireframe_zero_thickness() {
        let wf = make_wireframe("#00ff41", 0.0);
        let result = validate_wireframe(&wf);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("thickness"));
            }
            _ => panic!("Expected InvalidValue error about thickness"),
        }
    }

    #[test]
    fn test_validate_wireframe_invalid_rotation() {
        let mut wf = make_wireframe("#00ff41", 2.0);
        wf.rotation.y = AnimatedValue::Expression("invalid syntax".to_string());
        let result = validate_wireframe(&wf);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidExpression(_)) => {}
            _ => panic!("Expected InvalidExpression error"),
        }
    }

    #[test]
    fn test_validate_wireframe_invalid_color() {
        let wf = make_wireframe("notacolor", 2.0);
        let result = validate_wireframe(&wf);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    // ===========================================
    // Glyph Validation Tests
    // ===========================================

    #[test]
    fn test_validate_glyph_valid() {
        let glyph = make_glyph("HELLO", 1.0, "#00ff41");
        assert!(validate_glyph(&glyph).is_ok());
    }

    #[test]
    fn test_validate_glyph_empty_text() {
        let glyph = make_glyph("", 1.0, "#00ff41");
        let result = validate_glyph(&glyph);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("text"));
            }
            _ => panic!("Expected InvalidValue error about text"),
        }
    }

    #[test]
    fn test_validate_glyph_zero_font_size() {
        let glyph = make_glyph("HELLO", 0.0, "#00ff41");
        let result = validate_glyph(&glyph);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("font_size"));
            }
            _ => panic!("Expected InvalidValue error about font_size"),
        }
    }

    #[test]
    fn test_validate_glyph_negative_font_size() {
        let glyph = make_glyph("HELLO", -1.0, "#00ff41");
        let result = validate_glyph(&glyph);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("font_size"));
            }
            _ => panic!("Expected InvalidValue error about font_size"),
        }
    }

    #[test]
    fn test_validate_glyph_invalid_color() {
        let glyph = make_glyph("HELLO", 1.0, "bad");
        let result = validate_glyph(&glyph);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    // ===========================================
    // Line Validation Tests
    // ===========================================

    #[test]
    fn test_validate_line_valid() {
        let line = make_line(
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            0.5,
            "#00ff41",
            2.0,
        );
        assert!(validate_line(&line).is_ok());
    }

    #[test]
    fn test_validate_line_one_point() {
        let line = make_line(vec![[0.0, 0.0, 0.0]], 0.5, "#00ff41", 2.0);
        let result = validate_line(&line);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("2 points"));
            }
            _ => panic!("Expected InvalidValue error about points"),
        }
    }

    #[test]
    fn test_validate_line_zero_points() {
        let line = make_line(vec![], 0.5, "#00ff41", 2.0);
        let result = validate_line(&line);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("2 points"));
            }
            _ => panic!("Expected InvalidValue error about points"),
        }
    }

    #[test]
    fn test_validate_line_glow_valid() {
        // Test boundaries
        let line_zero = make_line(
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            0.0,
            "#00ff41",
            2.0,
        );
        assert!(validate_line(&line_zero).is_ok());

        let line_one = make_line(
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            1.0,
            "#00ff41",
            2.0,
        );
        assert!(validate_line(&line_one).is_ok());
    }

    #[test]
    fn test_validate_line_glow_exceeds() {
        let line = make_line(
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            1.1,
            "#00ff41",
            2.0,
        );
        let result = validate_line(&line);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("glow"));
            }
            _ => panic!("Expected InvalidValue error about glow"),
        }
    }

    #[test]
    fn test_validate_line_glow_negative() {
        let line = make_line(
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            -0.1,
            "#00ff41",
            2.0,
        );
        let result = validate_line(&line);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("glow"));
            }
            _ => panic!("Expected InvalidValue error about glow"),
        }
    }

    #[test]
    fn test_validate_line_zero_thickness() {
        let line = make_line(
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            0.5,
            "#00ff41",
            0.0,
        );
        let result = validate_line(&line);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("thickness"));
            }
            _ => panic!("Expected InvalidValue error about thickness"),
        }
    }

    #[test]
    fn test_validate_line_invalid_color() {
        let line = make_line(
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            0.5,
            "bad",
            2.0,
        );
        let result = validate_line(&line);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    // ===========================================
    // Particles Validation Tests
    // ===========================================

    #[test]
    fn test_validate_particles_valid() {
        let particles = make_particles(100, 2.0, "#00ff41");
        assert!(validate_particles(&particles).is_ok());
    }

    #[test]
    fn test_validate_particles_zero_count() {
        let particles = make_particles(0, 2.0, "#00ff41");
        let result = validate_particles(&particles);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("count"));
            }
            _ => panic!("Expected InvalidValue error about count"),
        }
    }

    #[test]
    fn test_validate_particles_zero_size() {
        let particles = make_particles(100, 0.0, "#00ff41");
        let result = validate_particles(&particles);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("size"));
            }
            _ => panic!("Expected InvalidValue error about size"),
        }
    }

    #[test]
    fn test_validate_particles_negative_size() {
        let particles = make_particles(100, -1.0, "#00ff41");
        let result = validate_particles(&particles);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("size"));
            }
            _ => panic!("Expected InvalidValue error about size"),
        }
    }

    #[test]
    fn test_validate_particles_invalid_color() {
        let particles = make_particles(100, 2.0, "bad");
        let result = validate_particles(&particles);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    // ===========================================
    // Axes Validation Tests
    // ===========================================

    #[test]
    fn test_validate_axes_valid() {
        let axes = make_axes(1.0, 2.0, AxisColors::default());
        assert!(validate_axes(&axes).is_ok());
    }

    #[test]
    fn test_validate_axes_zero_length() {
        let axes = make_axes(0.0, 2.0, AxisColors::default());
        let result = validate_axes(&axes);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("length"));
            }
            _ => panic!("Expected InvalidValue error about length"),
        }
    }

    #[test]
    fn test_validate_axes_negative_length() {
        let axes = make_axes(-1.0, 2.0, AxisColors::default());
        let result = validate_axes(&axes);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("length"));
            }
            _ => panic!("Expected InvalidValue error about length"),
        }
    }

    #[test]
    fn test_validate_axes_zero_thickness() {
        let axes = make_axes(1.0, 0.0, AxisColors::default());
        let result = validate_axes(&axes);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("thickness"));
            }
            _ => panic!("Expected InvalidValue error about thickness"),
        }
    }

    #[test]
    fn test_validate_axes_invalid_x_color() {
        let colors = AxisColors {
            x: "bad".to_string(),
            y: "#00ff00".to_string(),
            z: "#0000ff".to_string(),
        };
        let axes = make_axes(1.0, 2.0, colors);
        let result = validate_axes(&axes);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    #[test]
    fn test_validate_axes_invalid_y_color() {
        let colors = AxisColors {
            x: "#ff0000".to_string(),
            y: "bad".to_string(),
            z: "#0000ff".to_string(),
        };
        let axes = make_axes(1.0, 2.0, colors);
        let result = validate_axes(&axes);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    #[test]
    fn test_validate_axes_invalid_z_color() {
        let colors = AxisColors {
            x: "#ff0000".to_string(),
            y: "#00ff00".to_string(),
            z: "bad".to_string(),
        };
        let axes = make_axes(1.0, 2.0, colors);
        let result = validate_axes(&axes);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidColor(_)) => {}
            _ => panic!("Expected InvalidColor error"),
        }
    }

    // ===========================================
    // Post-Processing Validation Tests
    // ===========================================

    #[test]
    fn test_validate_post_valid_all() {
        let post = PostProcessing {
            bloom: 0.5,
            chromatic_aberration: 0.05,
            noise: 0.1,
            vignette: 0.3,
            crt_curvature: 0.2,
            scanlines: Some(Scanlines {
                intensity: 0.1,
                count: 300,
            }),
        };
        assert!(validate_post_processing(&post).is_ok());
    }

    #[test]
    fn test_validate_post_bloom_boundary() {
        let post_zero = make_post(0.0, 0.0);
        assert!(validate_post_processing(&post_zero).is_ok());

        let post_one = make_post(1.0, 0.0);
        assert!(validate_post_processing(&post_one).is_ok());
    }

    #[test]
    fn test_validate_post_bloom_exceeds() {
        let post = make_post(1.1, 0.0);
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("bloom"));
            }
            _ => panic!("Expected InvalidValue error about bloom"),
        }
    }

    #[test]
    fn test_validate_post_bloom_negative() {
        let post = make_post(-0.1, 0.0);
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("bloom"));
            }
            _ => panic!("Expected InvalidValue error about bloom"),
        }
    }

    #[test]
    fn test_validate_post_chrom_ab_max() {
        let post = make_post(0.0, 0.1);
        assert!(validate_post_processing(&post).is_ok());
    }

    #[test]
    fn test_validate_post_chrom_ab_exceeds() {
        let post = make_post(0.0, 0.11);
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("chromatic_aberration"));
            }
            _ => panic!("Expected InvalidValue error about chromatic_aberration"),
        }
    }

    #[test]
    fn test_validate_post_chrom_ab_negative() {
        let post = make_post(0.0, -0.01);
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("chromatic_aberration"));
            }
            _ => panic!("Expected InvalidValue error about chromatic_aberration"),
        }
    }

    #[test]
    fn test_validate_post_noise_boundary() {
        let mut post = make_post(0.0, 0.0);
        post.noise = 0.0;
        assert!(validate_post_processing(&post).is_ok());

        post.noise = 1.0;
        assert!(validate_post_processing(&post).is_ok());
    }

    #[test]
    fn test_validate_post_noise_exceeds() {
        let mut post = make_post(0.0, 0.0);
        post.noise = 1.1;
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("noise"));
            }
            _ => panic!("Expected InvalidValue error about noise"),
        }
    }

    #[test]
    fn test_validate_post_vignette_boundary() {
        let mut post = make_post(0.0, 0.0);
        post.vignette = 0.0;
        assert!(validate_post_processing(&post).is_ok());

        post.vignette = 1.0;
        assert!(validate_post_processing(&post).is_ok());
    }

    #[test]
    fn test_validate_post_vignette_exceeds() {
        let mut post = make_post(0.0, 0.0);
        post.vignette = 1.1;
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("vignette"));
            }
            _ => panic!("Expected InvalidValue error about vignette"),
        }
    }

    #[test]
    fn test_validate_post_crt_curvature_boundary() {
        let mut post = make_post(0.0, 0.0);
        post.crt_curvature = 0.0;
        assert!(validate_post_processing(&post).is_ok());

        post.crt_curvature = 1.0;
        assert!(validate_post_processing(&post).is_ok());
    }

    #[test]
    fn test_validate_post_crt_curvature_exceeds() {
        let mut post = make_post(0.0, 0.0);
        post.crt_curvature = 1.1;
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("crt_curvature"));
            }
            _ => panic!("Expected InvalidValue error about crt_curvature"),
        }
    }

    #[test]
    fn test_validate_post_scanlines_valid() {
        let mut post = make_post(0.0, 0.0);
        post.scanlines = Some(Scanlines {
            intensity: 0.5,
            count: 300,
        });
        assert!(validate_post_processing(&post).is_ok());
    }

    #[test]
    fn test_validate_post_scanlines_intensity_boundary() {
        let mut post = make_post(0.0, 0.0);

        post.scanlines = Some(Scanlines {
            intensity: 0.0,
            count: 300,
        });
        assert!(validate_post_processing(&post).is_ok());

        post.scanlines = Some(Scanlines {
            intensity: 1.0,
            count: 300,
        });
        assert!(validate_post_processing(&post).is_ok());
    }

    #[test]
    fn test_validate_post_scanlines_intensity_exceeds() {
        let mut post = make_post(0.0, 0.0);
        post.scanlines = Some(Scanlines {
            intensity: 1.1,
            count: 300,
        });
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("scanline intensity"));
            }
            _ => panic!("Expected InvalidValue error about scanline intensity"),
        }
    }

    #[test]
    fn test_validate_post_scanlines_zero_count() {
        let mut post = make_post(0.0, 0.0);
        post.scanlines = Some(Scanlines {
            intensity: 0.1,
            count: 0,
        });
        let result = validate_post_processing(&post);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("scanline count"));
            }
            _ => panic!("Expected InvalidValue error about scanline count"),
        }
    }

    // ===========================================
    // Thickness Validation Tests
    // ===========================================

    #[test]
    fn test_validate_thickness_valid() {
        assert!(validate_thickness(1.0).is_ok());
        assert!(validate_thickness(0.1).is_ok());
        assert!(validate_thickness(10.0).is_ok());
    }

    #[test]
    fn test_validate_thickness_zero() {
        let result = validate_thickness(0.0);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("thickness"));
            }
            _ => panic!("Expected InvalidValue error about thickness"),
        }
    }

    #[test]
    fn test_validate_thickness_negative() {
        let result = validate_thickness(-1.0);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidValue(msg)) => {
                assert!(msg.contains("thickness"));
            }
            _ => panic!("Expected InvalidValue error about thickness"),
        }
    }

    // ===========================================
    // Existing Tests (Opacity and Scale)
    // ===========================================

    #[test]
    fn test_validate_opacity_static_valid() {
        assert!(validate_opacity(&AnimatedValue::Static(0.0)).is_ok());
        assert!(validate_opacity(&AnimatedValue::Static(0.5)).is_ok());
        assert!(validate_opacity(&AnimatedValue::Static(1.0)).is_ok());
    }

    #[test]
    fn test_validate_opacity_static_invalid() {
        assert!(validate_opacity(&AnimatedValue::Static(-0.1)).is_err());
        assert!(validate_opacity(&AnimatedValue::Static(1.1)).is_err());
    }

    #[test]
    fn test_validate_opacity_expression_valid() {
        assert!(validate_opacity(&AnimatedValue::Expression("t".to_string())).is_ok());
        assert!(validate_opacity(&AnimatedValue::Expression("1 - t".to_string())).is_ok());
        assert!(validate_opacity(&AnimatedValue::Expression("sin(t * PI) * 0.5 + 0.5".to_string())).is_ok());
    }

    #[test]
    fn test_validate_opacity_expression_invalid_syntax() {
        let result = validate_opacity(&AnimatedValue::Expression("invalid syntax here".to_string()));
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidExpression(_)) => {}
            _ => panic!("Expected InvalidExpression error"),
        }
    }

    #[test]
    fn test_validate_scale_uniform_valid() {
        assert!(validate_scale(&Scale::Uniform(1.0)).is_ok());
        assert!(validate_scale(&Scale::Uniform(0.5)).is_ok());
        assert!(validate_scale(&Scale::Uniform(10.0)).is_ok());
    }

    #[test]
    fn test_validate_scale_uniform_invalid() {
        assert!(validate_scale(&Scale::Uniform(0.0)).is_err());
        assert!(validate_scale(&Scale::Uniform(-1.0)).is_err());
    }

    #[test]
    fn test_validate_scale_non_uniform_valid() {
        assert!(validate_scale(&Scale::NonUniform([1.0, 2.0, 3.0])).is_ok());
        assert!(validate_scale(&Scale::NonUniform([0.1, 0.1, 0.1])).is_ok());
    }

    #[test]
    fn test_validate_scale_non_uniform_invalid() {
        assert!(validate_scale(&Scale::NonUniform([0.0, 1.0, 1.0])).is_err());
        assert!(validate_scale(&Scale::NonUniform([1.0, -1.0, 1.0])).is_err());
        assert!(validate_scale(&Scale::NonUniform([1.0, 1.0, 0.0])).is_err());
    }

    #[test]
    fn test_validate_scale_uniform_expression_valid() {
        assert!(validate_scale(&Scale::UniformExpression("t * 4 + 1".to_string())).is_ok());
        assert!(validate_scale(&Scale::UniformExpression("1 + sin(t * PI) * 0.5".to_string())).is_ok());
    }

    #[test]
    fn test_validate_scale_uniform_expression_invalid() {
        let result = validate_scale(&Scale::UniformExpression("invalid syntax".to_string()));
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidExpression(_)) => {}
            _ => panic!("Expected InvalidExpression error"),
        }
    }

    #[test]
    fn test_validate_scale_per_axis_valid() {
        let scale = Scale::PerAxis(AnimatedScale {
            x: AnimatedValue::Expression("t * 2 + 1".to_string()),
            y: AnimatedValue::Static(1.0),
            z: AnimatedValue::Expression("1 + sin(t * PI)".to_string()),
        });
        assert!(validate_scale(&scale).is_ok());
    }

    #[test]
    fn test_validate_scale_per_axis_invalid_expression() {
        let scale = Scale::PerAxis(AnimatedScale {
            x: AnimatedValue::Expression("invalid".to_string()),
            y: AnimatedValue::Static(1.0),
            z: AnimatedValue::Static(1.0),
        });
        let result = validate_scale(&scale);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidExpression(_)) => {}
            _ => panic!("Expected InvalidExpression error"),
        }
    }
}
