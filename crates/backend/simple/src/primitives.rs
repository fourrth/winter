//! This module contains all of the primitive types
//!
//! All primitives implement the [`super::Drawable`]
//! as they are meant to be the targets to convert into
//! when drawing a construct.

use winter_core::{
    bindings::types::GLint,
    opengl::{GLIndexType, GLVertexType},
};

use crate::Drawable;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Component<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> {
    v_data: Box<[V]>,
    c_data: Box<[C]>,
    i_data: Box<[I]>,
}
impl<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> Component<V, I, C, L> {
    #[inline(always)]
    pub fn new(v_data: Box<[V]>, c_data: Box<[C]>, i_data: Box<[I]>) -> Self {
        Self {
            v_data,
            c_data,
            i_data,
        }
    }
    #[inline(always)]
    fn get_vertices(&self) -> &[V] {
        &self.v_data
    }
    #[inline(always)]
    pub fn get_vertices_mut(&mut self) -> &mut [V] {
        &mut self.v_data
    }

    #[inline(always)]
    fn get_colors(&self) -> &[C] {
        &self.c_data
    }
    #[inline(always)]
    pub fn get_color_mut(&mut self) -> &mut [C] {
        &mut self.c_data
    }
    #[inline(always)]
    fn get_indices(&self) -> &[I] {
        &self.i_data
    }
    #[inline(always)]
    pub fn get_indices_mut(&mut self) -> &mut [I] {
        &mut self.i_data
    }

    /// Merge two components together so
    /// they act as one
    pub fn merge(self, other: Self) -> Self {
        let Self {
            v_data,
            c_data,
            i_data,
        } = self;
        v_data.to_vec().extend(other.v_data);
        c_data.to_vec().extend(other.c_data);
        i_data.to_vec().extend(other.i_data);
        Self {
            v_data,
            c_data,
            i_data,
        }
    }
}
impl<V: GLVertexType, I: GLIndexType, C: GLVertexType, const L: GLint> Drawable<V, I, C, L>
    for Component<V, I, C, L>
{
    fn get_vertices(&self) -> &[V] {
        self.get_vertices()
    }
    fn get_colors(&self) -> &[C] {
        self.get_colors()
    }
    fn get_indices(&self) -> &[I] {
        self.get_indices()
    }
}
