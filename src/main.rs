use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

mod output;
mod primitives;
mod render;
mod scene;

use scene::Scene;

#[derive(Parser)]
#[command(name = "termcad")]
#[command(about = "Terminal CAD aesthetic GIF generator", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a scene to GIF or PNG frames
    Render {
        /// Scene JSON file
        scene: PathBuf,

        /// Output file (GIF) or directory (with --frames)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output PNG frames to directory instead of GIF
        #[arg(long)]
        frames: bool,

        /// Output JSON progress/status
        #[arg(long)]
        json: bool,
    },

    /// Validate a scene file without rendering
    Validate {
        /// Scene JSON file
        scene: PathBuf,
    },

    /// Generate a starter scene
    Init {
        /// Template name (spinning-cube, grid-flythrough, text-terminal)
        #[arg(long)]
        template: Option<String>,
    },

    /// List available primitives and their parameters
    Primitives {
        /// Specific primitive to show details for
        name: Option<String>,
    },

    /// Show tool info and capabilities
    Info {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Render {
            scene,
            output,
            frames,
            json,
        } => cmd_render(scene, output, frames, json),
        Commands::Validate { scene } => cmd_validate(scene),
        Commands::Init { template } => cmd_init(template),
        Commands::Primitives { name } => cmd_primitives(name),
        Commands::Info { json } => cmd_info(json),
    };

    match result {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::from(e.exit_code())
        }
    }
}

use output::{FrameWriteError, GifError};
use render::RenderError;
use scene::ValidationError;
use thiserror::Error;

#[derive(Debug, Error)]
enum TermcadError {
    #[error("Scene validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("Failed to parse scene: {0}")]
    Parse(#[source] serde_json::Error),

    #[error("Render failed: {0}")]
    Render(#[from] RenderError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Gif(#[from] GifError),

    #[error("{0}")]
    FrameWrite(#[from] FrameWriteError),

    #[error("Failed to serialize: {0}")]
    Serialization(#[source] serde_json::Error),

    #[error("Unknown template: {0}. Available: spinning-cube, grid-flythrough, text-terminal")]
    UnknownTemplate(String),

    #[error("Unknown primitive: {0}")]
    UnknownPrimitive(String),
}

impl TermcadError {
    fn exit_code(&self) -> u8 {
        match self {
            TermcadError::Validation(_) | TermcadError::Parse(_) => 1,
            TermcadError::Render(_) => 2,
            TermcadError::Io(_) | TermcadError::FrameWrite(_) => 3,
            TermcadError::Gif(GifError::FfmpegNotFound) => 4,
            TermcadError::Gif(_) => 3,
            TermcadError::Serialization(_) => 5,
            TermcadError::UnknownTemplate(_) | TermcadError::UnknownPrimitive(_) => 1,
        }
    }
}

fn cmd_render(
    scene_path: PathBuf,
    output: Option<PathBuf>,
    frames_mode: bool,
    json_output: bool,
) -> Result<(), TermcadError> {
    // Load and parse scene
    let scene_str = std::fs::read_to_string(&scene_path)?;

    let scene: Scene =
        serde_json::from_str(&scene_str).map_err(TermcadError::Parse)?;

    // Validate scene
    scene.validate()?;

    // Determine output path - default to Videos or Downloads folder
    let output_path = output.unwrap_or_else(|| {
        let stem = scene_path.file_stem().unwrap_or_default();
        let filename = if frames_mode {
            format!("{}_frames", stem.to_string_lossy())
        } else {
            format!("{}.gif", stem.to_string_lossy())
        };

        // Try Videos first, then Downloads, then current directory
        let base_dir = dirs::video_dir()
            .or_else(dirs::download_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        base_dir.join(filename)
    });

    // Render
    if json_output {
        println!(
            "{}",
            serde_json::json!({"status": "rendering", "frame": 0, "total": scene.total_frames()})
        );
    }

    let renderer = render::Renderer::new(&scene)?;
    let frames = renderer.render_all(json_output)?;

    if frames_mode {
        // Output PNG frames
        output::write_frames(&output_path, &frames)?;

        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "status": "complete",
                    "output": output_path.to_string_lossy(),
                    "frames": frames.len()
                })
            );
        } else {
            println!(
                "Wrote {} frames to {}",
                frames.len(),
                output_path.display()
            );
        }
    } else {
        // Assemble GIF
        if json_output {
            println!("{}", serde_json::json!({"status": "assembling"}));
        }

        let size_bytes = output::assemble_gif(&output_path, &frames, scene.fps)?;

        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "status": "complete",
                    "output": output_path.to_string_lossy(),
                    "frames": frames.len(),
                    "size_bytes": size_bytes
                })
            );
        } else {
            println!("Wrote {} ({} frames)", output_path.display(), frames.len());
        }
    }

    Ok(())
}

