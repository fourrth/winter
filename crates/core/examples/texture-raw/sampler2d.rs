use winter_core::{bindings, raw::textures::TextureUnit};

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
