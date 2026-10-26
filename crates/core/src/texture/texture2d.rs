use std::ffi::c_void;

use crate::{
    bindings::{self, types::GLsizei},
    raw::{
        self,
        textures::{
            TextureFormat, TextureInternalFormat, TextureParameter, TextureTarget, TextureType,
            TextureUnit,
        },
    },
    texture::{Guard, Texture},
    NonZeroUInt,
};

#[derive(Debug)]
pub struct Sampler2D(i32);

impl Sampler2D {
    pub fn update(&self, data: TextureUnit) {
        unsafe { bindings::Uniform1i(self.0, data.to_offset()) };
    }
}
impl From<i32> for Sampler2D {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

pub struct Texture2D {
    sampler_uniform: Sampler2D,
    tex: Guard,
    target: TextureTarget,
    program: NonZeroUInt,
}

impl Texture2D {
    pub fn new(
        program: NonZeroUInt,
        width: u32,
        height: u32,
        internal: TextureInternalFormat,
        format: TextureFormat,
        data: *const u8,
        sampler_loc: GLsizei,
        target: TextureTarget,
    ) -> Result<Self, String> {
        unsafe {
            let tex = raw::textures::CreateTexture2D(
                width as GLsizei,
                height as GLsizei,
                internal,
                format,
                TextureType::UnsignedByte,
                data as *const c_void,
            )?;
            bindings::UseProgram(program.into());
            raw::textures::BindTexture(TextureTarget::Texture2D, tex.into());

            raw::textures::TexParameteri(
                TextureTarget::Texture2D,
                TextureParameter::MinFilter,
                bindings::LINEAR as i32,
            );
            raw::textures::TexParameteri(
                TextureTarget::Texture2D,
                TextureParameter::MagFilter,
                bindings::LINEAR as i32,
            );
            let sampler_uniform = Sampler2D::from(sampler_loc);

            bindings::UseProgram(0);
            Ok(Self {
                sampler_uniform,
                tex: Guard { inner: tex },
                target,
                program,
            })
        }
    }
}
impl Texture for Texture2D {
    fn id(&self) -> crate::NonZeroUInt {
        self.tex.inner.into()
    }
    fn bind(&self) {
        unsafe {
            raw::textures::BindTexture(TextureTarget::Texture2D, self.tex.inner.into());
        }
    }
    fn target(&self) -> TextureTarget {
        self.target
    }
    /// leaks shader/auto binds program
    fn assign(&self, binding: TextureUnit) {
        unsafe {
            bindings::UseProgram(self.program.into());
            self.sampler_uniform.update(binding);
            raw::textures::ActiveTexture(binding);
            self.bind();
            let mut bound: i32 = 0;
            bindings::GetIntegerv(bindings::TEXTURE_BINDING_2D, &mut bound);
            assert_eq!(
                bound as u32,
                self.tex.inner.into(),
                "texture not on unit 0 at draw time"
            );
        }
    }
}
