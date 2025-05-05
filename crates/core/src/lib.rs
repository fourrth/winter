//! TODO: DOCUMENTATION

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub mod buffer;
pub mod opengl;
pub mod raw;
pub mod uniform;
pub mod vao;

#[cfg(target_pointer_width = "64")]
pub type NonZeroUInt = std::num::NonZeroU32;

#[cfg(target_pointer_width = "32")]
pub type NonZeroUInt = std::num::NonZeroU16;

pub fn roll_gl_errors() {
    unsafe {
        loop {
            let error = bindings::GetError();
            if error != bindings::NO_ERROR {
                println!("OpenGL error: {}", error);
                panic!("HIT GL ERROR!!!");
                // Handle or log the error as needed
            } else {
                break; // No more errors
            }
        }
    }
}
