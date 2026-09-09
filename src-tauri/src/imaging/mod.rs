//! Pure image processing: decoding, pixel helpers, encoders, thumbnails.
//! (Named `imaging` so it does not shadow the `image` crate.)

pub mod decode;
pub mod encode;
pub mod pixels;
pub mod resize;
pub mod thumbnail;
pub mod transform;
