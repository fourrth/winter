use std::marker::PhantomData;

use winter_core::{
    bindings::{self, types::GLint},
    buffer::{index, vertex, ElementArrayBuffer, VertexBuffer},
    opengl::{GLIndexType, GLVertexType},
    vao::{VertexArrayObject, VertexArrayObjectData},
    NonZeroUInt,
};

use crate::Guard;
use crate::Vao;
use crate::{Drawable, VertexBufferUpdater};

//TODO: create config setup builder for the vao data struct
#[derive(Debug, Clone)]
pub struct Builder<
    V: GLVertexType,
    I: GLIndexType,
    C: GLVertexType,
    const L: GLint,
    const N: bool,
    const M: u32,
> {
    pub vertex_data: vertex::DynamicData<V, L>,
    pub index_data: index::IndexBufferData,
    pub color_data: vertex::DynamicData<C, 3>,

    _pb: PhantomData<V>,
    _ib: PhantomData<I>,
    _cb: PhantomData<C>,
}

impl<
        V: GLVertexType,
        I: GLIndexType,
        C: GLVertexType,
        const L: GLint,
        const N: bool,
        const M: u32,
    > Builder<V, I, C, L, N, M>
{
    pub fn create() -> Self {
        Self {
            vertex_data: vertex::DynamicData::new::<V>(
                None,
                vertex::Layout::new(
                    0, // pos is defined as 0
                ),
            ),
            index_data: index::IndexBufferData::new::<I>(None),
            color_data: vertex::DynamicData::new::<V>(
                None,
                vertex::Layout::new(
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

impl<
        V: GLVertexType,
        I: GLIndexType,
        C: GLVertexType,
        const L: GLint,
        const N: bool,
        const M: u32,
    > VertexArrayObjectData for Builder<V, I, C, L, N, M>
{
    type VAO = Vao<V, I, C, L, N, M>;
    fn build(self) -> Self::VAO {
        let id = unsafe {
            let mut id: u32 = 0;
            bindings::GenVertexArrays(1, &mut id);
            bindings::BindVertexArray(id);
            id
        };

        let position_vb = vertex::DynamicBuffer::from(self.vertex_data);
        let color_vb = vertex::DynamicBuffer::from(self.color_data);
        let index_buffer = index::IndexBuffer::from(self.index_data);

        let vao: Vao<V, I, C, L, N, M> = Vao {
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

impl<
        'a,
        V: GLVertexType,
        I: GLIndexType,
        C: GLVertexType,
        const L: GLint,
        const N: bool,
        const M: u32,
    > Drop for Vao<V, I, C, L, N, M>
{
    fn drop(&mut self) {
        unsafe {
            bindings::BindVertexArray(0);
        }
    }
}
impl<
        'a,
        V: GLVertexType,
        I: GLIndexType,
        C: GLVertexType,
        const L: GLint,
        const N: bool,
        const M: u32,
    > Vao<V, I, C, L, N, M>
{
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
    pub fn update_color_component(&'a mut self) -> VertexBufferUpdater<'a, C, 3> {
        let id = self.color_vb.id().into();
        VertexBufferUpdater::from(unsafe { self.color_vb.as_data_mut() }, id)
    }
}

impl<
        V: GLVertexType,
        I: GLIndexType,
        C: GLVertexType,
        const L: GLint,
        const N: bool,
        const M: u32,
    > VertexArrayObject for Vao<V, I, C, L, N, M>
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
                M,
                self.index_buffer.len() as i32,
                I::to_glenum(),
                std::ptr::null(),
            );
        }
    }
}
