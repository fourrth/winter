use crate::{
    bindings::types::GLenum,
    opengl::{self, GLIndexType},
    raw::{self, buffers::BufferTarget},
    NonZeroUInt,
};

use std::ffi::c_void;

use super::{ElementArrayBuffer, Guard};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexBufferDataU32 {
    data: Vec<u32>,
    ty: GLenum,
    max_index: u32,
}

impl IndexBufferDataU32 {
    /// data is whatever you want to put in a VertexBuffer
    pub fn new(data: Option<&[u32]>) -> Self {
        if let Some(data_) = data {
            let max_index = *data_.iter().max().unwrap_or(&0u32);

            Self {
                data: data_.to_vec(),
                ty: u32::to_glenum(),
                max_index,
            }
        } else {
            Self {
                data: vec![],
                ty: u32::to_glenum(),
                max_index: 0,
            }
        }
    }
    pub fn merge(self, other: IndexBufferDataU32) -> Self {
        let Self {
            mut data,
            ty,
            max_index,
        } = self;
        let Self {
            data: other_data,
            ty: other_ty,
            max_index: other_max_index,
        } = other;

        debug_assert_eq!(ty, other_ty);

        if other_data.is_empty() {
            return Self {
                data,
                ty,
                max_index,
            };
        }

        // Only shift when `self` actually has vertices; an empty buffer has no
        // "next free" vertex index to offset against.
        let offset = if data.is_empty() { 0 } else { max_index + 1 };
        let new_max = other_max_index
            .checked_add(offset)
            .expect("index buffer overflow during merge");

        data.reserve(other_data.len());
        data.extend(other_data.into_iter().map(|i| i + offset));

        Self {
            data,
            ty,
            max_index: new_max,
        }
    }
    pub fn len(&self) -> usize {
        self.data.len() / opengl::get_size(self.ty).unwrap()
    }

    pub fn data(&self) -> &[u32] {
        &self.data
    }

    pub fn ty(&self) -> u32 {
        self.ty
    }

    pub unsafe fn data_mut(&mut self) -> &mut Vec<u32> {
        &mut self.data
    }
}

#[derive(Debug)]
pub struct IndexBuffer {
    id: Guard,
    data: IndexBufferDataU32,
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
    pub fn new(data: IndexBufferDataU32) -> Self {
        let id = unsafe {
            NonZeroUInt::new(
                raw::buffers::CreateBuffer(
                    data.data.as_ptr() as *const c_void,
                    (data.data.len() * std::mem::size_of::<u32>()) as isize,
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
    pub fn index_type(&self) -> GLenum {
        self.data.ty
    }
}
