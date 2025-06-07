use serde::{Deserialize, Serialize};

use super::{iv::*, seed::*};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Status {
    pub ivs: IVs,
    pub gender: i16,
    pub nature: i16,
    pub ability: i16,
    pub hidden_power_type: i16,
    pub hidden_power_power: u8,
    pub pid: PID,
}
