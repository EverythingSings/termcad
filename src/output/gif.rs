use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GifError {
    #[error("ffmpeg not found. Please install ffmpeg and ensure it's in your PATH")]
    FfmpegNotFound,

    #[error("Failed to create temp directory: {0}")]
    TempDirError(String),

    #[error("Failed to write frame: {0}")]
    FrameWriteError(String),

    #[error("ffmpeg failed: {0}")]
    FfmpegError(String),

    #[error("Failed to read output file: {0}")]
    OutputReadError(String),
}

pub fn assemble_gif(
    output_path: &Path,
    frames: &[image::RgbaImage],
    fps: u32,
) -> Result<u64, GifError> {
    // Check if ffmpeg is available
    let ffmpeg_check = Command::new("ffmpeg").arg("-version").output();

    if ffmpeg_check.is_err() {
        return Err(GifError::FfmpegNotFound);
    }

    // Create temp directory for frames
    let temp_dir = std::env::temp_dir().join(format!("termcad_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| GifError::TempDirError(e.to_string()))?;

    // Write frames as PNGs
    let num_digits = (frames.len() as f32).log10().ceil() as usize;
    for (i, frame) in frames.iter().enumerate() {
        let filename = format!("frame_{:0width$}.png", i, width = num_digits);
        let path = temp_dir.join(&filename);

        frame
            .save(&path)
            .map_err(|e| GifError::FrameWriteError(e.to_string()))?;
    }

    // Build ffmpeg command
    let frame_pattern = temp_dir.join(format!("frame_%0{}d.png", num_digits));

    // Use a high-quality palette for better GIF output
    let palette_path = temp_dir.join("palette.png");

    // Generate palette
    let palette_result = Command::new("ffmpeg")
        .args([
            "-y",
            "-framerate",
            &fps.to_string(),
            "-i",
            frame_pattern.to_str().unwrap(),
            "-vf",
            "palettegen=stats_mode=full",
            palette_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| GifError::FfmpegError(e.to_string()))?;

    if !palette_result.status.success() {
        let stderr = String::from_utf8_lossy(&palette_result.stderr);
        return Err(GifError::FfmpegError(format!(
            "Palette generation failed: {}",
            stderr
        )));
    }

    // Generate GIF with palette
    let output_result = Command::new("ffmpeg")
        .args([
            "-y",
            "-framerate",
            &fps.to_string(),
            "-i",
            frame_pattern.to_str().unwrap(),
            "-i",
            palette_path.to_str().unwrap(),
            "-lavfi",
            "paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle",
            "-loop",
            "0",
            output_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| GifError::FfmpegError(e.to_string()))?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(GifError::FfmpegError(format!("GIF creation failed: {}", stderr)));
    }

    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Get file size
    let metadata = std::fs::metadata(output_path)
        .map_err(|e| GifError::OutputReadError(e.to_string()))?;

    Ok(metadata.len())
}
