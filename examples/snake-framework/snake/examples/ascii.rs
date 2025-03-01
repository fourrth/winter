use crossterm::event::KeyCode;
use snake::{Coordinate, Direction};
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

fn main() {
    let mut cxt = snake::Builder::create()
        .add(snake::BuildOptions::ArenaDim(100, 36))
        .add(snake::BuildOptions::StartingHeadCoord(Coordinate((6, 6))))
        .add(snake::BuildOptions::StartingMoveDir(Direction::Right))
        .build();

    let mut writer = BufWriter::new(io::stdout());
    let _ = crossterm::terminal::enable_raw_mode();

    let target_frametime = 1f32 / 10f32;

    let mut timer;
    loop {
        timer = Instant::now();
        // std::thread::sleep(Duration::from_millis(100));

        // check input
        if let Ok(b) = crossterm::event::poll(Duration::from_millis(0)) {
            if b {
                if let Ok(e) = crossterm::event::read() {
                    match e {
                        crossterm::event::Event::Key(val) => match val.code {
                            KeyCode::Char('w') => {
                                cxt.move_dir = Direction::Up;
                            }
                            KeyCode::Char('a') => {
                                cxt.move_dir = Direction::Left;
                            }
                            KeyCode::Char('s') => {
                                cxt.move_dir = Direction::Down;
                            }
                            KeyCode::Char('d') => {
                                cxt.move_dir = Direction::Right;
                            }
                            KeyCode::Char(' ') => {
                                cxt.add_part = true;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }

        // tick

        cxt.tick();

        // now displaying...
        for (cx, &ca) in cxt.get_arena_iter().enumerate() {
            if cx % cxt.arena_dim.0 as usize == 0 {
                let _ = writeln!(writer, "");
            }
            match ca {
                snake::Cell::Empty => {
                    // write nothing/space
                    let _ = write!(writer, "_");
                }
                snake::Cell::SnakeBody => {
                    let _ = write!(writer, "#");
                }
                snake::Cell::SnakeHead => {
                    let _ = write!(writer, "0");
                }
                _ => todo!(),
            }
        }
        let _ = crossterm::execute!(
            io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        );

        let _ = writer.flush();

        let frametime = timer.elapsed().as_secs_f32();

        let wait_time = target_frametime - frametime;
        if wait_time > 0f32 {
            std::thread::sleep(Duration::from_secs_f32(wait_time));
        }
        timer = Instant::now();
    }
}
