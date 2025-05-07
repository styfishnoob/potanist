use std::ops::RangeInclusive;

pub type Rand = u16; // 0x0 ~ 0xffff
pub type Seed = u32; // 0x0 ~ 0xffffffff
pub type Pid = u32; // 0x0 ~ 0xffffffff
pub type IV = u8; // 0 ~ 31
pub type IVRange = RangeInclusive<IV>;

pub type IVGroup = [IV; 3];
pub type IVRangeGroup = [IVRange; 3];

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

pub struct IVRanges {
    pub hp: IVRange,
    pub attack: IVRange,
    pub defense: IVRange,
    pub speed: IVRange,
    pub sp_attack: IVRange,
    pub sp_defense: IVRange,
}

impl IVRanges {
    pub fn new(
        hp: IVRange,
        attack: IVRange,
        defense: IVRange,
        speed: IVRange,
        sp_attack: IVRange,
        sp_defense: IVRange,
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
