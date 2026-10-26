//! This module contains all of the constructs
//!
//! A construct is a general shape type that
//! is meant to be serialized and deserialized,
//! manipulated, and otherwise modified when
//! creating your mesh
//!
//! Every construct implements [`super::IntoDrawable`]
//! Which then implements [`super::Drawable`]

use std::collections::HashMap;

use glmath::{
    vector::{Vector2, Vector3},
    Element,
};

use winter_core::{
    buffer::{
        index::IndexBufferDataU32,
        vertex::{DynamicData, Layout},
    },
    opengl::GLVertexType,
};

use crate::{Drawable, Mesh};

use super::{primitives::Component, shapes, IndexGrid, IntoDrawable};
/// Basic one color triangles
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TriangleSolidColor<V: GLVertexType + Element, C: GLVertexType + Element> {
    pub data: shapes::Triangle<V>,
    pub color: Vector3<C>, // rgb
}
impl<V: GLVertexType + Element, C: GLVertexType + Element> TriangleSolidColor<V, C> {
    /// Create a triangle from the bare shape primitive
    pub fn new1(tri: shapes::Triangle<V>, color: Vector3<C>) -> Self {
        Self { data: tri, color }
    }
    /// Create a triangle using 3 points and 3 colors
    pub fn new2(
        bottom_left: Vector3<V>,
        bottom_right: Vector3<V>,
        top: Vector3<V>,
        color: Vector3<C>,
    ) -> Self {
        Self::new1(
            shapes::Triangle {
                bottom_left,
                bottom_right,
                top,
            },
            color,
        )
    }
}

impl<V: GLVertexType + Element, C: GLVertexType + Element> IntoDrawable
    for TriangleSolidColor<V, C>
{
    type IntoDrawable = Component<V, C>;
    #[inline(always)]
    fn into_drawable(self) -> Self::IntoDrawable {
        let v_data = Vec::from(bytemuck::must_cast::<shapes::Triangle<V>, [V; 9]>(
            self.data,
        ))
        .into_boxed_slice();
        let c_data = self.color.into_iter().cycle().take(9).collect();
        let i_data = [0u32, 1u32, 1u32 + 1u32].into_iter().collect();
        Component::new(v_data, c_data, i_data)
    }
}

/// Basic one colored rectanlges or just two triangles together via specified points
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RectangleSolidColor<V: GLVertexType + Element, C: GLVertexType + Element> {
    pub rect: shapes::Rectangle<V>,
    pub color: Vector3<C>, // rgb
}
impl<V: GLVertexType + Element, C: GLVertexType + Element> RectangleSolidColor<V, C> {
    /// Creates a rectangle using the bare shape primitive
    pub fn new1(rect: shapes::Rectangle<V>, color: Vector3<C>) -> Self {
        Self { rect, color }
    }
    /// Creates a rectangle using 4 points and color
    pub fn new2(
        bottom_left_corner: Vector3<V>,
        bottom_right_corner: Vector3<V>,
        top_right_corner: Vector3<V>,
        top_left_corner: Vector3<V>,
        color: Vector3<C>,
    ) -> Self {
        Self::new1(
            shapes::Rectangle {
                bottom_left_corner,
                bottom_right_corner,
                top_right_corner,
                top_left_corner,
            },
            color,
        )
    }

    pub fn to_triangles(self) -> [TriangleSolidColor<V, C>; 2] {
        self.rect
            .to_triangles()
            .map(|tri| TriangleSolidColor::new1(tri, self.color))
    }
}
impl<V: GLVertexType + Element, C: GLVertexType + Element> IntoDrawable
    for RectangleSolidColor<V, C>
{
    type IntoDrawable = Component<V, C>;
    #[inline(always)]
    fn into_drawable(self) -> Self::IntoDrawable {
        let v_data = [
            self.rect.bottom_left_corner.into_iter(),
            self.rect.bottom_right_corner.into_iter(),
            self.rect.top_right_corner.into_iter(),
            self.rect.top_left_corner.into_iter(),
        ]
        .into_iter()
        .flatten()
        .collect();
        let c_data = self.color.into_iter().cycle().take(12).collect();
        let i_data = [
            0u32,               // 0
            1u32,               // 1
            1u32 + 1u32,        // 2
            0u32,               // 0
            1u32 + 1u32,        // 2
            1u32 + 1u32 + 1u32, // 3
        ]
        .into_iter()
        .collect();

        Component::new(v_data, c_data, i_data)
    }
}

