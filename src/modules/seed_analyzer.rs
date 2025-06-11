use std::collections::HashMap;

use crate::{
    constants::time_sum_map,
    types::{iv::*, seed::*, status::*},
};

use super::{rand_analyzer::RandAnalyzer, rng_lc::RngLC};

pub struct SeedAnalyzer {
    rng_analyzer: RandAnalyzer,
    rng_lc: RngLC,
}

impl SeedAnalyzer {
    pub fn new() -> Self {
        Self {
            rng_analyzer: RandAnalyzer::new(),
            rng_lc: RngLC::new(),
        }
    }

    pub fn extract_status(&self, iv_1st_seed: IV1stSeed, tid: Rand, sid: Rand) -> Status {
        let pid_2nd_seed = self.rng_lc.prev(iv_1st_seed);
        let pid_1st_seed = self.rng_lc.prev(pid_2nd_seed);
        let iv_2nd_seed = self.rng_lc.next(iv_1st_seed);

        let pid_1st_rand = self.rng_analyzer.extract_rand(pid_1st_seed);
        let pid_2nd_rand = self.rng_analyzer.extract_rand(pid_2nd_seed);
        let iv_1st_rand = self.rng_analyzer.extract_rand(iv_1st_seed);
        let iv_2nd_rand = self.rng_analyzer.extract_rand(iv_2nd_seed);
        let pid = (pid_2nd_rand as PID) << 16 | (pid_1st_rand as PID);

        let ivs_1st = self.rng_analyzer.rand_to_iv_group(iv_1st_rand);
        let ivs_2nd = self.rng_analyzer.rand_to_iv_group(iv_2nd_rand);
        let nature_num = (pid % 100 % 25) as u8;
        let gender_num = (pid & 0xff) as u8;
        let ability_num = (pid & 1) as u8;
        let is_shiny = is_shiny(pid, tid, sid);

        let ivs = IVs {
            hp: ivs_1st[0],
            attack: ivs_1st[1],
            defense: ivs_1st[2],
            speed: ivs_2nd[0],
            sp_attack: ivs_2nd[1],
            sp_defense: ivs_2nd[2],
        };

        let hidden_power_type = extract_hidden_power_type(&ivs);
        let hidden_power_power = extract_hidden_power_power(&ivs);

        return Status {
            ivs: ivs,
            gender: gender_num,
            nature: nature_num,
            ability: ability_num,
            shiny: is_shiny,
            hidden_power_type: hidden_power_type,
            hidden_power_power: hidden_power_power,
            pid: pid,
        };
    }

    /*
      HashMap<year, ((month, day), (hour, minutes, boot_time_sec, second))>
      HashKey(2048): ((1, 1), (20, 03, 10, 45)) -> 2048年 1月1日 20時03分10秒に選択 45秒につづきから選択
    */
    pub fn create_boot_time_map(
        &self,
        initial_seed: InitialSeed,
        blank_frame: u16,
    ) -> Option<HashMap<u16, Vec<((u8, u8), (u8, u8, u8, u8))>>> {
        let mut boot_time_map: HashMap<u16, Vec<((u8, u8), (u8, u8, u8, u8))>> = HashMap::new();
        let time_sum_map = time_sum_map::load_bincode("./data/boot_time_sum_map.bin");

        let hour = u8::try_from((initial_seed >> 16) & 0xff)
            .ok()
            .filter(|&h| h < 24)?;
        let time_sum = ((initial_seed >> 24) & 0xff) as u16;
        let frame_sum = (initial_seed & 0xffff) as u16;

        for year in 0..=99 {
            let frame = frame_sum - year;

            if let Some(boot_time_vec) = time_sum_map.get(&time_sum) {
                let mut min_boot_time_sec_map: HashMap<(u8, u8), (u8, u8, u8)> = HashMap::new();

                for (month, day, minutes, second) in boot_time_vec {
                    let waiting_frame: u16 = frame + blank_frame; // ポケモンを選択してから待機する時間
                    let waiting_time: u16 = waiting_frame / 60; // 14秒が最速　14秒未満の組みは切り捨て

                    if waiting_time < 14 || (*second as u16) < waiting_time {
                        continue;
                    }

                    // ソフト選択時間
                    let boot_time_sec = ((*second as u16) - waiting_time) as u8;

                    // 10秒以下だと時間変更からDS再起動までに7~8秒かかるため間に合わない
                    if boot_time_sec < 10 {
                        continue;
                    }

                    min_boot_time_sec_map
                        .entry((*month, *day))
                        .and_modify(|entry| {
                            if boot_time_sec < entry.1 {
                                *entry = (*minutes, boot_time_sec, *second);
                            }
                        })
                        .or_insert_with(|| (*minutes, boot_time_sec, *second));
                }

                let mut sorted: Vec<((u8, u8), (u8, u8, u8))> =
                    min_boot_time_sec_map.into_iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));

                for value in &sorted {
                    let tuple_1 = value.0;
                    let tuple_2 = (hour, value.1.0, value.1.1, value.1.2);
                    boot_time_map
                        .entry(2000 + year)
                        .or_insert_with(Vec::new)
                        .push((tuple_1, tuple_2));
                }
            } else {
                println!("{} に一致する起動時間は見つかりませんでした。", time_sum)
            }
        }

        return Some(boot_time_map);
    }
}

fn is_shiny(pid: PID, tid: Rand, sid: Rand) -> bool {
    let tsid_xor = (tid ^ sid) as u32;
    let pid_xor = ((pid >> 16) ^ (pid & 0xffff)) as u32;
    (tsid_xor ^ pid_xor) <= 7
}

fn extract_hidden_power_type(ivs: &IVs) -> u8 {
    let hp = if ivs.hp % 2 == 0 { 0 } else { 1 };
    let attack = if ivs.attack % 2 == 0 { 0 } else { 2 };
    let defense = if ivs.defense % 2 == 0 { 0 } else { 4 };
    let speed = if ivs.speed % 2 == 0 { 0 } else { 8 };
    let sp_attack = if ivs.sp_attack % 2 == 0 { 0 } else { 16 };
    let sp_defense = if ivs.sp_defense % 2 == 0 { 0 } else { 32 };
    let sum = hp + attack + defense + speed + sp_attack + sp_defense;
    return (sum * 15 % 63) as u8;
}

fn extract_hidden_power_power(ivs: &IVs) -> u8 {
    let hp = if ivs.hp % 2 == 0 { 0 } else { 1 };
    let attack = if ivs.attack % 2 == 0 { 0 } else { 2 };
    let defense = if ivs.defense % 2 == 0 { 0 } else { 4 };
    let speed = if ivs.speed % 2 == 0 { 0 } else { 8 };
    let sp_attack = if ivs.sp_attack % 2 == 0 { 0 } else { 16 };
    let sp_defense = if ivs.sp_defense % 2 == 0 { 0 } else { 32 };
    let sum: u16 = hp + attack + defense + speed + sp_attack + sp_defense;
    return (sum * 40 / 63 + 30) as u8;
}
