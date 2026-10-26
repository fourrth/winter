use std::{collections::HashMap, iter};

use winter_core::{
    buffer::{
        index::IndexBufferDataU32,
        vertex::{DynamicData, Layout},
    },
    opengl::GLVertexType,
};
use winter_simple::Drawable;
use winter_simple::{
    glmath::{
        vector::{Vector2, Vector3},
        Element,
    },
    shapes,
};

/// A billboard of text placed at a location in 3D space.
///
/// Renders each non-whitespace character as a quad sampling a monospaced
/// glyph atlas (laid out in `atlas_columns` × `atlas_rows` cells).
#[derive(Debug, Clone, PartialEq)]
pub struct Text<V: GLVertexType + Element, C: GLVertexType + Element> {
    pub text: String,

    /// World-space position of the start (bottom-left) of the text.
    pub position: Vector3<V>,

    /// Flat RGB color applied to every vertex.
    pub color: Vector3<C>,

    /// Size of a single glyph quad, in world units.
    pub char_width: V,
    pub char_height: V,

    /// Atlas layout.
    pub atlas_columns: u32,
    pub atlas_rows: u32,
    pub atlas_last_row_length: u32,
    /// Character code occupying cell (0, 0) — usually `' '`.
    pub first_char: char,
}

impl<V: GLVertexType + Element, C: GLVertexType + Element> Text<V, C> {
    pub fn new(text: impl Into<String>, position: Vector3<V>, color: Vector3<C>) -> Self {
        Self {
            text: text.into(),
            position,
            color,
            char_width: V::from_f32(0.5),
            char_height: V::from_f32(0.5),
            atlas_columns: 16,
            atlas_rows: 6,
            atlas_last_row_length: 15,
            first_char: ' ',
        }
    }
    pub fn atlas_len(&self) -> u32 {
        self.atlas_rows * self.atlas_columns - (self.atlas_columns - self.atlas_last_row_length)
    }

    pub fn to_mesh(self, position_key: u32, color_key: u32, uvs_key: u32) -> winter_simple::Mesh {
        // De-interleaved CPU-side storage.
        let mut positions: Vec<shapes::Rectangle<V>> = Vec::new();
        let mut uvs: Vec<shapes::Rectangle2d<V>> = Vec::new();
        let mut colors: Vec<Vector3<C>> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // UV step per atlas cell.
        let u_step = V::from_f32(1.0 / self.atlas_columns as f32);
        let v_step = V::from_f32(1.0 / self.atlas_rows as f32);

        let x_start = self.position[0];
        let y_start = self.position[1];
        let z = self.position[2];

        // Column we're writing to (in glyph units), and the number of
        // quads we've actually emitted — they diverge when we skip spaces.
        let mut advance: u32 = 0;
        let mut quad: u32 = 0;

        let atlas_len = self.atlas_len();

        let invalid_char = (self.first_char as u32) + 5; // %  for invalid chars

        for ch in self.text.chars() {
            // Advance for whitespace without emitting geometry.
            if ch == ' ' {
                advance += 1;
                continue;
            }

            // Which cell in the atlas this character lives in.
            let code = {
                if let Some(car) = (ch as u32).checked_sub(self.first_char as u32) {
                    if car > atlas_len {
                        invalid_char
                    } else {
                        car
                    }
                } else {
                    invalid_char
                }
            };
            let col = code % self.atlas_columns;
            let row = code / self.atlas_columns;

            let uv1 = Vector2::from([
                V::from_f32(col as f32) * u_step,
                V::from_f32(row as f32) * v_step + v_step,
            ]);
            let uv2 = uv1.add(Vector2::from([u_step, -v_step]));

            // World-space quad corners.
            let xy1 = Vector2::from([
                x_start + V::from_f32(advance as f32) * self.char_width,
                y_start,
            ]);

            let xy2 = xy1.add(Vector2::from([self.char_width, self.char_height]));

            // // Top-left, top-right, bottom-right, bottom-left.
            positions.push(shapes::Rectangle::from_2d_2points(xy1, xy2, z));
            uvs.push(shapes::Rectangle2d::from_2points(uv1, uv2));
            colors.extend(iter::once(self.color).cycle().take(12));
            // Two triangles: (0,1,2) and (0,2,3).

            let base = (quad * 4) as usize;
            indices.extend_from_slice(&[
                (base) as u32,
                (base + 1) as u32,
                (base + 2) as u32,
                (base) as u32,
                (base + 2) as u32,
                (base + 3) as u32,
            ]);
            advance += 1;
            quad += 1;
        }

        let index = IndexBufferDataU32::new(Some(&indices));

        let vbs = HashMap::from([
            (
                position_key,
                DynamicData::new2(
                    Some(&positions),
                    quad as usize * 4,
                    Layout::new(position_key, 3, V::to_glenum()),
                ),
            ),
            (
                color_key,
                DynamicData::new2(
                    Some(&colors),
                    quad as usize * 4,
                    Layout::new(color_key, 3, C::to_glenum()),
                ),
            ),
            (
                uvs_key,
                DynamicData::new2(
                    Some(&uvs),
                    quad as usize * 4,
                    Layout::new(uvs_key, 2, V::to_glenum()),
                ),
            ),
        ]);
        winter_simple::Mesh::from_buffers(index, vbs)
    }
}
