use crate::{
    bindings::{
        self,
        types::{GLenum, GLfloat, GLint, GLsizei, GLuint},
    },
    NonZeroUInt,
};
use std::ffi::c_void;

//todo: find the correct file for this
macro_rules! texture_units {
    ($($name:ident = $n:expr),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum TextureUnit { $($name = $n),* }

        impl TextureUnit {
            #[inline]
            pub const fn to_offset(self) -> GLint { self as GLint }
            #[inline]
            pub const fn to_gl_enum(self) -> GLenum {
                bindings::TEXTURE0 + self.to_offset() as GLenum
            }
        }
    };
}

texture_units! {
    Texture0 = 0,  Texture1 = 1,  Texture2 = 2,  Texture3 = 3,
    Texture4 = 4,  Texture5 = 5,  Texture6 = 6,  Texture7 = 7,
    Texture8 = 8,  Texture9 = 9,  Texture10 = 10, Texture11 = 11,
    Texture12 = 12, Texture13 = 13, Texture14 = 14, Texture15 = 15,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureTarget {
    // Texture1D,
    Texture2D,
    Texture3D,
    // Texture1DArray,
    Texture2DArray,
    // TextureRectangle,
    TextureCubeMap,
    TextureCubeMapArray,
    TextureBuffer,
    Texture2DMultisample,
    Texture2DMultisampleArray,
    TextureCubeMapPositiveX,
    TextureCubeMapNegativeX,
    TextureCubeMapPositiveY,
    TextureCubeMapNegativeY,
    TextureCubeMapPositiveZ,
    TextureCubeMapNegativeZ,
}

impl TextureTarget {
    #[inline]
    pub fn get_glenum(self) -> GLenum {
        match self {
            // Self::Texture1D => bindings::TEXTURE_1D,
            Self::Texture2D => bindings::TEXTURE_2D,
            Self::Texture3D => bindings::TEXTURE_3D,
            // Self::Texture1DArray => bindings::TEXTURE_1D_ARRAY,
            Self::Texture2DArray => bindings::TEXTURE_2D_ARRAY,
            // Self::TextureRectangle => bindings::TEXTURE_RECTANGLE,
            Self::TextureCubeMap => bindings::TEXTURE_CUBE_MAP,
            Self::TextureCubeMapArray => bindings::TEXTURE_CUBE_MAP_ARRAY,
            Self::TextureBuffer => bindings::TEXTURE_BUFFER,
            Self::Texture2DMultisample => bindings::TEXTURE_2D_MULTISAMPLE,
            Self::Texture2DMultisampleArray => bindings::TEXTURE_2D_MULTISAMPLE_ARRAY,
            Self::TextureCubeMapPositiveX => bindings::TEXTURE_CUBE_MAP_POSITIVE_X,
            Self::TextureCubeMapNegativeX => bindings::TEXTURE_CUBE_MAP_NEGATIVE_X,
            Self::TextureCubeMapPositiveY => bindings::TEXTURE_CUBE_MAP_POSITIVE_Y,
            Self::TextureCubeMapNegativeY => bindings::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            Self::TextureCubeMapPositiveZ => bindings::TEXTURE_CUBE_MAP_POSITIVE_Z,
            Self::TextureCubeMapNegativeZ => bindings::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        }
    }
}

impl std::fmt::Debug for TextureTarget {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Texture2D => write!(f, "Texture2D"),
            Self::Texture3D => write!(f, "Texture3D"),
            Self::Texture2DArray => write!(f, "Texture2DArray"),
            Self::TextureCubeMap => write!(f, "TextureCubeMap"),
            Self::TextureCubeMapArray => write!(f, "TextureCubeMapArray"),
            Self::TextureBuffer => write!(f, "TextureBuffer"),
            Self::Texture2DMultisample => write!(f, "Texture2DMultisample"),
            Self::Texture2DMultisampleArray => write!(f, "Texture2DMultisampleArray"),
            Self::TextureCubeMapPositiveX => write!(f, "TextureCubeMapPositiveX"),
            Self::TextureCubeMapNegativeX => write!(f, "TextureCubeMapNegativeX"),
            Self::TextureCubeMapPositiveY => write!(f, "TextureCubeMapPositiveY"),
            Self::TextureCubeMapNegativeY => write!(f, "TextureCubeMapNegativeY"),
            Self::TextureCubeMapPositiveZ => write!(f, "TextureCubeMapPositiveZ"),
            Self::TextureCubeMapNegativeZ => write!(f, "TextureCubeMapNegativeZ"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureParameter {
    MinFilter,
    MagFilter,
    WrapS,
    WrapT,
    WrapR,
    MinLOD,
    MaxLOD,
    BaseLevel,
    MaxLevel,
    CompareMode,
    CompareFunc,
}

impl TextureParameter {
    #[inline]
    pub fn get_glenum(self) -> GLenum {
        match self {
            Self::MinFilter => bindings::TEXTURE_MIN_FILTER,
            Self::MagFilter => bindings::TEXTURE_MAG_FILTER,
            Self::WrapS => bindings::TEXTURE_WRAP_S,
            Self::WrapT => bindings::TEXTURE_WRAP_T,
            Self::WrapR => bindings::TEXTURE_WRAP_R,
            Self::MinLOD => bindings::TEXTURE_MIN_LOD,
            Self::MaxLOD => bindings::TEXTURE_MAX_LOD,
            Self::BaseLevel => bindings::TEXTURE_BASE_LEVEL,
            Self::MaxLevel => bindings::TEXTURE_MAX_LEVEL,
            Self::CompareMode => bindings::TEXTURE_COMPARE_MODE,
            Self::CompareFunc => bindings::TEXTURE_COMPARE_FUNC,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Red,
    RG,
    RGB,
    RGBA,
    RedInteger,
    RGInteger,
    RGBInteger,
    RGBAInteger,
    StencilIndex,
    DepthComponent,
    DepthStencil,
}

impl TextureFormat {
    #[inline]
    pub fn get_glenum(self) -> GLenum {
        match self {
            Self::Red => bindings::RED,
            Self::RG => bindings::RG,
            Self::RGB => bindings::RGB,
            Self::RGBA => bindings::RGBA,
            Self::RedInteger => bindings::RED_INTEGER,
            Self::RGInteger => bindings::RG_INTEGER,
            Self::RGBInteger => bindings::RGB_INTEGER,
            Self::RGBAInteger => bindings::RGBA_INTEGER,
            Self::StencilIndex => bindings::STENCIL_INDEX,
            Self::DepthComponent => bindings::DEPTH_COMPONENT,
            Self::DepthStencil => bindings::DEPTH_STENCIL,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureInternalFormat {
    R8,
    RG8,
    RGB8,
    RGBA8,
    SRGB8,
    SRGB8Alpha8,
    R16F,
    RG16F,
    RGB16F,
    RGBA16F,
    R32F,
    RG32F,
    RGB32F,
    RGBA32F,
    // ... many more
}

impl TextureInternalFormat {
    #[inline]
    pub fn get_glenum(self) -> GLint {
        match self {
            Self::R8 => bindings::R8 as GLint,
            Self::RG8 => bindings::RG8 as GLint,
            Self::RGB8 => bindings::RGB8 as GLint,
            Self::RGBA8 => bindings::RGBA8 as GLint,
            Self::SRGB8 => bindings::SRGB8 as GLint,
            Self::SRGB8Alpha8 => bindings::SRGB8_ALPHA8 as GLint,
            Self::R16F => bindings::R16F as GLint,
            Self::RG16F => bindings::RG16F as GLint,
            Self::RGB16F => bindings::RGB16F as GLint,
            Self::RGBA16F => bindings::RGBA16F as GLint,
            Self::R32F => bindings::R32F as GLint,
            Self::RG32F => bindings::RG32F as GLint,
            Self::RGB32F => bindings::RGB32F as GLint,
            Self::RGBA32F => bindings::RGBA32F as GLint,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureType {
    UnsignedByte,
    Byte,
    UnsignedShort,
    Short,
    UnsignedInt,
    Int,
    Float,
}

impl TextureType {
    #[inline]
    pub fn get_glenum(self) -> GLenum {
        match self {
            Self::UnsignedByte => bindings::UNSIGNED_BYTE,
            Self::Byte => bindings::BYTE,
            Self::UnsignedShort => bindings::UNSIGNED_SHORT,
            Self::Short => bindings::SHORT,
            Self::UnsignedInt => bindings::UNSIGNED_INT,
            Self::Int => bindings::INT,
            Self::Float => bindings::FLOAT,
        }
    }
}

#[inline]
pub unsafe fn GenTexture() -> GLuint {
    let mut id: GLuint = 0;
    bindings::GenTextures(1, &mut id);
    id
}
#[inline]
pub unsafe fn TexImage2D(
    target: TextureTarget,
    level: GLint,
    internal_format: GLint,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
    format: GLenum,
    ty: GLenum,
    data: *const c_void,
) {
    match target {
        TextureTarget::Texture2D => {
            bindings::TexImage2D(
                target.get_glenum(),
                level,
                internal_format,
                width,
                height,
                border,
                format,
                ty,
                data,
            );
        }
        _ => unimplemented!(),
    }
}

#[inline]
pub unsafe fn TexSubImage2D(
    target: TextureTarget,
    level: GLint,
    xoffset: GLint,
    yoffset: GLint,
    width: GLsizei,
    height: GLsizei,
    format: TextureFormat,
    ty: TextureType,
    data: *const c_void,
) {
    match target {
        TextureTarget::Texture2D => {
            bindings::TexSubImage2D(
                target.get_glenum(),
                level,
                xoffset,
                yoffset,
                width,
                height,
                format.get_glenum(),
                ty.get_glenum(),
                data,
            );
        }
        _ => unimplemented!(),
    }
}

#[inline]
pub unsafe fn CreateTexture2D(
    width: GLsizei,
    height: GLsizei,
    internal_format: TextureInternalFormat,
    format: TextureFormat,
    ty: TextureType,
    data: *const c_void,
) -> Result<NonZeroUInt, String> {
    unsafe {
        let id = GenTexture();
        BindTexture(TextureTarget::Texture2D, id);
        if let Some(r) = NonZeroUInt::new(id) {
            TexImage2D(
                TextureTarget::Texture2D,
                0,
                internal_format.get_glenum(),
                width,
                height,
                0,
                format.get_glenum(),
                ty.get_glenum(),
                data,
            );

            #[cfg(debug_assertions)]
            {
                let mut get_width: GLint = 0;
                bindings::GetTexLevelParameteriv(
                    TextureTarget::Texture2D.get_glenum(),
                    0,
                    bindings::TEXTURE_WIDTH,
                    &mut get_width,
                );
                if get_width != width as GLint {
                    DeleteTexture(id);
                    return Err(String::from("Incorrect Texture Width"));
                }
                let mut get_height: GLint = 0;
                bindings::GetTexLevelParameteriv(
                    TextureTarget::Texture2D.get_glenum(),
                    0,
                    bindings::TEXTURE_HEIGHT,
                    &mut get_height,
                );
                if get_height != height as GLint {
                    DeleteTexture(id);
                    return Err(String::from("Incorrect Texture Height"));
                }

                BindTexture(TextureTarget::Texture2D, 0);
            }
            Ok(r)
        } else {
            Err("GenTexture returned id of 0...".to_string())
        }
    }
}

#[inline]
pub unsafe fn DeleteTexture(id: GLuint) {
    bindings::DeleteTextures(1, &id)
}

#[inline]
pub unsafe fn BindTexture(target: TextureTarget, id: GLuint) {
    bindings::BindTexture(target.get_glenum(), id);
}

#[inline]
pub unsafe fn TexParameteri(target: TextureTarget, pname: TextureParameter, param: GLint) {
    bindings::TexParameteri(target.get_glenum(), pname.get_glenum(), param);
}

#[inline]
pub unsafe fn TexParameterf(target: TextureTarget, pname: TextureParameter, param: GLfloat) {
    bindings::TexParameterf(target.get_glenum(), pname.get_glenum(), param);
}

#[inline]
pub unsafe fn GenerateMipmap(target: TextureTarget) {
    bindings::GenerateMipmap(target.get_glenum());
}

#[inline]
pub unsafe fn ActiveTexture(texture: TextureUnit) {
    bindings::ActiveTexture(texture.to_gl_enum());
}
