mod frames;
mod gif;

pub use frames::{write_frames, FrameWriteError};
pub use gif::{assemble_gif, GifError};
