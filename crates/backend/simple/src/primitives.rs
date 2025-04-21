//! This module contains all of the primitive types
//!
//! All primitives implement the [`super::Drawable`]
//! as they are meant to be the targets to convert into
//! when drawing a construct.

use std::marker::PhantomData;

use winter_core::opengl::{GLIndexType, GLVertexType};

use crate::Drawable;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Component<V: GLVertexType, I: GLIndexType, C: GLVertexType> {
    data: Box<[u8]>,

    // indices for indexing into data
    // it is stored by what Drawable is,
    // but is the layout is:
    // vertices: [0,c_index)
    // color: [c_index,i_index)
    // indices: [i_index,data.len())
    c_index: usize,
    i_index: usize,

    _v: PhantomData<V>,
    _i: PhantomData<I>,
    _c: PhantomData<C>,
}
impl<V: GLVertexType, I: GLIndexType, C: GLVertexType> Component<V, I, C> {
    //TODO: fix this, but later because I am tired of looking at it
    // basically rust has bad support for unsafe iterator stuff, so
    // we can't just take in iterators if we want optimizations
    // rust does have stuff like ExactSizeIterator and the nightly Trustlen,
    // but they are not perfect (in fact, some I impl the above two
    // still does not always do what it should optimization wise, so screw them:
    // we allocate who cares)

