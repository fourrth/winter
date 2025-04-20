//! This is the library for my opengl application

#[cfg(not(feature = "raw"))]
pub mod buffer;
#[cfg(not(feature = "raw"))]
pub mod shader;
#[cfg(not(feature = "raw"))]
pub mod vao;

pub mod opengl;
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
pub mod raw;

#[cfg(not(feature = "raw"))]
pub mod context;

#[cfg(not(feature = "raw"))]
pub use context::*;
#[cfg(not(feature = "raw"))]
pub mod common;

#[cfg(target_pointer_width = "64")]
pub type NonZeroUInt = std::num::NonZeroU32;

#[cfg(target_pointer_width = "32")]
pub type NonZeroUInt = std::num::NonZeroU16;
