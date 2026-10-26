use crate::bindings::types::{GLenum, GLint, GLuint};

mod dynamic;
pub use dynamic::*;

mod r#static;
pub use r#static::*;

// Know that for layout:
// we always normalize
// we only do one attrib per VertexBuffer
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Layout {
    pub attrib_loc: GLuint, // layout location in shader
    pub attrib_len: GLint,
    pub ty: GLenum,
}

impl Layout {
    pub fn new(attrib_loc: GLuint, attrib_len: GLint, ty: GLenum) -> Self {
        Self {
            attrib_loc,
            attrib_len,
            ty,
        }
    }
}
