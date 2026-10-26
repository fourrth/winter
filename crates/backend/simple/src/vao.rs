use std::{collections::HashMap, marker::PhantomData};

use winter_core::{
    bindings::{self, types::GLsizei},
    buffer::{
        index::{IndexBuffer, IndexBufferDataU32},
        vertex::{DynamicBuffer, DynamicData, Layout},
        ElementArrayBuffer, VertexBuffer,
    },
    opengl::{GLIndexType, GLVertexType},
    raw,
    vao::{VertexArrayObject, VertexArrayObjectData},
    NonZeroUInt,
};

use crate::{Drawable, VaoGeneric, VaoSimple, VertexBufferUpdater};
use crate::{Guard, Mesh};
use crate::{IntoDrawable, Vao};

impl<V: GLVertexType, C: GLVertexType> Drop for Vao<V, C> {
    fn drop(&mut self) {
        unsafe {
            bindings::BindVertexArray(0);
            // note guard will drop after this point so it gets deleted then
        }
    }
}
impl<V: GLVertexType, C: GLVertexType> VertexArrayObject for Vao<V, C> {
    fn bind(&self) {
        unsafe {
            self.index_buffer.bind();
            bindings::BindVertexArray(self.id.inner.into())
        };
    }
    fn draw(&self) {
        // vao is autobound at draw, just like the index_buffer
        self.bind();
        unsafe {
            self.index_buffer.bind();
            bindings::DrawElements(
                bindings::TRIANGLES,
                (self.index_buffer.len() * std::mem::size_of::<u32>()) as GLsizei,
                <u32 as GLIndexType>::to_glenum(),
                std::ptr::null(),
            );
        }
    }
}

impl<'a, V: GLVertexType, C: GLVertexType> Vao<V, C> {
    /// This gives you a &mut to the position data.
    /// You can then modify this reference and
    /// when you drop the reference,
    /// it will write the new buffer to OpenGL
    pub fn update_position_component(&'a mut self) -> VertexBufferUpdater<'a, V> {
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

impl VertexArrayObject for VaoGeneric {
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
                (std::mem::size_of::<u32>() * self.index_buffer.len()) as GLsizei,
                self.index_buffer.index_type(),
                std::ptr::null(),
            );
        }
    }
}

