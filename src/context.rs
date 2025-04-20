mod window;

use std::{ffi::CString, mem::ManuallyDrop};

use glfw::ffi::GLFWkeyfun;

use crate::{
    shader::Program,
    vao::{VertexArrayObject, VertexArrayObjectData},
};

use self::window::{main_context_keyboard_input, GlfwInputFunction, Window, USER_KEY_FUNC};

pub struct Context<VAO: VertexArrayObject> {
    pub window: ManuallyDrop<Window>,
    pub program: ManuallyDrop<Program>,
    pub vao: ManuallyDrop<VAO>,
}
impl<VAO: VertexArrayObject> Context<VAO> {
    pub fn new<VAOD: VertexArrayObjectData<VAO = VAO>>(
        width: i32,
        height: i32,
        title: CString,
        vertex_shader_text: CString,
        fragment_shader_text: CString,
        input_function: Option<GlfwInputFunction>,
        vertex_array_object_data: VAOD,
    ) -> Result<Self, String> {
        let window = Window::new(width, height, title)?;
        let program = Program::new(vertex_shader_text, fragment_shader_text)?;
        unsafe {
            if let Some(input_f) = input_function {
                glfw::ffi::glfwSetKeyCallback(
                    window.handle,
                    Some(main_context_keyboard_input as GLFWkeyfun),
                );
                USER_KEY_FUNC = Some(input_f);
            } else {
                // then we either don't want input,
                // or we just aren't the ones handling it
                USER_KEY_FUNC = None;
            }
        }
        Ok(Self {
            window: ManuallyDrop::new(window),
            program: ManuallyDrop::new(program),
            vao: ManuallyDrop::new(vertex_array_object_data.build()),
        })
    }
}
impl<VAO: VertexArrayObject> Drop for Context<VAO> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.vao);
            ManuallyDrop::drop(&mut self.program);
            ManuallyDrop::drop(&mut self.window);
        }
    }
}
