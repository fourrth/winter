pub mod primitives;
use std::ffi;

use crate::{
    raw::buffers::{self, BufferTarget},
    simple::buffer::{GLType, IndexBuffer, Layout, VertexBuffer},
    Float,
};
use glad_gles2::gl;
use glmath::vector::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    component_kind: primitives::ComponentKind,
    data: Box<[u8]>,
}

#[derive(Debug, Clone)]
pub struct VertexArrayObject {
    id: u32,

    /// should we update the opengl data?
    /// this is set whenever a &mut is acquired
    pub data_update: bool,

    // a pocket is a (start,end) where it's purpose
    // is that you can do &data[pocket.0..pocket.1]
    // to get the required slice
    position_pocket_array: Vec<Vec<(usize, usize)>>,
    pub position: VertexBuffer,

    color_pocket_array: Vec<Vec<(usize, usize)>>,
    pub color: VertexBuffer,
    pub indices: IndexBuffer,
}

impl VertexArrayObject {
    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.id) };
    }
    pub fn draw(&self) {
        self.indices.bind();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len as i32,
                self.indices.ty.get_glenum(),
                std::ptr::null(),
            );
        }
    }
    /// get component data to position components
    pub fn get_position_component(&self, index: usize) -> Vec<&[u8]> {
        self.position_pocket_array[index]
            .iter()
            .map(|&(start, end)| {
                &self.position.buffer.data[start..end] //
            })
            .collect()
    }
    /// get mutable component data to position components
    pub fn get_position_component_mut(&mut self, index: usize) -> Vec<&mut [u8]> {
        unsafe {
            let data: *mut u8 = self.position.buffer.data.as_mut_ptr();
            self.position_pocket_array[index]
                .iter()
                .map(|&(start, end)| std::slice::from_raw_parts_mut(data.add(start), end))
                .collect()
        }
    }
    /// get component data to position components
    pub fn get_color_component(&self, index: usize) -> Vec<&[u8]> {
        self.color_pocket_array[index]
            .iter()
            .map(|&(start, end)| {
                &self.color.buffer.data[start..end] //
            })
            .collect()
    }
    /// get mutable component data to color components
    pub fn get_color_component_mut(&mut self, index: usize) -> Vec<&mut [u8]> {
        unsafe {
            let data: *mut u8 = self.color.buffer.data.as_mut_ptr();
            self.color_pocket_array[index]
                .iter()
                .map(|&(start, end)| std::slice::from_raw_parts_mut(data.add(start), end))
                .collect()
        }
    }
    /// take a component and tell opengl to update it
    pub fn update_position_component(&mut self, index: usize) {
        let _ = index;
        self.update_position_component_all();
    }
    /// reload the entire position buffer
    pub fn update_position_component_all(&mut self) {
        self.position.buffer.bind(BufferTarget::ArrayBuffer);
        // right now we just set the entire data again because
        // I want to make sure that this actually works
        // but in the future we can use glBufferSubData
        // and just update what we have to
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                self.position.buffer.data.len() as isize,
                self.position.buffer.data.as_ptr() as *const ffi::c_void,
                gl::STATIC_DRAW,
            );
        }
    }
    /// take a component and tell opengl to update it
    pub fn update_color_component(&mut self, index: usize) {
        let _ = index;
        self.update_color_component_all();
    }
    /// update the entire buffer
    pub fn update_color_component_all(&mut self) {
        self.color.buffer.bind(BufferTarget::ArrayBuffer);
        // right now we just set the entire data again because
        // I want to make sure that this actually works
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                self.color.buffer.data.len() as isize,
                self.color.buffer.data.as_ptr() as *const ffi::c_void,
                gl::STATIC_DRAW,
            );
        }
    }
}

impl Drop for VertexArrayObject {
    // TODO: make this have an unsafe variant which the context holds
    // that way it can drop all of its vaos at once and doesn't need to
    // do a function call for each vao, just one which gets rid of everything.
    fn drop(&mut self) {
        unsafe { buffers::DeleteVertexArray(self.id) };
    }
}

// turns Triangle (not triangle but its tuple) into its components of (pos,color)
fn converter_triangle<'a>(data: &'a [u8]) -> (&'a [u8], &'a [u8]) {
    data.split_at(std::mem::size_of::<[Vector3<Float>; 3]>())
}

#[derive(Debug)]
pub struct VertexArrayObjectBuilder {
    // not public because we want to be able
    // to change what the user puts stuff in
    // i.e., the buffer can change despite
    // what the user puts in
    components: Vec<Component>,
}

impl VertexArrayObjectBuilder {
    pub fn create() -> Self {
        Self { components: vec![] }
    }

    pub fn add(mut self, component: Component) -> Self {
        self.components.push(component);
        self
    }

