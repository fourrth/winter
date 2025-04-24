use crate::{
    bindings::types::GLenum,
    opengl::{self, GLIndexType},
    raw::{self, buffers::BufferTarget},
    NonZeroUInt,
};

use std::ffi::c_void;

use super::{ElementArrayBuffer, Guard};

#[derive(Debug, Clone)]
pub struct IndexBufferData {
    pub data: Vec<u8>,
    pub ty: GLenum,
}

impl IndexBufferData {
    /// data is whatever you want to put in a VertexBuffer
    pub fn new<T: GLIndexType>(data: Option<&[T]>) -> Self {
        if let Some(data_) = data {
            Self {
                data: Vec::from(bytemuck::must_cast_slice::<T, u8>(data_)),
                ty: T::to_glenum(),
            }
        } else {
            Self {
                data: vec![],
                ty: T::to_glenum(),
            }
        }
    }
    pub fn len(&self) -> usize {
        self.data.len() / opengl::get_size(self.ty).unwrap()
    }
}

#[derive(Debug)]
pub struct IndexBuffer {
    id: Guard,
    data: IndexBufferData,
}

impl ElementArrayBuffer for IndexBuffer {
    fn id(&self) -> NonZeroUInt {
        self.id.inner
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl IndexBuffer {
    /// Converts from your data into an OpenGL type
    pub fn from(data: IndexBufferData) -> Self {
        let id = unsafe {
            NonZeroUInt::new(
                raw::buffers::CreateBuffer(
                    data.data.as_ptr() as *const c_void,
                    data.data.len() as isize,
                    BufferTarget::ElementArrayBuffer,
                )
                .unwrap(),
            )
            .unwrap()
        };
        Self {
            id: Guard { inner: id },
            data: data,
        }
    }
}
