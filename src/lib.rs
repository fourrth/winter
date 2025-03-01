//! This is the library for my opengl application

mod buffer;
mod shader;
mod vao;

pub mod context;
pub use context::*;

pub mod raw;

#[cfg(feature = "common")]
pub mod common;

type Float = f32;
