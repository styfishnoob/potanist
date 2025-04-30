pub struct LCRng {
    multiplier: u32,
    increment: u32,
    inverse_multiplier: u32,
}

impl LCRng {
    pub fn new() -> Self {
        Self {
            multiplier: 0x41c64e6d,
            increment: 0x6073,
            inverse_multiplier: 0xeeb9eb65,
        }
    }

    pub fn next(&self, seed: u32) -> u32 {
        let next_seed = (seed as u64) * (self.multiplier as u64) + (self.increment as u64);
        return (next_seed & 0xffffffff) as u32;
    }

    pub fn prev(&self, seed: u32) -> u32 {
        let prev_seed = (self.inverse_multiplier as u64) * (seed as u64 - self.increment as u64);
        return (prev_seed & 0xffffffff) as u32;
    }

    pub fn extract_rand(&self, seed: u32) -> u16 {
        return (seed >> 16) as u16;
    }
}
