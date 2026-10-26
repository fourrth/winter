use std::ffi::c_void;

use bytemuck::Pod;

use crate::{
    buffer::{Guard, VertexBuffer},
    opengl::GLVertexType,
    raw::{self, buffers::BufferTarget},
    NonZeroUInt,
};

use super::Layout;

/// This VertexBuffer is meant for fast updates
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DynamicData {
    pub vertex_count: usize,
    pub layout: Layout,
    // This VertexBuffer type stores data on the cpu side
    pub data: Vec<u8>,
}

impl DynamicData {
    /// data is whatever you want to put in a VertexBuffer
    pub fn new<T: GLVertexType>(data: Option<&[T]>, layout: Layout) -> Self {
        if let Some(data_) = data {
            Self {
                // vertex_count: data_.len() / layout.attrib_len as usize,
                vertex_count: data_.len() / layout.attrib_len as usize,
                data: bytemuck::must_cast_slice::<T, u8>(data_).to_vec(),
                layout,
            }
        } else {
            Self {
                vertex_count: 0,
                data: vec![],
                layout,
            }
        }
    }

    pub fn new2<T: Pod>(data: Option<&[T]>, vertex_count: usize, layout: Layout) -> Self {
        if let Some(data_) = data {
            let data_b = bytemuck::must_cast_slice::<T, u8>(data_).to_vec();
            Self {
                vertex_count: vertex_count,
                data: data_b,
                layout,
            }
        } else {
            Self {
                vertex_count: 0,
                data: vec![],
                layout,
            }
        }
    }
}

/// This is the main VertexBuffer type for dynamic vbs
#[derive(Debug)]
pub struct DynamicBuffer {
    data: DynamicData,
    // OpenGL id
    id: Guard,
}

impl DynamicBuffer {
    /// Convert's your data into a useable OpenGL object
    pub fn new(data: DynamicData) -> Self {
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
        pub fn to_static(self) -> VertexBufferStatic {
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
    pub unsafe fn as_data_mut(&mut self) -> &mut DynamicData {
        &mut self.data
    }

    /// Gives you a reference to the inner data
    pub fn as_data(&self) -> &DynamicData {
        &self.data
    }
}

impl VertexBuffer for DynamicBuffer {
    fn id(&self) -> NonZeroUInt {
        self.id.inner
    }
    fn layout(&self) -> Layout {
        self.data.layout
    }
}
