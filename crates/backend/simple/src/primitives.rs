//! This module contains all of the primitive types
//!
//! All primitives implement the [`super::Drawable`]
//! as they are meant to be the targets to convert into
//! when drawing a construct.

use std::{collections::HashMap, marker::PhantomData};

use winter_core::{
    buffer::{
        index::IndexBufferDataU32,
        vertex::{DynamicData, Layout},
    },
    opengl::GLVertexType,
};

use crate::{Drawable, IntoDrawable, Mesh};

/// A component is a pos,color, and index mesh with implied
/// layouts on the verts and colors.
/// verts and colors are both GLFloat,atrib len 3, and
/// verts are in loc 0, colors are in loc 1
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Component<V: GLVertexType, C: GLVertexType> {
    mesh: Mesh,
    _vb: PhantomData<V>,
    _cb: PhantomData<C>,
}
impl<V: GLVertexType, C: GLVertexType> Component<V, C> {
    #[inline(always)]
    pub fn new(v_data: Box<[V]>, c_data: Box<[C]>, i_data: Box<[u32]>) -> Self {
        // these copies are not fully required, but actually are because
        // the alignment/mem layout is different between Vec<u8> and Vec<V>
        // the array underneath is the same, but on drop the actual drop is different
        // really we need to carry the type all the way, but thats not how the api for the
        // vertex buffers is right now. TODO: do the big refactor please.

        let v_data_len = v_data.len();
        let c_data_len = c_data.len();

        let v_data_u8 = bytemuck::must_cast_slice::<_, u8>(&v_data).to_vec();

        let c_data_u8 = bytemuck::must_cast_slice::<_, u8>(&c_data).to_vec();

        let v_buffer_data = DynamicData {
            // vertex_count: v_data_len / 3,
            vertex_count: v_data_len / 3,
            data: v_data_u8,
            layout: Layout::new(0, 3, V::to_glenum()),
        };
        let c_buffer_data = DynamicData {
            vertex_count: c_data_len / 3,
            data: c_data_u8,
            layout: Layout::new(1, 3, C::to_glenum()),
        };
        // dbg!(&v_buffer_data);

        let i_buffer_data = IndexBufferDataU32::new(Some(&i_data));

        // let vbs = HashMap::from([(0u32, v_buffer_data), (1u32, c_buffer_data)]);
        let mut vbs = HashMap::new();
        vbs.insert(0u32, v_buffer_data);
        vbs.insert(1u32, c_buffer_data);

        Self::from_buffers(i_buffer_data, vbs)
    }
    #[inline(always)]
    pub fn get_vertices(&self) -> &[V] {
        bytemuck::must_cast_slice(self.mesh.vertex_buffers.get(&0).unwrap().data.as_slice())
    }
    #[inline(always)]
    pub fn get_vertices_mut(&mut self) -> &mut [V] {
        bytemuck::must_cast_slice_mut(
            self.mesh
                .vertex_buffers
                .get_mut(&0)
                .unwrap()
                .data
                .as_mut_slice(),
        )
    }

    #[inline(always)]
    pub fn get_colors(&self) -> &[C] {
        bytemuck::must_cast_slice(self.mesh.vertex_buffers.get(&1).unwrap().data.as_slice())
    }
    #[inline(always)]
    pub fn get_color_mut(&mut self) -> &mut [C] {
        bytemuck::must_cast_slice_mut(
            self.mesh
                .vertex_buffers
                .get_mut(&1)
                .unwrap()
                .data
                .as_mut_slice(),
        )
    }
    #[inline(always)]
    pub fn get_indices(&self) -> &[u32] {
        bytemuck::must_cast_slice(self.mesh.index_buffer.data())
    }
    #[inline(always)]
    pub unsafe fn get_indices_mut(&mut self) -> &mut [u32] {
        bytemuck::must_cast_slice_mut(self.mesh.index_buffer.data_mut().as_mut_slice())
    }

    pub fn to_mesh(self) -> Mesh {
        let (a, b) = self.get_buffers();
        Mesh::from_buffers(a, b)
    }
}
impl<V: GLVertexType, C: GLVertexType> Drawable for Component<V, C> {
    fn from_buffers(
        index_buffers: IndexBufferDataU32,
        vertex_buffers: HashMap<u32, DynamicData>,
    ) -> Self {
        Self {
            mesh: Mesh::from_buffers(index_buffers, vertex_buffers),
            _vb: PhantomData,

            _cb: PhantomData,
        }
    }
    fn get_buffers(self) -> (IndexBufferDataU32, HashMap<u32, DynamicData>) {
        self.mesh.get_buffers()
    }
}
// how can i make this an automatic implmentation for the trait??
impl<V: GLVertexType, C: GLVertexType> IntoDrawable for Component<V, C> {
    type IntoDrawable = Self;
    fn into_drawable(self) -> Self::IntoDrawable {
        self
    }
}
