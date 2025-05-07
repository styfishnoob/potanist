use std::ops::RangeInclusive;

pub type IV = RangeInclusive<u8>;
pub type IVGroup = [IV; 3]; // [Hp, Atk, Def] [Spe, SpA, SpD]

pub struct IVs {
    pub hp: IV,
    pub attack: IV,
    pub defense: IV,
    pub speed: IV,
    pub sp_attack: IV,
    pub sp_defense: IV,
}

impl IVs {
    pub fn new(hp: IV, attack: IV, defense: IV, speed: IV, sp_attack: IV, sp_defense: IV) -> Self {
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
