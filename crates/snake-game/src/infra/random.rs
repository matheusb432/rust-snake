use std::num::NonZeroUsize;

use rand::{RngExt, SeedableRng, rngs::StdRng};
use snake_core::random::RandomSource;

pub struct RandRandomSource {
    rng: StdRng,
}
impl RandRandomSource {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl RandomSource for RandRandomSource {
    fn index(&mut self, length: NonZeroUsize) -> usize {
        self.rng.random_range(0..length.get())
    }
}