impl Mesh {
    pub fn empty() -> Mesh {
        Mesh {
            vertex_buffers: HashMap::new(),
            index_buffer: IndexBufferDataU32::new(None),
        }
    }
    pub fn new(into_drawable: impl IntoDrawable) -> Self {
        Self::empty().add(into_drawable)
    }
    pub fn add(self, into_drawable: impl IntoDrawable) -> Self {
        let (index_buffer, vertex_buffers) = into_drawable.into_drawable().get_buffers();
        let m = Mesh {
            vertex_buffers,
            index_buffer,
        };
        self.merge(m)
    }
    /// update or add a certain material into the mesh
    pub fn update_material(self, key: u32, material: &DynamicData) -> Self {
        let Self {
            mut vertex_buffers,
            index_buffer,
        } = self;

        if let Some(mat) = vertex_buffers.get_mut(&key) {
            debug_assert_eq!(material.layout, mat.layout);
            mat.data.extend(material.data.clone());
        } else {
            let _ = vertex_buffers.insert(key, material.clone());
        }
        Self {
            vertex_buffers,
            index_buffer,
        }
    }
    /// update the material at key. if the material did not exist, then returns Err(Mesh)
    pub fn update_material_data(self, key: u32, material_data: &Vec<u8>) -> Result<Self, Self> {
        let Self {
            mut vertex_buffers,
            index_buffer,
        } = self;

        if let Some(mat) = vertex_buffers.get_mut(&key) {
            mat.data.extend(material_data);

            Ok(Self {
                vertex_buffers,
                index_buffer,
            })
        } else {
            Err(Self {
                vertex_buffers,
                index_buffer,
            })
        }
    }
    /// removes material at key if any exists
    pub fn remove_material(self, key: u32) -> Self {
        let Self {
            mut vertex_buffers,
            index_buffer,
        } = self;
        let _ = vertex_buffers.remove(&key);
        Self {
            vertex_buffers,
            index_buffer,
        }
    }
}
impl Drawable for Mesh {
    fn from_buffers(
        index: IndexBufferDataU32,
        vbs: std::collections::HashMap<u32, DynamicData>,
    ) -> Self {
        Self {
            vertex_buffers: vbs,
            index_buffer: index,
        }
    }
    fn get_buffers(
        self,
    ) -> (
        IndexBufferDataU32,
        std::collections::HashMap<u32, DynamicData>,
    ) {
        let Self {
            vertex_buffers,
            index_buffer,
        } = self;
        (index_buffer, vertex_buffers)
    }
}
impl IntoDrawable for Mesh {
    type IntoDrawable = Self;
    fn into_drawable(self) -> Self::IntoDrawable {
        self
    }
}
impl VertexArrayObjectData for Mesh {
    type VAO = VaoGeneric;
    fn build(self) -> Self::VAO {
        let inner = unsafe {
            let mut id: u32 = 0;
            bindings::GenVertexArrays(1, &mut id);
            bindings::BindVertexArray(id);
            NonZeroUInt::new(id).expect("Could not create VertexArray")
        };
        let Self {
            vertex_buffers: vertex_buffers_data,
            index_buffer: index_buffer_data,
        } = self;

        let vertex_buffers = vertex_buffers_data
            .into_iter()
            .map(|(k, v)| (k, DynamicBuffer::new(v)))
            .collect();
        let index_buffer = IndexBuffer::new(index_buffer_data);
        let vao = VaoGeneric {
            id: Guard { inner },
            vertex_buffers,
            index_buffer,
        };

        for b in vao.vertex_buffers.values() {
            b.bind_to_vao(&vao);
        }
        vao
    }
}
impl<V: GLVertexType, C: GLVertexType> VaoSimple<V, C> {
    pub fn create() -> Self {
        Self {
            vertex_data: DynamicData::new::<V>(
                None,
                Layout::new(
                    0, // pos is defined as 0
                    3,
                    V::to_glenum(),
                ),
            ),
            index_data: IndexBufferDataU32::new(None),
            color_data: DynamicData::new::<V>(
                None,
                Layout::new(
                    1, // color is defined as 1
                    3,
                    C::to_glenum(),
                ),
            ),

            _pb: PhantomData,
            _cb: PhantomData,
        }
    }
    pub fn add(self, drawable: impl IntoDrawable) -> Self {
        self.merge(drawable.into_drawable())
    }
}

impl<V: GLVertexType, C: GLVertexType> IntoDrawable for VaoSimple<V, C> {
    type IntoDrawable = Self;

    fn into_drawable(self) -> Self::IntoDrawable {
        self
    }
}

impl<V: GLVertexType, C: GLVertexType> Drawable for VaoSimple<V, C> {
    fn from_buffers(index: IndexBufferDataU32, vbs: HashMap<u32, DynamicData>) -> Self {
        Self {
            vertex_data: vbs.get(&0).unwrap().clone(),
            index_data: index,
            color_data: vbs.get(&1).unwrap().clone(),
            _pb: PhantomData,
            _cb: PhantomData,
        }
    }

    fn get_buffers(self) -> (IndexBufferDataU32, HashMap<u32, DynamicData>) {
        let Self {
            vertex_data,
            index_data,
            color_data,
            _pb,
            _cb,
        } = self;
        (
            index_data,
            HashMap::from([(0, vertex_data), (1, color_data)]),
        )
    }
}

impl<V: GLVertexType, C: GLVertexType> VertexArrayObjectData for VaoSimple<V, C> {
    type VAO = Vao<V, C>;
    fn build(self) -> Self::VAO {
        let id = unsafe { raw::buffers::CreateVertexArray() };

        let position_vb = DynamicBuffer::new(self.vertex_data);
        let color_vb = DynamicBuffer::new(self.color_data);
        let index_buffer = IndexBuffer::new(self.index_data);

        let vao: Vao<V, C> = Vao {
            id: Guard {
                inner: NonZeroUInt::new(id).unwrap(),
            },
            position_vb,
            color_vb,
            index_buffer,
            _pb: PhantomData,
            _cb: PhantomData,
        };

        vao.position_vb.bind_to_vao(&vao);
        vao.color_vb.bind_to_vao(&vao);

        vao
    }
}
