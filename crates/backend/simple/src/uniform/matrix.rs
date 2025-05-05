use glmath::matrix::Matrix4x4;
use winter_core::{
    bindings::{self, types::GLfloat},
    uniform::Uniform,
};

use super::GLUniform;

#[derive(Debug)]
pub struct Mat4x4(i32);
impl Uniform<&Matrix4x4<GLfloat>> for Mat4x4 {
    fn update(&self, data: &Matrix4x4<GLfloat>) {
        unsafe {
            bindings::UniformMatrix4fv(
                self.0,
                1,
                bindings::FALSE,
                data as *const _ as *const GLfloat,
            )
        };
    }
}
impl From<i32> for Mat4x4 {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl GLUniform<&Matrix4x4<GLfloat>> for Mat4x4 {}
