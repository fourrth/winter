use std::ffi::c_void;

use crate::{
    bindings::types::GLint,
    buffer::{Guard, VertexBuffer},
    opengl::GLVertexType,
    raw::{self, buffers::BufferTarget},
    NonZeroUInt,
};

use super::{DynamicBuffer, Layout};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StaticData<V: GLVertexType, const L: GLint> {
    // we do need to know how the data is
    // actually formatted though
    pub layout: Layout<V, L>,

    // This VertexBuffer does not store it's data
    // past the time it is first initialized
    // So we store it here, but when it's time
    // to tell OpenGL, it will be dropped
    pub data: Vec<u8>,
}

impl<V: GLVertexType, const L: GLint> StaticData<V, L> {
    /// data is whatever you want to put in a VertexBuffer
    pub fn new<T: GLVertexType>(data: Option<&[T]>, layout: Layout<V, L>) -> Self {
        if let Some(data_) = data {
            Self {
                data: Vec::from(bytemuck::must_cast_slice::<T, u8>(data_)),
                layout,
            }
        } else {
            Self {
                data: vec![],
                layout,
            }
        }
    }
}

/// VertexBuffer meant for static data with infrequent updates.
#[derive(Debug)]
pub struct StaticBuffer<V: GLVertexType, const L: GLint, const N: bool> {
    id: Guard,
    layout: Layout<V, L>,
}

impl<V: GLVertexType, const L: GLint, const N: bool> StaticBuffer<V, L, N> {
    /// Convert's your data into a useable OpenGL object
    pub fn from(data: StaticData<V, L>) -> Self {
        let id = unsafe {
            NonZeroUInt::new(
                raw::buffers::CreateBuffer(
                    data.data.as_ptr() as *const c_void,
                    data.data.len() as isize,
                    BufferTarget::ArrayBuffer,
                )
                .unwrap(),
            )
            .unwrap()
        };
        Self {
            layout: data.layout,
            id: Guard { inner: id },
        }
    }

    /// Converts static VertexBuffer to a dynamic one
    /// for frequent writes
    pub fn to_dynamic(self) -> DynamicBuffer<V, L, N> {
        // It should be possible to copy the data
        // from OpenGL to get it back on the cpu side
        // not important now but might be nice later on
        unimplemented!()
    }
}

impl<V: GLVertexType, const L: GLint, const N: bool> VertexBuffer<V, L, N>
    for StaticBuffer<V, L, N>
{
    fn id(&self) -> NonZeroUInt {
        self.id.inner
    }
    fn layout(&self) -> Layout<V, L> {
        self.layout
    }
}
