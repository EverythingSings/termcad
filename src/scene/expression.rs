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

/// Extracts comma-separated arguments from a function call, handling nested parentheses.
/// `start` should point to the opening parenthesis.
/// Returns (arguments, end_position) where end_position is the index after the closing paren.
fn extract_function_args(expr: &str, start: usize) -> Option<(Vec<String>, usize)> {
    let bytes = expr.as_bytes();
    if start >= bytes.len() || bytes[start] != b'(' {
        return None;
    }

    let mut depth = 1;
    let mut args = Vec::new();
    let mut current_arg_start = start + 1;
    let mut pos = start + 1;

    while pos < bytes.len() && depth > 0 {
        match bytes[pos] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    // Capture final argument
                    let arg = expr[current_arg_start..pos].trim().to_string();
                    if !arg.is_empty() {
                        args.push(arg);
                    }
                }
            }
            b',' if depth == 1 => {
                // Split argument at depth 1
                let arg = expr[current_arg_start..pos].trim().to_string();
                args.push(arg);
                current_arg_start = pos + 1;
            }
            _ => {}
        }
        pos += 1;
    }

    if depth == 0 {
        Some((args, pos))
    } else {
        None // Unbalanced parentheses
    }
}

/// Checks if the character at the given index is a valid identifier character.
fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Finds all occurrences of `func_name(...)` and applies transformer to replace them.
/// Only matches whole function names (not substrings like "step" in "smoothstep").
fn replace_function<F>(expr: &str, func_name: &str, transformer: F) -> String
where
    F: Fn(&[String]) -> String,
{
    let mut result = expr.to_string();
    let pattern = format!("{}(", func_name);

    // Keep replacing until no more matches.
    // Start from the beginning each iteration to handle nested cases properly.
    loop {
        let mut found_valid_match = false;
        let mut search_start = 0;

        while let Some(rel_idx) = result[search_start..].find(&pattern) {
            let start_idx = search_start + rel_idx;

            // Check for word boundary - don't match "step" inside "smoothstep"
            if start_idx > 0 && is_ident_char(result.as_bytes()[start_idx - 1]) {
                // This is a substring match, skip past it
                search_start = start_idx + 1;
                continue;
            }

            let args_start = start_idx + func_name.len();
            if let Some((args, end_pos)) = extract_function_args(&result, args_start) {
                let replacement = transformer(&args);
                result.replace_range(start_idx..end_pos, &replacement);
                found_valid_match = true;
                break; // Restart from beginning to handle nested cases
            } else {
                // Malformed function call, skip past it
                search_start = start_idx + 1;
            }
        }

        if !found_valid_match {
            break;
        }
    }

    result
}

/// Transform if_else(cond, true_val, false_val) to evalexpr's if(cond, true_val, false_val)
fn transform_if_else(args: &[String]) -> String {
    if args.len() != 3 {
        // Return as-is for evalexpr to report the error
        return format!("if_else({})", args.join(", "));
    }
    format!("if({}, {}, {})", args[0], args[1], args[2])
}

/// Transform step(x, edge) to return 0 if x < edge, else 1
fn transform_step(args: &[String]) -> String {
    if args.len() != 2 {
        return format!("step({})", args.join(", "));
    }
    format!("if(({}) < ({}), 0.0, 1.0)", args[0], args[1])
}

/// Transform smoothstep(x, edge0, edge1) to Hermite interpolation with clamping
/// Uses a simplified approach that avoids deep expression nesting
fn transform_smoothstep(args: &[String]) -> String {
    if args.len() != 3 {
        return format!("smoothstep({})", args.join(", "));
    }
    let (x, e0, e1) = (&args[0], &args[1], &args[2]);
    // For simple cases (x is just a variable), we can use a more efficient form
    // ratio = (x - edge0) / (edge1 - edge0)
    // When edge0=0 and edge1=1, this simplifies to just x
    // result = if(x<e0, 0, if(x>e1, 1, hermite))
    //
    // To avoid expression explosion, we use conditional branches that short-circuit:
    // if x < edge0: 0
    // else if x > edge1: 1
    // else: compute hermite on (x-e0)/(e1-e0)
    let ratio = format!("(({x}) - ({e0})) / (({e1}) - ({e0}))");
    // Hermite polynomial: t^2 * (3 - 2t) where t is the ratio
    // We compute this only in the else branch to avoid duplication issues
    let hermite = format!(
        "(({ratio}) * ({ratio}) * (3.0 - 2.0 * ({ratio})))"
    );
    format!(
        "if(({x}) < ({e0}), 0.0, if(({x}) > ({e1}), 1.0, {hermite}))"
    )
}

/// Transform ease_in(expr) to expr^2
fn transform_ease_in(args: &[String]) -> String {
    if args.len() != 1 {
        return format!("ease_in({})", args.join(", "));
    }
    format!("(({}) * ({}))", args[0], args[0])
}

/// Transform ease_out(expr) to 1 - (1-expr)^2
fn transform_ease_out(args: &[String]) -> String {
    if args.len() != 1 {
        return format!("ease_out({})", args.join(", "));
    }
    format!("(1.0 - (1.0 - ({})) * (1.0 - ({})))", args[0], args[0])
}

/// Transform ease_in_out(expr) to smoothstep polynomial: 3*x^2 - 2*x^3
fn transform_ease_in_out(args: &[String]) -> String {
    if args.len() != 1 {
        return format!("ease_in_out({})", args.join(", "));
    }
    let x = &args[0];
    format!("(3.0 * ({}) * ({}) - 2.0 * ({}) * ({}) * ({}))", x, x, x, x, x)
}

