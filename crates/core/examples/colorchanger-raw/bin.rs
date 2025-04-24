#![allow(non_snake_case)]

use glfw::ffi::GLFWwindow;
use once_cell::sync::Lazy;
use rand::Rng;
use std::{collections::HashMap, env::args, ffi::c_void, time::Instant};
use winter_core::{
    bindings::{
        self,
        types::{GLfloat, GLint, GLuint},
    },
    raw::{self, buffers::BufferTarget},
};
fn proc_loader(str: &'static str) -> *const c_void {
    unsafe {
        let mut name = str.as_bytes().to_vec();
        name.push(b'\0');
        glfw::ffi::glfwGetProcAddress(name.as_ptr() as *const i8)
    }
}

#[derive(Clone, Copy, Debug)]
enum DrawKind {
    Square,
    Triangle,
}

static DRAWKIND_HASHMAP: Lazy<HashMap<&'static str, DrawKind>> = Lazy::new(|| {
    HashMap::from([
        ("square", DrawKind::Square),
        ("triangle", DrawKind::Triangle),
    ])
});

static mut RNG_GEN: Lazy<rand::rngs::OsRng> = Lazy::new(|| rand::rngs::OsRng);

struct SimpleStruct {
    pub vertex_shader_text: String,
    pub fragment_shader_text: String,
    pub vertices: &'static [GLfloat],
    pub indices: &'static [GLuint],
    pub title: String,
    pub width: GLint,
    pub height: GLint,
    pub window: Option<*mut GLFWwindow>,
}
impl SimpleStruct {
    pub fn new(width: GLint, height: GLint, kind: DrawKind) -> Self {
        let vertex_shader_text = String::from(include_str!("vertex_shader.glsl"));
        let fragment_shader_text = format!(
            include_str!("frag_shader.glsl"),
            unsafe { RNG_GEN.gen::<f32>() },
            unsafe { RNG_GEN.gen::<f32>() },
        );
        let (vertices, indices, title): (&'static [GLfloat], &'static [GLuint], String) = match kind
        {
            DrawKind::Square => {
                (
                    [
                        0.5, 0.5, 0.0, // top right
                        0.5, -0.5, 0.0, // bottom right
                        -0.5, -0.5, 0.0, // bottom left
                        -0.5, 0.5, 0.0, // top left
                    ]
                    .as_slice(),
                    [
                        0, 1, 3, // first Triangle
                        1, 2, 3, // second Triangle
                    ]
                    .as_slice(),
                    String::from("My OpenGL/GLFW Square ✓\0"),
                )
            }
            DrawKind::Triangle => (
                [0.75, -0.5, 0., 0., 0.5, 0., -0.75, -0.5, 0.].as_slice(),
                [0, 1, 2].as_slice(),
                String::from("My OpenGL/GLFW Triangle ✓\0"),
            ),
        };

        Self {
            width,
            height,
            title,
            vertex_shader_text,
            fragment_shader_text,
            vertices,
            indices,
            window: None,
        }
    }
    pub fn initialize(&mut self) -> Result<(), String> {
        unsafe {
            if glfw::ffi::glfwInit() == 0 {
                return Err(String::from("GLFW Failed to Initialize"));
            }
            glfw::ffi::glfwWindowHint(glfw::ffi::CLIENT_API, glfw::ffi::OPENGL_ES_API);

            glfw::ffi::glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MAJOR, 3);
            glfw::ffi::glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MINOR, 2);
            glfw::ffi::glfwWindowHint(glfw::ffi::OPENGL_PROFILE, glfw::ffi::OPENGL_CORE_PROFILE);

            glfw::ffi::glfwWindowHint(glfw::ffi::RESIZABLE, glfw::ffi::FALSE);
            glfw::ffi::glfwWindowHint(glfw::ffi::FOCUSED, glfw::ffi::TRUE);

