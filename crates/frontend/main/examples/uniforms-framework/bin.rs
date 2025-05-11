use glfw::ffi::GLFWwindow;
use glmath::vector::Vector3;
use std::io::Write;
use std::{
    io::{self, stdout, BufWriter},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use winter::context::{Context, ContextKind};
use winter_core::{
    bindings::{self, types::GLfloat},
    uniform::Uniform,
};
use winter_simple::{constructs, shapes, uniform, IndexGrid, IntoDrawable, VertexArrayObject};

// This isn't perfect as it is still possible to
// write while reading,
// but it's very unlikely
// who cares it just cursor pos
extern "C" fn cursor_callback(_window: *mut GLFWwindow, xpos: f64, ypos: f64) {
    let index = CPOS_INDEX.fetch_xor(1, Ordering::Relaxed);
    unsafe {
        CURSOR_POS[index] = CursorPos {
            xpos: xpos as f32,
            ypos: ypos as f32,
        };
    }
}

fn get_cursor_pos() -> CursorPos {
    unsafe { CURSOR_POS[CPOS_INDEX.load(Ordering::Relaxed)] }
}

#[derive(Debug, Clone, Copy)]
struct CursorPos {
    pub xpos: f32,
    pub ypos: f32,
}
impl Into<(f32, f32)> for CursorPos {
    fn into(self) -> (f32, f32) {
        (self.xpos, self.ypos)
    }
}
static CPOS_INDEX: AtomicUsize = AtomicUsize::new(0);
static mut CURSOR_POS: [CursorPos; 2] = [CursorPos {
    xpos: 0f32,
    ypos: 0f32,
}; 2];

fn main() -> Result<(), String> {
    let width = 800;
    let height = 800;

    let title = String::from("Hello Example Framework!");

    let (vertex_shader_text, fragment_shader_text) = {
        (
            String::from(include_str!("vertex_shader.glsl")),
            String::from(include_str!("frag_shader.glsl")),
        )
    };

    let arena_cell_length: u32 = 13;

    let color1 = Vector3::from([1.0, 0.0, 0.0]);
    let color2 = Vector3::from([0.0, 1.0, 0.0]);
    let color3 = Vector3::from([0.0, 0.0, 1.0]);

    let colors = [color1, color2, color3];

    let position = shapes::Rectangle {
        bottom_left_corner: Vector3::from([-1.0, -1.0, 0.0]),
        bottom_right_corner: Vector3::from([1.0, -1.0, 0.0]),
        top_right_corner: Vector3::from([1.0, 1.0, 0.0]),
        top_left_corner: Vector3::from([-1.0, 1.0, 0.0]),
    };

    let color_data: Box<[Vector3<f32>]> = Box::from(colors);
    let index_grid: IndexGrid<u32> = {
        let width = arena_cell_length as usize;
        let height = arena_cell_length as usize;
        IndexGrid::new(
            width,
            height,
            (0..(width * height) as u32).map(|cx| cx % 3).collect(),
        )
        .unwrap()
    };

    let vao_builder: winter_simple::vao::Builder<f32, u32, f32, 3, false, { bindings::TRIANGLES }> =
        winter_simple::vao::Builder::create().add(
            constructs::PixelGridSolidColorIndividual::new(position, index_grid, color_data)
                .into_drawable(),
        );

    let mut context: Context<
        winter_simple::vao::Builder<f32, u32, f32, 3, false, { bindings::TRIANGLES }>,
    > = winter::context::Builder::new()
        .add(ContextKind::WindowSize(width, height))
        .add(ContextKind::Title(title))
        .add(ContextKind::VertexShaderText(vertex_shader_text))
        .add(ContextKind::FragmentShaderText(fragment_shader_text))
        .add(ContextKind::InputFunction(None))
        .add(ContextKind::VertexArrayObjectData(vao_builder))
        .build()?;
    let mut writer: BufWriter<io::StdoutLock<'static>> = BufWriter::new(stdout().lock());

    unsafe {
        glfw::ffi::glfwSetCursorPosCallback(context.window.handle, Some(cursor_callback));

        context.program.enable();
        context.vao.bind();
        struct MyUniforms {
            pub xpos: uniform::Float,
            pub ypos: uniform::Float,
            pub width: uniform::Float,
            pub height: uniform::Float,
            pub arena_cell_length: uniform::Float,
        }
        impl MyUniforms {
            pub fn update_all(
                &self,
                xpos: GLfloat,
                ypos: GLfloat,
                width: GLfloat,
                height: GLfloat,
                arena_cell_length: GLfloat,
            ) {
                self.xpos.update(xpos);
                self.ypos.update(ypos);
                self.width.update(width);
                self.height.update(height);
                self.arena_cell_length.update(arena_cell_length);
            }
        }

        let uniforms = MyUniforms {
            xpos: uniform::Float::from(0),
            ypos: uniform::Float::from(1),
            width: context.program.uniform("widthu").unwrap(),
            height: context.program.uniform("heightu").unwrap(),
            arena_cell_length: context.program.uniform("arena_cell_lengthu").unwrap(),
        };

        glfw::ffi::glfwSwapInterval(0);

        let mut frame_time = 0f64;

        let time_max_before_reset = std::time::Duration::from_secs_f32(0.1);

        let mut frame_cnt = 1f64;

        let time_start_ = Instant::now();
        let mut fps_update_start = time_start_;
        while context.window.should_close() == false {
            bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
            bindings::Clear(bindings::COLOR_BUFFER_BIT);

            context.vao.draw();

            let (xpos, ypos) = get_cursor_pos().into();
            uniforms.update_all(
                xpos,
                ypos,
                context.window.width as f32,
                context.window.height as f32,
                arena_cell_length as f32,
            );

            glfw::ffi::glfwGetFramebufferSize(
                context.window.handle,
                &mut context.window.width,
                &mut context.window.height,
            );

            let _ = write!(
                writer,
                "fps: {:.2}, camera pos: ({:.0},{:.0})------------------\r",
                1f64 / frame_time,
                xpos,
                ypos
            );
            let _ = writer.flush();
            glfw::ffi::glfwSwapBuffers(context.window.handle);
            glfw::ffi::glfwPollEvents();
            if fps_update_start.elapsed() > time_max_before_reset {
                frame_time = fps_update_start.elapsed().as_secs_f64() / frame_cnt as f64;
                fps_update_start = Instant::now();
                frame_cnt = 1f64;
            } else {
                frame_cnt += 1f64;
            }
        }
    }

    Ok(())
}
