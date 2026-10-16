//! Video compression on top of the bundled FFmpeg sidecar (`sharpr-ffmpeg`).
//! Pure parts (formats, probe parsing, argument lists) are kept apart from
//! the process and file handling so they can be unit-tested without FFmpeg.

pub mod args;
pub mod ffmpeg;
pub mod formats;
pub mod posters;
pub mod probe;
pub mod workdir;

#[cfg(test)]
pub mod testing;
