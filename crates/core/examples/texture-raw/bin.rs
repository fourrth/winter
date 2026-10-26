use glfw::ffi::GLFWwindow;
use std::{
    error::Error,
    ffi::{c_int, c_void},
};

use winter_core::{
    bindings::{
        self,
        types::{GLfloat, GLint, GLsizei, GLuint},
    },
    raw::{
        self,
        buffers::BufferTarget,
        textures::{
            TextureFormat, TextureInternalFormat, TextureParameter, TextureTarget, TextureType,
            TextureUnit,
        },
    },
};

use crate::sampler2d::Sampler2D;
mod sampler2d;
fn proc_loader(str: &'static str) -> *const c_void {
    unsafe {
        let mut name = str.as_bytes().to_vec();
        name.push(b'\0');
        glfw::ffi::glfwGetProcAddress(name.as_ptr() as *const i8)
    }
}

fn initialize(width: c_int, height: c_int, title: &str) -> Result<*mut GLFWwindow, String> {
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
            width,
            height,
            title.as_bytes() as *const _ as *const i8,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if window.is_null() {
            return Err(String::from("Failed to Create Window"));
        }
        glfw::ffi::glfwMakeContextCurrent(window);

        bindings::load_with(proc_loader);
        Ok(window)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let width = 800;
    let height = 800;

    let vertex_shader_text: &'static [u8] = b"#version 320 es
        layout (location = 0) in vec3 vertPosition;
        layout (location = 1) in vec2 uv;
        out vec2 v_uv;
        void main()
        {
            v_uv = uv;
            gl_Position = vec4(vertPosition,1.0);
        };\0";

    let fragment_shader_text: &'static [u8] = b"#version 320 es
        precision mediump float;
        in vec2 v_uv;
        layout (location = 2) uniform sampler2D u_tex;

        out vec4 outputF;
        
        void main()
        {
           float gs = texture(u_tex,v_uv).r;// grayscale
           outputF = vec4(1.0,0.0,0.0,gs);
        };\0";

    let (vertices, uvs, indices, title): (
        &'static [GLfloat],
        &'static [GLfloat],
        &'static [GLuint],
        String,
    ) = {
        (
            [
                0.5, 0.5, 0.0, // top right
                0.5, -0.5, 0.0, // bottom right
                -0.5, -0.5, 0.0, // bottom left
                -0.5, 0.5, 0.0, // top left
            ]
            .as_slice(),
            [
                1.0, 0.0, // top right
                1.0, 1.0, // bottom right
                0.0, 1.0, // bottom left
                0.0, 0.0, // top left
            ]
            .as_slice(),
            [
                0, 1, 3, // first Triangle
                1, 2, 3, // second Triangle
            ]
            .as_slice(),
            String::from("My OpenGL/GLFW Square ✓\0"),
        )
    };

    // loading image
    let img = image::ImageReader::open("glyph_atlas.png")?
        .with_guessed_format()?
        .decode()?
        .into_luma8();
    let pixels = img.to_vec();

    let sampler = Sampler2D::from(2);

    let window = initialize(width, height, &title)?;

    unsafe {
        let tex = raw::textures::CreateTexture2D(
            img.width() as GLsizei,
            img.height() as GLsizei,
            TextureInternalFormat::R8,
            TextureFormat::Red,
            TextureType::UnsignedByte,
            pixels.as_ptr() as *const c_void,
        )?;

        let program = raw::shader::CreateProgram(
            vertex_shader_text.as_ptr() as *const i8,
            vertex_shader_text.len().try_into().unwrap(),
            fragment_shader_text.as_ptr() as *const i8,
            fragment_shader_text.len().try_into().unwrap(),
        )
        .unwrap();

        raw::textures::BindTexture(TextureTarget::Texture2D, tex.into());

        raw::textures::TexParameteri(
            TextureTarget::Texture2D,
            TextureParameter::MinFilter,
            bindings::LINEAR as i32,
        );
        raw::textures::TexParameteri(
            TextureTarget::Texture2D,
            TextureParameter::MagFilter,
            bindings::LINEAR as i32,
        );

        let vao = raw::buffers::CreateVertexArray();
        let vb = raw::buffers::CreateBuffer(
            vertices.as_ptr() as *const c_void,
            (vertices.len() * std::mem::size_of::<GLfloat>())
                .try_into()
                .unwrap(),
            BufferTarget::ArrayBuffer,
        )
        .unwrap();

        let uvb = raw::buffers::CreateBuffer(
            uvs.as_ptr() as *const c_void,
            (uvs.len() * std::mem::size_of::<GLfloat>())
                .try_into()
                .unwrap(),
            BufferTarget::ArrayBuffer,
        )
        .unwrap();

        let ib = raw::buffers::CreateBuffer(
            indices.as_ptr() as *const c_void,
            (indices.len() * std::mem::size_of::<GLuint>())
                .try_into()
                .unwrap(),
            BufferTarget::ElementArrayBuffer,
        )
        .unwrap();

        raw::buffers::BindBuffer(BufferTarget::ArrayBuffer, vb);

        bindings::VertexAttribPointer(
            0,
            3,
            bindings::FLOAT,
            bindings::FALSE,
            3 * std::mem::size_of::<GLfloat>() as GLint,
            std::ptr::null(),
        );
        bindings::EnableVertexAttribArray(0);

        raw::buffers::BindBuffer(BufferTarget::ArrayBuffer, uvb);
        bindings::VertexAttribPointer(
            1,
            2,
            bindings::FLOAT,
            bindings::FALSE,
            2 * std::mem::size_of::<GLfloat>() as GLint,
            std::ptr::null(),
        );
        bindings::EnableVertexAttribArray(1);

        raw::buffers::BindBuffer(BufferTarget::ElementArrayBuffer, ib);

        bindings::UseProgram(program.into());
        raw::textures::ActiveTexture(TextureUnit::Texture0);
        raw::textures::BindTexture(TextureTarget::Texture2D, tex.into());
        sampler.update(TextureUnit::Texture0);

        // Note that we can do:
        // use program, activate, update,
        // then later on in the render loop we bind program, activate texture, then bind texture
        // update persists so it's not necesary to change it

        bindings::BindVertexArray(vao);

        bindings::Enable(bindings::BLEND);
        bindings::BlendFunc(bindings::SRC_ALPHA, bindings::ONE_MINUS_SRC_ALPHA);
        while glfw::ffi::glfwWindowShouldClose(window) == 0 {
            bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
            bindings::Clear(bindings::COLOR_BUFFER_BIT);

            bindings::DrawElements(
                bindings::TRIANGLES,
                indices.len() as GLint,
                bindings::UNSIGNED_INT,
                std::ptr::null(),
            );

            glfw::ffi::glfwSwapBuffers(window);
            glfw::ffi::glfwPollEvents();
        }
        bindings::DeleteVertexArrays(1, &vao);
        bindings::DeleteBuffers(1, &vb);
        bindings::DeleteBuffers(1, &ib);
        bindings::DeleteProgram(program.into());

        glfw::ffi::glfwTerminate();
    }
    Ok(())
}
