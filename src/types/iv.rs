use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

pub type IV = u8; // 0 ~ 31
pub type IVRange = RangeInclusive<IV>;
pub type IVGroup = [IV; 3];
pub type IVRangeGroup = [IVRange; 3];

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IVs {
    pub hp: IV,
    pub attack: IV,
    pub defense: IV,
    pub speed: IV,
    pub sp_attack: IV,
    pub sp_defense: IV,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IVRanges {
    pub hp: IVRange,
    pub attack: IVRange,
    pub defense: IVRange,
    pub speed: IVRange,
    pub sp_attack: IVRange,
    pub sp_defense: IVRange,
}
