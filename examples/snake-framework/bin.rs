use std::{
    ffi::CString,
    io::{self, BufWriter, Write},
    num::NonZeroU32,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use glmath::vector::Vector2;
use snake::{Coordinate, Direction};
use winter::bindings;
use winter::{common, primitives};

#[inline(always)]
fn clamp_pos(num: f32) -> f32 {
    if num < 0f32 {
        0f32
    } else {
        num
    }
}

fn new_snake(width: u64, height: u64) -> snake::Context {
    snake::Builder::create()
        .add(snake::BuildOptions::ArenaDim(width, height))
        .add(snake::BuildOptions::StartingHeadCoord(Coordinate((6, 6))))
        .add(snake::BuildOptions::StartingMoveDir(Direction::Right))
        .build()
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();

    let width = if let Some(width_) = args.get(0) {
        if let Ok(width_) = width_.parse::<i32>() {
            width_
        } else {
            println!("Inputted Width Unknown: using 1000");
            1000
        }
    } else {
        println!("Inputted Width Unknown: using 1000");
        1000
    };

    let height = if let Some(height_) = args.get(1) {
        if let Ok(height_) = height_.parse::<i32>() {
            height_
        } else {
            println!("Inputted height Unknown: using 1000");
            1000
        }
    } else {
        println!("Inputted height Unknown: using 1000");
        1000
    };

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
        Some(|window, key, _, action, _| {
            if action == glfw::ffi::PRESS {
                if key == glfw::ffi::KEY_ESCAPE {
                    unsafe {
                        glfw::ffi::glfwSetWindowShouldClose(window, glfw::ffi::TRUE);
                    }
                } else if key == glfw::ffi::KEY_SPACE {
                    DEBUG_ADD_FOOD_PRESS.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_ENTER {
                    // toggle ticking the snake
                    SHOULD_TICK.fetch_xor(true, Ordering::Relaxed);
                    if SHOULD_TICK.load(Ordering::Relaxed) & SHOULD_DIE.load(Ordering::Relaxed) {
                        SHOULD_RESTART.store(true, Ordering::Relaxed);
                    }
                } else if key == glfw::ffi::KEY_UP || key == glfw::ffi::KEY_W {
                    let _ = PRESS_KEY.fetch_or(1, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_LEFT || key == glfw::ffi::KEY_A {
                    let _ = PRESS_KEY.fetch_or(2, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_DOWN || key == glfw::ffi::KEY_S {
                    let _ = PRESS_KEY.fetch_or(4, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_RIGHT || key == glfw::ffi::KEY_D {
                    let _ = PRESS_KEY.fetch_or(8, Ordering::Relaxed);
                }
            } else if action == glfw::ffi::RELEASE {
                if key == glfw::ffi::KEY_SPACE {
                    DEBUG_ADD_FOOD_RELEASE.store(true, Ordering::Relaxed);
                }
            }
        }),
        vao_builder, // board is fully dead right now
    )
    .unwrap();

    static SHOULD_CLOSE: AtomicBool = AtomicBool::new(false);
    static SHOULD_TICK: AtomicBool = AtomicBool::new(false);
    static SHOULD_DIE: AtomicBool = AtomicBool::new(false);
    static SHOULD_RESTART: AtomicBool = AtomicBool::new(false);

    static TICKS_PER_SECOND: AtomicU64 = AtomicU64::new(10);

    static PRESS_KEY: AtomicU8 = AtomicU8::new(0);

    static DEBUG_ADD_FOOD_PRESS: AtomicBool = AtomicBool::new(false);
    static DEBUG_ADD_FOOD_RELEASE: AtomicBool = AtomicBool::new(false);

    let snake_context = Arc::new(Mutex::new(new_snake(
        ARENA_CELL_LENGTH as u64,
        ARENA_CELL_LENGTH as u64,
    )));

    context.program.enable();
    context.vao.bind();

    let snake_context_1 = Arc::clone(&snake_context);

    let tick_th = thread::spawn(move || {
        let mut start = Instant::now();
        while SHOULD_CLOSE.load(Ordering::Relaxed) == false {
            if let Ok(mut cxt) = snake_context_1.lock() {
                if SHOULD_RESTART.load(Ordering::Relaxed) == true {
                    *cxt = new_snake(ARENA_CELL_LENGTH as u64, ARENA_CELL_LENGTH as u64);
                    SHOULD_RESTART.store(false, Ordering::Relaxed);
                }

                if SHOULD_TICK.load(Ordering::Relaxed) {
                    if DEBUG_ADD_FOOD_PRESS.load(Ordering::Relaxed) {
                        cxt.add_part = true;
                    }
                    match cxt.tick() {
                        snake::GameState::Running => {
                            SHOULD_DIE.store(false, Ordering::Relaxed);
                        }
                        snake::GameState::Dead => {
                            // do death screen
                            SHOULD_DIE.store(true, Ordering::Relaxed);
                            SHOULD_TICK.store(false, Ordering::Relaxed);
                        }
                    };
                    if DEBUG_ADD_FOOD_RELEASE.load(Ordering::Relaxed) {
                        cxt.add_part = false;
                        DEBUG_ADD_FOOD_PRESS.store(false, Ordering::Relaxed)
                    }
                }

                let press = PRESS_KEY.load(Ordering::Relaxed);
                let mut release = 0;
                // w,a,s,d
                // 1,2,4,8
                if press != 0 {
                    for (mask, key) in [1u8, 2, 4, 8].into_iter().zip([1u8, 2, 3, 4].into_iter()) {
                        let dir = {
                            let zero_or_one: u8 =
                                (press & mask) >> (key.checked_sub(1).unwrap_or(0));
                            // 255 is not a valid dir, so gives Err(_)
                            Direction::try_from((key * zero_or_one).wrapping_sub(1))
                        };

                        if let Ok(val) = dir {
                            if cxt.move_dir
                                != match val {
                                    Direction::Left => Direction::Right,
                                    Direction::Right => Direction::Left,
                                    Direction::Up => Direction::Down,
                                    Direction::Down => Direction::Up,
                                }
                            {
                                cxt.move_dir = val;
                                release |= mask;
                                // we did our thing, so now push to the atomic
                                PRESS_KEY.fetch_xor(release, Ordering::Relaxed);
                                break;
                            } else {
                                // so we tried to go in the opposite direction,
                                // so remove that and continue to the next press
                                release |= mask;
                                continue;
                            }
                        }
                    }
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
            if !(SHOULD_DIE.load(Ordering::Relaxed) | SHOULD_RESTART.load(Ordering::Relaxed)) {
                bindings::ClearColor(0.8, 0.7, 0.7, 1.0);
                bindings::Clear(bindings::COLOR_BUFFER_BIT);

                context.vao.bind();

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
                bindings::ClearColor(1.0, 0.0, 0.0, 1.0);
                bindings::Clear(bindings::COLOR_BUFFER_BIT);
                bindings::BindVertexArray(0)
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
                "FPS: {:.2}, TICKS/SECOND: {}, MOVE_DIRECTION: {}, SCORE: {} --------------\r",
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
