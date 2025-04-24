use std::ffi::c_void;

use crate::{
    bindings::types::GLint,
    buffer::{Guard, VertexBuffer},
    opengl::GLVertexType,
    raw::{self, buffers::BufferTarget},
    NonZeroUInt,
};

use super::Layout;

/// This VertexBuffer is meant for fast updates
#[repr(C)]
#[derive(Debug, Clone)]
pub struct DynamicData<V: GLVertexType, const L: GLint> {
    pub layout: Layout<V, L>,
    // This VertexBuffer type stores data on the cpu side
    pub data: Vec<u8>,
}

impl<V: GLVertexType, const L: GLint> DynamicData<V, L> {
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

/// This is the main VertexBuffer type for dynamic vbs
#[derive(Debug)]
pub struct DynamicBuffer<V: GLVertexType, const L: GLint> {
    data: DynamicData<V, L>,
    // OpenGL id
    id: Guard,
}

impl<V: GLVertexType, const L: GLint> DynamicBuffer<V, L> {
    /// Convert's your data into a useable OpenGL object
    pub fn from(data: DynamicData<V, L>) -> Self {
        let id: NonZeroUInt = unsafe {
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
            data,
            id: Guard { inner: id },
        }
    }
    /*
        /// Converts a dynamic buffer to a static one.
        /// Use this if the buffer stopped being read very frequently
        /// and you want to free the memory associated with it on the cpu side
        pub fn to_static(self) -> VertexBufferStatic<V, L> {
            VertexBufferStatic {
                id: self.id,
                layout: self.data.layout,
            }
        }
    */
    /// Gives you a &mut to the inner data
    /// Unsafe because it gives you the ability to
    /// 'unsync' OpenGL and our local data storage
    /// and corrupt data
    pub unsafe fn as_data_mut(&mut self) -> &mut DynamicData<V, L> {
        &mut self.data
    }

    /// Gives you a reference to the inner data
    pub fn as_data(&self) -> &DynamicData<V, L> {
        &self.data
    }
}

impl<V: GLVertexType, const L: GLint> VertexBuffer<V, L> for DynamicBuffer<V, L> {
    fn id(&self) -> NonZeroUInt {
        self.id.inner
    }
    fn layout(&self) -> Layout<V, L> {
        self.data.layout
    }
}
