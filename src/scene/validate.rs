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
}
