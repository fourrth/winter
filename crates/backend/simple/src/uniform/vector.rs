use glmath::vector::Vector4;
use winter_core::{
    bindings::{self, types::GLfloat},
    uniform::Uniform,
};

use super::GLUniform;

#[derive(Debug)]
pub struct Vec4(i32);
impl Uniform<&Vector4<GLfloat>> for Vec4 {
    fn update(&self, data: &Vector4<GLfloat>) {
        unsafe { bindings::Uniform4fv(self.0, 1, data as *const _ as *const GLfloat) };
    }
}
impl From<i32> for Vec4 {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl GLUniform<&Vector4<GLfloat>> for Vec4 {}
