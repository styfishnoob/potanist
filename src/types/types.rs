use std::ops::RangeInclusive;

pub type Rand = u16; // 0x0 ~ 0xffff
pub type Seed = u32; // 0x0 ~ 0xffffffff
pub type Pid = u32; // 0x0 ~ 0xffffffff
pub type IV = u8; // 0 ~ 31
pub type IVRange = RangeInclusive<IV>;

pub type IVGroup = [IV; 3];
pub type IVRangeGroup = [IVRange; 3];
