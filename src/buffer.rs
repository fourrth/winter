use crate::{
    bindings::{
        self,
        types::{GLenum, GLint, GLuint},
    },
    opengl::{self, GLIndexType, GLVertexType},
    raw,
    vao::VertexArrayObject,
};
use std::{ffi::c_void, num::NonZeroU32};

use crate::raw::buffers::{self, BufferTarget};

// Know that for layout:
// we always normalize
// we only do one attrib per VertexBuffer
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub attrib_loc: GLuint, // layout location in shader
    pub attrib_len: GLint,  // how many elements per vertex: is [1,4]
    pub attrib_type: GLenum,
}

#[derive(Debug)]
struct Guard {
    pub inner: NonZeroU32,
}

impl Drop for Guard {
    fn drop(&mut self) {
        unsafe { buffers::DeleteBuffer(self.inner.into()) }
    }
}

/// This VertexBuffer is meant for fast updates
#[repr(C)]
#[derive(Debug, Clone)]
pub struct VertexBufferDynamicData {
    pub layout: Layout,
    // This VertexBuffer type stores data on the cpu side
    pub data: Vec<u8>,
}
impl VertexBufferDynamicData {
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

/// This is the main VertexBuffer type for dynamic vbs
#[derive(Debug)]
pub struct VertexBufferDynamic {
    data: VertexBufferDynamicData,
    // OpenGL id
    id: Guard,
}
impl VertexBufferDynamic {
    /// Convert's your data into a useable OpenGL object
    pub fn from(data: VertexBufferDynamicData) -> Self {
        let id: NonZeroU32 = unsafe {
            NonZeroU32::new(
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
    /// Converts a dynamic buffer to a static one.
    /// Use this if the buffer stopped being read very frequently
    /// and you want to free the memory associated with it on the cpu side
    pub fn to_static(self) -> VertexBufferStatic {
        VertexBufferStatic {
            id: self.id,
            layout: self.data.layout,
        }
    }

    /// Gives you a &mut to the inner data
    /// Unsafe because it gives you the ability to
    /// 'unsync' OpenGL and our local data storage
    /// and corrupt data
    pub unsafe fn as_data_mut(&mut self) -> &mut VertexBufferDynamicData {
        &mut self.data
    }

    /// Gives you a reference to the inner data
    pub fn as_data(&self) -> &VertexBufferDynamicData {
        &self.data
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VertexBufferStaticData {
    // we do need to know how the data is
    // actually formatted though
    pub layout: Layout,

    // This VertexBuffer does not store it's data
    // past the time it is first initialized
    // So we store it here, but when it's time
    // to tell OpenGL, it will be dropped
    pub data: Vec<u8>,
}

impl VertexBufferStaticData {
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
#[derive(Debug)]
pub struct VertexBufferStatic {
    id: Guard,
    pub layout: Layout,
}
impl VertexBufferStatic {
    /// Convert's your data into a useable OpenGL object
    pub fn from(data: VertexBufferStaticData) -> Self {
        let id = unsafe {
            NonZeroU32::new(
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
    pub fn to_dynamic(self) -> VertexBufferDynamic {
        // It should be possible to copy the data
        // from OpenGL to get it back on the cpu side
        // not important now but might be nice later on
        unimplemented!()
    }
}

pub trait VertexBufferT {
    /// Get internal gl id
    fn id(&self) -> NonZeroU32;
    /// Get the layout of the data
    fn layout(&self) -> Layout;
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
                self.layout().attrib_len,
                self.layout().attrib_type,
                bindings::FALSE,
                0,
                std::ptr::null_mut::<c_void>(),
            );
            bindings::EnableVertexAttribArray(self.layout().attrib_loc);
        }
    }
}
pub trait IndexBufferT {
    /// Get internal gl id
    fn id(&self) -> NonZeroU32;
    /// Get the amount of indices stored in the buffer
    fn len(&self) -> usize;
    fn bind(&self) {
        unsafe { bindings::BindBuffer(bindings::ELEMENT_ARRAY_BUFFER, self.id().into()) }
    }
}

impl VertexBufferT for VertexBufferDynamic {
    fn id(&self) -> NonZeroU32 {
        self.id.inner
    }
    fn layout(&self) -> Layout {
        self.data.layout
    }
}

impl VertexBufferT for VertexBufferStatic {
    fn id(&self) -> NonZeroU32 {
        self.id.inner
    }
    fn layout(&self) -> Layout {
        self.layout
    }
}
#[derive(Debug, Clone)]
pub struct IndexBufferData {
    pub data: Vec<u8>,
    ty: GLenum,
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
impl IndexBufferT for IndexBuffer {
    fn id(&self) -> NonZeroU32 {
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
            NonZeroU32::new(
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
