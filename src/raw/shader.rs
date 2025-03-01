use std::{mem::MaybeUninit, ptr};

use glad_gles2::gl::{self, GLchar, GLenum, GLint, GLsizei, GLuint};

#[allow(invalid_value)]
fn Message_Error_Helper(
    id: GLuint,
    GetWhativ: unsafe fn(GLuint, GLenum, *mut GLint) -> (),
    GetWhatInfoLog: unsafe fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
) -> String {
    unsafe {
        let mut length = MaybeUninit::uninit().assume_init();

        GetWhativ(id, gl::INFO_LOG_LENGTH, &mut length);

        // length cannot be -1 because we have an info log
        // if we didn't then it would be -1
        let mut msg: Vec<i8> = Vec::with_capacity(length as usize);
        msg.set_len(length as usize);

        GetWhatInfoLog(id, length, ptr::null_mut(), msg.as_mut_ptr());
        let a = std::slice::from_raw_parts(msg.as_ptr() as *const u8, (length - 1) as usize);
        std::str::from_utf8(a).unwrap().to_string()
    }
}

#[inline]
#[allow(invalid_value)]
pub unsafe fn CreateShader(
    SHADERTYPE: GLuint,
    source: *const i8,
    source_len: GLint,
) -> Result<GLuint, String> {
    unsafe {
        let id: GLuint = gl::CreateShader(SHADERTYPE);
        gl::ShaderSource(id, 1, &source, &source_len);
        gl::CompileShader(id);

        #[cfg(debug_assertions)]
        {
            let mut result: GLint = MaybeUninit::uninit().assume_init();
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result as *mut GLint);

            if result == 0 {
                let msg_str = Message_Error_Helper(id, gl::GetShaderiv, gl::GetShaderInfoLog);

                DeleteShader(id);
                match SHADERTYPE {
                    gl::FRAGMENT_SHADER => {
                        return Err(format!("Error in Fragment Shader; {}\n", msg_str));
                    }
                    gl::VERTEX_SHADER => {
                        return Err(format!("Error in Vertex Shader; {}\n", msg_str))
                    }
                    _ => {
                        return Err(format!("Error: SHADERTYPE: {:#X}; {}", SHADERTYPE, msg_str));
                    }
                }
            }
        }
        Ok(id)
    }
}

#[inline]
pub unsafe fn DeleteShader(id: GLuint) {
    gl::DeleteShader(id);
}

#[inline]
pub unsafe fn CreateProgram(
    vertex_shader_text: *const i8,
    vertex_shader_text_len: GLint,
    fragment_shader_text: *const i8,
    fragment_shader_text_len: GLint,
) -> Result<GLuint, String> {
    unsafe {
        let vertex_shader = match CreateShader(
            gl::VERTEX_SHADER,
            vertex_shader_text,
            vertex_shader_text_len,
        ) {
            Ok(val) => val,
            E => return E,
        };

        let fragment_shader = match CreateShader(
            gl::FRAGMENT_SHADER,
            fragment_shader_text,
            fragment_shader_text_len,
        ) {
            Ok(val) => val,
            E => return E,
        };

        let program: GLuint = match gl::CreateProgram() {
            0 => {
                return Err(String::from("Could Not Create Program"));
            }
            val => val,
        };

        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);

        gl::LinkProgram(program);

        #[cfg(debug_assertions)]
        {
            let mut result: GLint = 0;

            gl::GetProgramiv(program, gl::LINK_STATUS, &mut result as *mut GLint);

            if result == 0 {
                // if we did not link correctly

                let msg_str =
                    Message_Error_Helper(program, gl::GetProgramiv, gl::GetProgramInfoLog);

                return Err(format!("Error Linking Program; {}", msg_str));
            }

            gl::ValidateProgram(program);
            gl::GetProgramiv(program, gl::VALIDATE_STATUS, &mut result as *mut GLint);
            if result == 0 {
                let msg_str =
                    Message_Error_Helper(program, gl::GetProgramiv, gl::GetProgramInfoLog);

                return Err(format!("Error Validating Program; {}", msg_str));
            }
        }

        DeleteShader(vertex_shader);
        DeleteShader(fragment_shader);

        Ok(program)
    }
}

#[inline]
pub unsafe fn DeleteProgram(id: GLuint) {
    gl::DeleteProgram(id);
}
