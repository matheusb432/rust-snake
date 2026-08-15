use std::io;

fn main() -> io::Result<()> {
    // TODO: impl the newtypes and instantiate the snek
    // let mut snake = Snake {hp:};
    let mut snake_hp = 5_u32;
    loop {
        // TODO: add crossterm to handle input events
        let _move_to = {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer)?;
            println!("{}", buffer.clone());
            buffer
        };

        snake_hp = snake_hp.saturating_sub(3);
        println!("snake_hp: {}", snake_hp);
        if snake_hp == 0 {
            break Ok(());
        }
    }
}

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
