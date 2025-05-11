use std::{ffi::CString, mem::ManuallyDrop};

use glfw::ffi::GLFWkeyfun;
use winter_core::vao::{VertexArrayObject, VertexArrayObjectData};

use crate::shader::program::{self, Program, ProgramKind};

use self::window::{main_context_keyboard_input, GlfwInputFunction, USER_KEY_FUNC};

mod window;
use window::Window;

/// The arguments to be passed into the context builder.
/// ### Important Note
/// All arguments that take a String that
/// have b'\0' inside will have them
/// replaced with b' '
#[derive(Debug, Clone, PartialEq)]
pub enum ContextKind<VAO: VertexArrayObject, VAOD: Clone + VertexArrayObjectData<VAO = VAO>> {
    /// width, height
    WindowSize(i32, i32),
    Title(String),
    InputFunction(Option<GlfwInputFunction>),
    VertexShaderText(String),
    FragmentShaderText(String),
    // maybe should be &VAOD but who cares
    VertexArrayObjectData(VAOD),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Builder<VAO: VertexArrayObject, VAOD: Clone + VertexArrayObjectData<VAO = VAO>> {
    data: Vec<ContextKind<VAO, VAOD>>,
}

// rep is the replacement byte for b'\0'
fn to_cstring(mut s: String, rep: u8) -> CString {
    unsafe {
        for c in s.as_bytes_mut() {
            if *c == b'\0' {
                *c = rep;
            }
        }
        CString::new(s).unwrap_unchecked()
    }
}
impl<VAO: VertexArrayObject, VAOD: Clone + VertexArrayObjectData<VAO = VAO>> Builder<VAO, VAOD> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn add(mut self, kind: ContextKind<VAO, VAOD>) -> Self {
        self.data.push(kind);
        self
    }
    /// Build into context. Will use
    /// last given value for a specific kind.
    pub fn build(self) -> Result<Context<VAOD>, String> {
        let mut width_height: Option<(i32, i32)> = None;
        let mut title: Option<CString> = None;
        let mut vertex_shader_text: Option<CString> = None;
        let mut fragment_shader_text: Option<CString> = None;
        //TODO: need to put log for this when that's done
        // would say using default input (which for now will be nothing)
        let mut input_function: Option<GlfwInputFunction> = None;
        let mut vertex_array_object_data: Option<VAOD> = None;
        //TODO: eventually this will have defaults,

        for kind in self.data {
            match kind {
                ContextKind::WindowSize(width, height) => {
                    width_height = Some((width, height));
                }
                ContextKind::Title(title_) => {
                    title = Some(to_cstring(title_, b' '));
                }
                ContextKind::InputFunction(callback) => {
                    input_function = callback;
                }
                ContextKind::VertexShaderText(vertex_shader) => {
                    vertex_shader_text = Some(to_cstring(vertex_shader, b' '));
                }
                ContextKind::FragmentShaderText(fragment_shader) => {
                    fragment_shader_text = Some(to_cstring(fragment_shader, b' '));
                }
                ContextKind::VertexArrayObjectData(vao_data) => {
                    vertex_array_object_data = Some(vao_data);
                }
            }
        }

        const WIDTH_HEIGHT_ERR_MSG: &str = "No dimensions given";
        const TITLE_ERR_MSG: &str = "No title given";
        const VERTEX_SHADER_TEXT_ERR_MSG: &str = "No vertex shader given";
        const FRAGMENT_SHADER_TEXT_ERR_MSG: &str = "No fragment shader given";
        const VERTEX_ARRAY_OBJECT_DATA_ERR_MSG: &str = "No mesh data given";

        let mut err_string: String = String::new();
        if width_height.is_none() {
            err_string.push_str(WIDTH_HEIGHT_ERR_MSG);
        }
        if title.is_none() {
            err_string.push_str(TITLE_ERR_MSG);
        }
        if vertex_shader_text.is_none() {
            err_string.push_str(VERTEX_SHADER_TEXT_ERR_MSG);
        }
        if fragment_shader_text.is_none() {
            err_string.push_str(FRAGMENT_SHADER_TEXT_ERR_MSG);
        }
        if vertex_array_object_data.is_none() {
            err_string.push_str(VERTEX_ARRAY_OBJECT_DATA_ERR_MSG);
        }
        if !err_string.is_empty() {
            Err(err_string)
        } else {
            unsafe {
                Context::new(
                    width_height.unwrap_unchecked().0,
                    width_height.unwrap_unchecked().1,
                    title.unwrap_unchecked(),
                    vertex_shader_text.unwrap_unchecked(),
                    fragment_shader_text.unwrap_unchecked(),
                    input_function,
                    vertex_array_object_data.unwrap_unchecked(),
                )
            }
        }
    }
}

pub struct Context<VAOD: VertexArrayObjectData> {
    pub window: ManuallyDrop<Window>,
    pub program: ManuallyDrop<Program>,
    pub vao: ManuallyDrop<VAOD::VAO>,
}
impl<VAOD: VertexArrayObjectData> Context<VAOD> {
    pub fn new(
        width: i32,
        height: i32,
        title: CString,
        vertex_shader_text: CString,
        fragment_shader_text: CString,
        input_function: Option<GlfwInputFunction>,
        vertex_array_object_data: VAOD,
    ) -> Result<Self, String> {
        let window = Window::new(width, height, title)?;
        let program = {
            program::Builder::create()
                .add(ProgramKind::VertexShader(vertex_shader_text))
                .add(ProgramKind::FragmentShader(fragment_shader_text))
                .build()?
        };
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
impl<VAOD: VertexArrayObjectData> Drop for Context<VAOD> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.vao);
            ManuallyDrop::drop(&mut self.program);
            ManuallyDrop::drop(&mut self.window);
        }
    }
}