/// A grid of squares or pixels which are individually colored, but
/// entirely that color (so similar to many of [`PlaneSolidColor`])
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PixelGridSolidColorIndividual<V: GLVertexType + Element, C: GLVertexType + Element> {
    /// Position of the grid in space
    position: shapes::Rectangle<V>,

    /// size of the grid in width-1,height-1
    /// So 0,0 is a 1x1 grid, or single square
    /// note we count from the top left,
    /// left to right, top to bottom
    dimensions: (usize, usize),

    /// Our color data
    color_data: Vec<Vector3<C>>,

    /// Our index data which tells us
    /// what color is each square.
    /// so index_data[5] gives the 6th pixel color's
    /// index in color_data. color_data then has the
    /// pixel's actual color
    index_data: Vec<u32>,
}

impl<V: GLVertexType + Element, C: GLVertexType + Element> PixelGridSolidColorIndividual<V, C> {
    fn get_actual_dimensions(&self) -> (usize, usize) {
        (self.dimensions.0 + 1, self.dimensions.1 + 1)
    }
    fn get_grid_iter(&self) -> impl Iterator<Item = (V, V)> {
        let (width, height) = self.get_actual_dimensions();
        (0..height)
            .flat_map(move |cy| (0..width).map(move |cx| (V::from_usize(cx), V::from_usize(cy))))
    }
    /// The way to create a new pixel grid
    pub fn new(
        position: shapes::Rectangle<V>,
        index_grid: IndexGrid<u32>,
        color_data: Box<[Vector3<C>]>,
    ) -> Self {
        Self {
            position,
            dimensions: (
                match index_grid.width.checked_sub(1) {
                    Some(val) => val,
                    None => 0,
                },
                match index_grid.height.checked_sub(1) {
                    Some(val) => val,
                    None => 0,
                },
            ),
            color_data: color_data.into_vec(),
            index_data: index_grid.indices,
        }
    }

    fn _get_pixel_color(&self, xy_comb: usize) -> Option<u32> {
        /*         debug_assert_eq!(
            self.index_data.len(),
            (self.dimensions.0 + 1) * (self.dimensions.1 + 1)
        ); */
        let color_index = unsafe { *self.index_data.get_unchecked(xy_comb) };
        debug_assert!((color_index as usize) < self.color_data.len());
        Some(color_index)
    }
    pub fn get_pixel_color(&self, x: usize, y: usize) -> Option<u32> {
        if x > self.dimensions.0 || y > self.dimensions.1 {
            None
        } else {
            self._get_pixel_color((self.dimensions.0 + 1) * y + x)
        }
    }