    // also, this will get better later on when we bundle the shader stuff together
    // then we can do SSBOs and maybe some 1D texture stuff:
    // it will make this a whole lot simpler
    #[inline(always)]
    pub fn new(v_data: Box<[V]>, c_data: Box<[C]>, i_data: Box<[I]>) -> Self {
        let v_size = std::mem::size_of_val::<[_]>(v_data.as_ref());
        let c_size = std::mem::size_of_val::<[_]>(c_data.as_ref());
        let i_size = std::mem::size_of_val::<[_]>(i_data.as_ref());

        let mut data: Vec<u8> = Vec::with_capacity(v_size + c_size + i_size);

        data.extend(bytemuck::cast_slice::<_, u8>(&v_data));
        data.extend(bytemuck::cast_slice::<_, u8>(&c_data));
        data.extend(bytemuck::cast_slice::<_, u8>(&i_data));

        // LOG POINT
        // let vertex_data_tmp_compare = bytemuck::cast_slice::<u8, [V; 3]>(&data[0..v_size]);
        // let color_data_tmp_compare =
        //     bytemuck::cast_slice::<u8, [C; 3]>(&data[v_size..v_size + c_size]);
        // let index_data_tmp_compare = bytemuck::cast_slice::<u8, [I; 6]>(&data[v_size + c_size..]);

        // std::hint::black_box((
        //     vertex_data_tmp_compare,
        //     color_data_tmp_compare,
        //     index_data_tmp_compare,
        // ));

        Self {
            data: data.into_boxed_slice(),
            c_index: v_size,
            i_index: v_size + c_size,
            _v: PhantomData,
            _i: PhantomData,
            _c: PhantomData,
        }
    }
    #[inline(always)]
    fn get_vertices(&self) -> &[V] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const V,
                self.c_index / std::mem::size_of::<V>(),
            )
        }
    }
    #[inline(always)]
    pub fn get_vertices_mut(&mut self) -> &mut [V] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_ptr() as *mut V,
                self.c_index / std::mem::size_of::<V>(),
            )
        }
    }

    #[inline(always)]
    fn get_colors(&self) -> &[C] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr().add(self.c_index) as *const C,
                (self.i_index - self.c_index) / std::mem::size_of::<C>(),
            )
        }
    }
    #[inline(always)]
    pub fn get_color_mut(&mut self) -> &mut [C] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_ptr().add(self.c_index) as *mut C,
                (self.i_index - self.c_index) / std::mem::size_of::<C>(),
            )
        }
    }
    #[inline(always)]
    fn get_indices(&self) -> &[I] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr().add(self.i_index) as *const I,
                (self.data.len() - self.i_index) / std::mem::size_of::<I>(),
            )
        }
    }
    #[inline(always)]
    pub fn get_indices_mut(&mut self) -> &mut [I] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_ptr().add(self.i_index) as *mut I,
                (self.data.len() - self.i_index) / std::mem::size_of::<I>(),
            )
        }
    }

    /// Merge two components together so
    /// they act as one
    pub fn merge(self, mut other: Self) -> Self {
        // This is a pretty naive way of doing it
        // but the faster way of doing it extends
        // very well to generalized buffers
        // (dynamic buffer layouts with some type knowledge
        // of each buffer type), so I'll just
        // update it when I update everything else
        //
        // also, in the future this could do index optimizations
        // or maybe that could be separate because that
        // gets confusing unless you know exactly what
        // kind of data you are getting and have shader control
        //
        // also also, maybe in the future we could have an
        // arbitrary merge_n component function which would
        // also be a bit faster for merging many components
        // i'm not very worried about that right now though

        // note weird stuff can happen if
        // other's vertices and color are nothing (there aren't any)
        // but that's not something to worry about now,
        // as we strictly control how components are created right now
        //TODO: worry about this, but later

        // We're doing the same index shifting thing from
        // the builder here. I don't know what to do about it
        // and how it should actually be implemented.
        // I feel like a lot of things can be made up of pieces,
        // and each separate piece can be broken apart and added,
        // or merged into larger pieces
        {
            let indices = other.get_indices_mut();

            // VERY IMPORTANT TODO HERE
            //TODO: make component know about buffer layouts
            // currently, component knows nothing about buffer layouts
            // and this is below is simply hardcoded as 3 (each point is a vec3)
            // this needs to change, along with pretty much
            // everything about layouts. The whole idea of layouts
            // can be done via const generics and associated functions
            // we literally already do it with the C::gl_enum stuff,
            // just need to do more.
            // However,... that can be done another time as it's really
            // not important. We will never not use vector 3s for the time being.

            let len: usize = self.get_vertices().len() / (3 as usize);

            for ca in indices {
                *ca = *ca + I::from_usize(len);
            }
        }

        let (mut vec, self_c_index, self_i_index) =
            (self.data.into_vec(), self.c_index, self.i_index);
        let initial_vec_len = vec.len();

        vec.reserve_exact(other.data.len() - (vec.capacity() - initial_vec_len));
        unsafe { vec.set_len(vec.capacity()) };
        let new_c_index = self_c_index + other.c_index;
        let new_i_index = self_i_index + other.i_index;

        unsafe {
            {
                let src = other.get_indices();
                let size = std::mem::size_of_val::<[_]>(src);
                let dst = vec.as_mut_ptr().add(vec.len()).sub(size);
                std::ptr::copy_nonoverlapping(src.as_ptr() as *const u8, dst, size);
            }
            // move up indices in vec up to copied in

            {
                let src = vec.as_mut_ptr().add(self_i_index);
                let dst = vec.as_mut_ptr().add(new_i_index);
                std::ptr::copy_nonoverlapping(src, dst, initial_vec_len - self_i_index);
            }

            // now repeat for color
            {
                let src = other.get_colors();
                let size = std::mem::size_of_val::<[_]>(src);
                let dst = vec.as_mut_ptr().add(new_i_index).sub(size);
                std::ptr::copy_nonoverlapping(src.as_ptr() as *const u8, dst, size);
            }
            {
                let src = vec.as_mut_ptr().add(self_c_index);
                let dst = vec.as_mut_ptr().add(new_c_index);
                std::ptr::copy_nonoverlapping(src, dst, self_i_index - self_c_index);
            }

            // now our vertices
            {
                let src = other.get_vertices();
                let size = std::mem::size_of_val::<[_]>(src);
                let dst = vec.as_mut_ptr().add(new_c_index).sub(size);
                std::ptr::copy_nonoverlapping(src.as_ptr() as *const u8, dst, size);
            }
            // and don't move anything for vertices
            // because we are already there...
        }

        Self {
            data: vec.into_boxed_slice(),
            c_index: new_c_index,
            i_index: new_i_index,
            _v: PhantomData,
            _i: PhantomData,
            _c: PhantomData,
        }
    }
}
impl<V: GLVertexType, I: GLIndexType, C: GLVertexType> Drawable<V, I, C> for Component<V, I, C> {
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
