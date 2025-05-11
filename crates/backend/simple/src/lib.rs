//! This module contains the simple VertexArrayObject implementation
//!
//! # Main Purpose
//! You should use this crate if you believe that you
//! need something simple and quick to use, and don't
//! care too much on how things are actually done
//!
//! # Notes About Primitives and their Function Implementations
//! The main way of creating any of the primitives
//! given here is by invoking the `new#()` function of
//! the associated type.
//! It is important to read the function documentation
//! instead relying solely on the function signature,
//! because many simplifications were made with naming.
//! For example, most `new#()` functions contain
//! directional names like *bottom_left* or *top*.
//! These are only for reference as of course any
//! point can be set anywhere, however note that
//! when drawing or getting the data via the
//! [`Drawable`] trait, you will most likely
//! get the data counter clockwise and back to front
//!
//! Something else to note is that
//! all primitives are repr(C) with the intention
//! that you can cast to the lower vector types if need be,
//! however you should be able to use the Drawable trait
//! if you simply need the shape data.
//! Make sure you know what you're doing and
//! know your alignment requirements

pub mod constructs;
pub mod primitives;
pub mod shapes;
pub mod uniform;
pub mod vao;

pub mod updater;
pub use updater::*;

pub use glmath;
pub use winter_core::vao::VertexArrayObject;

use winter_core::{
    bindings::types::GLint,
    buffer::{index, vertex},
    opengl::{GLIndexType, GLVertexType},
    raw, NonZeroUInt,
};

use std::{fmt::Debug, marker::PhantomData};

//TODO: find a place to put this that makes sense
pub struct IndexGrid<I> {
    pub width: usize,
    pub height: usize,
    pub indices: Vec<I>,
}
impl<I> IndexGrid<I> {
    /// Create new grid
    ///
    /// Returns None if:
    /// `indices.len() != width * height`
    pub fn new(width: usize, height: usize, indices: Vec<I>) -> Option<Self> {
        if indices.len() != width * height {
            None
        } else {
            Some(Self {
                width,
                height,
                indices,
            })
        }
    }
}

#[derive(Debug)]
struct Guard {
    inner: NonZeroUInt,
}
impl Drop for Guard {
    fn drop(&mut self) {
        unsafe { raw::buffers::DeleteVertexArray(self.inner.into()) };
    }
}

/// Trait for all drawable things
///
/// All things that implement drawable
/// are themselves ready to be drawn.
/// This means any higher level types
/// should really be converted into their base
/// representations before implementing [`Drawable`]
pub trait Drawable<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint>:
    Debug
{
    fn get_vertices(&self) -> &[V];

    /// if none, then it is inferred that
    /// your vertices are in GL_TRIANGLES form
    fn get_indices(&self) -> &[I];
    fn get_colors(&self) -> &[C];
}
pub trait IntoDrawable<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> {
    type IntoDrawable: Drawable<V, I, C, L>;
    fn into_drawable(self) -> Self::IntoDrawable;
}

/// Note L is the attrib len for both pos and color
#[derive(Debug)]
pub struct Vao<
    V: GLVertexType,
    I: GLIndexType,
    C: GLVertexType,
    const L: GLint,
    const N: bool,
    const M: u32,
> {
    id: Guard,
    position_vb: vertex::DynamicBuffer<V, L, N>,
    color_vb: vertex::DynamicBuffer<C, 3, N>,
    index_buffer: index::IndexBuffer,

    _pb: PhantomData<V>,
    _ib: PhantomData<I>,
    _cb: PhantomData<C>,
}
