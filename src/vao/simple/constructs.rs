//! This module contains all of the constructs
//!
//! A construct is a general shape type that
//! is meant to be serialized and deserialized,
//! manipulated, and otherwise modified when
//! creating your mesh
//!
//! Every construct implements Into<Component<>>
//! Which then implements [`Drawable`]

use std::marker::PhantomData;

use glmath::{vector::Vector3, Element};

use crate::opengl::{GLIndexType, GLVertexType};

use super::{primitives::Component, shapes, IntoDrawable};
/// Basic one color triangles
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangleSolidColor<V: GLVertexType + Element, I: GLIndexType, C: GLVertexType + Element>
{
    pub data: shapes::Triangle<V>,
    pub color: Vector3<C>, // rgb
    _i: PhantomData<I>,
}
impl<V: GLVertexType + Element, I: GLIndexType, C: GLVertexType + Element>
    TriangleSolidColor<V, I, C>
{
    /// Create a triangle from the bare shape primitive
    pub fn new1(tri: shapes::Triangle<V>, color: Vector3<C>) -> Self {
        Self {
            data: tri,
            color,
            _i: PhantomData,
        }
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

impl<V: GLVertexType + Element, I: GLIndexType, C: GLVertexType + Element> IntoDrawable<V, I, C>
    for TriangleSolidColor<V, I, C>
{
    type IntoDrawable = Component<V, I, C>;
    #[inline(always)]
    fn into_drawable(self) -> Self::IntoDrawable {
        let v_data = Box::new(bytemuck::must_cast::<shapes::Triangle<V>, [V; 9]>(
            self.data,
        ));
        let c_data = self.color.into_iter().cycle().take(9).collect();
        let i_data = [I::zero(), I::one(), I::one() + I::one()]
            .into_iter()
            .collect();
        Component::new(v_data, c_data, i_data)
    }
}

/// Basic one colored rectanlges or just two triangles together via specified points
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectangleSolidColor<V: GLVertexType + Element, I: GLIndexType, C: GLVertexType + Element>
{
    pub rect: shapes::Rectangle<V>,
    pub color: Vector3<C>, // rgb
    _i: PhantomData<I>,
}
impl<V: GLVertexType + Element, I: GLIndexType, C: GLVertexType + Element>
    RectangleSolidColor<V, I, C>
{
    /// Creates a rectangle using the bare shape primitive
    pub fn new1(rect: shapes::Rectangle<V>, color: Vector3<C>) -> Self {
        Self {
            rect,
            color,
            _i: PhantomData,
        }
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

    pub fn to_triangles(self) -> [TriangleSolidColor<V, I, C>; 2] {
        self.rect
            .to_triangles()
            .map(|tri| TriangleSolidColor::new1(tri, self.color))
    }
}
impl<V: GLVertexType + Element, I: GLIndexType, C: GLVertexType + Element> IntoDrawable<V, I, C>
    for RectangleSolidColor<V, I, C>
{
    type IntoDrawable = Component<V, I, C>;
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
            I::zero(),                      // 0
            I::one(),                       // 1
            I::one() + I::one(),            // 2
            I::zero(),                      // 0
            I::one() + I::one(),            // 2
            I::one() + I::one() + I::one(), // 3
        ]
        .into_iter()
        .collect();

        Component::new(v_data, c_data, i_data)
    }
}

/// A grid of squares or pixels which are individually colored, but
/// entirely that color (so similar to many of [`PlaneSolidColor`])
#[derive(Debug, Clone, PartialEq)]
pub struct PixelGridSolidColorIndividual<
    V: GLVertexType + Element,
    I: GLIndexType,
    C: GLVertexType + Element,
> {
    // since this is just a square grid with
    // different colors for each square,
    // we do the simple thing
    /// Position of the grid in space
    position: shapes::Rectangle<V>,

    /// size of the grid in width-1,height-1
    /// So 0,0 is a 1x1 grid, or single square
    /// note we count from the top left,
    /// left to right, top to bottom
    dimensions: (usize, usize),

    /// Our color data
    pub color_data: Vec<Vector3<C>>,

    /// Our index data which tells us
    /// what color is each square.
    /// so index_data[5] gives the 6th pixel color's
    /// index in color_data. color_data then has the
    /// pixel's actual color
    index_data: Vec<I>,
}

impl<V: GLVertexType + Element, I: GLIndexType, C: GLVertexType + Element>
    PixelGridSolidColorIndividual<V, I, C>
{
    fn _get_pixel_color(&self, xy_comb: usize) -> Option<I> {
        debug_assert_eq!(
            self.index_data.len(),
            (self.dimensions.0 + 1) * (self.dimensions.1 + 1)
        );
        let color_index = unsafe { *self.index_data.get_unchecked(xy_comb) };
        debug_assert!(color_index.to_usize() < self.color_data.len());
        Some(color_index)
    }
    pub fn get_pixel_color(&self, x: usize, y: usize) -> Option<I> {
        if x > self.dimensions.0 || y > self.dimensions.1 {
            None
        } else {
            self._get_pixel_color((self.dimensions.0 + 1) * y + x)
        }
    }

    fn _get_mut_pixel_color(&mut self, xy_comb: usize) -> Option<&mut I> {
        debug_assert_eq!(
            self.index_data.len(),
            (self.dimensions.0 + 1) * (self.dimensions.1 + 1)
        );
        let color_index = unsafe { self.index_data.get_unchecked_mut(xy_comb) };
        debug_assert!(color_index.to_usize() < self.color_data.len());
        Some(color_index)
    }
    pub fn get_mut_pixel_color(&mut self, x: usize, y: usize) -> Option<&mut I> {
        self._get_mut_pixel_color((self.dimensions.0 + 1) * y + x)
    }

    /// Gives an iterator over all the pixels and gives their color
    pub fn pixel_color_iter<'a>(&'a self) -> impl Iterator<Item = Vector3<C>> + 'a {
        self.index_data.iter().map(|&_color_index| {
            let color_index = _color_index.to_usize();
            // now each index must be within color
            debug_assert!(color_index < self.color_data.len());

            unsafe { *self.color_data.get_unchecked(color_index) }
        })
    }

    /// Gives an iterator over all the pixels and gives their color
    pub fn pixel_color_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Vector3<C>> + 'a {
        let color_data_len = self.color_data.len();
        let data_p = self.color_data.as_mut_ptr();

        self.index_data.iter().map(move |&_color_index| {
            let color_index = _color_index.to_usize();
            // now each index must be within color
            debug_assert!(color_index < color_data_len);

            unsafe { &mut *data_p.add(color_index) }
        })
    }
    pub fn drawable(self) -> Component<V, I, C> {
        return unimplemented!();
        let v_data = todo!();
        let c_data = self
            .pixel_color_iter()
            .flat_map(|color| std::iter::repeat(color).take(4))
            .flat_map(|vec| vec.into_iter())
            .collect();

        let i_data = todo!();
        Component::new(v_data, c_data, i_data)
    }
}
