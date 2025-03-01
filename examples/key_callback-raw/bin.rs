use std::{
    ffi::{c_int, c_void},
    sync::Mutex,
};

use glad_gles2::gl;
use glfw::ffi::GLFWwindow;

extern "C" fn key_callback(
    window: *mut GLFWwindow,
    key: c_int,
    _scancode: c_int,
    action: c_int,
    _mods: c_int,
) {
    static LAST_KEY: Mutex<Option<c_int>> = Mutex::new(None);

    // Note we are not handling any mutex poisoning
    // because that is kind of specific and if that
    // is possible, you can handle the setup yourself

    if action == glfw::ffi::PRESS {
        print!("Key Press: {key}");
        if key == glfw::ffi::KEY_ESCAPE {
            if let Some(&last_key) = LAST_KEY.lock().unwrap().as_ref() {
                if last_key == glfw::ffi::KEY_ESCAPE {
                    unsafe {
                        glfw::ffi::glfwSetWindowShouldClose(window, glfw::ffi::TRUE);
                    }
                }
            }
        }
        println!();
        *LAST_KEY.lock().unwrap() = Some(key);
    }
}
fn proc_loader(str: &'static str) -> *const c_void {
    unsafe {
        let mut name = str.as_bytes().to_vec();
        name.push(b'\0');
        glfw::ffi::glfwGetProcAddress(name.as_ptr() as *const i8)
    }
}

fn main() -> Result<(), &'static str> {
    unsafe {
        if glfw::ffi::glfwInit() == 0 {
            return Err("Failed to initialize glfw");
        }

        let window = glfw::ffi::glfwCreateWindow(
            640,
            640,
            b"My GLFW Window\0".as_ptr() as *const i8,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if window.is_null() {
            glfw::ffi::glfwTerminate();
            return Err("Failed to Create Window");
        }

        glfw::ffi::glfwMakeContextCurrent(window);
        gl::load(proc_loader);

        glfw::ffi::glfwSetKeyCallback(window, Some(key_callback));

        glfw::ffi::glfwSwapInterval(1);
        println!("Press 'ESCAPE' twice to exit");
        while glfw::ffi::glfwWindowShouldClose(window) == 0 {
            gl::ClearColor(0.8, 0.7, 0.7, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            glfw::ffi::glfwSwapBuffers(window);
            glfw::ffi::glfwPollEvents();
        }
        glfw::ffi::glfwDestroyWindow(window);
        glfw::ffi::glfwTerminate();
    }
    Ok(())
}
