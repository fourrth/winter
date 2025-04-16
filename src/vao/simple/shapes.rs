use bytemuck::{Pod, Zeroable};
use glmath::{vector::Vector3, Element};

use crate::opengl::GLVertexType;

//TODO: make this more expansive
pub trait Translate<V: GLVertexType + Element> {
    fn shift(self, direction: Vector3<V>) -> Self;
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle<V: GLVertexType + Element> {
    pub bottom_left: Vector3<V>,
    pub bottom_right: Vector3<V>,
    pub top: Vector3<V>,
}
impl<V: GLVertexType + Element> Triangle<V> {
    pub fn new(bottom_left: Vector3<V>, bottom_right: Vector3<V>, top: Vector3<V>) -> Self {
        Triangle {
            bottom_left,
            bottom_right,
            top,
        }
    }
}
unsafe impl<V: GLVertexType + Element> Pod for Triangle<V> {}
unsafe impl<V: GLVertexType + Element> Zeroable for Triangle<V> {
    fn zeroed() -> Self {
        Self {
            bottom_left: Vector3::zeroed(),
            bottom_right: Vector3::zeroed(),
            top: Vector3::zeroed(),
        }
    }
}
impl<V: GLVertexType + Element> Translate<V> for Triangle<V> {
    fn shift(self, direction: Vector3<V>) -> Self {
        Self {
            bottom_left: self.bottom_left.add(direction),
            bottom_right: self.bottom_right.add(direction),
            top: self.top.add(direction),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rectangle<V: GLVertexType + Element> {
    pub bottom_left_corner: Vector3<V>,
    pub bottom_right_corner: Vector3<V>,
    pub top_right_corner: Vector3<V>,
    pub top_left_corner: Vector3<V>,
}
impl<V: GLVertexType + Element> Rectangle<V> {
    pub fn new(
        bottom_left_corner: Vector3<V>,
        bottom_right_corner: Vector3<V>,
        top_right_corner: Vector3<V>,
        top_left_corner: Vector3<V>,
    ) -> Self {
        Self {
            bottom_left_corner,
            bottom_right_corner,
            top_right_corner,
            top_left_corner,
        }
    }
    pub fn to_triangles(self) -> [Triangle<V>; 2] {
        [
            Triangle {
                bottom_left: self.bottom_left_corner,
                bottom_right: self.bottom_right_corner,
                top: self.top_right_corner,
            },
            Triangle {
                bottom_left: self.bottom_left_corner,
                bottom_right: self.top_right_corner,
                top: self.top_left_corner,
            },
        ]
    }
}

unsafe impl<V: GLVertexType + Element> Pod for Rectangle<V> {}
unsafe impl<V: GLVertexType + Element> Zeroable for Rectangle<V> {
    fn zeroed() -> Self {
        Self {
            bottom_left_corner: Vector3::zeroed(),
            bottom_right_corner: Vector3::zeroed(),
            top_right_corner: Vector3::zeroed(),
            top_left_corner: Vector3::zeroed(),
        }
    }
}
impl<V: GLVertexType + Element> Translate<V> for Rectangle<V> {
    fn shift(self, direction: Vector3<V>) -> Self {
        Rectangle {
            bottom_left_corner: self.bottom_left_corner.add(direction),
            bottom_right_corner: self.bottom_right_corner.add(direction),
            top_right_corner: self.top_right_corner.add(direction),
            top_left_corner: self.top_left_corner.add(direction),
        }
    }
}
