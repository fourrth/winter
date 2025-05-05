use winter_core::uniform::Uniform;

pub trait GLUniform<T>: Uniform<T> + From<i32> + Sized {
    fn new(id: i32) -> Option<Self> {
        if id == -1 {
            None
        } else {
            Some(Self::from(id))
        }
    }
}

//TODO: add more uniform implementations
// and uniform block implementation
mod scalar;
pub use scalar::*;

mod matrix;
pub use matrix::*;

mod vector;
pub use vector::*;
