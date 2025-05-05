/// Trait for objects that can fill
/// a uniform in the shader.
/// Type T is the type that the uniform
/// actually represents
pub trait Uniform<T: Sized> {
    /// This function sends the data
    /// and updates the gpu side
    fn update(&self, data: T);
}
