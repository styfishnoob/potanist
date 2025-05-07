use crate::constants::boot_time_sum_map;
use crate::constants::pokemon_nature::POKEMON_NATURES;
use crate::modules::lcrng::LCRng;
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

    pub fn display_pokemon_data_from_iv_1st_seed(&self, iv_1st_seed: Seed) {
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
        let gender_num = pid & 0xff;
        let ability_num = pid & 1;
        let (nature_en, nature_ja) = POKEMON_NATURES.get(&nature_num).unwrap();

        println!("性格値 : {:#010x}", pid);
        println!("性格   : {}({})", nature_ja, nature_en);
        println!("性別   : {}", gender_num);
        println!("特性   : {}", ability_num);
        println!(
            "個体値 : {}-{}-{}-{}-{}-{}",
            ivs_1st[0], ivs_1st[1], ivs_1st[2], ivs_2nd[0], ivs_2nd[1], ivs_2nd[2]
        );
        println!("---------------------------------------------");
    }

    pub fn display_boot_time_list(&self, time_sum: u16, hour: u8, frame_sum: u16, blank_frame: u16) {
        let boot_time_map = boot_time_sum_map::load_bincode("./data/boot_time_sum_map.bin");

        for year in 0..=99 {
            let frame = frame_sum - year;

            if let Some(boot_time_vec) = boot_time_map.get(&time_sum) {
                let mut min_boot_time_sec_map: HashMap<(u8, u8), (u8, u16, u8)> = HashMap::new();

                for (month, day, min, sec) in boot_time_vec {
                    let waiting_frame: u16 = frame + blank_frame; // ポケモンを選択してから待機する時間
                    let waiting_time: u16 = waiting_frame / 60;

                    if (*sec as u16) < waiting_time {
                        continue;
                    }

                    let boot_time_sec: u16 = (*sec as u16) - waiting_time;

                    if boot_time_sec < 10 {
                        continue; // 10秒以下だと時間変更からDS再起動までに7~8秒かかるため間に合わない
                    }

                    min_boot_time_sec_map
                        .entry((*month, *day))
                        .and_modify(|entry| {
                            if boot_time_sec < entry.1 {
                                *entry = (*min, boot_time_sec, *sec);
                            }
                        })
                        .or_insert_with(|| (*min, boot_time_sec, *sec));
                }

                let mut sorted: Vec<((u8, u8), (u8, u16, u8))> = min_boot_time_sec_map.into_iter().collect();

                sorted.sort_by(|a, b| a.0.cmp(&b.0));

                for ((month, day), (min, bsec, sec)) in sorted {
                    println!(
                      "20{:02}年 {:02}月 {:02}日 {:02}時 {:02}分 {:02}秒に選択 | 開始: {:02}秒 | 月*日+分+秒: {} | 待機時間: {}",
                      year, month, day, hour, min, bsec, sec, month * day + min + sec, (frame + blank_frame) / 60
                  );
                }

                println!("-----------------------------------------------------")
            } else {
                println!("{} に一致する起動時間は見つかりませんでした。", time_sum)
            }
        }
    }
}
