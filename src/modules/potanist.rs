use itertools::iproduct;
use std::ops::RangeInclusive;

use crate::constants::pokemon_nature::POKEMON_NATURES;
use crate::modules::lcrng::LCRng;
use crate::types::ivs::IVs;

pub struct Potanist {
    lcrng: LCRng,
}

impl Potanist {
    pub fn new() -> Self {
        Self {
            lcrng: LCRng::new(),
        }
    }

    pub fn ivs_to_rand(&self, ivs: [u8; 3]) -> u16 {
        let rand = (ivs[0] as u16) | (ivs[1] as u16) << 5 | (ivs[2] as u16) << 10;
        return rand & 0xffff;
    }

    pub fn rand_to_ivs(&self, rand: u16) -> [u8; 3] {
        return [
            (rand & 0x1f) as u8,
            ((rand >> 5) & 0x1f) as u8,
            ((rand >> 10) & 0x1f) as u8,
        ];
    }

    pub fn rands_to_seed(&self, rand_1: u16, rand_2: u16) -> u32 {
        return (rand_1 as u32) << 16 | (rand_2 as u32);
    }

    fn extract_pokemon_data_from_ivs_seeds(&self, ivs_seed_1: u32, ivs_seed_2: u32) {
        let pid_seed_2 = self.lcrng.prev(ivs_seed_1);
        let pid_seed_1 = self.lcrng.prev(pid_seed_2);
        let pid_rand_2 = self.lcrng.extract_rand(pid_seed_2);
        let pid_rand_1 = self.lcrng.extract_rand(pid_seed_1);
        let pid = (pid_rand_2 as u32) << 16 | (pid_rand_1 as u32);

        let nature_num = (pid % 100 % 25) as u8;
        let gender_num = pid & 0xff;
        let ability_num = pid & 1;

        let (nature_en, nature_ja) = POKEMON_NATURES.get(&nature_num).unwrap();

        println!("pid_seed_1 : {:#010x}", pid_seed_1);
        println!("pid_seed_2 : {:#010x}", pid_seed_2);
        println!("ivs_seed_1 : {:#010x}", ivs_seed_1);
        println!("ivs_seed_2 : {:#010x}", ivs_seed_2);
        println!("pid        : {:#010x}", pid);
        println!("nature     : {}({})", nature_ja, nature_en);
        println!("gender     : {}", gender_num);
        println!("ability    : {}", ability_num);
        println!("------------------");
    }

    pub fn find_seed_from_ivs(&self, ivs: IVs) {
        let calc_time_comp = |ivs_ranges: [&RangeInclusive<u8>; 3]| {
            ivs_ranges
                .iter()
                .map(|r| (*r.end() as u32) - (*r.start() as u32))
                .sum::<u32>()
        };

        let ivs_1 = [&ivs.hp, &ivs.attack, &ivs.defense];
        let ivs_2 = [&ivs.speed, &ivs.sp_attack, &ivs.sp_defense];

        let (faster_ivs_group, slower_ivs_group, forward) =
            if calc_time_comp(ivs_1) < calc_time_comp(ivs_2) {
                (ivs_1, ivs_2, true)
            } else {
                (ivs_2, ivs_1, false)
            };

        for (iv_1, iv_2, iv_3) in iproduct!(
            faster_ivs_group[0].clone(),
            faster_ivs_group[1].clone(),
            faster_ivs_group[2].clone()
        ) {
            let ivs_rand_1_0 = self.ivs_to_rand([iv_1, iv_2, iv_3]);
            let ivs_rand_1_1 = (1 << 15) | ivs_rand_1_0;

            for (ivs_rand_1, ivs_rand_2) in iproduct!([ivs_rand_1_0, ivs_rand_1_1], 0..=0xffff) {
                let (ivs_1st_seed, ivs_2nd_seed, generated_ivs) = if forward {
                    let ivs_1st_seed = self.rands_to_seed(ivs_rand_1, ivs_rand_2);
                    let ivs_2nd_seed = self.lcrng.next(ivs_1st_seed);
                    let ivs = self.rand_to_ivs(self.lcrng.extract_rand(ivs_2nd_seed));
                    (ivs_1st_seed, ivs_2nd_seed, ivs)
                } else {
                    let ivs_2nd_seed = self.rands_to_seed(ivs_rand_1, ivs_rand_2);
                    let ivs_1st_seed = self.lcrng.prev(ivs_2nd_seed);
                    let ivs = self.rand_to_ivs(self.lcrng.extract_rand(ivs_1st_seed));
                    (ivs_1st_seed, ivs_2nd_seed, ivs)
                };

                if slower_ivs_group
                    .iter()
                    .zip(generated_ivs.iter())
                    .all(|(range, value)| range.contains(value))
                {
                    self.extract_pokemon_data_from_ivs_seeds(ivs_1st_seed, ivs_2nd_seed);
                }
            }
        }
    }

    pub fn find_seed_from_pid(&self, pid: u32) {
        let pid_1st_rand_1 = (pid & 0xffff) as u16;
        let pid_2nd_rand_1 = (pid >> 16) as u16;

        for pid_1st_rand_2 in 0..=0xffff {
            let pid_1st_seed = self.rands_to_seed(pid_1st_rand_1, pid_1st_rand_2);
            let pid_2nd_seed = self.lcrng.next(pid_1st_seed);

            if pid_2nd_rand_1 == self.lcrng.extract_rand(pid_2nd_seed) {
                println!("pid_1st_seed    : {:#010x}", pid_1st_seed);
            }
        }
    }

    pub fn find_seed_from_boot_time(&self, seed: u32) {
        let mut current_seed = self.lcrng.next(seed);
        let mut found = false;
        let mut moves = -1;

        while found == false && moves < 500 {
            current_seed = self.lcrng.prev(current_seed);
            let time_sum_0 = (current_seed >> 24) & 0xff;
            let time_sum_1 = (1 << 8) | time_sum_0;
            let hour = (current_seed >> 16) & 0xff;
            let frame_sum = current_seed & 0xffff;
            moves += 1;

            // hour が 24 を超えている場合、いつ起動してもシードに到達不可能
            if hour >= 24 || frame_sum >= 1000 {
                continue;
            }

            found = true;
            println!("moves                   : {}", moves);
            println!("month * day + min + sec : {} or {}", time_sum_0, time_sum_1);
            println!("hour                    : {}", hour);
            println!("frame + year - 2000     : {}", frame_sum);
        }
    }
}
