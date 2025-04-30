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
}
