use glmath::vector::Vector3;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use winter::context::{Context, ContextKind};
use winter_core::bindings;
use winter_simple::{
    constructs,
    shapes::{self, Translate},
    IntoDrawable, VertexArrayObject,
};
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum DrawKind {
    Square,
    Triangles,
    TrianglesMerged,
}
static DRAW_KIND_MAP: Lazy<HashMap<&'static str, DrawKind>> = Lazy::new(|| {
    use DrawKind::*;
    let mut map = HashMap::new();
    map.insert("square", Square);
    map.insert("triangles", Triangles);
    map.insert("trianglesmerged", TrianglesMerged);
    map
});

fn main() -> Result<(), String> {
    let (width, height, kind) = {
        let args: Vec<_> = std::env::args().skip(1).collect();

        let width = if let Some(width_) = args.get(0) {
            if let Ok(width_) = width_.parse::<i32>() {
                width_
            } else {
                println!("Inputted Width Unknown: using 800");
                800
            }
        } else {
            println!("Inputted Width Unknown: using 800");
            800
        };

        let height = if let Some(height_) = args.get(1) {
            if let Ok(height_) = height_.parse::<i32>() {
                height_
            } else {
                println!("Inputted height Unknown: using 800");
                800
            }
        } else {
            println!("Inputted height Unknown: using 800");
            800
        };

        let kind = if let Some(kind_) = args.get(2) {
            if let Some(&kind_) = DRAW_KIND_MAP.get(kind_.to_ascii_lowercase().as_str()) {
                kind_
            } else {
                println!("DrawKind unknown: using Square");
                DrawKind::Square
            }
        } else {
            println!("DrawKind unknown: using Square");
            DrawKind::Square
        };

        (width, height, kind)
    };

    let title = String::from("Hello Example Framework!");

    let (vertex_shader_text, fragment_shader_text) = {
        (
            String::from(include_str!("vertex_shader.glsl")),
            String::from(include_str!("frag_shader.glsl")),
        )
    };

    let bottom_left_corner = Vector3::from([-0.5, -0.5, 0.0]);
    let bottom_right_corner = Vector3::from([0.5, -0.5, 0.0]);
    let top_right_corner = Vector3::from([0.5, 0.5, 0.0]);
    let top_left_corner = Vector3::from([-0.5, 0.5, 0.0]);

    let rect = shapes::Rectangle::new(
        bottom_left_corner,
        bottom_right_corner,
        top_right_corner,
        top_left_corner,
    );
    let shift = Vector3::from([0.1, 0.0, 0.0]);

    let (mut tri_left, mut tri_right) = rect.to_triangles().into();

    tri_left = tri_left.shift(shift.mul_scalar(1.0));
    tri_right = tri_right.shift(shift.mul_scalar(-1.0));

    let color1 = Vector3::from([0.0, 1.0, 1.0]);
    let color2 = Vector3::from([1.0, 0.0, 1.0]);

    let tri_left_comp = constructs::TriangleSolidColor::new1(tri_left, color1).into_drawable();
    let tri_right_comp = constructs::TriangleSolidColor::new1(tri_right, color2).into_drawable();

    let mut vao_builder: winter_simple::vao::Builder<
        f32,
        u32,
        f32,
        3,
        false,
        { bindings::TRIANGLES },
    > = winter_simple::vao::Builder::create();
    match kind {
        DrawKind::Square => {
            vao_builder = vao_builder
                .add(constructs::RectangleSolidColor::new1(rect, color1).into_drawable());
        }
        DrawKind::Triangles => vao_builder = vao_builder.add(tri_left_comp).add(tri_right_comp),
        DrawKind::TrianglesMerged => {
            vao_builder = vao_builder.add(tri_left_comp.merge(tri_right_comp));
        }
    }

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
        while context.window.should_close() == false {
            bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
            bindings::Clear(bindings::COLOR_BUFFER_BIT);

            context.vao.draw();

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
