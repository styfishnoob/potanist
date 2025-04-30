use std::ops::RangeInclusive;

use crate::lcrng::LCRng;
use crate::types::ivs::IVs;

pub struct Potanist {
    lcrng: LCRng,
}

impl Potanist {
    pub fn new() -> Self {
        Self {
            lcrng: LCRng::new(),
        }
    }
}
