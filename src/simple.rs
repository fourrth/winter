use std::ffi::{c_int, c_void, CString};

use self::shader::Program;
use glad_gles2::gl;
use glfw::ffi::{glfwWindowShouldClose, GLFWkeyfun, GLFWwindow};

pub mod buffer;
pub mod misic;
pub mod shader;
pub mod vao;

pub fn roll_gl_errors() {
    unsafe {
        loop {
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL error: {}", error);
                panic!("HIT GL ERROR!!!");
                // Handle or log the error as needed
            } else {
                break; // No more errors
            }
        }
    }
}

fn proc_loader(str: &'static str) -> *const c_void {
    unsafe {
        let mut name = str.as_bytes().to_vec();
        name.push(b'\0');
        glfw::ffi::glfwGetProcAddress(name.as_ptr() as *const i8)
    }
}

#[derive(Debug)]
pub struct Window {
    pub handle: *mut GLFWwindow,
    pub width: i32,
    pub height: i32,
    pub title: CString,
}
impl Window {
    pub fn new(width: i32, height: i32, title: CString) -> Result<Window, String> {
        unsafe {
            if glfw::ffi::glfwInit() == 0 {
                return Err(String::from("GLFW Failed to Initialize"));
            }
            glfw::ffi::glfwWindowHint(glfw::ffi::CLIENT_API, glfw::ffi::OPENGL_ES_API);

            glfw::ffi::glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MAJOR, 3);
            glfw::ffi::glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MINOR, 1);
            glfw::ffi::glfwWindowHint(glfw::ffi::OPENGL_PROFILE, glfw::ffi::OPENGL_CORE_PROFILE);

            glfw::ffi::glfwWindowHint(glfw::ffi::RESIZABLE, glfw::ffi::FALSE);
            glfw::ffi::glfwWindowHint(glfw::ffi::FOCUSED, glfw::ffi::TRUE);

            let window = glfw::ffi::glfwCreateWindow(
                width,
                height,
                title.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if window.is_null() {
                return Err(String::from("Failed to Create Window"));
            }
            glfw::ffi::glfwMakeContextCurrent(window);
            // glfw::ffi::glfwSetFramebufferSizeCallback(window, cbfun);

            gl::load(proc_loader);
            Ok(Window {
                handle: window,
                width,
                height,
                title,
            })
        }
    }
    pub fn should_close(&self) -> bool {
        unsafe {
            if glfwWindowShouldClose(self.handle) == 1 {
                true
            } else {
                false
            }
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            glfw::ffi::glfwMakeContextCurrent(std::ptr::null_mut());
            glfw::ffi::glfwTerminate();
        }
    }
}
type GlfwInputFunction =
    fn(window: *mut GLFWwindow, key: c_int, scancode: c_int, action: c_int, mods: c_int);

static mut USER_KEY_FUNC: Option<GlfwInputFunction> = None;

extern "C" fn main_context_keyboard_input(
    // window won't be needed as
    // the user is aware of window
    // they are refering to as they
    // set this up when they are making the context
    window: *mut GLFWwindow,
    key: c_int,
    scancode: c_int,
    action: c_int,
    mods: c_int,
) {
    // it should be impossible to get here
    // without also setting input_f
    unsafe { USER_KEY_FUNC.unwrap_unchecked()(window, key, scancode, action, mods) }
}

pub struct Context {
    pub window: Window,
    pub program: Program,
    pub vao: vao::VertexArrayObject,
}
impl Context {
    pub fn new(
        width: i32,
        height: i32,
        title: CString,
        vertex_shader_text: CString,
        fragment_shader_text: CString,
        input_function: Option<GlfwInputFunction>,
        vao: vao::VertexArrayObjectBuilder,
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
            window,
            program,
            vao: vao.build(),
        })
    }
}
