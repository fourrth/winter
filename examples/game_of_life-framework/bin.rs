use std::{
    ffi::CString,
    fs::File,
    io::{self, BufWriter, Read, Write},
    num::NonZeroU32,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

use game_of_life::Context;
use glad_gles2::gl;
use glmath::{vector::Vector2, Element};
use winter::{common, primitives};

const SAVE_FILE_OUTPUT_DIR: &str = "./target/save_data.txt";
const DEFAULT_TPS: u64 = 10;

#[inline(always)]
fn clampmin<T: Element>(input: T, min: T) -> T {
    if input < min {
        min
    } else {
        input
    }
}

// Some things I could add is like a way to back up generations
// (basically) just do the rules I have set up but in reverse,
// but the point of this is to show what framework can do now

// i'm also not handling proper shutdown for when a thread panics,
// and the render thread crashes/stops responding when trying to
// load from file when the file doesn't exist
// I dont really care tho... same reason above

// also I hate the way we are saving the data. Not only is it
// incredibly space innefficent, but it does no checks to make sure
// we have the same sort of context (like arena size checks and what not)

fn create_gol_cxt(arena_cell_length: usize, file_load: Option<&str>) -> game_of_life::Context {
    let setup_kind = match file_load {
        Some(file_path) => {
            let thing = File::open(file_path);

            match thing {
                Ok(mut file) => {
                    let mut buf = String::new();
                    // if we read incorrect data,
                    // then serde might complain
                    file.read_to_string(&mut buf).unwrap();
                    return serde_json::from_str::<Context>(&buf).unwrap();
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
            // if we get here, then we had an error
            // and are just defaulting to Random
            game_of_life::SetupKind::Random
        }
        None => {
            // means that we want to do random
            game_of_life::SetupKind::Random
        }
    };
    game_of_life::Builder::create()
        .add(game_of_life::Attribute::BoardSize(
            arena_cell_length,
            arena_cell_length,
        ))
        .add(game_of_life::Attribute::BoardSetup(setup_kind))
        .build()
        .unwrap()
}

static TICKS_PER_SECOND: AtomicU64 = AtomicU64::new(DEFAULT_TPS);

const TICK_INC_AMT: u64 = 1;

fn press_left() {
    // decrease the ticks per second by inc amount
    let tps = TICKS_PER_SECOND.load(Ordering::Relaxed);
    let store_val = if tps <= TICK_INC_AMT {
        TICK_INC_AMT
    } else {
        tps - TICK_INC_AMT
    };
    TICKS_PER_SECOND.store(store_val, Ordering::Relaxed);
}

fn press_right() {
    // increase the ticks per second by inc amount
    let tps = TICKS_PER_SECOND.load(Ordering::Relaxed);
    let store_val = tps + TICK_INC_AMT;
    TICKS_PER_SECOND.store(store_val, Ordering::Relaxed);
}

fn main() -> Result<(), String> {
    println!("Format for program args is: width height arena_cell_width arena_cell_height");
    println!("Spacebar starts and stops it,");
    println!("Enter will restart it: if you press backspace it will toggle it to load from file");
    println!("Speaking of file: you can save via backslash");
    println!("Left and right arrow change ticks per second\n");

    let (width, height, arena_size) = {
        let args: Vec<_> = std::env::args().skip(1).collect();

        let width = if let Some(width_) = args.get(0) {
            if let Ok(width_) = width_.parse::<i32>() {
                width_
            } else {
                println!("Inputted Width Unknown: using 800");
                800
            }
        } else {
            println!("Inputted Width Unknown: using 800");
            800
        };

        let height = if let Some(height_) = args.get(1) {
            if let Ok(height_) = height_.parse::<i32>() {
                height_
            } else {
                println!("Inputted height Unknown: using 800");
                800
            }
        } else {
            println!("Inputted height Unknown: using 800");
            800
        };

        let arena_size = if let Some(arena_size_) = args.get(2) {
            if let Ok(arena_size_) = arena_size_.parse::<u32>() {
                arena_size_
            } else {
                println!("Inputted arena_size Unknown: using 50");
                800
            }
        } else {
            println!("Inputted arena_size Unknown: using 50");
            50
        };

        (width, height, arena_size)
    };

    let title = CString::new("Conway's Game of Life").unwrap();

    let (vertex_shader_text, fragment_shader_text) = {
        (
            CString::new(include_str!("vertex_shader.glsl")).unwrap(),
            CString::new(include_str!("frag_shader.glsl")).unwrap(),
        )
    };

    let color_alive = primitives::Color::from_rgb(0, 255, 255);
    let color_dead = primitives::Color::from_rgb(0, 0, 255);

    let vao_builder = common::create_grid(
        Vector2::from([-1.; 2]).add(Vector2::from([0.1; 2])),
        Vector2::from([1.; 2]).sub(Vector2::from([0.1; 2])),
        NonZeroU32::new(arena_size as u32).unwrap(),
        NonZeroU32::new(arena_size as u32).unwrap(),
        0.,
        |_, _, _| color_dead,
    );

    // pretty sure I have said this before, but this should really all be
    // put into some manager type along with gol context and
    // also just be some Atomic int with & 0b for what setting
    static TICKING: AtomicBool = AtomicBool::new(false);
    static mut GOL_CXT: Option<Context> = None;

    // should we try to save to file
    static DO_SAVE: AtomicBool = AtomicBool::new(false);

    // restart the current context
    static DO_RESTART: AtomicBool = AtomicBool::new(false);

    // if we should load from file when we restart
    static SHOULD_LOAD_FROM_FILE: AtomicBool = AtomicBool::new(false);

    // some input statics
    static PRESS_LEFT: AtomicBool = AtomicBool::new(false);
    static PRESS_RIGHT: AtomicBool = AtomicBool::new(false);

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
                    if GOL_CXT.is_some() {
                        if TICKING.load(Ordering::Relaxed) == false {
                            TICKING.store(true, Ordering::Relaxed);
                        } else {
                            TICKING.store(false, Ordering::Relaxed);
                        }
                    }
                } else if key == glfw::ffi::KEY_ENTER {
                    DO_RESTART.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_BACKSLASH {
                    // save the current state
                    DO_SAVE.store(true, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_BACKSPACE {
                    // set that we should load from file
                    if SHOULD_LOAD_FROM_FILE.load(Ordering::Relaxed) {
                        SHOULD_LOAD_FROM_FILE.store(false, Ordering::Relaxed);
                    } else {
                        SHOULD_LOAD_FROM_FILE.store(true, Ordering::Relaxed);
                    }
                } else if key == glfw::ffi::KEY_LEFT {
                    PRESS_LEFT.store(true, Ordering::Relaxed);
                    press_left();
                } else if key == glfw::ffi::KEY_RIGHT {
                    PRESS_RIGHT.store(true, Ordering::Relaxed);
                    press_right();
                } else {
                    // println!("Key Press: {key}");
                }
            } else if action == glfw::ffi::RELEASE {
                if key == glfw::ffi::KEY_LEFT {
                    PRESS_LEFT.store(false, Ordering::Relaxed);
                } else if key == glfw::ffi::KEY_RIGHT {
                    PRESS_RIGHT.store(false, Ordering::Relaxed);
                }
            }
        }),
        vao_builder,
    )?;

    unsafe {
        GOL_CXT = Some(create_gol_cxt(arena_size as usize, None));

        context.program.enable();
        context.vao.bind();

        static SHOULD_CLOSE: AtomicBool = AtomicBool::new(false);

        const EVENT_TICKS_PER_SECOND: u64 = 20;

        let event_th = thread::spawn(move || {
            while SHOULD_CLOSE.load(Ordering::Relaxed) == false {
                //TODO: create an event thread
                if PRESS_LEFT.load(Ordering::Relaxed) {
                    press_left();
                }
                if PRESS_RIGHT.load(Ordering::Relaxed) {
                    press_right();
                }
                if DO_RESTART.load(Ordering::Relaxed) {
                    // then restart the game_of_life context
                    DO_RESTART.store(false, Ordering::Relaxed);
                    if SHOULD_LOAD_FROM_FILE.load(Ordering::Relaxed) {
                        GOL_CXT = Some(create_gol_cxt(
                            arena_size as usize,
                            Some(SAVE_FILE_OUTPUT_DIR),
                        ));
                    } else {
                        GOL_CXT = Some(create_gol_cxt(arena_size as usize, None));
                    }
                }
                if DO_SAVE.load(Ordering::Relaxed) {
                    if let Some(cxt) = &mut GOL_CXT {
                        DO_SAVE.store(false, Ordering::Relaxed);
                        // for now we just write out to a file with
                        // json because I don't really care...
                        let thing: Result<File, io::Error> = File::create(SAVE_FILE_OUTPUT_DIR);
                        match thing {
                            Ok(mut file) => {
                                let _ = writeln!(file, "{}", serde_json::to_string(&cxt).unwrap());
                                let _ = file.sync_all();
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_millis(1000 / EVENT_TICKS_PER_SECOND));
            }
        });

        let tick_th = thread::spawn(move || {
            let mut last_t: Option<Instant> = None;

            while SHOULD_CLOSE.load(Ordering::Relaxed) == false {
                if let Some(last) = last_t {
                    // how long a tick took
                    thread::sleep(Duration::from_secs_f64(clampmin(
                        (1f64 / (TICKS_PER_SECOND.load(Ordering::Relaxed) as f64))
                            - last.elapsed().as_secs_f64(),
                        0f64,
                    )));
                }
                if let Some(cxt) = &mut GOL_CXT {
                    if TICKING.load(Ordering::Relaxed) {
                        // want to tick: go to target tps
                        cxt.tick();
                    }
                }
                last_t = Some(Instant::now());
            }
        });

        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());

        let mut frame_count = 0usize;
        let mut time_start = Instant::now();

        let mut generation_count = 0;
        while context.window.should_close() == false {
            gl::ClearColor(0.8, 0.7, 0.7, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            context.vao.indices.bind();
            if let Some(cxt) = &mut GOL_CXT {
                generation_count = cxt.cnt;
                for (cx, cell) in cxt.get_data().into_iter().enumerate() {
                    // for each cell, paint it each frame and move it upwards
                    let c = {
                        if cell.is_alive() {
                            color_alive
                        } else {
                            // if dead, paint it dead
                            color_dead
                        }
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

            glfw::ffi::glfwGetFramebufferSize(
                context.window.handle,
                &mut context.window.width,
                &mut context.window.height,
            );

            glfw::ffi::glfwSwapBuffers(context.window.handle);
            glfw::ffi::glfwPollEvents();

            let frametime = time_start.elapsed().as_secs_f32();
            let fps = 1f32 / frametime;

            let _ = write!(
                writer,
                "FPS: {:.2}, GENERATION: {}, TICKS/SECOND: {}, LOAD FROM FILE: {}--------------\r",
                fps,
                generation_count,
                TICKS_PER_SECOND.load(Ordering::Relaxed),
                match SHOULD_LOAD_FROM_FILE.load(Ordering::Relaxed) {
                    true => "yes",
                    false => "no",
                }
            );
            if frame_count % 10 == 0 {
                let _ = writer.flush();
            }
            frame_count += 1;
            // at the end, reset the start time
            time_start = Instant::now();
        }
        SHOULD_CLOSE.store(true, Ordering::Relaxed);
        // let _ = write!(writer, "\x1b[2J");
        let _ = write!(writer, "\n");
        tick_th.join().unwrap();
        event_th.join().unwrap();
    }

    Ok(())
}
