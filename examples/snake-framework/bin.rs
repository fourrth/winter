use std::{
    ffi::CString,
    io::{self, BufWriter, Write},
    num::NonZeroU32,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use glad_gles2::gl;
use glmath::vector::Vector2;
use snake::{Coordinate, Direction};
use winter::{common, primitives};

#[inline(always)]
fn clamp_pos(num: f32) -> f32 {
    if num < 0f32 {
        0f32
    } else {
        num
    }
}

fn main() {
    let width = 1000;
    let height = 1000;

    let title = CString::new("Snake Game!").unwrap();

    let (vertex_shader_text, fragment_shader_text) = {
        (
            CString::new(include_str!("vertex_shader.glsl")).unwrap(),
            CString::new(include_str!("frag_shader.glsl")).unwrap(),
        )
    };

    const ARENA_CELL_LENGTH: usize = 30;

    let color_snake_head = primitives::Color::from_rgb(0, 255, 255);
    let color_snake_body = primitives::Color::from_rgb(255, 255, 255);
    let color_snake_empty = primitives::Color::from_rgb(0, 0, 255);
    let color_snake_food = primitives::Color::from_rgb(255, 0, 0);

    let vao_builder = common::create_grid(
        Vector2::from([-1.; 2]).add(Vector2::from([0.1; 2])),
        Vector2::from([1.; 2]).sub(Vector2::from([0.1; 2])),
        NonZeroU32::new(ARENA_CELL_LENGTH as u32).unwrap(),
        NonZeroU32::new(ARENA_CELL_LENGTH as u32).unwrap(),
        0.,
        |_, _, _| color_snake_empty,
    );

    let mut context = winter::Context::new(
        width,
        height,
        title,
        vertex_shader_text,
        fragment_shader_text,
        Some(|window, key, _, action, _| unsafe {
            if action == glfw::ffi::PRESS {
                if key == glfw::ffi::KEY_ESCAPE {
                    glfw::ffi::glfwSetWindowShouldClose(window, glfw::ffi::TRUE);
                } else if key == glfw::ffi::KEY_SPACE {
                    DEBUG_ADD_FOOD_PRESS.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_ENTER {
                    // toggle ticking the snake
                    SHOULD_TICK.fetch_xor(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_UP || key == glfw::ffi::KEY_W {
                    GO_UP_PRESS.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_LEFT || key == glfw::ffi::KEY_A {
                    GO_LEFT_PRESS.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_DOWN || key == glfw::ffi::KEY_S {
                    GO_DOWN_PRESS.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_RIGHT || key == glfw::ffi::KEY_D {
                    GO_RIGHT_PRESS.store(true, Ordering::Relaxed);
                }
            } else if action == glfw::ffi::RELEASE {
                if key == glfw::ffi::KEY_SPACE {
                    DEBUG_ADD_FOOD_RELEASE.store(true, Ordering::Relaxed);
                }
                if key == glfw::ffi::KEY_UP || key == glfw::ffi::KEY_W {
                    GO_UP_RELEASE.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_LEFT || key == glfw::ffi::KEY_A {
                    GO_LEFT_RELEASE.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_DOWN || key == glfw::ffi::KEY_S {
                    GO_DOWN_RELEASE.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_RIGHT || key == glfw::ffi::KEY_D {
                    GO_RIGHT_RELEASE.store(true, Ordering::Relaxed);
                }
            }
        }),
        vao_builder, // board is fully dead right now
    )
    .unwrap();

    static SHOULD_CLOSE: AtomicBool = AtomicBool::new(false);
    static SHOULD_TICK: AtomicBool = AtomicBool::new(false);
    static SHOULD_DIE: AtomicBool = AtomicBool::new(false);

    static TICKS_PER_SECOND: AtomicU64 = AtomicU64::new(10);

    // TODO: switch to using an atomic u8,u32,etc... seriously...
    static GO_UP_PRESS: AtomicBool = AtomicBool::new(false);
    static GO_UP_RELEASE: AtomicBool = AtomicBool::new(false);
    static GO_LEFT_PRESS: AtomicBool = AtomicBool::new(false);
    static GO_LEFT_RELEASE: AtomicBool = AtomicBool::new(false);
    static GO_DOWN_PRESS: AtomicBool = AtomicBool::new(false);
    static GO_DOWN_RELEASE: AtomicBool = AtomicBool::new(false);
    static GO_RIGHT_PRESS: AtomicBool = AtomicBool::new(false);
    static GO_RIGHT_RELEASE: AtomicBool = AtomicBool::new(false);

    static DEBUG_ADD_FOOD_PRESS: AtomicBool = AtomicBool::new(false);
    static DEBUG_ADD_FOOD_RELEASE: AtomicBool = AtomicBool::new(false);

    let snake_context = Arc::new(Mutex::new(
        snake::Builder::create()
            .add(snake::BuildOptions::ArenaDim(
                ARENA_CELL_LENGTH as u64,
                ARENA_CELL_LENGTH as u64,
            ))
            .add(snake::BuildOptions::StartingHeadCoord(Coordinate((6, 6))))
            .add(snake::BuildOptions::StartingMoveDir(Direction::Right))
            .build(),
    ));

    context.program.enable();
    context.vao.bind();

    let snake_context_1 = Arc::clone(&snake_context);
    let tick_th = thread::spawn(move || {
        let mut start = Instant::now();
        while SHOULD_CLOSE.load(Ordering::Relaxed) == false {
            if let Ok(mut cxt) = snake_context_1.lock() {
                if DEBUG_ADD_FOOD_PRESS.load(Ordering::Relaxed) {
                    cxt.add_part = true;
                    if DEBUG_ADD_FOOD_RELEASE.load(Ordering::Relaxed) {
                        DEBUG_ADD_FOOD_PRESS.store(false, Ordering::Relaxed);
                        DEBUG_ADD_FOOD_RELEASE.store(false, Ordering::Relaxed);
                    }
                }
                if SHOULD_TICK.load(Ordering::Relaxed) {
                    match cxt.tick() {
                        snake::GameState::Paused => todo!(),
                        snake::GameState::Running => (),
                        snake::GameState::Stopped => {
                            // for now we just ignore
                        }
                        snake::GameState::Dead => {
                            // do death screen
                            SHOULD_DIE.store(true, Ordering::Relaxed);
                        }
                    };
                }
                // now do movement changes
                // we will just be simple and
                // do it in the order of wasd
                // who cares...
                let dir = if GO_UP_PRESS.load(Ordering::Relaxed) {
                    Direction::Up as u8 + 1
                } else if GO_LEFT_PRESS.load(Ordering::Relaxed) {
                    Direction::Left as u8 + 1
                } else if GO_DOWN_PRESS.load(Ordering::Relaxed) {
                    Direction::Down as u8 + 1
                } else if GO_RIGHT_PRESS.load(Ordering::Relaxed) {
                    Direction::Right as u8 + 1
                } else {
                    0
                };
                if dir != 0 {
                    cxt.move_dir = Direction::try_from(dir - 1).unwrap();
                }
                // now check if we just did a quick move
                // basically we could press and unpress before
                // we have a chance to process the inital press,
                // so we have to use two variables
                if GO_UP_RELEASE.load(Ordering::Relaxed) {
                    GO_UP_PRESS.store(false, Ordering::Relaxed);
                }
                if GO_LEFT_RELEASE.load(Ordering::Relaxed) {
                    GO_LEFT_PRESS.store(false, Ordering::Relaxed);
                }
                if GO_DOWN_RELEASE.load(Ordering::Relaxed) {
                    GO_DOWN_PRESS.store(false, Ordering::Relaxed);
                }
                if GO_RIGHT_RELEASE.load(Ordering::Relaxed) {
                    GO_RIGHT_PRESS.store(false, Ordering::Relaxed);
                }
            }
            let target = 1f32 / (TICKS_PER_SECOND.load(Ordering::Relaxed) as f32);
            let sleep_time = clamp_pos(target - start.elapsed().as_secs_f32());
            thread::sleep(Duration::from_secs_f32(sleep_time));

            start = Instant::now();
        }
    });

    unsafe {
        let stdout = io::stdout();

        let mut writer = BufWriter::new(stdout.lock());

        let mut start = Instant::now();

        let mut move_dir: Direction = Direction::Right;
        let mut score: u64 = 0;

        while context.window.should_close() == false {
            if SHOULD_DIE.load(Ordering::Relaxed) == false {
                gl::ClearColor(0.8, 0.7, 0.7, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                context.vao.indices.bind();

                if let Ok(cxt) = snake_context.lock() {
                    move_dir = cxt.move_dir;
                    score = cxt.score;
                    for (cx, &ca) in cxt.get_arena_iter().enumerate() {
                        let c = match ca {
                            snake::Cell::SnakeHead => color_snake_head,
                            snake::Cell::SnakeBody => color_snake_body,
                            snake::Cell::Empty => color_snake_empty,
                            snake::Cell::Food => color_snake_food,
                        };

                        for triangle_color in context.vao.get_color_component_mut(cx) {
                            for point_color in common::convert_comp_triangle(triangle_color) {
                                *point_color = c.0;
                            }
                        }
                    }
                }
                context.vao.update_color_component_all();

                context.vao.draw();
            } else {
                // otherwise, do our death screen
                // we do this by unbinding the vao
                // and just changing the clearcolor
                gl::ClearColor(1.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::BindVertexArray(0)
            }

            glfw::ffi::glfwGetFramebufferSize(
                context.window.handle,
                &mut context.window.width,
                &mut context.window.height,
            );

            glfw::ffi::glfwSwapBuffers(context.window.handle);
            glfw::ffi::glfwPollEvents();
            let elapsed = start.elapsed();
            let fps = 1f32 / elapsed.as_secs_f32();

            let _ = write!(
                writer,
                "FPS: {:.2}, TICKS/SECOND: {}, MOVE_DIRECTION: {}, SCORE: {}--------------\r",
                fps,
                TICKS_PER_SECOND.load(Ordering::Relaxed),
                move_dir,
                score
            );
            let _ = writer.flush();
            start = Instant::now();
        }
        SHOULD_CLOSE.store(true, Ordering::Relaxed);
    }
    println!();
    tick_th.join().unwrap();
}
