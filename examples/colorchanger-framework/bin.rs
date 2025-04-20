use glmath::vector::Vector3;
use std::{ffi::CString, time::Instant};
use winter::bindings;
use winter::vao::simple::IntoDrawable;
use winter::vao::{simple, VertexArrayObject};

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

    let arena_cell_length = 8;
    // let margin = 0.05;

    let color1 = Vector3::from([1.0, 0.0, 0.0]);
    let color2 = Vector3::from([0.0, 1.0, 0.0]);
    let color3 = Vector3::from([0.0, 0.0, 1.0]);

    let colors = [color1, color2, color3];

    let position = simple::shapes::Rectangle {
        bottom_left_corner: Vector3::from([-0.5, -0.5, 0.0]).mul_scalar(2f32),
        bottom_right_corner: Vector3::from([0.5, -0.5, 0.0]).mul_scalar(2f32),
        top_right_corner: Vector3::from([0.5, 0.5, 0.0]).mul_scalar(2f32),
        top_left_corner: Vector3::from([-0.5, 0.5, 0.0]).mul_scalar(2f32),
    };

    let color_data = Box::from(colors);
    let index_grid: simple::IndexGrid<u32> = {
        let width = arena_cell_length;
        let height = arena_cell_length;
        simple::IndexGrid::new(
            width,
            height,
            (0..(width * height) as u32).map(|cx| cx % 3).collect(),
        )
        .unwrap()
    };

    let vao_builder = simple::Builder::create().add(
        simple::constructs::PixelGridSolidColorIndividual::new(position, index_grid, color_data)
            .into_drawable(),
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
