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

pub mod updater;
pub use updater::*;

use winter_core::{
    bindings::{self, types::GLint},
    buffer::{
        IndexBuffer, IndexBufferData, IndexBufferT, Layout, VertexBufferDynamic,
        VertexBufferDynamicData, VertexBufferT,
    },
    opengl::{GLIndexType, GLVertexType},
    raw,
    vao::{VertexArrayObject, VertexArrayObjectData},
    NonZeroUInt,
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
pub struct Vao<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> {
    id: Guard,
    position_vb: VertexBufferDynamic<V, L>,
    color_vb: VertexBufferDynamic<C, L>,
    index_buffer: IndexBuffer,

    _pb: PhantomData<V>,
    _ib: PhantomData<I>,
    _cb: PhantomData<C>,
}

impl<'a, V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> Drop
    for Vao<V, I, C, L>
{
    fn drop(&mut self) {
        unsafe {
            bindings::BindVertexArray(0);
        }
    }
}
impl<'a, V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> Vao<V, I, C, L> {
    /// This gives you a &mut to the position data.
    /// You can then modify this reference and
    /// when you drop the reference,
    /// it will write the new buffer to OpenGL
    pub fn update_position_component(&'a mut self) -> VertexBufferUpdater<'a, V, L> {
        let id = self.position_vb.id().into();
        VertexBufferUpdater::from(unsafe { self.position_vb.as_data_mut() }, id)
    }
    /// This gives you a &mut to the color data.
    /// You can then modify this reference and
    /// when you drop the reference,
    /// it will write the new buffer to OpenGL
    pub fn update_color_component(&'a mut self) -> VertexBufferUpdater<'a, C, L> {
        let id = self.color_vb.id().into();
        VertexBufferUpdater::from(unsafe { self.color_vb.as_data_mut() }, id)
    }
}

impl<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> VertexArrayObject
    for Vao<V, I, C, L>
{
    fn bind(&self) {
        unsafe { bindings::BindVertexArray(self.id.inner.into()) };
    }
    fn draw(&self) {
        // vao is autobound at draw, just like the index_buffer
        self.bind();
        unsafe {
            self.index_buffer.bind();
            bindings::DrawElements(
                bindings::TRIANGLES,
                self.index_buffer.len() as i32,
                I::to_glenum(),
                std::ptr::null(),
            );
        }
    }
}

#[derive(Debug)]
pub struct Builder<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> {
    vertex_data: VertexBufferDynamicData<V, L>,
    index_data: IndexBufferData,
    color_data: VertexBufferDynamicData<C, L>,

    _pb: PhantomData<V>,
    _ib: PhantomData<I>,
    _cb: PhantomData<C>,
}

impl<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> Builder<V, I, C, L> {
    pub fn create() -> Self {
        Self {
            vertex_data: VertexBufferDynamicData::new::<V>(
                None,
                Layout::new(
                    0, // pos is defined as 0
                ),
            ),
            index_data: IndexBufferData::new::<I>(None),
            color_data: VertexBufferDynamicData::new::<V>(
                None,
                Layout::new(
                    1, // color is defined as 1
                ),
            ),

            _pb: PhantomData,
            _ib: PhantomData,
            _cb: PhantomData,
        }
    }
    pub fn add(mut self, drawable: impl Drawable<V, I, C, L>) -> Self {
        // maybe we should just keep track of it instead
        // of doing division every time, but idk
        let len: usize = self.vertex_data.data.len() / L as usize / std::mem::size_of::<V>();

        self.vertex_data
            .data
            .extend(bytemuck::must_cast_slice::<_, u8>(drawable.get_vertices()));
        self.color_data
            .data
            .extend(bytemuck::must_cast_slice::<_, u8>(drawable.get_colors()));

        //TODO: maybe make this smarter so we don't always allocate
        let tmp_data = drawable
            .get_indices()
            .into_iter()
            .map(|&index| I::from_usize(len) + index)
            .collect::<Vec<_>>();

        self.index_data
            .data
            .extend(bytemuck::must_cast_slice::<_, u8>(&tmp_data));

        // LOG POINT

        // let vertex_data_tmp = bytemuck::cast_slice::<u8, [V; 3]>(&self.vertex_data.data);
        // let color_data_tmp = bytemuck::cast_slice::<u8, [C; 3]>(&self.color_data.data);
        // let index_data_tmp = bytemuck::cast_slice::<u8, I>(&self.index_data.data);
        // std::hint::black_box((vertex_data_tmp, color_data_tmp, index_data_tmp));

        self
    }
}

impl<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> VertexArrayObjectData
    for Builder<V, I, C, L>
{
    type VAO = Vao<V, I, C, L>;
    fn build(self) -> Self::VAO {
        let id = unsafe {
            let mut id: u32 = 0;
            bindings::GenVertexArrays(1, &mut id);
            bindings::BindVertexArray(id);
            id
        };

        let position_vb = VertexBufferDynamic::from(self.vertex_data);
        let color_vb = VertexBufferDynamic::from(self.color_data);
        let index_buffer = IndexBuffer::from(self.index_data);

        let vao: Vao<V, I, C, L> = Vao {
            id: Guard {
                inner: NonZeroUInt::new(id).unwrap(),
            },
            position_vb,
            color_vb,
            index_buffer,
            _pb: PhantomData,
            _ib: PhantomData,
            _cb: PhantomData,
        };

        vao.position_vb.bind_to_vao(&vao);
        vao.color_vb.bind_to_vao(&vao);

        vao
    }
}
