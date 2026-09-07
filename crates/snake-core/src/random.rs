use std::num::NonZeroUsize;

pub trait RandomSource {
    fn index(&mut self, length: NonZeroUsize) -> usize;
}
