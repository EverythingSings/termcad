use super::{LineVertex, Primitive};
use crate::scene::{parse_hex_color, ExpressionContext, GlyphAnimation, GlyphElement};

pub struct GlyphPrimitive {
    element: GlyphElement,
    base_color: [f32; 4],
}

impl GlyphPrimitive {
    pub fn from_element(element: &GlyphElement) -> Self {
        let base_color = parse_hex_color(&element.color).unwrap_or([0.0, 1.0, 0.25, 1.0]);

        Self {
            element: element.clone(),
            base_color,
        }
    }

    fn get_visible_text(&self, ctx: &ExpressionContext) -> &str {
        match self.element.animation {
            GlyphAnimation::None => &self.element.text,
            GlyphAnimation::Type => {
                let total_chars = self.element.text.len();
                let visible_chars = ((ctx.t * total_chars as f32).floor() as usize).min(total_chars);
                &self.element.text[..visible_chars]
            }
            GlyphAnimation::Flicker => &self.element.text,
        }
    }

    fn get_opacity(&self, ctx: &ExpressionContext) -> f32 {
        // Evaluate base opacity from AnimatedValue
        let base_opacity = self.element.opacity.evaluate(ctx).clamp(0.0, 1.0);

        match self.element.animation {
            GlyphAnimation::None | GlyphAnimation::Type => base_opacity,
            GlyphAnimation::Flicker => {
                // Simple flicker based on frame
                let flicker = ((ctx.frame as f32 * 7.3).sin() * 0.5 + 0.5) * 0.3 + 0.7;
                base_opacity * flicker
            }
        }
    }
}

impl Primitive for GlyphPrimitive {
    fn vertices(&self, ctx: &ExpressionContext) -> Vec<LineVertex> {
        let text = self.get_visible_text(ctx);
        let opacity = self.get_opacity(ctx);
        let color = [
            self.base_color[0],
            self.base_color[1],
            self.base_color[2],
            opacity,
        ];

        let mut vertices = Vec::new();
        let char_width = self.element.font_size * 0.6;
        let char_height = self.element.font_size;

        // Calculate starting position to center text
        let total_width = text.len() as f32 * char_width;
        let start_x = self.element.position[0] - total_width / 2.0;

        for (i, ch) in text.chars().enumerate() {
            let x = start_x + i as f32 * char_width;
            let y = self.element.position[1];
            let z = self.element.position[2];

            // Generate simple line-based character representation
            let char_lines = get_char_lines(ch, char_width, char_height);

            for line in char_lines {
                vertices.push(LineVertex::new(
                    [x + line.0[0], y + line.0[1], z],
                    color,
                ));
                vertices.push(LineVertex::new(
                    [x + line.1[0], y + line.1[1], z],
                    color,
                ));
            }
        }

        vertices
    }
}

