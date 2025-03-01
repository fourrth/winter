use std::{io::stdout, time::Duration};

use crossterm::{
    event,
    style::{Color, SetBackgroundColor},
};

struct CGOLASCII(game_of_life::Context);

impl std::fmt::Display for CGOLASCII {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.0.get_data().len() {
            let cell = match self.0.get_data()[i] {
                game_of_life::Cell::Alive => {
                    let _ = crossterm::execute!(stdout(), SetBackgroundColor(Color::Red));
                    "  "
                }
                game_of_life::Cell::Dead => {
                    let _ = crossterm::execute!(stdout(), SetBackgroundColor(Color::Blue));
                    "  "
                }
            };
            write!(f, "{}", cell)?;
            if (i + 1) % self.0.board_size.0 == 0 {
                let _ = crossterm::execute!(stdout(), SetBackgroundColor(Color::Reset));
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}
fn main() -> std::io::Result<()> {
    let mut cxt = CGOLASCII(
        game_of_life::Builder::create()
            .add(game_of_life::Attribute::BoardSize(16, 16))
            .build()
            .unwrap(),
    );
    let _ = crossterm::execute!(
        stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::Hide
    );
    print!("{}", cxt);
    const DO_DEBUG: bool = true;
    'main: loop {
        if event::poll(Duration::from_secs_f32(0.5))? {
            match event::read()? {
                event::Event::Key(ev) => {
                    match ev.kind {
                        event::KeyEventKind::Press => {
                            match ev.code {
                                event::KeyCode::Esc => {
                                    // quit
                                    break 'main;
                                }
                                _ => {
                                    // really any key event triggers
                                    // a tick and writes out

                                    cxt.0.tick();
                                    let _ = crossterm::execute!(
                                        stdout(),
                                        crossterm::terminal::Clear(
                                            crossterm::terminal::ClearType::All
                                        )
                                    );
                                    println!("{}", cxt);
                                    if DO_DEBUG {
                                        let coords = cxt
                                            .0
                                            .get_data()
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, &cell)| cell.is_alive())
                                            .map(|(cx, _)| {
                                                (cx % cxt.0.board_size.0, cx / cxt.0.board_size.0)
                                            })
                                            .collect::<Vec<_>>();

                                        println!(
                                        "iteration:\t{}\t board size:\t{} x {} \ncoordinates:\t{:?}\tindex:\t{:?}",
                                        cxt.0.cnt,
                                        cxt.0.board_size.0,
                                        cxt.0.board_size.1,
                                        &coords,
                                        coords.iter().map(|(x,y)|x+ y*cxt.0.board_size.0).collect::<Vec<_>>()
                                    );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        } else {
            // nothing happened
        }
    }

    /*
    let mut stdout = stdout();
         for _ in 0..1000 {
        cxt.0.tick();
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        );
        print!("{}", cxt);
    } */
    Ok(())
}
