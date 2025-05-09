use crate::constants::time_sum_map;
use crate::modules::lcrng::LCRng;
use crate::types::pokemon_data::PokemonData;
use crate::types::pokemon_ivs::IVs;
use crate::types::types::*;
use std::collections::HashMap;

pub struct SeedAnalyzer {
    lcrng: LCRng,
}

impl SeedAnalyzer {
    pub fn new() -> Self {
        Self { lcrng: LCRng::new() }
    }

    pub fn iv_group_to_rand(&self, iv_group: IVGroup) -> Rand {
        let rand = (iv_group[0] as u16) | (iv_group[1] as u16) << 5 | (iv_group[2] as u16) << 10;
        return rand & 0xffff;
    }

    pub fn rand_to_ivs(&self, rand: Rand) -> IVGroup {
        return [
            (rand & 0x1f) as IV,
            ((rand >> 5) & 0x1f) as IV,
            ((rand >> 10) & 0x1f) as IV,
        ];
    }

    pub fn rands_to_seed(&self, rand_high: Rand, rand_low: Rand) -> Seed {
        return (rand_high as Seed) << 16 | (rand_low as Seed);
    }

    pub fn extract_pokemon_data_from_iv_1st_seed(&self, iv_1st_seed: Seed) -> PokemonData {
        let pid_2nd_seed = self.lcrng.prev(iv_1st_seed);
        let pid_1st_seed = self.lcrng.prev(pid_2nd_seed);
        let iv_2nd_seed = self.lcrng.next(iv_1st_seed);

        let pid_1st_rand = self.lcrng.extract_rand(pid_1st_seed);
        let pid_2nd_rand = self.lcrng.extract_rand(pid_2nd_seed);
        let iv_1st_rand = self.lcrng.extract_rand(iv_1st_seed);
        let iv_2nd_rand = self.lcrng.extract_rand(iv_2nd_seed);
        let pid = (pid_2nd_rand as Pid) << 16 | (pid_1st_rand as Pid);

        let ivs_1st = self.rand_to_ivs(iv_1st_rand);
        let ivs_2nd = self.rand_to_ivs(iv_2nd_rand);
        let nature_num = (pid % 100 % 25) as u8;
        let gender_num = (pid & 0xff) as u8;
        let ability_num = (pid & 1) as u8;

        let ivs = IVs {
            hp: ivs_1st[0],
            attack: ivs_1st[1],
            defense: ivs_1st[2],
            speed: ivs_2nd[0],
            sp_attack: ivs_2nd[1],
            sp_defense: ivs_2nd[2],
        };

        return PokemonData {
            ability: ability_num,
            gender: gender_num,
            hidden_power_type: 0,
            hidden_power_power: 0,
            ivs: ivs,
            nature: nature_num,
            pid: pid,
        };
    }

    /*
      HashMap<year, ((month, day), (hour, minutes, boot_time_sec, second))>
      HashKey(2048): ((1, 1), (20, 03, 10, 45)) -> 2048年 1月1日 20時03分10秒に選択 45秒につづきから選択
    */
    pub fn create_boot_time_map_from_initial_seed(
        &self,
        initial_seed: Seed,
        blank_frame: u16,
    ) -> Option<HashMap<u16, Vec<((u8, u8), (u8, u8, u8, u8))>>> {
        let mut boot_time_map: HashMap<u16, Vec<((u8, u8), (u8, u8, u8, u8))>> = HashMap::new();
        let time_sum_map = time_sum_map::load_bincode("./data/boot_time_sum_map.bin");

        let hour = u8::try_from((initial_seed >> 16) & 0xff).ok().filter(|&h| h < 24)?;
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

                let mut sorted: Vec<((u8, u8), (u8, u8, u8))> = min_boot_time_sec_map.into_iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));

                for value in &sorted {
                    let tuple_1 = value.0;
                    let tuple_2 = (hour, value.1 .0, value.1 .1, value.1 .2);
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
