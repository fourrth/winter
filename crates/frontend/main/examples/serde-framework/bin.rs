use glmath::vector::Vector3;
use std::time::Instant;
use winter::context::{Context, ContextKind};
use winter_core::bindings;
use winter_simple::{
    constructs, primitives,
    shapes::{self, Translate},
    IndexGrid, IntoDrawable, VertexArrayObject,
};

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

    let arena_cell_length = 8;

    let color1 = Vector3::from([1.0, 0.0, 0.0]);
    let color2 = Vector3::from([0.0, 1.0, 0.0]);
    let color3 = Vector3::from([0.0, 0.0, 1.0]);

    let colors = [color1, color2, color3];

    let position = shapes::Rectangle {
        bottom_left_corner: Vector3::from([-0.5, -0.5, 0.0]).mul_scalar(1f32),
        bottom_right_corner: Vector3::from([0.5, -0.5, 0.0]).mul_scalar(1f32),
        top_right_corner: Vector3::from([0.5, 0.5, 0.0]).mul_scalar(1f32),
        top_left_corner: Vector3::from([-0.5, 0.5, 0.0]).mul_scalar(1f32),
    };

    let color_data: Box<[Vector3<f32>]> = Box::from(colors);
    let index_grid: IndexGrid<u32> = {
        let width = arena_cell_length;
        let height = arena_cell_length;
        IndexGrid::new(
            width,
            height,
            (0..(width * height) as u32).map(|cx| cx % 3).collect(),
        )
        .unwrap()
    };

    let rect_tri_serialized =
        serde_json::to_string::<winter_simple::primitives::Component<f32, u32, f32, 3>>(
            &constructs::RectangleSolidColor::<f32, u32, f32>::new1(
                position.shift(Vector3::from([0.5, 0.0, 0.0])),
                color2,
            )
            .into_drawable()
            .merge(
                constructs::TriangleSolidColor::new1(
                    position.to_triangles()[0].shift(Vector3::from([0.0, -0.5, 0.0])),
                    color3,
                )
                .into_drawable(),
            ),
        )
        .unwrap();
    let pixel_grid = serde_json::to_string(&constructs::PixelGridSolidColorIndividual::new(
        position, index_grid, color_data,
    ))
    .unwrap();
    let vao_builder: winter_simple::vao::Builder<f32, u32, f32, 3, false, { bindings::TRIANGLES }> =
        winter_simple::vao::Builder::create()
            .add(
                serde_json::from_str::<constructs::PixelGridSolidColorIndividual<_, _, _>>(
                    &pixel_grid,
                )
                .unwrap()
                .into_drawable(),
            )
            .add(
                serde_json::from_str::<primitives::Component<_, _, _, 3>>(&rect_tri_serialized)
                    .unwrap(),
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