    fn _get_mut_pixel_color(&mut self, xy_comb: usize) -> Option<&mut u32> {
        /*         debug_assert_eq!(
            self.index_data.len(),
            (self.dimensions.0 + 1) * (self.dimensions.1 + 1)
        ); */
        let color_index = unsafe { self.index_data.get_unchecked_mut(xy_comb) };
        debug_assert!((*color_index as usize) < self.color_data.len());
        Some(color_index)
    }
    pub fn get_mut_pixel_color(&mut self, x: usize, y: usize) -> Option<&mut u32> {
        self._get_mut_pixel_color((self.dimensions.0 + 1) * y + x)
    }
    /// Gives an iterator over all the pixels and gives their color
    pub fn pixel_color_iter<'a>(&'a self) -> impl Iterator<Item = Vector3<C>> + 'a {
        self.index_data.iter().map(|&_color_index| {
            let color_index = _color_index as usize;

            // now each index must be within color
            debug_assert!(color_index < self.color_data.len());

            if let Some(val) = self.color_data.get(color_index) {
                *val
            } else {
                // in the futureog out the error
                Vector3::from([C::zero(); 3])
            }

            // unsafe { *self.color_data.get_unchecked(color_index) }
        })
    }

    /// Gives an iterator over all the pixels and gives their color
    pub fn pixel_color_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Vector3<C>> + 'a {
        let color_data_len = self.color_data.len();
        let data_p = self.color_data.as_mut_ptr();

        self.index_data.iter().map(move |&_color_index| {
            let color_index = _color_index as usize;
            // now each index must be within color
            debug_assert!(color_index < color_data_len);

            unsafe { &mut *data_p.add(color_index) }
        })
    }

    /// Gives an iterator of the grid positions
    /// as rectangles
    pub fn get_position_iter(&self) -> impl Iterator<Item = shapes::Rectangle<V>> {
        let (width, height) = {
            // note that we use the 0 based dims on purpose
            let (w, h) = self.get_actual_dimensions();
            (V::from_usize(w), V::from_usize(h))
        };
        let pos = self.position;

        let u_inc = V::one() / width;
        let v_inc = V::one() / height;

        self.get_grid_iter().map(move |(cx, cy)| {
            // now we have our iteration of cx and cy

            let u = cx * u_inc;
            let du = (cx + V::one()) * u_inc;
            let v = cy * v_inc;
            let dv = (cy + V::one()) * v_inc;

            // In the future we will switch to matrices,
            // but this simple variant is fine for now

            // note: cx and cy go top left to right, top to bottom
            // so u and v will do so as well
            let rect_uv_space = [(u, dv), (du, dv), (du, v), (u, v)];
            let r = rect_uv_space.map(|pair| {
                Vector3::lerp(
                    pos.top_left_corner.lerp(pos.top_right_corner, pair.0),
                    pos.bottom_left_corner.lerp(pos.bottom_right_corner, pair.0),
                    pair.1,
                )
            });

            shapes::Rectangle::from(r)
        })
    }

    pub fn get_pixels_iter(&self) -> impl Iterator<Item = RectangleSolidColor<V, C>> + '_ {
        self.get_position_iter()
            .zip(self.pixel_color_iter())
            .map(|(rect, color)| RectangleSolidColor::new1(rect, color))
    }
}
impl<V: GLVertexType + Element, C: GLVertexType + Element> IntoDrawable
    for PixelGridSolidColorIndividual<V, C>
{
    type IntoDrawable = Component<V, C>;
    fn into_drawable(self) -> Self::IntoDrawable {
        let (width, height) = self.get_actual_dimensions();
        let v_data = self
            .get_position_iter()
            .flat_map(|rect| {
                // i've said this elsewhere,
                // but this is really inefficient
                // without color having it's own indices
                // but, shader generalization is coming
                // somewhat soon so idc right now
                // do all of that stuff later
                bytemuck::must_cast::<_, [V; 4 * 3]>(rect)
            })
            .collect();
        let c_data = self
            .pixel_color_iter()
            .flat_map(|color| std::iter::repeat(color).take(4))
            .flat_map(|vec| vec.into_iter())
            .collect();

        let i_data = [[
            0u32,               // 0
            1u32,               // 1
            1u32 + 1u32,        // 2
            0u32,               // 0
            1u32 + 1u32,        // 2
            1u32 + 1u32 + 1u32, // 3
        ]]
        .into_iter()
        .cycle()
        .take(width * height)
        .enumerate()
        .flat_map(|(cx, rect_indices)| rect_indices.map(|val| val + (cx * 4) as u32))
        .collect();

        Component::new(v_data, c_data, i_data)
    }
}

/// Solid colored Rectangular Prism
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RectPrismSolidColor<V: GLVertexType + Element, C: GLVertexType + Element> {
    pub verts: [Vector3<V>; 8],
    pub color: Vector3<C>, // rgb
}

