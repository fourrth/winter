//! This is a collection of quick and dirty functions that are good to have
//!
//! This module is very unstable and very subject to change

use crate::bindings;

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
