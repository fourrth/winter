//! This module contains marker traits for OpenGL types,
//! along with some OpenGL functions for the types

use bytemuck::Pod;
use num_integer::Integer;

use crate::bindings::types::*;

const OPENGL_TYPES: &[GLenum] = &[
    0x1400, // GL_BYTE
    0x1401, // GL_UNSIGNED_BYTE
    0x1402, // GL_SHORT
    0x1403, // GL_UNSIGNED_SHORT
    0x1404, // GL_INT
    0x1405, // GL_UNSIGNED_INT
    0x1406, // GL_FLOAT
];

const OPENGL_TYPES_SIZES: &[usize] = &[
    std::mem::size_of::<GLbyte>(),   // GL_BYTE
    std::mem::size_of::<GLubyte>(),  // GL_UNSIGNED_BYTE
    std::mem::size_of::<GLshort>(),  // GL_SHORT
    std::mem::size_of::<GLushort>(), // GL_UNSIGNED_SHORT
    std::mem::size_of::<GLint>(),    // GL_INT
    std::mem::size_of::<GLuint>(),   // GL_UNSIGNED_INT
    std::mem::size_of::<GLfloat>(),  // GL_FLOAT
];

/// Returns the size of the GLenum type.
/// Note that you must have a supported
/// OpenGL type in order to use this,
/// otherwise it returns None
pub const fn get_size(gl_type: GLenum) -> Option<usize> {
    match gl_type as usize - 0x1400 {
        0 => Some(OPENGL_TYPES_SIZES[0]),
        1 => Some(OPENGL_TYPES_SIZES[1]),
        2 => Some(OPENGL_TYPES_SIZES[2]),
        3 => Some(OPENGL_TYPES_SIZES[3]),
        4 => Some(OPENGL_TYPES_SIZES[4]),
        5 => Some(OPENGL_TYPES_SIZES[5]),
        6 => Some(OPENGL_TYPES_SIZES[6]),
        _ => None,
    }
}

pub trait GLIndexType: Integer + Pod {
    fn to_glenum() -> GLenum;
    fn from_usize(value: usize) -> Self;
}
impl GLIndexType for GLubyte {
    fn to_glenum() -> GLenum {
        0x1401 // GL_UNSIGNED_BYTE
    }
    fn from_usize(value: usize) -> Self {
        value as Self
    }
}
impl GLIndexType for GLushort {
    fn to_glenum() -> GLenum {
        0x1403 // GL_UNSIGNED_SHORT
    }
    fn from_usize(value: usize) -> Self {
        value as Self
    }
}
impl GLIndexType for GLuint {
    fn to_glenum() -> GLenum {
        0x1405 // GL_UNSIGNED_INT
    }
    fn from_usize(value: usize) -> Self {
        value as Self
    }
}

pub trait GLVertexType: Pod {
    fn to_glenum() -> GLenum;
}
impl GLVertexType for GLbyte {
    fn to_glenum() -> GLenum {
        0x1400 // GL_BYTE
    }
}
impl GLVertexType for GLubyte {
    fn to_glenum() -> GLenum {
        0x1401 // GL_UNSIGNED_BYTE
    }
}
impl GLVertexType for GLshort {
    fn to_glenum() -> GLenum {
        0x1402 // GL_SHORT
    }
}
impl GLVertexType for GLushort {
    fn to_glenum() -> GLenum {
        0x1403 // GL_UNSIGNED_SHORT
    }
}
impl GLVertexType for GLint {
    fn to_glenum() -> GLenum {
        0x1404 // GL_INT
    }
}
impl GLVertexType for GLuint {
    fn to_glenum() -> GLenum {
        0x1405 // GL_UNSIGNED_INT
    }
}
impl GLVertexType for GLfloat {
    fn to_glenum() -> GLenum {
        0x1406 // GL_FLOAT
    }
}
