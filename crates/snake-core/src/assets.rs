use crate::{Texture, TextureColor};

pub const SNAKE_HEAD: Texture = Texture::new('>', TextureColor::DarkGreen);
pub const SNAKE_PART: Texture = Texture::new('~', TextureColor::DarkGreen);
pub const WALL_X: Texture = Texture::new('|', TextureColor::White);
pub const WALL_Y: Texture = Texture::new('_', TextureColor::White);
pub const WALL_DIAGONAL: Texture = Texture::new('x', TextureColor::White);
pub const BLANK: Texture = Texture::new(' ', TextureColor::White);
pub const APPLE: Texture = Texture::new('&', TextureColor::Red);
