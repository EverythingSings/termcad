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

#[derive(Debug)]
#[allow(dead_code)]
enum TermcadError {
    InvalidScene(String),
    RenderError(String),
    IoError(String),
    DependencyMissing(String),
}

impl TermcadError {
    fn exit_code(&self) -> u8 {
        match self {
            TermcadError::InvalidScene(_) => 1,
            TermcadError::RenderError(_) => 2,
            TermcadError::IoError(_) => 3,
            TermcadError::DependencyMissing(_) => 4,
        }
    }
}

impl std::fmt::Display for TermcadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermcadError::InvalidScene(msg) => write!(f, "Invalid scene: {}", msg),
            TermcadError::RenderError(msg) => write!(f, "Render error: {}", msg),
            TermcadError::IoError(msg) => write!(f, "IO error: {}", msg),
            TermcadError::DependencyMissing(msg) => write!(f, "Dependency missing: {}", msg),
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
    let scene_str = std::fs::read_to_string(&scene_path)
        .map_err(|e| TermcadError::IoError(format!("Failed to read scene file: {}", e)))?;

    let scene: Scene = serde_json::from_str(&scene_str)
        .map_err(|e| TermcadError::InvalidScene(format!("Parse error: {}", e)))?;

    // Validate scene
    scene
        .validate()
        .map_err(|e| TermcadError::InvalidScene(e.to_string()))?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let stem = scene_path.file_stem().unwrap_or_default();
        if frames_mode {
            PathBuf::from(format!("{}_frames", stem.to_string_lossy()))
        } else {
            PathBuf::from(format!("{}.gif", stem.to_string_lossy()))
        }
    });

    // Render
    if json_output {
        println!(
            "{}",
            serde_json::json!({"status": "rendering", "frame": 0, "total": scene.total_frames()})
        );
    }

    let renderer = render::Renderer::new(&scene)
        .map_err(|e| TermcadError::RenderError(format!("Failed to initialize renderer: {}", e)))?;

    let frames = renderer
        .render_all(json_output)
        .map_err(|e| TermcadError::RenderError(e.to_string()))?;

    if frames_mode {
        // Output PNG frames
        output::write_frames(&output_path, &frames)
            .map_err(|e| TermcadError::IoError(e.to_string()))?;

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

        let size_bytes = output::assemble_gif(&output_path, &frames, scene.fps)
            .map_err(|e| TermcadError::IoError(e.to_string()))?;

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
    let scene_str = std::fs::read_to_string(&scene_path)
        .map_err(|e| TermcadError::IoError(format!("Failed to read scene file: {}", e)))?;

    let scene: Scene = serde_json::from_str(&scene_str)
        .map_err(|e| TermcadError::InvalidScene(format!("Parse error: {}", e)))?;

    scene
        .validate()
        .map_err(|e| TermcadError::InvalidScene(e.to_string()))?;

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
            return Err(TermcadError::InvalidScene(format!(
                "Unknown template: {}. Available: spinning-cube, grid-flythrough, text-terminal",
                name
            )));
        }
    };

    let json = serde_json::to_string_pretty(&scene).unwrap();
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
            return Err(TermcadError::InvalidScene(format!(
                "Unknown primitive: {}",
                name
            )));
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