    pub fn build(self) -> VertexArrayObject {
        // should convert the components
        // into Attributes

        let id = unsafe {
            let mut id: u32 = 0;
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);
            id
        };

        let mut vao = VertexArrayObject {
            id,
            position_pocket_array: vec![],
            color_pocket_array: vec![],
            data_update: false,
            position: VertexBuffer::new(
                vec![],
                Layout {
                    attrib_len: 3,
                    attrib_type: GLType::Float,
                    attrib_loc: 0, // pos is defined as 0
                },
            ),
            color: VertexBuffer::new(
                vec![],
                Layout {
                    attrib_len: 3,
                    attrib_type: GLType::Float,
                    attrib_loc: 1, // color is defined as 1
                },
            ),

            indices: IndexBuffer::new(vec![], GLType::UInt, 0),
        };

        #[inline(always)]
        fn rectangle(data: &[u8], vao: &mut VertexArrayObject) {
            //TODO: Make this do the optimized indices...
            // this doesn't right now because we are giving straight
            // triangle data, which would only work if the indices are sequential.
            // we need to change how the data is done.
            let data_uint =
                [0u32, 1u32, 2u32, 3u32, 4u32, 5u32].map(|val: u32| vao.indices.len + val);
            let ind = bytemuck::must_cast_slice::<u32, u8>(&data_uint);

            let triangle_dos = data.split_at(std::mem::size_of::<primitives::triangle::Data>());

            debug_assert_eq!(triangle_dos.0.len(), triangle_dos.1.len());

            let mut position_pockets = Vec::with_capacity(2);
            let mut color_pockets = Vec::with_capacity(2);

            for cb in [triangle_dos.0, triangle_dos.1].into_iter() {
                let (pos, col) = converter_triangle(cb);

                debug_assert_eq!(pos.len(), col.len());
                debug_assert_eq!(pos.len(), std::mem::size_of::<[Vector3<Float>; 3]>());

                let pos_pocket = (
                    vao.position.buffer.data.len(),
                    vao.position.buffer.data.len() + pos.len(),
                );
                position_pockets.push(pos_pocket);

                let color_pocket = (
                    vao.color.buffer.data.len(),
                    vao.color.buffer.data.len() + col.len(),
                );
                color_pockets.push(color_pocket);

                vao.position.buffer.data.extend(pos.into_iter());

                vao.color.buffer.data.extend(col.into_iter());
            }
            vao.indices.buffer.data.extend(ind.into_iter());
            vao.indices.len += data_uint.len() as u32;

            vao.position_pocket_array.push(position_pockets);
            vao.color_pocket_array.push(color_pockets);
        }

        for ca in self.components {
            match ca.component_kind {
                primitives::ComponentKind::Triangle => {
                    let data_uint = [0u32, 1u32, 2u32].map(|val: u32| vao.indices.len + val);
                    let ind = bytemuck::must_cast_slice::<u32, u8>(&data_uint);

                    let (pos, col) = converter_triangle(&ca.data);

                    debug_assert_eq!(pos.len(), col.len());
                    debug_assert_eq!(pos.len(), std::mem::size_of::<[Vector3<Float>; 3]>());

                    let pos_pocket = (
                        vao.position.buffer.data.len(),
                        vao.position.buffer.data.len() + pos.len(),
                    );
                    vao.position_pocket_array.push(vec![pos_pocket]);

                    let color_pocket = (
                        vao.color.buffer.data.len(),
                        vao.color.buffer.data.len() + col.len(),
                    );
                    vao.color_pocket_array.push(vec![color_pocket]);

                    vao.position.buffer.data.extend(pos.into_iter());

                    vao.color.buffer.data.extend(col.into_iter());

                    vao.indices.buffer.data.extend(ind.into_iter());
                    vao.indices.len += data_uint.len() as u32;
                }
                primitives::ComponentKind::Rectangle => rectangle(&ca.data, &mut vao),
                primitives::ComponentKind::RectangularPrism => {
                    debug_assert_eq!(
                        ca.data.len() / 6,
                        std::mem::size_of::<primitives::rectangle::Data>()
                    );
                    for cx in 0..6 {
                        rectangle(
                            &ca.data[cx * std::mem::size_of::<primitives::rectangle::Data>()
                                ..(cx + 1) * std::mem::size_of::<primitives::rectangle::Data>()],
                            &mut vao,
                        );
                    }
                }
            }
        }

        // Now we complete setting up the buffers

        vao.position.buffer.setup_buffer(BufferTarget::ArrayBuffer);
        vao.color.buffer.setup_buffer(BufferTarget::ArrayBuffer);
        vao.indices
            .buffer
            .setup_buffer(BufferTarget::ElementArrayBuffer);

        vao.position.bind_buffer(BufferTarget::ArrayBuffer);
        vao.color.bind_buffer(BufferTarget::ArrayBuffer);
        vao
    }
}
