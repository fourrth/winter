//! This module holds the VertexArrayObject information
//!
//! You can either create your own vao implementation
//! or you can use the preexisting one given in [`default`]

/// Trait for implementing VertexArrayObjects
///
/// Implement this trait on anything that you want
/// to manage the various buffers OpenGL has access to
pub trait VertexArrayObject {
    /// draw should automatically bind the VertexArrayObject
    /// and in general, draw with the mesh
    fn draw(&self);
    fn bind(&self);
}

/// This trait holds the data before telling OpenGL about it
///
/// A type that implements this trait holds the data until
/// it is time to tell OpenGL about it
pub trait VertexArrayObjectData {
    type VAO: VertexArrayObject;
    fn build(self) -> Self::VAO;
}

pub mod default {
    //! This module holds the default implementation for VertexArrayObjects
    //!
    //! This module will most likely be refactored/moved into a
    //! completely separate crate eventually, as it is the main
    //! design principle for this engine section of the crate
    //! to only give the user access to creating their own
    //! OpenGL engines

    use std::{marker::PhantomData, num::NonZeroU32};

    use glmath::vector::Vector3;

    use crate::{
        bindings::{self, types::GLint},
        buffer::{
            IndexBuffer, IndexBufferData, IndexBufferT, Layout, VertexBufferDynamic,
            VertexBufferDynamicData, VertexBufferT,
        },
        common::roll_gl_errors,
        opengl::{GLIndexType, GLVertexType},
        primitives,
        raw::{
            self,
            buffers::{BufferTarget, MapAccess, MapAccessBF},
        },
        Float,
    };

    use super::{VertexArrayObject, VertexArrayObjectData};