fn preprocess_expression(expr: &str) -> String {
    let mut result = expr.to_string();

    // Replace conditional helper functions (order matters for nested calls)
    result = replace_function(&result, "if_else", transform_if_else);
    result = replace_function(&result, "step", transform_step);
    result = replace_function(&result, "smoothstep", transform_smoothstep);

    // Replace easing functions with their expanded forms
    // Now using the generic replacer to support arbitrary expressions, not just `t`
    result = replace_function(&result, "ease_in_out", transform_ease_in_out);
    result = replace_function(&result, "ease_in", transform_ease_in);
    result = replace_function(&result, "ease_out", transform_ease_out);

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

    // Argument extraction tests
    #[test]
    fn test_extract_simple_args() {
        let expr = "(a, b, c)";
        let (args, end) = extract_function_args(expr, 0).unwrap();
        assert_eq!(args, vec!["a", "b", "c"]);
        assert_eq!(end, 9);
    }

    #[test]
    fn test_extract_nested_args() {
        let expr = "(foo(1, 2), bar(3))";
        let (args, end) = extract_function_args(expr, 0).unwrap();
        assert_eq!(args, vec!["foo(1, 2)", "bar(3)"]);
        assert_eq!(end, 19);
    }

    #[test]
    fn test_extract_args_with_spaces() {
        let expr = "(  a  ,  b  )";
        let (args, _) = extract_function_args(expr, 0).unwrap();
        assert_eq!(args, vec!["a", "b"]);
    }

    #[test]
    fn test_extract_unbalanced_returns_none() {
        let expr = "(a, b";
        assert!(extract_function_args(expr, 0).is_none());
    }

    // if_else tests
    #[test]
    fn test_if_else_basic_true() {
        let ctx = ExpressionContext::new(0, 30); // t = 0
        let result = evaluate_expression("if_else(t < 0.5, 1.0, 0.0)", &ctx).unwrap();
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_if_else_basic_false() {
        let ctx = ExpressionContext::new(29, 30); // t = 1
        let result = evaluate_expression("if_else(t < 0.5, 1.0, 0.0)", &ctx).unwrap();
        assert!((result - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_if_else_with_expressions() {
        let ctx = ExpressionContext::new(15, 30); // t ≈ 0.517
        // When t < 0.5, returns t * 2; otherwise returns (1 - t) * 2
        let result = evaluate_expression("if_else(t < 0.5, t * 2, (1 - t) * 2)", &ctx).unwrap();
        // t > 0.5, so result = (1 - 0.517) * 2 ≈ 0.966
        assert!((result - 0.966).abs() < 0.01);
    }

    // step tests
    #[test]
    fn test_step_below_edge() {
        let ctx = ExpressionContext::new(0, 30); // t = 0
        let result = evaluate_expression("step(t, 0.5)", &ctx).unwrap();
        assert!((result - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_step_at_edge() {
        let ctx = ExpressionContext::new(15, 30); // t ≈ 0.517, at/above edge 0.5
        let result = evaluate_expression("step(t, 0.5)", &ctx).unwrap();
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_step_above_edge() {
        let ctx = ExpressionContext::new(29, 30); // t = 1
        let result = evaluate_expression("step(t, 0.5)", &ctx).unwrap();
        assert!((result - 1.0).abs() < 0.001);
    }

    // smoothstep tests
    #[test]
    fn test_smoothstep_at_zero() {
        let ctx = ExpressionContext::new(0, 30); // t = 0
        let result = evaluate_expression("smoothstep(t, 0.0, 1.0)", &ctx).unwrap();
        assert!((result - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_smoothstep_at_one() {
        let ctx = ExpressionContext::new(29, 30); // t = 1
        let result = evaluate_expression("smoothstep(t, 0.0, 1.0)", &ctx).unwrap();
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_smoothstep_at_middle() {
        let ctx = ExpressionContext::new(15, 30); // t ≈ 0.517
        let result = evaluate_expression("smoothstep(t, 0.0, 1.0)", &ctx).unwrap();
        // smoothstep(0.517) = 0.517^2 * (3 - 2*0.517) ≈ 0.526
        assert!(result > 0.4 && result < 0.6);
    }

    // Nested conditional tests
    #[test]
    fn test_nested_if_else() {
        let ctx = ExpressionContext::new(15, 30); // t ≈ 0.517
        let result = evaluate_expression(
            "if_else(t < 0.33, 0.0, if_else(t < 0.66, 0.5, 1.0))",
            &ctx,
        )
        .unwrap();
        // t is between 0.33 and 0.66, so result = 0.5
        assert!((result - 0.5).abs() < 0.001);
    }

    // Improved easing function tests (now work with arbitrary expressions)
    #[test]
    fn test_ease_in_with_expression() {
        let ctx = ExpressionContext::new(15, 30); // t ≈ 0.517
        let result = evaluate_expression("ease_in(t * 2 - 1)", &ctx).unwrap();
        // t * 2 - 1 ≈ 0.034, squared ≈ 0.001
        assert!(result.abs() < 0.01);
    }

    #[test]
    fn test_ease_out_with_expression() {
        let ctx = ExpressionContext::new(15, 30);
        let result = evaluate_expression("ease_out(t)", &ctx).unwrap();
        // ease_out(0.517) = 1 - (1-0.517)^2 ≈ 0.767
        assert!(result > 0.7 && result < 0.8);
    }

    #[test]
    fn test_ease_in_out_with_expression() {
        let ctx = ExpressionContext::new(15, 30);
        let result = evaluate_expression("ease_in_out(t)", &ctx).unwrap();
        // Should be close to t at t=0.5
        assert!(result > 0.4 && result < 0.6);
    }
}
