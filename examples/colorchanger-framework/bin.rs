use glmath::vector::Vector2;
use std::{ffi::CString, time::Instant};
use winter::vao::VertexArrayObject;
use winter::{bindings, NonZeroUInt};
use winter::{common, primitives};

fn main() -> Result<(), String> {
    let width = 800;
    let height = 800;

    let title = CString::new("Hello Example Framework!").unwrap();

    let (vertex_shader_text, fragment_shader_text) = {
        (
            CString::new(include_str!("vertex_shader.glsl")).unwrap(),
            CString::new(include_str!("frag_shader.glsl")).unwrap(),
        )
    };

    let arena_cell_length = 5;

    let color1 = primitives::Color::from_rgb(255, 0, 0);
    let color2 = primitives::Color::from_rgb(0, 255, 0);
    let color3 = primitives::Color::from_rgb(0, 0, 255);

    let margin = Vector2::from([0.2; 2]);
    let vao_builder = common::create_grid(
        Vector2::from([-1.; 2]).add(margin),
        Vector2::from([1.; 2]).sub(margin),
        NonZeroUInt::new(arena_cell_length).unwrap(),
        NonZeroUInt::new(arena_cell_length).unwrap(),
        0.,
        |_, _, color_change| {
            if color_change % 3 == 0 {
                color1
            } else if color_change % 3 == 1 {
                color2
            } else
            /* if color_change % 3 == 2 */
            {
                color3
            }
        },
    );

    let mut context = winter::Context::new(
        width,
        height,
        title,
        vertex_shader_text,
        fragment_shader_text,
        None,
        vao_builder,
    )?;

    unsafe {
        context.program.enable();
        context.vao.bind();
        let time_start = Instant::now();
        while context.window.should_close() == false {
            bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
            bindings::Clear(bindings::COLOR_BUFFER_BIT);

            context.vao.draw();
            bindings::Uniform1f(2, time_start.elapsed().as_secs_f32());

            glfw::ffi::glfwGetFramebufferSize(
                context.window.handle,
                &mut context.window.width,
                &mut context.window.height,
            );

            glfw::ffi::glfwSwapBuffers(context.window.handle);
            glfw::ffi::glfwPollEvents();
        }
    }

    Ok(())
}
