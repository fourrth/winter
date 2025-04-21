//! This module holds the VertexArrayObject information
//!
//! You can either create your own vao implementation
//! or you can use the preexisting one given in [`default`]

/// Trait for implementing VertexArrayObjects
///
/// Implement this trait on anything that you want
/// to manage the various buffers OpenGL has access to
pub trait VertexArrayObject {
    /// draw should automatically bind the VertexArrayObject
    /// and in general, draw with the mesh
    fn draw(&self);
    fn bind(&self);
}

/// This trait holds the data before telling OpenGL about it
///
/// A type that implements this trait holds the data until
/// it is time to tell OpenGL about it
pub trait VertexArrayObjectData {
    type VAO: VertexArrayObject;
    fn build(self) -> Self::VAO;
}