fn cmd_validate(scene_path: PathBuf) -> Result<(), TermcadError> {
    let scene_str = std::fs::read_to_string(&scene_path)?;

    let scene: Scene =
        serde_json::from_str(&scene_str).map_err(TermcadError::Parse)?;

    scene.validate()?;

    println!("Scene is valid");
    println!("  Canvas: {}x{}", scene.canvas.width, scene.canvas.height);
    println!("  Duration: {}s @ {} fps", scene.duration, scene.fps);
    println!("  Total frames: {}", scene.total_frames());
    println!("  Elements: {}", scene.elements.len());

    Ok(())
}

fn cmd_init(template: Option<String>) -> Result<(), TermcadError> {
    let scene = match template.as_deref() {
        Some("spinning-cube") | None => scene::templates::spinning_cube(),
        Some("grid-flythrough") => scene::templates::grid_flythrough(),
        Some("text-terminal") => scene::templates::text_terminal(),
        Some(name) => {
            return Err(TermcadError::UnknownTemplate(name.to_string()));
        }
    };

    let json = serde_json::to_string_pretty(&scene).map_err(TermcadError::Serialization)?;
    println!("{}", json);
    Ok(())
}

fn cmd_primitives(name: Option<String>) -> Result<(), TermcadError> {
    match name.as_deref() {
        None => {
            println!("Available primitives:");
            println!();
            println!("  grid        Infinite perspective plane");
            println!("  wireframe   Edge-only geometry (cube, sphere, torus, ico, cylinder)");
            println!("  glyph       Monospace text in 3D space");
            println!("  line        Vector path with glow");
            println!("  particles   Scattered point field");
            println!("  axes        XYZ indicator");
            println!();
            println!("Use `termcad primitives <name>` for details on a specific primitive.");
        }
        Some("grid") => {
            println!("grid - Infinite perspective plane");
            println!();
            println!("Parameters:");
            println!("  divisions       Number of grid lines (default: 20)");
            println!("  fade_distance   Distance at which grid fades out (default: 50.0)");
            println!("  color           Hex color (default: \"#00ff41\")");
            println!("  opacity         0.0 to 1.0 (default: 0.5)");
        }
        Some("wireframe") => {
            println!("wireframe - Edge-only geometry");
            println!();
            println!("Parameters:");
            println!("  geometry    Shape: cube, sphere, torus, ico, cylinder");
            println!("  scale       Uniform scale or [x, y, z] (default: 1.0)");
            println!("  color       Hex color (default: \"#00ff41\")");
            println!("  thickness   Line width in pixels (default: 2.0)");
            println!("  position    [x, y, z] (default: [0, 0, 0])");
            println!("  rotation    {{ x, y, z }} in degrees, supports expressions");
        }
        Some("glyph") => {
            println!("glyph - Monospace text in 3D space");
            println!();
            println!("Parameters:");
            println!("  text        Text string to display");
            println!("  font_size   Size in world units (default: 1.0)");
            println!("  position    [x, y, z] (default: [0, 0, 0])");
            println!("  color       Hex color (default: \"#00ff41\")");
            println!("  animation   \"type\", \"flicker\", or \"none\" (default: \"none\")");
        }
        Some("line") => {
            println!("line - Vector path with glow");
            println!();
            println!("Parameters:");
            println!("  points      Array of [x, y, z] coordinates");
            println!("  closed      Connect last point to first (default: false)");
            println!("  thickness   Line width in pixels (default: 2.0)");
            println!("  glow        Glow intensity 0.0-1.0 (default: 0.5)");
            println!("  color       Hex color (default: \"#00ff41\")");
        }
        Some("particles") => {
            println!("particles - Scattered point field");
            println!();
            println!("Parameters:");
            println!("  count       Number of particles (default: 100)");
            println!("  bounds      [x, y, z] extents (default: [10, 10, 10])");
            println!("  size        Particle size in pixels (default: 2.0)");
            println!("  depth_fade  Fade based on depth (default: true)");
            println!("  color       Hex color (default: \"#00ff41\")");
        }
        Some("axes") => {
            println!("axes - XYZ indicator");
            println!();
            println!("Parameters:");
            println!("  length      Axis length (default: 1.0)");
            println!("  colors      {{ x, y, z }} hex colors");
            println!("  position    [x, y, z] (default: [0, 0, 0])");
            println!("  thickness   Line width in pixels (default: 2.0)");
        }
        Some(name) => {
            return Err(TermcadError::UnknownPrimitive(name.to_string()));
        }
    }
    Ok(())
}

