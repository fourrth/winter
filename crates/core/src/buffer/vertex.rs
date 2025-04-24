use crate::{
    bindings::types::{GLint, GLuint},
    opengl::GLVertexType,
};

use std::marker::PhantomData;

mod dynamic;
pub use dynamic::*;

mod stati;
pub use stati::*;

// Know that for layout:
// we always normalize
// we only do one attrib per VertexBuffer
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Layout<V: GLVertexType, const L: GLint> {
    pub attrib_loc: GLuint, // layout location in shader
    _v: PhantomData<V>,
}

impl<V: GLVertexType, const L: GLint> Layout<V, L> {
    pub fn new(attrib_loc: GLuint) -> Self {
        Self {
            attrib_loc,
            _v: PhantomData,
        }
    }
}
