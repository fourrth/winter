use winter_core::{
    bindings::{
        self,
        types::{GLfloat, GLuint},
    },
    uniform::Uniform,
};

use super::GLUniform;

#[derive(Debug)]
pub struct Float(i32);
impl Uniform<GLfloat> for Float {
    fn update(&self, data: GLfloat) {
        unsafe { bindings::Uniform1f(self.0, data) };
    }
}
impl From<i32> for Float {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl GLUniform<GLfloat> for Float {}

#[derive(Debug)]
pub struct Uint(i32);
impl Uniform<GLuint> for Uint {
    fn update(&self, data: GLuint) {
        unsafe { bindings::Uniform1ui(self.0, data) };
    }
}
impl From<i32> for Uint {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl GLUniform<GLuint> for Uint {}
