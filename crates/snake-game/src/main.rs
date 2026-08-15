use std::io;

fn main() -> io::Result<()> {
    let mut snake_hp = 5_u32;
    loop {
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
