//! This is the library for my opengl application

mod buffer;
mod shader;
mod vao;

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
pub mod raw;

#[cfg(not(feature = "raw"))]
pub mod context;
#[cfg(not(feature = "raw"))]
pub use context::*;

#[cfg(feature = "common")]
pub mod common;

type Float = f32;