            let window = glfw::ffi::glfwCreateWindow(
                self.width,
                self.height,
                self.title.as_bytes().as_ptr() as *const i8,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if window.is_null() {
                return Err(String::from("Failed to Create Window"));
            }
            glfw::ffi::glfwMakeContextCurrent(window);
            // glfw::ffi::glfwSetFramebufferSizeCallback(window, cbfun);

            bindings::load_with(proc_loader);
            self.window = Some(window);
            Ok(())
        }
    }
}

fn main() -> Result<(), String> {
    let kind_str = match args().skip(1).next() {
        Some(val) => val,
        None => {
            println!("Did not give a valid argument. Valid arguments are triangle or square. Using triangle");
            String::from("triangle")
        }
    };
    let kind = match DRAWKIND_HASHMAP.get(kind_str.as_str()) {
        Some(val) => val,
        None => {
            println!("Did not give a valid argument. Valid arguments are triangle or square. Using Triangle");
            &DrawKind::Triangle
        }
    };
    let mut ss = SimpleStruct::new(640, 640, *kind);
    ss.initialize()?;
    unsafe {
        let program = raw::shader::CreateProgram(
            ss.vertex_shader_text.as_ptr() as *const i8,
            ss.vertex_shader_text.len().try_into().unwrap(),
            ss.fragment_shader_text.as_ptr() as *const i8,
            ss.fragment_shader_text.len().try_into().unwrap(),
        )
        .unwrap();

        let vao = raw::buffers::CreateVertexArray();
        let vb = raw::buffers::CreateBuffer(
            ss.vertices.as_ptr() as *const c_void,
            (ss.vertices.len() * std::mem::size_of::<GLfloat>())
                .try_into()
                .unwrap(),
            BufferTarget::ArrayBuffer,
        )
        .unwrap();

        let ib = raw::buffers::CreateBuffer(
            ss.indices.as_ptr() as *const c_void,
            (ss.indices.len() * std::mem::size_of::<GLuint>())
                .try_into()
                .unwrap(),
            BufferTarget::ElementArrayBuffer,
        )
        .unwrap();

        bindings::BindBuffer(bindings::ARRAY_BUFFER, vb);

        bindings::VertexAttribPointer(
            0,
            3,
            bindings::FLOAT,
            bindings::FALSE,
            3 * std::mem::size_of::<GLfloat>() as GLint,
            std::ptr::null(),
        );
        let vertPositionL: GLint =
            bindings::GetAttribLocation(program, b"vertPosition\0".as_ptr() as *const i8);
        assert!(vertPositionL != -1);
        bindings::EnableVertexAttribArray(vertPositionL as GLuint);

        let mut time: Instant;
        let timeuL: GLint = bindings::GetUniformLocation(program, b"timeu\0".as_ptr() as *const i8);
        if timeuL == -1 {
            return Err(String::from("Could not Get the Location of time uniform"));
        }

        bindings::BindBuffer(bindings::ELEMENT_ARRAY_BUFFER, ib);
        bindings::UseProgram(program);
        bindings::BindVertexArray(vao);
        let time_start = Instant::now();
        while glfw::ffi::glfwWindowShouldClose(ss.window.unwrap()) == 0 {
            time = Instant::now();

            bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
            bindings::Clear(bindings::COLOR_BUFFER_BIT);

            bindings::DrawElements(
                bindings::TRIANGLES,
                ss.indices.len() as GLint,
                bindings::UNSIGNED_INT,
                std::ptr::null(),
            );

            // update the uniforms
            bindings::Uniform1f(timeuL, time.duration_since(time_start).as_secs_f32());

            glfw::ffi::glfwSwapBuffers(ss.window.unwrap());
            glfw::ffi::glfwPollEvents();
        }
        raw::buffers::DeleteVertexArray(vao);
        raw::buffers::DeleteBuffer(vb);
        raw::buffers::DeleteBuffer(ib);
        raw::shader::DeleteProgram(program);

        glfw::ffi::glfwTerminate();
    }
    Ok(())
}
