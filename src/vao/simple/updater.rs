use std::marker::PhantomData;

use crate::{
    bindings::{self, types::GLint},
    buffer::VertexBufferDynamicData,
    common::roll_gl_errors,
    opengl::GLVertexType,
    raw::{
        self,
        buffers::{BufferTarget, MapAccess, MapAccessBF},
    },
    NonZeroUInt,
};

/// Wrapper type for updating your VertexBuffer
///
/// Will push changes to OpenGL when dropped
pub struct VertexBufferUpdater<'a, V: GLVertexType> {
    id: NonZeroUInt,
    inner: &'a mut VertexBufferDynamicData,
    _v: PhantomData<V>,
}

//TODO: impl deref for this
impl<'a, V: GLVertexType> VertexBufferUpdater<'a, V> {
    pub fn from(data: &'a mut VertexBufferDynamicData, id: NonZeroUInt) -> Self {
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
