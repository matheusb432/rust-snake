use snake_core::{Texture, TextureColor};

pub(super) const SNAKE_HEAD: Texture = Texture::new('{', TextureColor::DarkGreen);
pub(super) const SNAKE_PART: Texture = Texture::new('~', TextureColor::DarkGreen);
pub(super) const WALL_X: Texture = Texture::new('|', TextureColor::White);
pub(super) const WALL_Y: Texture = Texture::new('_', TextureColor::White);
pub(super) const WALL_DIAGONAL: Texture = Texture::new('x', TextureColor::White);
pub(super) const BLANK: Texture = Texture::new(' ', TextureColor::White);
pub(super) const APPLE: Texture = Texture::new('&', TextureColor::Red);
