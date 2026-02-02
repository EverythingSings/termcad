use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrameWriteError {
    #[error("Failed to create directory: {0}")]
    DirectoryError(String),

    #[error("Failed to write frame: {0}")]
    WriteError(String),
}

pub fn write_frames(
    output_dir: &Path,
    frames: &[image::RgbaImage],
) -> Result<(), FrameWriteError> {
    // Create output directory
    std::fs::create_dir_all(output_dir)
        .map_err(|e| FrameWriteError::DirectoryError(e.to_string()))?;

    let num_digits = (frames.len() as f32).log10().ceil() as usize;

    for (i, frame) in frames.iter().enumerate() {
        let filename = format!("frame_{:0width$}.png", i, width = num_digits);
        let path = output_dir.join(filename);

        frame
            .save(&path)
            .map_err(|e| FrameWriteError::WriteError(format!("{}: {}", path.display(), e)))?;
    }

    Ok(())
}
