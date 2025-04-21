use bytemuck::{Pod, Zeroable};
use glmath::{
    vector::{Vector3, Vector4},
    Element,
};
use winter_core::opengl::GLVertexType;

//TODO: change this when glmath updates
fn vector4_to_vector3<T: Element>(v: Vector4<T>) -> Vector3<T> {
    Vector3::from((v[0], v[1], v[2]))
}
// homo norm too
fn vector4_to_vector3_norm<T: Element>(v: Vector4<T>) -> Vector3<T> {
    Vector3::from((v[0] / v[3], v[1] / v[3], v[2] / v[3]))
}

//TODO: make this more expansive
pub trait Translate<V: GLVertexType + Element> {
    fn shift(self, direction: Vector3<V>) -> Self;
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Triangle<V: GLVertexType + Element> {
    pub bottom_left: Vector3<V>,
    pub bottom_right: Vector3<V>,
    pub top: Vector3<V>,
}
impl<V: GLVertexType + Element> From<[Vector3<V>; 3]> for Triangle<V> {
    fn from(value: [Vector3<V>; 3]) -> Self {
        Self {
            bottom_left: value[0],
            bottom_right: value[1],
            top: value[2],
        }
    }
}
impl<V: GLVertexType + Element> Triangle<V> {
    pub fn new(bottom_left: Vector3<V>, bottom_right: Vector3<V>, top: Vector3<V>) -> Self {
        Self {
            bottom_left,
            bottom_right,
            top,
        }
    }
    #[allow(non_snake_case)]
    pub fn to_4D(self) -> Triangle4D<V> {
        Triangle4D {
            bottom_left: Vector4::from((self.bottom_left, V::one())),
            bottom_right: Vector4::from((self.bottom_right, V::one())),
            top: Vector4::from((self.top, V::one())),
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Triangle4D<V: GLVertexType + Element> {
    pub bottom_left: Vector4<V>,
    pub bottom_right: Vector4<V>,
    pub top: Vector4<V>,
}
impl<V: GLVertexType + Element> From<[Vector4<V>; 3]> for Triangle4D<V> {
    fn from(value: [Vector4<V>; 3]) -> Self {
        Self {
            bottom_left: value[0],
            bottom_right: value[1],
            top: value[2],
        }
    }
}

impl<V: GLVertexType + Element> Triangle4D<V> {
    #[allow(non_snake_case)]
    pub fn to_3D(self) -> Triangle<V> {
        Triangle {
            bottom_left: vector4_to_vector3(self.bottom_left),
            bottom_right: vector4_to_vector3(self.bottom_right),
            top: vector4_to_vector3(self.top),
        }
    }
    #[allow(non_snake_case)]
    pub fn to_3D_norm(self) -> Triangle<V> {
        Triangle {
            bottom_left: vector4_to_vector3_norm(self.bottom_left),
            bottom_right: vector4_to_vector3_norm(self.bottom_right),
            top: vector4_to_vector3_norm(self.top),
        }
    }
    pub fn new(bottom_left: Vector4<V>, bottom_right: Vector4<V>, top: Vector4<V>) -> Self {
        Triangle4D {
            bottom_left,
            bottom_right,
            top,
        }
    }
}
unsafe impl<V: GLVertexType + Element> Pod for Triangle4D<V> {}
unsafe impl<V: GLVertexType + Element> Zeroable for Triangle4D<V> {
    fn zeroed() -> Self {
        Self {
            bottom_left: Vector4::zeroed(),
            bottom_right: Vector4::zeroed(),
            top: Vector4::zeroed(),
        }
    }
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rectangle<V: GLVertexType + Element> {
    pub bottom_left_corner: Vector3<V>,
    pub bottom_right_corner: Vector3<V>,
    pub top_right_corner: Vector3<V>,
    pub top_left_corner: Vector3<V>,
}

impl<V: GLVertexType + Element> From<[Vector3<V>; 4]> for Rectangle<V> {
    fn from(value: [Vector3<V>; 4]) -> Self {
        Self {
            bottom_left_corner: value[0],
            bottom_right_corner: value[1],
            top_right_corner: value[2],
            top_left_corner: value[3],
        }
    }
}

impl<V: GLVertexType + Element> Rectangle<V> {
    #[allow(non_snake_case)]
    pub fn to_4D(self) -> Rectangle4D<V> {
        Rectangle4D {
            bottom_left_corner: Vector4::from((self.bottom_left_corner, V::one())),
            bottom_right_corner: Vector4::from((self.bottom_right_corner, V::one())),
            top_right_corner: Vector4::from((self.top_right_corner, V::one())),
            top_left_corner: Vector4::from((self.top_left_corner, V::one())),
        }
    }
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
#[repr(C, packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rectangle4D<V: GLVertexType + Element> {
    pub bottom_left_corner: Vector4<V>,
    pub bottom_right_corner: Vector4<V>,
    pub top_right_corner: Vector4<V>,
    pub top_left_corner: Vector4<V>,
}

impl<V: GLVertexType + Element> From<[Vector4<V>; 4]> for Rectangle4D<V> {
    fn from(value: [Vector4<V>; 4]) -> Self {
        Self {
            bottom_left_corner: value[0],
            bottom_right_corner: value[1],
            top_right_corner: value[2],
            top_left_corner: value[3],
        }
    }
}
impl<V: GLVertexType + Element> Rectangle4D<V> {
    #[allow(non_snake_case)]
    pub fn to_3D(self) -> Rectangle<V> {
        Rectangle {
            bottom_left_corner: vector4_to_vector3(self.bottom_left_corner),
            bottom_right_corner: vector4_to_vector3(self.bottom_right_corner),
            top_right_corner: vector4_to_vector3(self.top_right_corner),
            top_left_corner: vector4_to_vector3(self.top_left_corner),
        }
    }
    #[allow(non_snake_case)]
    pub fn to_3D_norm(self) -> Rectangle<V> {
        Rectangle {
            bottom_left_corner: vector4_to_vector3_norm(self.bottom_left_corner),
            bottom_right_corner: vector4_to_vector3_norm(self.bottom_right_corner),
            top_right_corner: vector4_to_vector3_norm(self.top_right_corner),
            top_left_corner: vector4_to_vector3_norm(self.top_left_corner),
        }
    }
    pub fn new(
        bottom_left_corner: Vector4<V>,
        bottom_right_corner: Vector4<V>,
        top_right_corner: Vector4<V>,
        top_left_corner: Vector4<V>,
    ) -> Self {
        Self {
            bottom_left_corner,
            bottom_right_corner,
            top_right_corner,
            top_left_corner,
        }
    }
    pub fn to_triangles(self) -> [Triangle4D<V>; 2] {
        [
            Triangle4D {
                bottom_left: self.bottom_left_corner,
                bottom_right: self.bottom_right_corner,
                top: self.top_right_corner,
            },
            Triangle4D {
                bottom_left: self.bottom_left_corner,
                bottom_right: self.top_right_corner,
                top: self.top_left_corner,
            },
        ]
    }
}

unsafe impl<V: GLVertexType + Element> Pod for Rectangle4D<V> {}
unsafe impl<V: GLVertexType + Element> Zeroable for Rectangle4D<V> {
    fn zeroed() -> Self {
        Self {
            bottom_left_corner: Vector4::zeroed(),
            bottom_right_corner: Vector4::zeroed(),
            top_right_corner: Vector4::zeroed(),
            top_left_corner: Vector4::zeroed(),
        }
    }
}
