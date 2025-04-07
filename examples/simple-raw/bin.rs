use glfw::ffi::GLFWwindow;
use once_cell::sync::Lazy;
use std::{collections::HashMap, ffi::c_void};
use winter::bindings::{
    self,
    types::{GLfloat, GLint, GLuint},
};
use winter::raw::buffers::BufferTarget;

fn proc_loader(str: &'static str) -> *const c_void {
    unsafe {
        let mut name = str.as_bytes().to_vec();
        name.push(b'\0');
        glfw::ffi::glfwGetProcAddress(name.as_ptr() as *const i8)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
enum DrawKind {
    Square,
    Triangle,
}
static DRAW_KIND_MAP: Lazy<HashMap<&'static str, DrawKind>> = Lazy::new(|| {
    use DrawKind::*;
    let mut map = HashMap::new();
    map.insert("square", Square);
    map.insert("triangle", Triangle);
    map
});

struct SimpleStruct {
    pub vertex_shader_text: &'static [u8],
    pub fragment_shader_text: &'static [u8],
    pub vertices: &'static [GLfloat],
    pub indices: &'static [GLuint],
    pub title: String,
    pub width: GLint,
    pub height: GLint,
    pub window: Option<*mut GLFWwindow>,
}
impl SimpleStruct {
    pub fn new(width: GLint, height: GLint, kind: DrawKind) -> Self {
        let vertex_shader_text: &'static [u8] = b"#version 320 es
        layout (location = 0) in vec3 vertPosition;
    
        void main()
        {
           gl_Position = vec4(vertPosition,1.0);
        };\0";

        let fragment_shader_text: &'static [u8] = b"#version 320 es
        precision mediump float;
        out vec4 outputF;
        void main()
        {
           outputF = vec4(1.0,1.0,1.0,1.0);
        };\0";

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
                self.title.as_bytes() as *const _ as *const i8,
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
    let drawkind = match std::env::args().skip(1).next() {
        Some(str) => {
            if let Some(&kind) = DRAW_KIND_MAP.get(str.to_ascii_lowercase().as_str()) {
                kind
            } else {
                // then no good match, log and do default
                dbg!("Failed to get a good match, using default triangle (options: triangle, square)");
                DrawKind::Triangle
            }
        }
        None => {
            // means no input, do default like above
            dbg!("Failed to get a good match, using default triangle (options: triangle, square)");
            DrawKind::Triangle
        }
    };
    let mut ss = SimpleStruct::new(640, 640, drawkind);
    ss.initialize()?;
    unsafe {
        let program = winter::raw::shader::CreateProgram(
            ss.vertex_shader_text.as_ptr() as *const i8,
            ss.vertex_shader_text.len().try_into().unwrap(),
            ss.fragment_shader_text.as_ptr() as *const i8,
            ss.fragment_shader_text.len().try_into().unwrap(),
        )
        .unwrap();

        let vao = winter::raw::buffers::CreateVertexArray();
        let vb = winter::raw::buffers::CreateBuffer(
            ss.vertices.as_ptr() as *const c_void,
            (ss.vertices.len() * std::mem::size_of::<GLfloat>())
                .try_into()
                .unwrap(),
            BufferTarget::ArrayBuffer,
        )
        .unwrap();

        let ib = winter::raw::buffers::CreateBuffer(
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
        bindings::EnableVertexAttribArray(0);

        bindings::BindBuffer(bindings::ELEMENT_ARRAY_BUFFER, ib);

        bindings::UseProgram(program);
        bindings::BindVertexArray(vao);

        while glfw::ffi::glfwWindowShouldClose(ss.window.unwrap()) == 0 {
            bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
            bindings::Clear(bindings::COLOR_BUFFER_BIT);

            bindings::DrawElements(
                bindings::TRIANGLES,
                ss.indices.len() as GLint,
                bindings::UNSIGNED_INT,
                std::ptr::null(),
            );

            glfw::ffi::glfwSwapBuffers(ss.window.unwrap());
            glfw::ffi::glfwPollEvents();
        }
        bindings::DeleteVertexArrays(1, &vao);
        bindings::DeleteBuffers(1, &vb);
        bindings::DeleteBuffers(1, &ib);
        bindings::DeleteProgram(program);

        glfw::ffi::glfwTerminate();
    }
    Ok(())
}
