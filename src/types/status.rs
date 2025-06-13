use serde::{Deserialize, Serialize};

use super::{iv::*, seed::*};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Status {
    pub ivs: IVs,
    pub gender: u8,
    pub nature: u8,
    pub ability: u8,
    pub shiny: bool,
    pub pid: PID,
}
