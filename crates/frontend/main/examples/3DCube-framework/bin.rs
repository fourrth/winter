// #![allow(unused)]
use std::{
    io::{BufWriter, Write},
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use nalgebra as na;
use winter::{
    camera::{Camera, InputFrame},
    context::{Context, ContextKind},
};
use winter_core::{bindings, uniform::Uniform};
use winter_simple::{
    constructs,
    uniform::{self, GLUniform},
    IntoDrawable, VertexArrayObject,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

fn get_color_from_direction(dir: Direction) -> (f32, f32, f32) {
    match dir {
        // Red
        Direction::PosX => (1f32, 0f32, 0f32),

        // Orange
        Direction::NegX => (1f32, 165f32 / 255f32, 0f32),

        // Green
        Direction::PosY => (0f32, 1f32, 0f32),

        // Blue
        Direction::NegY => (0f32, 0f32, 1f32),

        // Yellow
        Direction::PosZ => (1f32, 1f32, 0f32),

        // White
        Direction::NegZ => (1f32, 1f32, 1f32),
    }
}
// front,back,left,right,top,bottom
const SYSTEM_DIRECTIONS: [Direction; 6] = [
    Direction::PosZ,
    Direction::NegZ,
    Direction::PosX,
    Direction::NegX,
    Direction::PosY,
    Direction::NegY,
];

const PRESS_FORWARD: u32 = 1 << 0;
const PRESS_LEFT: u32 = 1 << 1;
const PRESS_BACK: u32 = 1 << 2;
const PRESS_RIGHT: u32 = 1 << 3;
const PRESS_UP: u32 = 1 << 4;
const PRESS_DOWN: u32 = 1 << 5;
const PRESS_SPRINT: u32 = 1 << 6;

static SHOULD_CLOSE: AtomicBool = AtomicBool::new(false);
static PRESS_KEY: AtomicU32 = AtomicU32::new(0);
static WINDOW_FOCUSED: AtomicBool = AtomicBool::new(true);

const INPUT_TICKS_PER_SECOND: AtomicU32 = AtomicU32::new(100);

// getting around rust and lifetimes...
// this is a ptr to glfw window. starts null/0
static GLFW_WINDOW_HANDLE: AtomicUsize = AtomicUsize::new(0);

fn main() -> Result<(), String> {
    let width = 1600;
    let height = 1600;
    let aspect_ratio = width as f32 / height as f32;

    let title = String::from("Hello Example Framework!");

    let (vertex_shader_text, fragment_shader_text) = {
        (
            String::from(include_str!("vertex_shader.glsl")),
            String::from(include_str!("frag_shader.glsl")),
        )
    };

    let colors =
        SYSTEM_DIRECTIONS.map(|d| glmath::vector::Vector3::from(get_color_from_direction(d)));

    let vao_builder: winter_simple::vao::Builder<f32, u32, f32, 3, false, { bindings::TRIANGLES }> =
        winter_simple::vao::Builder::create().add(
            constructs::RectPrismMultiColor::new2(
                glmath::vector::Vector3::from([0f32, 0f32, 0f32]),
                1f32,
                1f32,
                1f32,
                colors,
            )
            .into_drawable(),
        );

    let mut context: Context<
        winter_simple::vao::Builder<f32, u32, f32, 3, false, { bindings::TRIANGLES }>,
    > = winter::context::Builder::new()
        .add(ContextKind::WindowSize(width, height))
        .add(ContextKind::Title(title))
        .add(ContextKind::VertexShaderText(vertex_shader_text))
        .add(ContextKind::FragmentShaderText(fragment_shader_text))
        .add(ContextKind::VertexArrayObjectData(vao_builder))
        .add(ContextKind::InputFunction(Some(
            |window, key, _, action, _| {
                if action == glfw::ffi::PRESS {
                    match key {
                        glfw::ffi::KEY_ESCAPE => unsafe {
                            glfw::ffi::glfwSetInputMode(
                                window,
                                glfw::ffi::CURSOR,
                                glfw::ffi::CURSOR_NORMAL,
                            );

                            let last_focused = WINDOW_FOCUSED.swap(false, Ordering::Release);
                            if last_focused == false {
                                // so if we unfocsed (like esc'd) and then press escape again, quit
                                glfw::ffi::glfwSetWindowShouldClose(window, glfw::ffi::TRUE);
                            }
                        },
                        glfw::ffi::KEY_UP | glfw::ffi::KEY_W => {
                            let _ = PRESS_KEY.fetch_or(PRESS_FORWARD, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_LEFT | glfw::ffi::KEY_A => {
                            let _ = PRESS_KEY.fetch_or(PRESS_LEFT, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_DOWN | glfw::ffi::KEY_S => {
                            let _ = PRESS_KEY.fetch_or(PRESS_BACK, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_RIGHT | glfw::ffi::KEY_D => {
                            let _ = PRESS_KEY.fetch_or(PRESS_RIGHT, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_SPACE => {
                            let _ = PRESS_KEY.fetch_or(PRESS_UP, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_LEFT_CONTROL => {
                            let _ = PRESS_KEY.fetch_or(PRESS_DOWN, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_LEFT_SHIFT => {
                            let _ = PRESS_KEY.fetch_or(PRESS_SPRINT, Ordering::Relaxed);
                        }

                        _ => {} // nothing
                    }
                } else if action == glfw::ffi::RELEASE {
                    match key {
                        glfw::ffi::KEY_UP | glfw::ffi::KEY_W => {
                            let _ = PRESS_KEY.fetch_and(!PRESS_FORWARD, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_LEFT | glfw::ffi::KEY_A => {
                            let _ = PRESS_KEY.fetch_and(!PRESS_LEFT, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_DOWN | glfw::ffi::KEY_S => {
                            let _ = PRESS_KEY.fetch_and(!PRESS_BACK, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_RIGHT | glfw::ffi::KEY_D => {
                            let _ = PRESS_KEY.fetch_and(!PRESS_RIGHT, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_SPACE => {
                            let _ = PRESS_KEY.fetch_and(!PRESS_UP, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_LEFT_CONTROL => {
                            let _ = PRESS_KEY.fetch_and(!PRESS_DOWN, Ordering::Relaxed);
                        }
                        glfw::ffi::KEY_LEFT_SHIFT => {
                            let _ = PRESS_KEY.fetch_and(!PRESS_SPRINT, Ordering::Relaxed);
                        }

                        _ => {} // nothing
                    }
                }
            },
        )))
        .add(ContextKind::MouseInputFunction(Some(
            |window, button, action, _mods| unsafe {
                if action == glfw::ffi::PRESS {
                    match button {
                        glfw::ffi::MOUSE_BUTTON_LEFT => {
                            WINDOW_FOCUSED.store(true, Ordering::Release);
                            glfw::ffi::glfwSetInputMode(
                                window,
                                glfw::ffi::CURSOR,
                                glfw::ffi::CURSOR_DISABLED,
                            );
                        }
                        _ => {}
                    }
                }
            },
        )))
        .add(ContextKind::WindowFocusFunction(Some(|window, focused| {
            WINDOW_FOCUSED.store(focused == glfw::ffi::TRUE, Ordering::Release);
            if focused == glfw::ffi::TRUE {
                unsafe {
                    // so alt+tab gets rid of the mouse
                    glfw::ffi::glfwSetInputMode(
                        window,
                        glfw::ffi::CURSOR,
                        glfw::ffi::CURSOR_DISABLED,
                    );
                }
            }
        })))
        .add(ContextKind::FrameBufferSizeFunction(Some(
            |_window, width, height| {
                //
                unsafe {
                    bindings::Viewport(0, 0, width, height);
                }
            },
        )))
        .build()?;
    unsafe {
        glfw::ffi::glfwSetInputMode(
            context.window.handle,
            glfw::ffi::RAW_MOUSE_MOTION,
            glfw::ffi::TRUE,
        );
        glfw::ffi::glfwSetInputMode(
            context.window.handle,
            glfw::ffi::CURSOR,
            glfw::ffi::CURSOR_DISABLED,
        );
    }

    // set the window in the atomic
    GLFW_WINDOW_HANDLE.store(context.window.handle as usize, Ordering::Release);

    let camera = Arc::new(Mutex::new(Camera {
        position: na::Point3::new(0f32, 0f32, 3f32),
        yaw: 0f32,
        pitch: 0f32,

        sensitivity: 0.002f32,
        speed: 5f32,
        sprint_multiplier: 2f32,

        fov_y: 60f32.to_radians(),
        aspect: aspect_ratio,
        znear: 0.1f32,
        zfar: 100f32,
    }));

    let camera_clone1 = camera.clone();

    let input_th: thread::JoinHandle<()> = {
        thread::spawn(move || {
            let mut start_time = Instant::now();
            let window = GLFW_WINDOW_HANDLE.load(Ordering::Acquire) as *mut glfw::ffi::GLFWwindow;

            let mut last_xpos = 0f64;
            let mut last_ypos = 0f64;

            let mut xpos = 0f64;
            let mut ypos = 0f64;

            while SHOULD_CLOSE.load(Ordering::Relaxed) == false {
                let now = Instant::now();

                unsafe {
                    glfw::ffi::glfwGetCursorPos(window, &mut xpos, &mut ypos);
                }
                let delta_x = xpos - last_xpos;
                let delta_y = ypos - last_ypos;

                // period of event ticks
                let target = 1f32 / (INPUT_TICKS_PER_SECOND.load(Ordering::Relaxed) as f32);

                let press = PRESS_KEY.load(Ordering::Relaxed);

                let forward = press & PRESS_FORWARD != 0;
                let left = press & PRESS_LEFT != 0;
                let backward = press & PRESS_BACK != 0;
                let right = press & PRESS_RIGHT != 0;
                let up = press & PRESS_UP != 0;
                let down = press & PRESS_DOWN != 0;
                let sprint = press & PRESS_SPRINT != 0;

                let input = InputFrame {
                    forward,
                    backward,
                    left,
                    right,
                    up,
                    down,
                    sprint,
                };

                if WINDOW_FOCUSED.load(Ordering::Acquire) {
                    let mut cam = camera_clone1.lock().unwrap();
                    cam.apply_mouse_delta(delta_x as f32, delta_y as f32);
                    cam.apply_keyboard(&input, (now - start_time).as_secs_f32());
                }

                let sleep_time = target - start_time.elapsed().as_secs_f32();
                thread::sleep(Duration::from_secs_f32(
                    sleep_time.clamp(0f32, f32::INFINITY),
                ));

                start_time = now;

                last_xpos = xpos;
                last_ypos = ypos;
            }
        })
    };

    let u_mvp = uniform::Mat4x4::new(5).unwrap();

    unsafe {
        let mut writer: BufWriter<std::io::StdoutLock<'static>> =
            BufWriter::new(std::io::stdout().lock());

        context.program.enable();
        context.vao.bind();
        let mut start_time = Instant::now();

        let mut cnt = 0u32;

        bindings::Enable(bindings::DEPTH_TEST);

        let _start = Instant::now();
        while context.window.should_close() == false {
            glfw::ffi::glfwPollEvents();

            glfw::ffi::glfwGetFramebufferSize(
                context.window.handle,
                &mut context.window.width,
                &mut context.window.height,
            );

            let (view, proj, model) = {
                let cam = camera.lock().unwrap();
                (
                    cam.view_matrix(),
                    cam.projection_matrix(),
                    na::Matrix4::identity(),
                )
            };

            let mvp = proj * view * model;

            u_mvp.update(&glmath::matrix::Matrix4x4::from(
                mvp.data.0.map(|r| glmath::vector::Vector4::from(r)),
            ));

            bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
            bindings::Clear(bindings::COLOR_BUFFER_BIT | bindings::DEPTH_BUFFER_BIT);
            context.vao.draw();

            glfw::ffi::glfwSwapBuffers(context.window.handle);

            let elapsed = start_time.elapsed();
            start_time = Instant::now();
            let fps = 1f32 / elapsed.as_secs_f32();

            let _ = write!(writer, "FPS: {:.2}----------------------------\r", fps,);
            if cnt % 10 == 0 {
                let _ = writer.flush();
            }
            cnt += 1;
        }
        SHOULD_CLOSE.store(true, Ordering::Relaxed);
    }
    input_th.join().unwrap();

    Ok(())
}