impl<V: GLVertexType + Element, C: GLVertexType + Element> RectPrismSolidColor<V, C> {
    /// Creates a Rectangular Prism using a corner and displacement magnitudes
    pub fn new1(base: Vector3<V>, xm: V, ym: V, zm: V, color: Vector3<C>) -> Self {
        // a Rectangular Prism is just 6 rectangles
        //first 0,1,2,3 are xz plane, then next 4 are shifted back z
        let xm_ = Vector3::from([xm, V::zero(), V::zero()]);
        let ym_ = Vector3::from([V::zero(), ym, V::zero()]);
        let zm_ = Vector3::from([V::zero(), V::zero(), zm]);

        let base_shifted = base.add(zm_);

        let verts = [
            base,                           // bottom left
            base.add(xm_),                  //bottom right
            base.add(xm_).add(ym_),         // top right
            base.add(ym_),                  // top left
            base_shifted,                   // bottom left
            base_shifted.add(xm_),          //bottom right
            base_shifted.add(xm_).add(ym_), // top right
            base_shifted.add(ym_),          // top left
        ];
        Self { verts, color }
    }
    /// Creates a Rectangular Prism using a center base and given lengths. So the edge is base + m/2 for example
    pub fn new2(origin_base: Vector3<V>, xm: V, ym: V, zm: V, color: Vector3<C>) -> Self {
        // basically just the above comment
        let base = origin_base.sub(Vector3::from([xm, ym, zm]).div_scalar(V::one() + V::one()));
        Self::new1(base, xm, ym, zm, color)
    }
}
impl<V: GLVertexType + Element, C: GLVertexType + Element> IntoDrawable
    for RectPrismSolidColor<V, C>
{
    type IntoDrawable = Component<V, C>;
    #[inline(always)]
    fn into_drawable(self) -> Self::IntoDrawable {
        let v_data = self
            .verts
            .into_iter()
            .map(|v| v.into_iter())
            .flatten()
            .collect::<Box<_>>();

        let i_data = [
            // Face 1: Front
            [0usize, 1usize, 2usize, 0, 2, 3],
            // Face 2: Back
            [4, 5, 6, 4, 6, 7],
            // Face 3: Left
            [4, 0, 3, 4, 3, 7],
            // Face 4: Right
            [1, 5, 6, 1, 6, 2],
            // Face 5: Top
            [3, 2, 6, 3, 6, 7],
            // Face 6: Bottom
            [4, 5, 1, 4, 1, 0],
        ]
        .into_iter()
        .flatten()
        .map(|i| i as u32)
        .collect::<Box<_>>();
        let c_data = self
            .color
            .into_iter()
            .cycle()
            .take(3 * 8)
            .collect::<Box<_>>();
        Component::new(v_data, c_data, i_data)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RectPrismMultiColor<V: GLVertexType + Element, C: GLVertexType + Element> {
    pub verts: [Vector3<V>; 8],
    pub colors: [Vector3<C>; 6], // rgb
}

impl<V: GLVertexType + Element, C: GLVertexType + Element> RectPrismMultiColor<V, C> {
    /// Creates a Rectangular Prism using a corner and displacement magnitudes
    /// the colors are supplied in the following order:
    /// front,back,left,right,top,bottom
    pub fn new1(base: Vector3<V>, xm: V, ym: V, zm: V, colors: [Vector3<C>; 6]) -> Self {
        // a Rectangular Prism is just 8 points
        //first 0,1,2,3 are xz plane, then next 4 are shifted back z
        let xm_ = Vector3::from([xm, V::zero(), V::zero()]);
        let ym_ = Vector3::from([V::zero(), ym, V::zero()]);
        let zm_ = Vector3::from([V::zero(), V::zero(), zm]);

        let base_shifted = base.add(zm_);

        let verts = [
            base,                           // bottom left
            base.add(xm_),                  //bottom right
            base.add(xm_).add(ym_),         // top right
            base.add(ym_),                  // top left
            base_shifted,                   // bottom left
            base_shifted.add(xm_),          //bottom right
            base_shifted.add(xm_).add(ym_), // top right
            base_shifted.add(ym_),          // top left
        ];
        Self { verts, colors }
    }
    /// Creates a Rectangular Prism using a center base and given lengths. So the edge is base + m/2 for example
    /// the colors are supplied in the following order:
    /// front,back,left,right,top,bottom
    pub fn new2(origin_base: Vector3<V>, xm: V, ym: V, zm: V, colors: [Vector3<C>; 6]) -> Self {
        // basically just the above comment
        let base = origin_base.sub(Vector3::from([xm, ym, zm]).div_scalar(V::one() + V::one()));
        Self::new1(base, xm, ym, zm, colors)
    }
}
impl<V: GLVertexType + Element, C: GLVertexType + Element> IntoDrawable
    for RectPrismMultiColor<V, C>
{
    type IntoDrawable = Component<V, C>;
    #[inline(always)]
    fn into_drawable(self) -> Self::IntoDrawable {
        let vertices = self.verts;

        let colors = self.colors;

        let indices = [
            // Face 1: Front
            [0usize, 1usize, 2usize, 0, 2, 3],
            // Face 2: Back
            [4, 5, 6, 4, 6, 7],
            // Face 3: Left
            [4, 0, 3, 4, 3, 7],
            // Face 4: Right
            [1, 5, 6, 1, 6, 2],
            // Face 5: Top
            [3, 2, 6, 3, 6, 7],
            // Face 6: Bottom
            [4, 5, 1, 4, 1, 0],
        ];

        let v_data = indices
            .into_iter()
            .map(|face| face.into_iter().map(|i| vertices[i].into_iter()).flatten())
            .flatten()
            .collect::<Box<_>>();

        let i_data = (0usize..36)
            .into_iter()
            .map(|i| i as u32)
            .collect::<Box<_>>();

        let c_data = colors
            .into_iter()
            .map(|face| face.into_iter().cycle().take(3 * 6))
            .flatten()
            .collect::<Box<_>>();

        Component::new(v_data, c_data, i_data)
    }
}

/// A rectangle with uvs as well
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RectangleUVs<V: GLVertexType + Element, U: GLVertexType + Element> {
    pub rect: shapes::Rectangle<V>,
    pub uvs: shapes::Rectangle2d<U>,
}
impl<V: GLVertexType + Element, U: GLVertexType + Element> RectangleUVs<V, U> {
    /// Creates a rectangle using the bare shape primitive
    pub fn new1(rect: shapes::Rectangle<V>, uvs: shapes::Rectangle2d<U>) -> Self {
        Self { rect, uvs }
    }
    /// Creates a rectangle using 4 points and the uv rect
    pub fn new2(
        bottom_left_corner: Vector3<V>,
        bottom_right_corner: Vector3<V>,
        top_right_corner: Vector3<V>,
        top_left_corner: Vector3<V>,
        uv: shapes::Rectangle2d<U>,
    ) -> Self {
        Self::new1(
            shapes::Rectangle {
                bottom_left_corner,
                bottom_right_corner,
                top_right_corner,
                top_left_corner,
            },
            uv,
        )
    }
    /// does Self::new1() but sets the uvs to be top left 0,0, bottom right 1,1
    pub fn new1_auto(rect: shapes::Rectangle<V>) -> Self {
        let uvs = shapes::Rectangle2d::new(
            Vector2::from([U::zero(), U::one()]),
            Vector2::from([U::one(), U::one()]),
            Vector2::from([U::one(), U::zero()]),
            Vector2::from([U::zero(), U::zero()]),
        );
        Self::new1(rect, uvs)
    }
    pub fn to_mesh(self, key_position: u32, key_uvs: u32) -> Mesh {
        let Self { rect, uvs: uv } = self;

        let i_data = [
            0u32,               // 0
            1u32,               // 1
            1u32 + 1u32,        // 2
            0u32,               // 0
            1u32 + 1u32,        // 2
            1u32 + 1u32 + 1u32, // 3
        ];

        let pos = [
            rect.bottom_left_corner,
            rect.bottom_right_corner,
            rect.top_right_corner,
            rect.top_left_corner,
        ];

        let uvs = [
            uv.bottom_left_corner,
            uv.bottom_right_corner,
            uv.top_right_corner,
            uv.top_left_corner,
        ];

        let pos_data =
            DynamicData::new2(Some(&pos), 3, Layout::new(key_position, 3, V::to_glenum()));
        let uv_data = DynamicData::new2(Some(&uvs), 3, Layout::new(key_uvs, 2, U::to_glenum()));

        Mesh::from_buffers(
            IndexBufferDataU32::new(Some(&i_data)),
            HashMap::from([(key_position, pos_data), (key_uvs, uv_data)]),
        )
    }
}
