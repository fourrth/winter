use glad_gles2::gl::{self, GLfloat, GLint, GLuint};
use glfw::ffi::GLFWwindow;
use lazy_static::lazy_static;
use std::{collections::HashMap, ffi::c_void};
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
lazy_static! {
    static ref DrawKindMap: HashMap<&'static str, DrawKind> = {
        use DrawKind::*;
        let mut map = HashMap::new();
        map.insert("square", Square);
        map.insert("triangle", Triangle);
        map
    };
}

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
        precision lowp float;
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

            gl::load(proc_loader);
            self.window = Some(window);
            Ok(())
        }
    }
}
fn main() -> Result<(), String> {
    let drawkind = match std::env::args().skip(1).next() {
        Some(str) => {
            if let Some(&kind) = DrawKindMap.get(str.to_ascii_lowercase().as_str()) {
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

        gl::BindBuffer(gl::ARRAY_BUFFER, vb);

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<GLfloat>() as GLint,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib);

        gl::UseProgram(program);
        gl::BindVertexArray(vao);

        while glfw::ffi::glfwWindowShouldClose(ss.window.unwrap()) == 0 {
            gl::ClearColor(0.8, 0.7, 0.7, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawElements(
                gl::TRIANGLES,
                ss.indices.len() as GLint,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            glfw::ffi::glfwSwapBuffers(ss.window.unwrap());
            glfw::ffi::glfwPollEvents();
        }
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vb);
        gl::DeleteBuffers(1, &ib);
        gl::DeleteProgram(program);

        glfw::ffi::glfwTerminate();
    }
    Ok(())
}
