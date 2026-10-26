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
    buffer::{
        index::{self, IndexBufferDataU32},
        vertex::{self, DynamicData},
    },
    opengl::GLVertexType,
    raw, NonZeroUInt,
};

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

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
pub struct Guard {
    pub inner: NonZeroUInt,
}
impl Drop for Guard {
    fn drop(&mut self) {
        unsafe { raw::buffers::DeleteVertexArray(self.inner.into()) };
    }
}

/// Trait for all drawable things
pub trait Drawable: Debug + Sized + IntoDrawable {
    fn from_buffers(index: IndexBufferDataU32, vbs: HashMap<u32, DynamicData>) -> Self;
    fn get_buffers(self) -> (IndexBufferDataU32, HashMap<u32, DynamicData>);
    fn merge(self, other: impl Drawable) -> Self {
        let (other_index_buffer, other_vbs) = other.into_drawable().get_buffers();
        assert_ne!(other_vbs.len(), 0);
        let (our_index_buffer, mut our_vbs) = self.get_buffers();

        // merge our vbs with their vbs
        for (other_key, other_buffer) in other_vbs {
            if let Some(val) = our_vbs.get_mut(&other_key) {
                debug_assert_eq!(val.layout, other_buffer.layout);
                val.data.extend_from_slice(&other_buffer.data);
                val.vertex_count = val.vertex_count + other_buffer.vertex_count;
            } else {
                // then we have not had that key yet,
                // so do 0 + other,.. == other
                our_vbs.insert(other_key, other_buffer);
            }
        }

        debug_assert_eq!(our_index_buffer.ty(), other_index_buffer.ty());

        let combined_index = our_index_buffer.merge(other_index_buffer);
        Self::from_buffers(combined_index, our_vbs)
    }
}
pub trait IntoDrawable {
    type IntoDrawable: Drawable;
    fn into_drawable(self) -> Self::IntoDrawable;
}

#[derive(Debug)]
pub struct Vao<V: GLVertexType, C: GLVertexType> {
    pub id: Guard,
    pub position_vb: vertex::DynamicBuffer,
    pub color_vb: vertex::DynamicBuffer,
    pub index_buffer: index::IndexBuffer,
    pub _pb: PhantomData<V>,
    pub _cb: PhantomData<C>,
}

// does the key need to be u32? or can it be real generic
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mesh {
    pub vertex_buffers: HashMap<u32, vertex::DynamicData>,
    pub index_buffer: index::IndexBufferDataU32,
}

#[derive(Debug, Clone)]
pub struct VaoSimple<V: GLVertexType, C: GLVertexType> {
    pub vertex_data: DynamicData,
    pub index_data: IndexBufferDataU32,
    pub color_data: DynamicData,

    _pb: PhantomData<V>,
    _cb: PhantomData<C>,
}

#[derive(Debug)]
pub struct VaoGeneric {
    id: Guard,
    pub vertex_buffers: HashMap<u32, vertex::DynamicBuffer>,
    pub index_buffer: index::IndexBuffer,
}