    #[derive(Debug)]
    struct Guard {
        inner: NonZeroU32,
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe { raw::buffers::DeleteVertexArray(self.inner.into()) };
        }
    }

    #[derive(Debug)]
    pub struct Vao<P: GLVertexType, I: GLIndexType, C: GLVertexType> {
        id: Guard,
        position_vb: VertexBufferDynamic,
        color_vb: VertexBufferDynamic,
        index_buffer: IndexBuffer,
        /*     static_vertex_buffers: Vec<VertexBufferStatic>,
        dynamic_vertex_buffers: Vec<VertexBufferDynamic>,
        index_buffers: IndexBuffer,*/
        _pb: PhantomData<P>,
        _ib: PhantomData<I>,
        _cb: PhantomData<C>,
    }

    /// Wrapper type for updating your VertexBuffer
    ///
    /// Will push changes to OpenGL when dropped
    pub struct VertexBufferUpdater<'a, V: GLVertexType> {
        id: NonZeroU32,
        inner: &'a mut VertexBufferDynamicData,
        _v: PhantomData<V>,
    }
    //TODO: impl deref for this
    impl<'a, V: GLVertexType> VertexBufferUpdater<'a, V> {
        pub fn from(data: &'a mut VertexBufferDynamicData, id: NonZeroU32) -> Self {
            Self {
                id,
                inner: data,
                _v: PhantomData,
            }
        }
        pub fn data_mut(&mut self) -> &mut [V] {
            bytemuck::cast_slice_mut::<u8, V>(&mut self.inner.data)
        }
        /// Writes your changes to OpenGL.
        ///  No different than simply dropping the Updater
        pub fn write(self) {}
    }
    impl<'a, V: GLVertexType> Drop for VertexBufferUpdater<'a, V> {
        fn drop(&mut self) {
            unsafe {
                // will push the changes to OpenGL
                bindings::BindBuffer(bindings::ARRAY_BUFFER, self.id.into());
                if cfg!(debug_assertions) {
                    // let's check if our src size == buffer size
                    let mut gl_buffer_size = 0i32;
                    bindings::GetBufferParameteriv(
                        bindings::ARRAY_BUFFER,
                        bindings::BUFFER_SIZE,
                        &mut gl_buffer_size as &mut _ as *mut GLint,
                    );
                    assert_eq!(gl_buffer_size as usize, self.inner.data.len());
                }

                let dst = match raw::buffers::MapBufferRange(
                    BufferTarget::ArrayBuffer,
                    0,
                    self.inner.data.len() as isize,
                    MapAccessBF::new()
                        .add(MapAccess::Write)
                        .add(MapAccess::DiscardBuffer),
                ) {
                    Some(val) => val,
                    None => {
                        // then pop gl error
                        roll_gl_errors();
                        panic!()
                    }
                }
                .as_ptr() as *mut u8;

                std::ptr::copy(self.inner.data.as_ptr(), dst, self.inner.data.len());

                raw::buffers::UnmapBuffer(BufferTarget::ArrayBuffer);
                bindings::BindBuffer(bindings::ARRAY_BUFFER, 0);
            }
        }
    }

    impl<'a, P: GLVertexType, I: GLIndexType, C: GLVertexType> Vao<P, I, C> {
        /// This gives you a &mut to the position data.
        /// You can then modify this reference and
        /// when you drop the reference,
        /// it will write the new buffer to OpenGL
        pub fn update_position_component(&'a mut self) -> VertexBufferUpdater<'a, P> {
            let id = self.position_vb.id().into();
            VertexBufferUpdater::from(unsafe { self.position_vb.as_data_mut() }, id)
        }
        /// This gives you a &mut to the color data.
        /// You can then modify this reference and
        /// when you drop the reference,
        /// it will write the new buffer to OpenGL
        pub fn update_color_component(&'a mut self) -> VertexBufferUpdater<'a, C> {
            let id = self.color_vb.id().into();
            VertexBufferUpdater::from(unsafe { self.color_vb.as_data_mut() }, id)
        }
    }
    impl<P: GLVertexType, I: GLIndexType, C: GLVertexType> VertexArrayObject for Vao<P, I, C> {
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
    pub struct BuilderDefault<P: GLVertexType, I: GLIndexType, C: GLVertexType> {
        vertex_data: VertexBufferDynamicData,
        index_data: IndexBufferData,
        color_data: VertexBufferDynamicData,

        _pb: PhantomData<P>,
        _ib: PhantomData<I>,
        _cb: PhantomData<C>,
    }

    impl<P: GLVertexType, I: GLIndexType, C: GLVertexType> BuilderDefault<P, I, C> {
        pub fn create() -> Self {
            Self {
                vertex_data: VertexBufferDynamicData::new::<P>(
                    None,
                    Layout {
                        attrib_len: 3,
                        attrib_type: P::to_glenum(),
                        attrib_loc: 0, // pos is defined as 0
                    },
                ),
                index_data: IndexBufferData::new::<I>(None),
                color_data: VertexBufferDynamicData::new::<P>(
                    None,
                    Layout {
                        attrib_len: 3,
                        attrib_type: C::to_glenum(),
                        attrib_loc: 1, // pos is defined as 0
                    },
                ),

                _pb: PhantomData,
                _ib: PhantomData,
                _cb: PhantomData,
            }
        }
        fn triangle(&mut self, tri: &[u8]) {
            debug_assert_eq!(tri.len(), std::mem::size_of::<primitives::triangle::Data>());

            let len = self.index_data.len() as u32;
            let (p, c) = converter_triangle(tri);

            let i = [0u32, 1u32, 2u32].map(|val| val + len);
            self.vertex_data.data.extend(p);
            self.color_data.data.extend(c);
            self.index_data
                .data
                .extend(bytemuck::must_cast_slice::<u32, u8>(&i));
        }
        fn rectangle(&mut self, rect: &[u8]) {
            for tri in rect.chunks_exact(std::mem::size_of::<primitives::triangle::Data>()) {
                self.triangle(tri);
            }
        }
        pub fn add(mut self, dob: Component) -> Self {
            match dob.component_kind {
                primitives::ComponentKind::Triangle => {
                    self.triangle(&dob.data) //
                }
                primitives::ComponentKind::Rectangle => {
                    self.rectangle(&dob.data) //
                }
                primitives::ComponentKind::RectangularPrism => {
                    for rect in dob
                        .data
                        .chunks_exact(std::mem::size_of::<primitives::rectangle::Data>())
                    {
                        self.rectangle(rect);
                    }
                }
            }
            self
        }
    }

    impl<P: GLVertexType, I: GLIndexType, C: GLVertexType> VertexArrayObjectData
        for BuilderDefault<P, I, C>
    {
        type VAO = Vao<P, I, C>;
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

            let vao: Vao<P, I, C> = Vao {
                id: Guard {
                    inner: NonZeroU32::new(id).unwrap(),
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

    // Keeping component and having it work
    // like this is such a hack, but it's more
    // important to just get something working first.
    // After it's working, we can revamp primitives
    // to be disappeared and we can switch to external
    // vertex object files
    #[derive(Debug, Clone, PartialEq)]
    pub struct Component {
        component_kind: primitives::ComponentKind,
        data: Box<[u8]>,
    }

    impl Component {
        pub fn new<'a>(kind: primitives::ComponentKind, data: &'a [u8]) -> Self {
            Self {
                component_kind: kind,
                data: Box::from(data),
            }
        }
    }

    // old stuff coppied in
    // turns Triangle (not triangle but its tuple) into its components of (pos,color)
    fn converter_triangle<'a>(data: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        data.split_at(std::mem::size_of::<[Vector3<Float>; 3]>())
    }
}
