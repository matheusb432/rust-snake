use std::{
    io::{self, Write},
    process::{self, ExitCode, Termination},
    thread,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

pub const QUIT: char = 'q';

fn main() -> io::Result<ExitCode> {
    let game = Game::start()?;
    // TODO: impl the newtypes and instantiate the snek
    // TODO: use Arc<Mutex<T>> ?
    // let mut snake = Snake {hp:};
    // let mut snake_hp = 5_u32;

    // TODO: use newtype for directional inputs
    let (input_tx, input_rx) = std::sync::mpsc::channel();

    // TODO: cleanup once game drop works
    println!("enter move ['{QUIT}' to quit]: ");
    thread::spawn(move || {
        loop {
            // TODO: this is blocking. try to have a loop with tx/rx before using async EventStream
            if let Ok(Event::Key(key)) = event::read().inspect_err(|err| {
                eprintln!("error on event read: {}", err);
            }) {
                println!("sending...");
                // best effort to send key
                let _ = input_tx.send(key);
            }
        }
    });
    // TODO cleanup once game drop works
    // disable_raw_mode()?;

    // TODO: without async i think 2 loops are necessary since input_rx can't listen on a closed channel, review for a simpler solution later
    let exit_code = loop {
        println!("receiving...");
        match input_rx.recv() {
            Ok(key) => match key.code {
                arrow_key_code @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right) => {
                    println!("pressed direction: {}", arrow_key_code);
                }
                KeyCode::Char(QUIT) => {
                    break game.exit();
                }
                _ => {}
            },
            Err(err) => {
                eprintln!("{}", err);
                break ExitCode::FAILURE;
            }
        }
    };
    // snake_hp = snake_hp.saturating_sub(3);
    // println!("snake_hp: {}", snake_hp);
    // if snake_hp == 0 {
    //     break Ok(());
    // }

    Ok(exit_code)
}

// TODO: write and make it receive tx
// fn handle_inputs(...)

// TODO: move any 'mod's here to own files or snake-core once they get complex enough
mod snake {
    use crate::core::Transform;

    /// snake player
    pub struct Snake {
        hp: SnakeHp,
        transform: Transform,
    }

    impl Snake {
        pub fn kill(&mut self) {
            self.hp = SnakeHp::ZERO;
        }

        // TODO: create based on `>~~~~~{`
        pub fn render_body(&self) -> String {
            todo!()
        }
    }

    pub struct SnakeHp(u32);

    impl SnakeHp {
        pub const ZERO: Self = Self(0);
    }
}

mod core {

    pub struct Transform {
        position: Vector2,
        rotation: Rotation,
    }
    // TODO: implement vec floating point calcs
    /// vector 2 for game coords
    pub struct Vector2 {
        x: f32,
        y: f32,
    }

    /// actors can only change in 90deg increments. might be better to use a simple 0-3 enum but i want to practice something closer to gamedev engines
    pub enum Rotation {
        Deg90,
        Deg180,
        Deg270,
        Deg0,
    }
}

struct Game;
impl Game {
    pub fn start() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }

    /// drops `Game`, which in turn calls its `Drop` impl to exit the process
    pub fn exit(self) -> ExitCode {
        self.report()
    }
}
impl Drop for Game {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        println!("exiting game...");
    }
}
impl Termination for Game {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}

// fn exit_game() {
//     process::exit(ExitCode::SUCCESS);
// }
