use crate::{
    bindings::{self, types::GLint},
    opengl::GLVertexType,
    vao::VertexArrayObject,
    NonZeroUInt,
};
use std::ffi::c_void;

use crate::raw::buffers::{self};

pub mod index;
pub mod vertex;

#[derive(Debug)]
struct Guard {
    pub inner: NonZeroUInt,
}

impl Drop for Guard {
    fn drop(&mut self) {
        unsafe { buffers::DeleteBuffer(self.inner.into()) }
    }
}

pub trait VertexBuffer<V: GLVertexType, const L: GLint> {
    /// Get internal gl id
    fn id(&self) -> NonZeroUInt;
    /// Get the layout of the data
    fn layout(&self) -> vertex::Layout<V, L>;
    fn bind(&self) {
        unsafe {
            bindings::BindBuffer(bindings::ARRAY_BUFFER, self.id().into());
        }
    }
    /// Binds the buffer to the specified VertexArrayObject
    fn bind_to_vao<VAO: VertexArrayObject>(&self, vao: &VAO) {
        vao.bind();
        self.bind();

        unsafe {
            bindings::VertexAttribPointer(
                self.layout().attrib_loc,
                L,
                V::to_glenum(),
                bindings::FALSE,
                0,
                std::ptr::null_mut::<c_void>(),
            );
            bindings::EnableVertexAttribArray(self.layout().attrib_loc);
        }
    }
}
pub trait ElementArrayBuffer {
    /// Get internal gl id
    fn id(&self) -> NonZeroUInt;
    /// Get the amount of indices stored in the buffer
    fn len(&self) -> usize;
    fn bind(&self) {
        unsafe { bindings::BindBuffer(bindings::ELEMENT_ARRAY_BUFFER, self.id().into()) }
    }
}
