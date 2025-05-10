use crate::types::pokemon_ivs::IVs;
use crate::types::types::*;

pub struct PokemonData {
    pub ability: u8,
    pub gender: u8,
    pub hidden_power_type: u8,
    pub hidden_power_power: u8,
    pub ivs: IVs,
    pub nature: u8,
    pub pid: Pid,
}

impl PokemonData {
    pub fn new(
        ability: u8,
        gender: u8,
        hidden_power_type: u8,
        hidden_power_power: u8,
        ivs: IVs,
        nature: u8,
        pid: Pid,
    ) -> Self {
        Self {
            ability,
            gender,
            hidden_power_type,
            hidden_power_power,
            ivs,
            nature,
            pid,
        }
    }
}
