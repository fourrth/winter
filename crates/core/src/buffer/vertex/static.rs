use std::ffi::c_void;

use crate::{
    buffer::{Guard, VertexBuffer},
    opengl::GLVertexType,
    raw::{self, buffers::BufferTarget},
    NonZeroUInt,
};

use super::{DynamicBuffer, Layout};

#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StaticData {
    // we do need to know how the data is
    // actually formatted though
    pub layout: Layout,

    // This VertexBuffer does not store it's data
    // past the time it is first initialized
    // So we store it here, but when it's time
    // to tell OpenGL, it will be dropped
    pub data: Vec<u8>,
}

impl StaticData {
    /// data is whatever you want to put in a VertexBuffer
    pub fn new<T: GLVertexType>(data: Option<&[T]>, layout: Layout) -> Self {
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
pub struct StaticBuffer {
    id: Guard,
    layout: Layout,
}

impl StaticBuffer {
    /// Convert's your data into a useable OpenGL object
    pub fn from(data: StaticData) -> Self {
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
    pub fn to_dynamic(self) -> DynamicBuffer {
        // It should be possible to copy the data
        // from OpenGL to get it back on the cpu side
        // not important now but might be nice later on
        unimplemented!()
    }
}

impl VertexBuffer for StaticBuffer {
    fn id(&self) -> NonZeroUInt {
        self.id.inner
    }
    fn layout(&self) -> Layout {
        self.layout
    }
}
