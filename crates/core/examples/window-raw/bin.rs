use std::ffi::c_void;

use winter_core::bindings;

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
        bindings::load_with(proc_loader);

        glfw::ffi::glfwSwapInterval(1);
        while glfw::ffi::glfwWindowShouldClose(window) == 0 {
            bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
            bindings::Clear(bindings::COLOR_BUFFER_BIT);

            glfw::ffi::glfwSwapBuffers(window);
            glfw::ffi::glfwPollEvents();
        }
        glfw::ffi::glfwDestroyWindow(window);
        glfw::ffi::glfwTerminate();
    }
    Ok(())
}
