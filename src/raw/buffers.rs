use crate::bindings::{
    self,
    types::{GLbitfield, GLenum, GLint, GLintptr, GLsizei, GLsizeiptr, GLuint},
    MAP_FLUSH_EXPLICIT_BIT, MAP_INVALIDATE_BUFFER_BIT, MAP_INVALIDATE_RANGE_BIT, MAP_READ_BIT,
    MAP_UNSYNCHRONIZED_BIT, MAP_WRITE_BIT,
};
use std::{ffi::c_void, ptr::NonNull};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BufferTarget {
    ArrayBuffer,
    ElementArrayBuffer,
    CopyWriteBuffer,
}

impl BufferTarget {
    #[inline]
    pub fn get_glenum(self) -> u32 {
        match self {
            BufferTarget::ArrayBuffer => bindings::ARRAY_BUFFER,
            BufferTarget::ElementArrayBuffer => bindings::ELEMENT_ARRAY_BUFFER,
            BufferTarget::CopyWriteBuffer => bindings::COPY_WRITE_BUFFER,
        }
    }
}

impl std::fmt::Debug for BufferTarget {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BufferTarget::ArrayBuffer => {
                write!(f, "Array Buffer")
            }
            BufferTarget::ElementArrayBuffer => {
                write!(f, "Element Array Buffer")
            }
            BufferTarget::CopyWriteBuffer => {
                write!(f, "Copy Write Buffer")
            }
        }
    }
}

#[inline]
pub unsafe fn CreateVertexArray() -> GLuint {
    let mut id: GLuint = 0;
    bindings::GenVertexArrays(1, &mut id);
    bindings::BindVertexArray(id);
    id
}

#[inline]
pub unsafe fn DeleteVertexArray(id: GLuint) {
    bindings::DeleteVertexArrays(1, &id);
}

#[inline]
pub unsafe fn DrawElements(mode: GLenum, count: GLsizei, ty: GLenum, indices: *const c_void) {
    bindings::DrawElements(mode, count, ty, indices);
}

#[inline]
pub unsafe fn CreateBuffer(
    data: *const c_void,
    size: GLsizeiptr,
    target: BufferTarget,
) -> Result<GLuint, String> {
    unsafe {
        let mut id: GLuint = 0;
        bindings::GenBuffers(1, &mut id);
        bindings::BindBuffer(target.get_glenum(), id);
        bindings::BufferData(target.get_glenum(), size, data, bindings::STATIC_DRAW);

        #[cfg(debug_assertions)]
        {
            let mut get_size: GLint = 0;
            bindings::GetBufferParameteriv(
                target.get_glenum(),
                bindings::BUFFER_SIZE,
                std::ptr::from_mut(&mut get_size),
            );
            if get_size != size as GLint {
                bindings::DeleteBuffers(1, std::ptr::from_mut(&mut id));
                return Err(String::from("Incorrect Buffer Size"));
            }

            // unbind to make sure we don't have any problems later on
            bindings::BindBuffer(target.get_glenum(), 0);

            // This is only in debug because you should assume
            // that the buffer is null if you did not bind anything.
        }
        Ok(id)
    }
}

#[inline]
pub unsafe fn DeleteBuffer(id: GLuint) {
    bindings::DeleteBuffers(1, &id)
}

//TODO: make better debug impl that breaks this
//TODO: apart into the used bitfields
#[derive(Debug, Clone, Copy)]
pub enum MapAccess {
    // necessary
    Read,  // GL_MAP_READ_BIT
    Write, // GL_MAP_WRITE_BIT

    // optional
    DiscardRange,   // GL_MAP_INVALIDATE_RANGE_BIT
    DiscardBuffer,  // GL_MAP_INVALIDATE_BUFFER_BIT
    FlushExplicit,  // GL_MAP_FLUSH_EXPLICIT_BIT
    Unsynchronized, // GL_MAP_UNSYNCHRONIZED_BIT
}
pub struct MapAccessBF(pub GLbitfield);
impl MapAccessBF {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }
    #[inline]
    pub const fn add(self, other: MapAccess) -> Self {
        Self(
            self.0
                | match other {
                    MapAccess::Read => MAP_READ_BIT,
                    MapAccess::Write => MAP_WRITE_BIT,

                    MapAccess::DiscardRange => MAP_INVALIDATE_RANGE_BIT,
                    MapAccess::DiscardBuffer => MAP_INVALIDATE_BUFFER_BIT,
                    MapAccess::FlushExplicit => MAP_FLUSH_EXPLICIT_BIT,
                    MapAccess::Unsynchronized => MAP_UNSYNCHRONIZED_BIT,
                },
        )
    }
}

/// This is glMapBufferRange, so it is equivalent in usage
///
/// Returns raw pointer to data as the lifetime of the pointer
/// is tied to the lifetime of the vertex buffer it represents,
/// and eventually the OpenGL context
#[inline]
pub unsafe fn MapBufferRange(
    target: BufferTarget,
    offset: GLintptr,
    length: GLsizeiptr,
    access: MapAccessBF,
) -> Option<NonNull<[u8]>> {
    let data = bindings::MapBufferRange(target.get_glenum(), offset, length, access.0);
    if let Some(ptr) =
        NonNull::new(std::ptr::slice_from_raw_parts_mut(data, length as usize) as *mut [u8])
    {
        Some(ptr)
    } else {
        None
    }
}

#[inline]
pub unsafe fn UnmapBuffer(target: BufferTarget) -> bool {
    if bindings::UnmapBuffer(target.get_glenum()) != 0 {
        true
    } else {
        false
    }
}
