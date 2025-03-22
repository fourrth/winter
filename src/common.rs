//! This is a collection of quick and dirty functions that are good to have

use std::num::NonZeroU32;

use bytemuck::Contiguous;
use glad_gles2::gl;
use glmath::vector::{Vector2, Vector3};

use crate::{primitives, vao::VertexArrayObjectBuilder, Float};

#[inline(always)]
pub fn create_rectangle(
    p1: Vector3<Float>,
    c: primitives::Color,
    width: Float,
    height: Float,
) -> primitives::rectangle::Data {
    let p2 = Vector3::from([p1.0[0] + width, p1.0[1], 0.]);
    let p4 = Vector3::from([p1.0[0], p1.0[1] + height, 0.]);
    let p3 = Vector3::from([p1.0[0] + width, p1.0[1] + height, 0.]);
    primitives::rectangle::new2(p1, p2, p3, p4, c)
}

/// create a grid
/// pos_bottom_left is the bottom left corner pos
/// pos_top_right is the top right corner pos
/// Note that all positions are done relative
/// to the normal opengl [-1,1] thing
/// for colors, you are required to pass in a closure
/// which will be ran every cell iteration
/// the cells are created like a book,
/// left to right, top to bottom
/// you are provided the current x and y indices from the
/// top left, where 0,0 is the top, leftest cell
pub fn create_grid<F: Fn(u32, u32, u32) -> primitives::Color>(
    pos_bottom_left: Vector2<Float>,
    pos_top_right: Vector2<Float>,
    grid_width: NonZeroU32,
    grid_height: NonZeroU32,
    inner_margin: Float,
    // color_function: fn(x_index: u32, y_index: u32, cnt: u32) -> primitives::Color,
    color_function: F,
) -> VertexArrayObjectBuilder {
    let grid_width_f = grid_width.into_integer() as Float;
    let grid_height_f = grid_height.into_integer() as Float;

    let cell_width = ((pos_top_right.0[0] - pos_bottom_left.0[0]).abs()
        - (inner_margin * (grid_width_f - 1.)))
        / grid_width_f;

    let cell_height = ((pos_top_right.0[1] - pos_bottom_left.0[1]).abs()
        - (inner_margin * (grid_height_f - 1.)))
        / grid_height_f;

    let mut builder = VertexArrayObjectBuilder::create();

    let mut p1 = Vector3::from([pos_bottom_left.0[0], pos_top_right.0[1] - cell_height, 0.]);

    let mut cnt: u32 = 0;
    for cy in 0..grid_height.into() {
        for cx in 0..grid_width.into() {
            let c = color_function(cx, cy, cnt);

            builder = builder.add(primitives::rectangle::export(create_rectangle(
                p1,
                c,
                cell_width,
                cell_height,
            )));

            debug_assert!(!(p1.0[0].abs() > 1.));
            debug_assert!(!(p1.0[1].abs() > 1.));
            debug_assert!(!(p1.0[2].abs() != 0.));

            p1.0[0] += cell_width + inner_margin;

            cnt += 1;
        }
        p1.0[0] = pos_bottom_left.0[0];
        p1.0[1] -= cell_height + inner_margin;
    }
    builder
}

#[inline(always)]
pub fn convert_comp_triangle(data: &mut [u8]) -> &mut [Vector3<Float>; 3] {
    unsafe {
        (data.as_mut_ptr() as *mut [Vector3<Float>; 3])
            .as_mut()
            .unwrap_unchecked()
    }
}
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