fn cmd_info(json: bool) -> Result<(), TermcadError> {
    if json {
        println!(
            "{}",
            serde_json::json!({
                "name": "termcad",
                "version": env!("CARGO_PKG_VERSION"),
                "primitives": ["grid", "wireframe", "glyph", "line", "particles", "axes"],
                "geometries": ["cube", "sphere", "torus", "ico", "cylinder"],
                "post_effects": ["bloom", "scanlines", "chromatic_aberration", "noise", "vignette", "crt_curvature"],
                "output_formats": ["gif", "png"],
                "features": {
                    "animation_expressions": true,
                    "json_output": true,
                    "headless_rendering": true
                }
            })
        );
    } else {
        println!("termcad v{}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Terminal CAD aesthetic GIF generator");
        println!();
        println!("Primitives: grid, wireframe, glyph, line, particles, axes");
        println!("Geometries: cube, sphere, torus, ico, cylinder");
        println!("Post-effects: bloom, scanlines, chromatic_aberration, noise, vignette");
        println!("Output: GIF, PNG frames");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_exit_code() {
        let err = TermcadError::Validation(ValidationError::InvalidDimensions(
            "test".to_string(),
        ));
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_parse_error_exit_code() {
        let json_err = serde_json::from_str::<Scene>("invalid").unwrap_err();
        let err = TermcadError::Parse(json_err);
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_io_error_exit_code() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = TermcadError::Io(io_err);
        assert_eq!(err.exit_code(), 3);
    }

    #[test]
    fn test_ffmpeg_not_found_exit_code() {
        let err = TermcadError::Gif(GifError::FfmpegNotFound);
        assert_eq!(err.exit_code(), 4);
    }

    #[test]
    fn test_gif_error_other_exit_code() {
        let err = TermcadError::Gif(GifError::TempDirError("test".to_string()));
        assert_eq!(err.exit_code(), 3);
    }

    #[test]
    fn test_unknown_template_exit_code() {
        let err = TermcadError::UnknownTemplate("test".to_string());
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_unknown_primitive_exit_code() {
        let err = TermcadError::UnknownPrimitive("test".to_string());
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_validation_error_display() {
        let err = TermcadError::Validation(ValidationError::InvalidColor(
            "'xyz' is not a valid hex color".to_string(),
        ));
        let msg = format!("{}", err);
        assert!(msg.contains("validation failed"));
        assert!(msg.contains("Invalid color"));
    }

    #[test]
    fn test_error_from_validation() {
        let validation_err = ValidationError::InvalidValue("test".to_string());
        let termcad_err: TermcadError = validation_err.into();
        assert!(matches!(termcad_err, TermcadError::Validation(_)));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let termcad_err: TermcadError = io_err.into();
        assert!(matches!(termcad_err, TermcadError::Io(_)));
    }

    #[test]
    fn test_error_from_gif() {
        let gif_err = GifError::FfmpegNotFound;
        let termcad_err: TermcadError = gif_err.into();
        assert!(matches!(termcad_err, TermcadError::Gif(_)));
    }

    #[test]
    fn test_error_from_frame_write() {
        let frame_err = FrameWriteError::DirectoryError("test".to_string());
        let termcad_err: TermcadError = frame_err.into();
        assert!(matches!(termcad_err, TermcadError::FrameWrite(_)));
    }
}
