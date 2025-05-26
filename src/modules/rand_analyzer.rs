use crate::types::{iv::*, types::*};

pub struct RandAnalyzer {}

impl RandAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn extract_rand(&self, seed: Seed) -> Rand {
        return (seed >> 16) as Rand;
    }

    pub fn rands_to_seed(&self, rand_high: Rand, rand_low: Rand) -> Seed {
        return (rand_high as Seed) << 16 | (rand_low as Seed);
    }

    pub fn iv_group_to_rand(&self, iv_group: IVGroup) -> Rand {
        let rand = (iv_group[0] as u16) | (iv_group[1] as u16) << 5 | (iv_group[2] as u16) << 10;
        return rand & 0xffff;
    }

    pub fn rand_to_iv_group(&self, rand: Rand) -> IVGroup {
        return [
            (rand & 0x1f) as IV,
            ((rand >> 5) & 0x1f) as IV,
            ((rand >> 10) & 0x1f) as IV,
        ];
    }
}
