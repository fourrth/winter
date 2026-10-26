//

use crate::{
    raw::{
        self,
        textures::{TextureTarget, TextureUnit},
    },
    NonZeroUInt,
};

pub mod texture2d;

#[derive(Debug)]
struct Guard {
    pub inner: NonZeroUInt,
}
impl Drop for Guard {
    fn drop(&mut self) {
        unsafe { raw::textures::DeleteTexture(self.inner.into()) }
    }
}

pub trait Texture {
    fn id(&self) -> NonZeroUInt;
    fn target(&self) -> TextureTarget;
    fn assign(&self, binding: TextureUnit);
    fn bind(&self);
}
