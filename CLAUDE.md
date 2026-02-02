# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

termcad is a headless CLI tool that renders terminal-aesthetic 3D wireframe scenes to animated GIFs. It uses wgpu for GPU-accelerated rendering and ffmpeg for GIF assembly.

## Build and Test Commands

```bash
cargo build                           # Build
cargo test                            # Run all unit tests
cargo test expression                 # Run tests in a specific module
cargo run -- render examples/spinning_cube.json  # Render scene to GIF
cargo run -- render scene.json --frames -o output_dir  # Output PNG frames for visual inspection
cargo run -- validate scene.json      # Validate scene without rendering
cargo run -- init --template spinning-cube > new_scene.json  # Generate starter scene
```

**Dependencies:** Requires ffmpeg in PATH for GIF assembly.

## Development Standards

### Functional Programming Style

Prefer pure functions and immutability throughout:

- **Pure transformation functions** - Functions like `rotate_x`, `rotate_y`, `rotate_z` in `src/primitives/wireframe.rs` take input and return new values without mutation
- **Immutable data flow** - Scene data flows through the pipeline without modification: parse → validate → render → output
- **Composition over inheritance** - The `Primitive` trait composes vertex generation; each primitive is a standalone transformation
- **No hidden state** - `ExpressionContext` explicitly passes frame/time state rather than using globals

When adding new functionality:
- Write pure functions that take inputs and return outputs
- Avoid `&mut` where possible; prefer returning new values
- Use `Result<T, E>` for fallible operations, not panics
- Compose small functions rather than writing large stateful methods

### Test-Driven Development

**Write tests first.** Before implementing new features:

1. **Unit tests** for pure logic (expressions, geometry generation, color parsing):
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_function_name() {
           let input = ...;
           let result = function_under_test(input);
           assert_eq!(result, expected);
       }
   }
   ```

2. **Visual regression testing** using `--frames` output:
   ```bash
   # Render frames for visual inspection
   cargo run -- render test_scene.json --frames -o test_output
   # Compare specific frames against known-good references
   ```

3. **Validation tests** - The validation layer (`src/scene/validate.rs`) should reject invalid input before rendering. Add validation tests for edge cases.

4. **Integration tests** - Test full render pipeline with minimal scenes in `examples/`.

**Test organization:**
- Unit tests live in the same file as the code they test (`#[cfg(test)]` modules)
- Example scenes in `examples/` serve as integration test fixtures
- Use `cargo run -- validate` to quickly check scene validity without full render

### Documentation

- **Doc comments on public items** - All public functions, structs, and traits need `///` doc comments explaining purpose and usage
- **Module-level docs** - Each module should have `//!` comments at the top explaining its role in the pipeline
- **Inline comments for non-obvious logic** - Especially in shaders and geometry generation
- **Keep examples current** - `examples/*.json` files demonstrate features; update them when adding capabilities

### Error Handling

- Use `thiserror` for error types with meaningful messages
- Propagate errors with `?`; don't unwrap in library code
- Exit codes are semantic (see below); preserve this contract

## Architecture

### Pipeline Flow

1. **Scene parsing** (`src/scene/schema.rs`) - JSON scene files define canvas, camera, elements, and post-processing
2. **Validation** (`src/scene/validate.rs`) - Pure validation functions check all constraints before rendering
3. **Rendering** (`src/render/pipeline.rs`) - wgpu-based headless rendering, one frame at a time
4. **Primitives** (`src/primitives/`) - Each element type implements `Primitive` trait to generate `LineVertex` data
5. **Post-processing** (`src/render/post.rs`, `src/shaders/post.wgsl`) - Bloom, scanlines, chromatic aberration, noise, vignette, CRT curvature
6. **Output** (`src/output/`) - Either PNG frames or ffmpeg-assembled GIF

### Key Abstractions

**Primitive trait** (`src/primitives/mod.rs:19-21`): Pure vertex generation - takes context, returns vertices:
```rust
pub trait Primitive {
    fn vertices(&self, ctx: &ExpressionContext) -> Vec<LineVertex>;
}
```

**AnimatedValue** (`src/scene/schema.rs:237-257`): Supports static values or expressions (e.g., `"t * 360"`). Expressions use `evalexpr` with variables: `t` (0-1 progress), `frame`, `total_frames`, `PI`, `TAU`, and easing functions.

**ExpressionContext** (`src/scene/expression.rs`): Immutable context passed to primitives each frame.

### Scene JSON Structure

```json
{
  "canvas": { "width": 800, "height": 600, "background": "#0a0a0a" },
  "camera": { "position": [5, 5, 5], "target": [0, 0, 0], "fov": 45 },
  "duration": 2.0,
  "fps": 30,
  "elements": [
    { "type": "wireframe", "geometry": "cube", "rotation": { "y": "t * 360" } }
  ],
  "post": { "bloom": 0.3, "scanlines": { "intensity": 0.1, "count": 300 } }
}
```

**Element types:** `grid`, `wireframe`, `glyph`, `line`, `particles`, `axes`

**Wireframe geometries:** `cube`, `sphere`, `torus`, `ico`, `cylinder`

### Exit Codes

- 0: Success
- 1: Invalid scene (validation failed)
- 2: Render error (GPU/shader issues)
- 3: IO error (file read/write)
- 4: Dependency missing (ffmpeg)
