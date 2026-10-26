use std::ffi::{CStr, CString};

use winter_core::{bindings, raw::shader, NonZeroUInt};
use winter_simple::uniform::GLUniform;

pub enum ProgramKind {
    VertexShader(CString),
    FragmentShader(CString),
}

#[derive(Debug)]
pub struct Builder {
    vertex_shader_text: Option<CString>,
    fragment_shader_text: Option<CString>,
}

impl Builder {
    pub fn create() -> Self {
        Builder {
            vertex_shader_text: None,
            fragment_shader_text: None,
        }
    }
    pub fn add(self, kind: ProgramKind) -> Self {
        match kind {
            ProgramKind::FragmentShader(frag_shader) => Self {
                fragment_shader_text: Some(frag_shader),
                vertex_shader_text: self.vertex_shader_text,
            },
            ProgramKind::VertexShader(vertex_shader) => Self {
                fragment_shader_text: self.fragment_shader_text,
                vertex_shader_text: Some(vertex_shader),
            },
        }
    }
    pub fn build(self) -> Result<Program, String> {
        let (vertex_shader_text_, fragment_shader_text_) =
            (self.vertex_shader_text, self.fragment_shader_text);

        if let Some(vertex_shader_text) = vertex_shader_text_ {
            if let Some(fragment_shader_text) = fragment_shader_text_ {
                unsafe {
                    match shader::CreateProgram(
                        vertex_shader_text.as_ptr(),
                        -1,
                        fragment_shader_text.as_ptr(),
                        -1,
                    ) {
                        Ok(id) => Ok(Program { id }),
                        Err(s) => Err(s),
                    }
                }
            } else {
                Err(String::from("vertex shader text not given"))
            }
        } else {
            Err(String::from("fragment shader text not given"))
        }
    }
}

#[derive(Debug)]
pub struct Program {
    id: NonZeroUInt,
}

impl Program {
    pub fn id(&self) -> NonZeroUInt {
        self.id
    }
    pub fn enable(&self) {
        unsafe { bindings::UseProgram(self.id.into()) }
    }
    pub fn uniform<T, U: GLUniform<T>>(&self, name: &CStr) -> Option<U> {
        let out = unsafe { bindings::GetUniformLocation(self.id.into(), name.as_ptr()) };
        U::new(out)
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { shader::DeleteProgram(self.id.into()) };
    }
}
