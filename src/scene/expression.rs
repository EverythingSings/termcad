use evalexpr::{context_map, eval_float_with_context, EvalexprError};
use std::f32::consts::{PI, TAU};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExpressionError {
    #[error("Failed to create evaluation context")]
    ContextCreationFailed,

    #[error("Expression evaluation failed: {0}")]
    EvaluationFailed(#[from] EvalexprError),
}

#[derive(Debug, Clone, Copy)]
pub struct ExpressionContext {
    pub t: f32,
    pub frame: u32,
    pub total_frames: u32,
}

impl ExpressionContext {
    pub fn new(frame: u32, total_frames: u32) -> Self {
        let t = if total_frames > 1 {
            frame as f32 / (total_frames - 1) as f32
        } else {
            0.0
        };
        Self {
            t,
            frame,
            total_frames,
        }
    }
}

pub fn evaluate_expression(expr: &str, ctx: &ExpressionContext) -> Result<f32, ExpressionError> {
    let context = context_map! {
        "t" => ctx.t as f64,
        "frame" => ctx.frame as i64,
        "total_frames" => ctx.total_frames as i64,
        "PI" => PI as f64,
        "TAU" => TAU as f64,
    }
    .map_err(|_| ExpressionError::ContextCreationFailed)?;

    // Pre-process expression to handle custom functions
    let processed = preprocess_expression(expr);

    let result = eval_float_with_context(&processed, &context)?;
    Ok(result as f32)
}

fn preprocess_expression(expr: &str) -> String {
    let mut result = expr.to_string();

    // Replace easing functions with their expanded forms
    // ease_in(x) = x^2
    // ease_out(x) = 1 - (1-x)^2
    // ease_in_out(x) = 2*x^2 if x < 0.5, else 1 - (-2*x + 2)^2 / 2

    // For simplicity, we'll handle these as approximations
    // A more robust solution would use a proper expression transformer

    // Replace ease_in_out(t) with a polynomial approximation
    if result.contains("ease_in_out") {
        // Approximate: 3*t^2 - 2*t^3 (smoothstep)
        result = result.replace("ease_in_out(t)", "(3.0 * t * t - 2.0 * t * t * t)");
    }

    // Replace ease_in(t) with t^2
    if result.contains("ease_in") {
        result = result.replace("ease_in(t)", "(t * t)");
    }

    // Replace ease_out(t) with 1 - (1-t)^2
    if result.contains("ease_out") {
        result = result.replace("ease_out(t)", "(1.0 - (1.0 - t) * (1.0 - t))");
    }

    // Add math:: prefix to trig functions for evalexpr compatibility
    // This allows users to write sin(x) instead of math::sin(x)
    for func in ["sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "sqrt", "abs", "floor", "ceil", "round"] {
        let pattern = format!("{}(", func);
        let replacement = format!("math::{}(", func);
        // Only replace if not already prefixed with math::
        if result.contains(&pattern) && !result.contains(&replacement) {
            result = result.replace(&pattern, &replacement);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_expression() {
        let ctx = ExpressionContext::new(15, 30);
        assert!((ctx.t - 0.5172).abs() < 0.01);

        let result = evaluate_expression("t * 360", &ctx).expect("expression should evaluate");
        assert!((result - 186.2).abs() < 1.0);
    }

    #[test]
    fn test_constants() {
        let ctx = ExpressionContext::new(0, 30);
        let result = evaluate_expression("PI", &ctx).expect("PI should evaluate");
        assert!((result - PI).abs() < 0.001);
    }

    #[test]
    fn test_trig() {
        let ctx = ExpressionContext::new(0, 30);
        let result = evaluate_expression("sin(0)", &ctx).expect("sin(0) should evaluate");
        assert!(result.abs() < 0.001);
    }

    #[test]
    fn test_invalid_expression_returns_error() {
        let ctx = ExpressionContext::new(0, 30);
        let result = evaluate_expression("undefined_var + 1", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_syntax_error_returns_error() {
        let ctx = ExpressionContext::new(0, 30);
        let result = evaluate_expression("1 + + 2", &ctx);
        assert!(result.is_err());
    }
}
