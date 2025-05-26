use crate::types::types::*;

pub struct RngLC {
    multiplier: u32,
    increment: u32,
    inverse_multiplier: u32,
}

impl RngLC {
    pub fn new() -> Self {
        Self {
            multiplier: 0x41c64e6d,
            increment: 0x6073,
            inverse_multiplier: 0xeeb9eb65,
        }
    }

    pub fn next(&self, seed: Seed) -> Seed {
        let next_seed = (seed as u64) * (self.multiplier as u64) + (self.increment as u64);
        return (next_seed & 0xffffffff) as Seed;
    }

    pub fn prev(&self, seed: Seed) -> Seed {
        let diff = seed.wrapping_sub(self.increment);
        let prev_seed = (self.inverse_multiplier as u64 * diff as u64) & 0xffffffff;
        return (prev_seed & 0xffffffff) as Seed;
    }
}
