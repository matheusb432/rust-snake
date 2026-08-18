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

    println!("\renter move ['{QUIT}' to quit]: ");
    thread::spawn(move || {
        loop {
            // TODO: this is blocking. try to have a loop with tx/rx before using async EventStream
            if let Ok(Event::Key(key)) = event::read().inspect_err(|err| {
                // TODO: emit error event. do not print to the tui outside main thread
                // eprintln!("error on event read: {}", err);
            }) {
                // best effort to send key
                let _ = input_tx.send(key);
            }
        }
    });

    // TODO: without async i think 2 loops are necessary since input_rx can't listen on a closed channel, review for a simpler solution later
    let exit_code = loop {
        match input_rx.recv() {
            Ok(key) => match key.code {
                arrow_key_code @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right) => {
                    println!("\rpressed direction: {}", arrow_key_code);
                }
                KeyCode::Char(QUIT) => {
                    break game.exit();
                }
                _ => {}
            },
            Err(err) => {
                eprintln!("\r{}", err);
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
    use std::default;

    use crate::core::{Rotation, Transform};

    /// snake player
    pub struct Snake {
        hp: SnakeHp,
        body: SnakeBody,
        transform: Transform,
    }

    impl Snake {
        // pub const DEFAULT_SIZE: ActorSize = ActorSize{
        //     width
        // };

        pub fn kill(&mut self) {
            self.hp = SnakeHp::ZERO;
        }

        // TODO: create based on `>~~{`
        pub fn render_body(&self) -> String {
            todo!()
        }
    }

    pub enum SnakeError {
        SizeTooBig { max: u16, actual: u16 },
    }

    pub struct SnakeHp(u32);

    impl SnakeHp {
        pub const ZERO: Self = Self(0);
    }

    // TODO: make logic to order each snake part
    /// body that governs which part is head/appendage/tail of the snake.
    pub struct SnakeBody(Vec<SnakePart>);
    impl SnakeBody {
        pub const MAX_SIZE: u16 = 500;
        pub fn new(size: u16) -> Result<Self, SnakeError> {
            if size > Self::MAX_SIZE {
                return Err(SnakeError::SizeTooBig {
                    max: Self::MAX_SIZE,
                    actual: size,
                });
            }
            let parts = vec![SnakePart::default(); size.into()];
            Ok(Self(parts))
        }
    }

    /// the snake's body part
    #[derive(Default, Debug, PartialEq, Eq, Clone)]
    pub struct SnakePart {
        /// direction the part is facing
        rotation: SnakeRotation,
    }

    /// snake can only move in 90 deg increments
    #[derive(Default, Debug, PartialEq, Eq, Clone)]
    pub enum SnakeRotation {
        #[default]
        Deg0 = 0,
        // TODO: how to use u16
        // Deg0 = 0_u16,
        Deg90 = 90,
        Deg180 = 180,
        Deg270 = 270,
    }
    impl From<Rotation> for SnakeRotation {
        // TODO impl
        fn from(value: Rotation) -> Self {}
    }
}

mod core {
    // TODO: set less permissive access modifiers after abstractions

    use std::ops::Add;

    #[derive(Default, Debug, PartialEq, Clone)]
    pub struct Transform {
        pub position: Vector2,
        pub rotation: Rotation,
    }
    // TODO: implement vec floating point calcs
    /// vector 2 for game coords
    #[derive(Default, Debug, PartialEq, Clone)]
    pub struct Vector2 {
        pub x: f32,
        pub y: f32,
    }

    #[repr(transparent)]
    #[derive(Default, Debug, PartialEq, Eq, Clone)]
    pub struct Rotation(u16);
    impl Rotation {
        pub const RIGHT: Rotation = Self(0);
        pub const DOWN: Rotation = Self(90);
        pub const LEFT: Rotation = Self(180);
        pub const UP: Rotation = Self(270);
        pub const DEGREES_UPPER: u16 = 360;
        // TODO: implement (might be better to be infallible. 750° is fine to be interpreted as 30°)
        pub fn new(value: u16) -> Self {
            Self(value.rem_euclid(Self::DEGREES_UPPER))
        }

        pub fn rotate(self, rotation: Rotation) -> Self {
            Self::new(self.into_inner() + rotation.into_inner())
        }

        pub fn into_inner(self) -> u16 {
            self.0
        }
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
        println!("\rexiting game...");
    }
}
impl Termination for Game {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}