// Simple vector font - returns line segments for each character
fn get_char_lines(ch: char, w: f32, h: f32) -> Vec<([f32; 2], [f32; 2])> {
    let w = w * 0.8; // Character width with spacing
    let h2 = h / 2.0;

    match ch.to_ascii_uppercase() {
        'A' => vec![
            ([0.0, 0.0], [w / 2.0, h]),
            ([w / 2.0, h], [w, 0.0]),
            ([w * 0.2, h2], [w * 0.8, h2]),
        ],
        'B' => vec![
            ([0.0, 0.0], [0.0, h]),
            ([0.0, h], [w * 0.7, h]),
            ([w * 0.7, h], [w, h * 0.8]),
            ([w, h * 0.8], [w, h * 0.6]),
            ([w, h * 0.6], [w * 0.7, h2]),
            ([0.0, h2], [w * 0.7, h2]),
            ([w * 0.7, h2], [w, h * 0.4]),
            ([w, h * 0.4], [w, h * 0.2]),
            ([w, h * 0.2], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [0.0, 0.0]),
        ],
        'C' => vec![
            ([w, h * 0.8], [w * 0.5, h]),
            ([w * 0.5, h], [0.0, h * 0.7]),
            ([0.0, h * 0.7], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w * 0.5, 0.0]),
            ([w * 0.5, 0.0], [w, h * 0.2]),
        ],
        'D' => vec![
            ([0.0, 0.0], [0.0, h]),
            ([0.0, h], [w * 0.6, h]),
            ([w * 0.6, h], [w, h * 0.7]),
            ([w, h * 0.7], [w, h * 0.3]),
            ([w, h * 0.3], [w * 0.6, 0.0]),
            ([w * 0.6, 0.0], [0.0, 0.0]),
        ],
        'E' => vec![
            ([w, h], [0.0, h]),
            ([0.0, h], [0.0, 0.0]),
            ([0.0, 0.0], [w, 0.0]),
            ([0.0, h2], [w * 0.7, h2]),
        ],
        'F' => vec![
            ([w, h], [0.0, h]),
            ([0.0, h], [0.0, 0.0]),
            ([0.0, h2], [w * 0.7, h2]),
        ],
        'G' => vec![
            ([w, h * 0.8], [w * 0.5, h]),
            ([w * 0.5, h], [0.0, h * 0.7]),
            ([0.0, h * 0.7], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w * 0.5, 0.0]),
            ([w * 0.5, 0.0], [w, h * 0.3]),
            ([w, h * 0.3], [w, h2]),
            ([w, h2], [w * 0.5, h2]),
        ],
        'H' => vec![
            ([0.0, 0.0], [0.0, h]),
            ([w, 0.0], [w, h]),
            ([0.0, h2], [w, h2]),
        ],
        'I' => vec![
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
            ([w * 0.5, 0.0], [w * 0.5, h]),
            ([w * 0.3, h], [w * 0.7, h]),
        ],
        'J' => vec![
            ([w * 0.3, h], [w * 0.7, h]),
            ([w * 0.5, h], [w * 0.5, h * 0.2]),
            ([w * 0.5, h * 0.2], [w * 0.3, 0.0]),
            ([w * 0.3, 0.0], [0.0, h * 0.2]),
        ],
        'K' => vec![
            ([0.0, 0.0], [0.0, h]),
            ([w, h], [0.0, h2]),
            ([0.0, h2], [w, 0.0]),
        ],
        'L' => vec![
            ([0.0, h], [0.0, 0.0]),
            ([0.0, 0.0], [w, 0.0]),
        ],
        'M' => vec![
            ([0.0, 0.0], [0.0, h]),
            ([0.0, h], [w / 2.0, h2]),
            ([w / 2.0, h2], [w, h]),
            ([w, h], [w, 0.0]),
        ],
        'N' => vec![
            ([0.0, 0.0], [0.0, h]),
            ([0.0, h], [w, 0.0]),
            ([w, 0.0], [w, h]),
        ],
        'O' => vec![
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [w, h * 0.3]),
            ([w, h * 0.3], [w, h * 0.7]),
            ([w, h * 0.7], [w * 0.7, h]),
            ([w * 0.7, h], [w * 0.3, h]),
            ([w * 0.3, h], [0.0, h * 0.7]),
            ([0.0, h * 0.7], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w * 0.3, 0.0]),
        ],
        'P' => vec![
            ([0.0, 0.0], [0.0, h]),
            ([0.0, h], [w * 0.7, h]),
            ([w * 0.7, h], [w, h * 0.8]),
            ([w, h * 0.8], [w, h * 0.6]),
            ([w, h * 0.6], [w * 0.7, h2]),
            ([w * 0.7, h2], [0.0, h2]),
        ],
        'Q' => vec![
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [w, h * 0.3]),
            ([w, h * 0.3], [w, h * 0.7]),
            ([w, h * 0.7], [w * 0.7, h]),
            ([w * 0.7, h], [w * 0.3, h]),
            ([w * 0.3, h], [0.0, h * 0.7]),
            ([0.0, h * 0.7], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w * 0.3, 0.0]),
            ([w * 0.5, h * 0.3], [w, 0.0]),
        ],
        'R' => vec![
            ([0.0, 0.0], [0.0, h]),
            ([0.0, h], [w * 0.7, h]),
            ([w * 0.7, h], [w, h * 0.8]),
            ([w, h * 0.8], [w, h * 0.6]),
            ([w, h * 0.6], [w * 0.7, h2]),
            ([w * 0.7, h2], [0.0, h2]),
            ([w * 0.5, h2], [w, 0.0]),
        ],
        'S' => vec![
            ([w, h * 0.8], [w * 0.5, h]),
            ([w * 0.5, h], [0.0, h * 0.7]),
            ([0.0, h * 0.7], [w * 0.3, h2]),
            ([w * 0.3, h2], [w * 0.7, h2]),
            ([w * 0.7, h2], [w, h * 0.3]),
            ([w, h * 0.3], [w * 0.5, 0.0]),
            ([w * 0.5, 0.0], [0.0, h * 0.2]),
        ],
        'T' => vec![
            ([0.0, h], [w, h]),
            ([w / 2.0, h], [w / 2.0, 0.0]),
        ],
        'U' => vec![
            ([0.0, h], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w * 0.3, 0.0]),
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [w, h * 0.3]),
            ([w, h * 0.3], [w, h]),
        ],
        'V' => vec![
            ([0.0, h], [w / 2.0, 0.0]),
            ([w / 2.0, 0.0], [w, h]),
        ],
        'W' => vec![
            ([0.0, h], [w * 0.25, 0.0]),
            ([w * 0.25, 0.0], [w / 2.0, h2]),
            ([w / 2.0, h2], [w * 0.75, 0.0]),
            ([w * 0.75, 0.0], [w, h]),
        ],
        'X' => vec![
            ([0.0, h], [w, 0.0]),
            ([w, h], [0.0, 0.0]),
        ],
        'Y' => vec![
            ([0.0, h], [w / 2.0, h2]),
            ([w, h], [w / 2.0, h2]),
            ([w / 2.0, h2], [w / 2.0, 0.0]),
        ],
        'Z' => vec![
            ([0.0, h], [w, h]),
            ([w, h], [0.0, 0.0]),
            ([0.0, 0.0], [w, 0.0]),
        ],
        '0' => vec![
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [w, h * 0.3]),
            ([w, h * 0.3], [w, h * 0.7]),
            ([w, h * 0.7], [w * 0.7, h]),
            ([w * 0.7, h], [w * 0.3, h]),
            ([w * 0.3, h], [0.0, h * 0.7]),
            ([0.0, h * 0.7], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w * 0.3, 0.0]),
            ([w * 0.2, h * 0.2], [w * 0.8, h * 0.8]),
        ],
        '1' => vec![
            ([w * 0.3, h * 0.8], [w * 0.5, h]),
            ([w * 0.5, h], [w * 0.5, 0.0]),
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
        ],
        '2' => vec![
            ([0.0, h * 0.8], [w * 0.3, h]),
            ([w * 0.3, h], [w * 0.7, h]),
            ([w * 0.7, h], [w, h * 0.7]),
            ([w, h * 0.7], [w, h * 0.5]),
            ([w, h * 0.5], [0.0, 0.0]),
            ([0.0, 0.0], [w, 0.0]),
        ],
        '3' => vec![
            ([0.0, h], [w, h]),
            ([w, h], [w * 0.5, h2]),
            ([w * 0.5, h2], [w, h * 0.3]),
            ([w, h * 0.3], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [0.0, 0.0]),
        ],
        '4' => vec![
            ([w * 0.7, 0.0], [w * 0.7, h]),
            ([w * 0.7, h], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w, h * 0.3]),
        ],
        '5' => vec![
            ([w, h], [0.0, h]),
            ([0.0, h], [0.0, h2]),
            ([0.0, h2], [w * 0.7, h2]),
            ([w * 0.7, h2], [w, h * 0.3]),
            ([w, h * 0.3], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [0.0, 0.0]),
        ],
        '6' => vec![
            ([w, h * 0.8], [w * 0.5, h]),
            ([w * 0.5, h], [0.0, h * 0.5]),
            ([0.0, h * 0.5], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w * 0.3, 0.0]),
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [w, h * 0.2]),
            ([w, h * 0.2], [w, h * 0.4]),
            ([w, h * 0.4], [w * 0.7, h2]),
            ([w * 0.7, h2], [0.0, h2]),
        ],
        '7' => vec![
            ([0.0, h], [w, h]),
            ([w, h], [w * 0.3, 0.0]),
        ],
        '8' => vec![
            ([w * 0.3, h2], [0.0, h * 0.7]),
            ([0.0, h * 0.7], [w * 0.3, h]),
            ([w * 0.3, h], [w * 0.7, h]),
            ([w * 0.7, h], [w, h * 0.7]),
            ([w, h * 0.7], [w * 0.7, h2]),
            ([w * 0.7, h2], [w * 0.3, h2]),
            ([w * 0.3, h2], [0.0, h * 0.3]),
            ([0.0, h * 0.3], [w * 0.3, 0.0]),
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [w, h * 0.3]),
            ([w, h * 0.3], [w * 0.7, h2]),
        ],
        '9' => vec![
            ([0.0, h * 0.2], [w * 0.5, 0.0]),
            ([w * 0.5, 0.0], [w, h * 0.5]),
            ([w, h * 0.5], [w, h * 0.7]),
            ([w, h * 0.7], [w * 0.7, h]),
            ([w * 0.7, h], [w * 0.3, h]),
            ([w * 0.3, h], [0.0, h * 0.8]),
            ([0.0, h * 0.8], [0.0, h * 0.6]),
            ([0.0, h * 0.6], [w * 0.3, h2]),
            ([w * 0.3, h2], [w, h2]),
        ],
        ' ' => vec![],
        '-' => vec![
            ([w * 0.2, h2], [w * 0.8, h2]),
        ],
        '_' => vec![
            ([0.0, 0.0], [w, 0.0]),
        ],
        '.' => vec![
            ([w * 0.4, 0.0], [w * 0.6, 0.0]),
            ([w * 0.6, 0.0], [w * 0.6, h * 0.1]),
            ([w * 0.6, h * 0.1], [w * 0.4, h * 0.1]),
            ([w * 0.4, h * 0.1], [w * 0.4, 0.0]),
        ],
        ':' => vec![
            ([w * 0.4, h * 0.2], [w * 0.6, h * 0.2]),
            ([w * 0.6, h * 0.2], [w * 0.6, h * 0.3]),
            ([w * 0.6, h * 0.3], [w * 0.4, h * 0.3]),
            ([w * 0.4, h * 0.3], [w * 0.4, h * 0.2]),
            ([w * 0.4, h * 0.7], [w * 0.6, h * 0.7]),
            ([w * 0.6, h * 0.7], [w * 0.6, h * 0.8]),
            ([w * 0.6, h * 0.8], [w * 0.4, h * 0.8]),
            ([w * 0.4, h * 0.8], [w * 0.4, h * 0.7]),
        ],
        '>' => vec![
            ([0.0, h], [w, h2]),
            ([w, h2], [0.0, 0.0]),
        ],
        '<' => vec![
            ([w, h], [0.0, h2]),
            ([0.0, h2], [w, 0.0]),
        ],
        '/' => vec![
            ([0.0, 0.0], [w, h]),
        ],
        '\\' => vec![
            ([0.0, h], [w, 0.0]),
        ],
        '=' => vec![
            ([w * 0.1, h * 0.6], [w * 0.9, h * 0.6]),
            ([w * 0.1, h * 0.4], [w * 0.9, h * 0.4]),
        ],
        '+' => vec![
            ([w * 0.1, h2], [w * 0.9, h2]),
            ([w * 0.5, h * 0.2], [w * 0.5, h * 0.8]),
        ],
        '*' => vec![
            ([w * 0.1, h2], [w * 0.9, h2]),
            ([w * 0.2, h * 0.2], [w * 0.8, h * 0.8]),
            ([w * 0.2, h * 0.8], [w * 0.8, h * 0.2]),
        ],
        '[' => vec![
            ([w * 0.7, h], [w * 0.3, h]),
            ([w * 0.3, h], [w * 0.3, 0.0]),
            ([w * 0.3, 0.0], [w * 0.7, 0.0]),
        ],
        ']' => vec![
            ([w * 0.3, h], [w * 0.7, h]),
            ([w * 0.7, h], [w * 0.7, 0.0]),
            ([w * 0.7, 0.0], [w * 0.3, 0.0]),
        ],
        '(' => vec![
            ([w * 0.7, h], [w * 0.3, h * 0.7]),
            ([w * 0.3, h * 0.7], [w * 0.3, h * 0.3]),
            ([w * 0.3, h * 0.3], [w * 0.7, 0.0]),
        ],
        ')' => vec![
            ([w * 0.3, h], [w * 0.7, h * 0.7]),
            ([w * 0.7, h * 0.7], [w * 0.7, h * 0.3]),
            ([w * 0.7, h * 0.3], [w * 0.3, 0.0]),
        ],
        _ => vec![
            // Unknown character - draw a box
            ([0.0, 0.0], [w, 0.0]),
            ([w, 0.0], [w, h]),
            ([w, h], [0.0, h]),
            ([0.0, h], [0.0, 0.0]),
        ],
    }
}
