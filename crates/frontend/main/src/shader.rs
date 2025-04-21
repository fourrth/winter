use std::ffi::CString;

use winter_simple::winter_core::{bindings, raw::shader};

#[derive(Debug)]
pub struct Program {
    pub id: u32,
}

impl Program {
    pub fn new(vertex_shader_text: CString, fragment_shader_text: CString) -> Result<Self, String> {
        unsafe {
            let id = shader::CreateProgram(
                vertex_shader_text.as_ptr(),
                -1,
                fragment_shader_text.as_ptr(),
                -1,
            )?;
            Ok(Self { id })
        }
    }
    pub fn enable(&self) {
        unsafe { bindings::UseProgram(self.id) }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { shader::DeleteProgram(self.id) };
    }
}
