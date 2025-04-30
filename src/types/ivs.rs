use std::ops::RangeInclusive;

pub struct IVs {
    pub hp: RangeInclusive<u8>,
    pub attack: RangeInclusive<u8>,
    pub defense: RangeInclusive<u8>,
    pub speed: RangeInclusive<u8>,
    pub sp_attack: RangeInclusive<u8>,
    pub sp_defense: RangeInclusive<u8>,
}

impl IVs {
    pub fn new(
        hp: RangeInclusive<u8>,
        attack: RangeInclusive<u8>,
        defense: RangeInclusive<u8>,
        speed: RangeInclusive<u8>,
        sp_attack: RangeInclusive<u8>,
        sp_defense: RangeInclusive<u8>,
    ) -> Self {
        Self {
            hp,
            attack,
            defense,
            sp_attack,
            sp_defense,
            speed,
        }
    }
}
