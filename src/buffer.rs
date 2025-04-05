use crate::bindings;
use std::{ffi::c_void, num::NonZeroU32, ptr};

use crate::raw::buffers::{self, BufferTarget};
#[derive(Clone, Copy)]
pub enum GLType {
    Float,
    Int,
    UInt,
}

impl GLType {
    pub fn get_glenum(self) -> u32 {
        match self {
            GLType::Float => bindings::FLOAT,
            GLType::Int => bindings::INT,
            GLType::UInt => bindings::UNSIGNED_INT,
        }
    }
    pub fn get_size(self) -> i32 {
        // right now all types have the same size
        4i32
        /* match self {
            GLType::Float => 4,
            GLType::Int => 4,
            GLType::UInt => 4,
        } */
    }
}

impl std::fmt::Debug for GLType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            GLType::Float => {
                write!(f, "float")
            }
            GLType::Int => {
                write!(f, "int")
            }
            GLType::UInt => {
                write!(f, "uint")
            }
        }
    }
}

// Know that for layout:
// we always normalize
// we only do one attrib per VertexBuffer
#[derive(Debug, Clone)]
pub struct Layout {
    pub attrib_loc: u32, // layout location in shader
    pub attrib_len: i32, // how many elements per vertex: is [1,4]
    pub attrib_type: GLType,
}
impl Layout {
    /*  pub fn new(
        program: &Program,
        attrib_name: CString,
        attrib_len: usize,
        attrib_type: GLType,
    ) -> Result<Self, String> {
        let attrib_location =
            match unsafe { bindings::GetAttribLocation(program.id, attrib_name.as_ptr()) } {
                -1 => {
                    return Err(format!(
                        "Could not find attrib of name: {}",
                        attrib_name.to_str().unwrap()
                    ));
                }
                val => val as u32,
            };
        Ok(Self {
            attrib_len,
            attrib_type,
        })
    } */
}

#[derive(Debug)]
pub struct Buffer {
    id: Option<NonZeroU32>,
    pub data: Vec<u8>,
}
impl Buffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { id: None, data }
    }
    pub fn setup_buffer(&mut self, target: BufferTarget) {
        unsafe {
            //TODO: get rid of these unwraps for unchecks
            self.id = Some(
                NonZeroU32::new(
                    buffers::CreateBuffer(
                        self.data.as_ptr() as *const c_void,
                        self.data.len() as isize,
                        target,
                    )
                    .unwrap(),
                )
                .unwrap(),
            );
        }
    }
    pub fn bind(&self, target: BufferTarget) {
        if let Some(id) = self.id {
            unsafe { bindings::BindBuffer(target.get_glenum(), id.into()) };
        } else {
            println!("Tried to bind before creating buffer: silently failing");
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        match self.id {
            Some(id) => {
                unsafe { buffers::DeleteBuffer(id.into()) };
            }
            None => {
                // trying to drop a buffer which does not exist
                // is a good way to do nothing...
            }
        }
    }
}
#[derive(Debug)]
pub struct IndexBuffer {
    pub buffer: Buffer,
    pub ty: GLType,
    pub len: u32,
}
impl Clone for IndexBuffer {
    fn clone(&self) -> Self {
        Self::new(self.buffer.data.clone(), self.ty, self.len)
    }
}
impl IndexBuffer {
    pub fn new(data: Vec<u8>, ty: GLType, len: u32) -> Self {
        Self {
            buffer: Buffer::new(data),
            len,
            ty,
        }
    }
    pub fn bind(&self) {
        self.buffer.bind(BufferTarget::ElementArrayBuffer);
    }
}

#[derive(Debug)]
pub struct VertexBuffer {
    pub buffer: Buffer,
    pub layout: Layout,
}

impl Clone for VertexBuffer {
    fn clone(&self) -> Self {
        Self::new(self.buffer.data.clone(), self.layout.clone())
    }
}

impl VertexBuffer {
    pub fn new(data: Vec<u8>, layout: Layout) -> Self {
        Self {
            buffer: Buffer::new(data),
            layout,
        }
    }
    /// Creates the buffer and binds to the current vao
    #[allow(invalid_value)]
    pub fn bind_buffer(&mut self, target: BufferTarget) {
        unsafe {
            // now bind it to the current vao
            self.buffer.bind(target);
            bindings::VertexAttribPointer(
                self.layout.attrib_loc,
                self.layout.attrib_len,
                self.layout.attrib_type.get_glenum(),
                bindings::FALSE,
                0, /* self.layout.attrib_len as i32 * self.layout.attrib_type.get_size() */
                ptr::null_mut::<c_void>(),
            );
            bindings::EnableVertexAttribArray(self.layout.attrib_loc);
        }
    }
}
