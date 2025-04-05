use crate::bindings::{
    self,
    types::{GLint, GLsizeiptr, GLuint},
};
use std::{ffi::c_void, mem::MaybeUninit};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BufferTarget {
    ArrayBuffer,
    ElementArrayBuffer,
}

impl BufferTarget {
    pub fn get_glenum(self) -> u32 {
        match self {
            BufferTarget::ArrayBuffer => bindings::ARRAY_BUFFER,
            BufferTarget::ElementArrayBuffer => bindings::ELEMENT_ARRAY_BUFFER,
        }
    }
}

impl std::fmt::Debug for BufferTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BufferTarget::ArrayBuffer => {
                write!(f, "Array Buffer")
            }
            BufferTarget::ElementArrayBuffer => {
                write!(f, "Element Array Buffer")
            }
        }
    }
}

#[inline]
#[allow(invalid_value)]
pub unsafe fn CreateVertexArray() -> GLuint {
    let mut id: GLuint = MaybeUninit::uninit().assume_init();
    bindings::GenVertexArrays(1, &mut id);
    bindings::BindVertexArray(id);
    id
}

#[inline]
pub unsafe fn DeleteVertexArray(id: GLuint) {
    bindings::DeleteVertexArrays(1, &id);
}

#[inline]
#[allow(invalid_value)]
pub unsafe fn CreateBuffer(
    data: *const c_void,
    size: GLsizeiptr,
    target: BufferTarget,
) -> Result<GLuint, String> {
    unsafe {
        let mut id: GLuint = MaybeUninit::uninit().assume_init();
        bindings::GenBuffers(1, &mut id);
        bindings::BindBuffer(target.get_glenum(), id);
        bindings::BufferData(target.get_glenum(), size, data, bindings::STATIC_DRAW);

        #[cfg(debug_assertions)]
        {
            let mut get_size: GLint = MaybeUninit::uninit().assume_init();
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
