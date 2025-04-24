use crate::{
    bindings::{
        self,
        types::{GLenum, GLint, GLuint},
    },
    opengl::{self, GLIndexType, GLVertexType},
    raw,
    vao::VertexArrayObject,
    NonZeroUInt,
};
use std::{ffi::c_void, marker::PhantomData};

use crate::raw::buffers::{self, BufferTarget};

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

#[derive(Debug)]
struct Guard {
    pub inner: NonZeroUInt,
}

impl Drop for Guard {
    fn drop(&mut self) {
        unsafe { buffers::DeleteBuffer(self.inner.into()) }
    }
}

/// This VertexBuffer is meant for fast updates
#[repr(C)]
#[derive(Debug, Clone)]
pub struct VertexBufferDynamicData<V: GLVertexType, const L: GLint> {
    pub layout: Layout<V, L>,
    // This VertexBuffer type stores data on the cpu side
    pub data: Vec<u8>,
}
impl<V: GLVertexType, const L: GLint> VertexBufferDynamicData<V, L> {
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
pub struct VertexBufferDynamic<V: GLVertexType, const L: GLint> {
    data: VertexBufferDynamicData<V, L>,
    // OpenGL id
    id: Guard,
}
impl<V: GLVertexType, const L: GLint> VertexBufferDynamic<V, L> {
    /// Convert's your data into a useable OpenGL object
    pub fn from(data: VertexBufferDynamicData<V, L>) -> Self {
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
    /// Converts a dynamic buffer to a static one.
    /// Use this if the buffer stopped being read very frequently
    /// and you want to free the memory associated with it on the cpu side
    pub fn to_static(self) -> VertexBufferStatic<V, L> {
        VertexBufferStatic {
            id: self.id,
            layout: self.data.layout,
        }
    }

    /// Gives you a &mut to the inner data
    /// Unsafe because it gives you the ability to
    /// 'unsync' OpenGL and our local data storage
    /// and corrupt data
    pub unsafe fn as_data_mut(&mut self) -> &mut VertexBufferDynamicData<V, L> {
        &mut self.data
    }

    /// Gives you a reference to the inner data
    pub fn as_data(&self) -> &VertexBufferDynamicData<V, L> {
        &self.data
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VertexBufferStaticData<V: GLVertexType, const L: GLint> {
    // we do need to know how the data is
    // actually formatted though
    pub layout: Layout<V, L>,

    // This VertexBuffer does not store it's data
    // past the time it is first initialized
    // So we store it here, but when it's time
    // to tell OpenGL, it will be dropped
    pub data: Vec<u8>,
}

impl<V: GLVertexType, const L: GLint> VertexBufferStaticData<V, L> {
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
pub struct VertexBufferStatic<V: GLVertexType, const L: GLint> {
    id: Guard,
    pub layout: Layout<V, L>,
}
impl<V: GLVertexType, const L: GLint> VertexBufferStatic<V, L> {
    /// Convert's your data into a useable OpenGL object
    pub fn from(data: VertexBufferStaticData<V, L>) -> Self {
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
    pub fn to_dynamic(self) -> VertexBufferDynamic<V, L> {
        // It should be possible to copy the data
        // from OpenGL to get it back on the cpu side
        // not important now but might be nice later on
        unimplemented!()
    }
}

pub trait VertexBufferT<V: GLVertexType, const L: GLint> {
    /// Get internal gl id
    fn id(&self) -> NonZeroUInt;
    /// Get the layout of the data
    fn layout(&self) -> Layout<V, L>;
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
pub trait IndexBufferT {
    /// Get internal gl id
    fn id(&self) -> NonZeroUInt;
    /// Get the amount of indices stored in the buffer
    fn len(&self) -> usize;
    fn bind(&self) {
        unsafe { bindings::BindBuffer(bindings::ELEMENT_ARRAY_BUFFER, self.id().into()) }
    }
}

impl<V: GLVertexType, const L: GLint> VertexBufferT<V, L> for VertexBufferDynamic<V, L> {
    fn id(&self) -> NonZeroUInt {
        self.id.inner
    }
    fn layout(&self) -> Layout<V, L> {
        self.data.layout
    }
}

impl<V: GLVertexType, const L: GLint> VertexBufferT<V, L> for VertexBufferStatic<V, L> {
    fn id(&self) -> NonZeroUInt {
        self.id.inner
    }
    fn layout(&self) -> Layout<V, L> {
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
