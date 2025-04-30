use std::ops::RangeInclusive;

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

    pub fn make_seed_from_rands(&self, rand_1: u16, rand_2: u16) -> u32 {
        return (rand_1 as u32) << 16 | (rand_2 as u32);
    }

    fn extract_pokemon_data_from_ivs_seeds(&self, ivs_seed_1: u32, ivs_seed_2: u32) {
        let pid_seed_2 = self.lcrng.prev(ivs_seed_1);
        let pid_seed_1 = self.lcrng.prev(pid_seed_2);
        let pid_rand_2 = self.lcrng.extract_rand(pid_seed_2);
        let pid_rand_1 = self.lcrng.extract_rand(pid_seed_1);
        let pid = (pid_rand_2 as u32) << 16 | (pid_rand_1 as u32);
        let nature_num = pid % 100 % 25;
        let gender_num = pid & 0xff;
        let ability_num = pid & 1;

        println!("pid_seed_1 : {:#010x}", pid_seed_1);
        println!("pid_seed_2 : {:#010x}", pid_seed_2);
        println!("ivs_seed_1 : {:#010x}", ivs_seed_1);
        println!("ivs_seed_2 : {:#010x}", ivs_seed_2);
        println!("pid        : {:#010x}", pid);
        println!("nature     : {}", nature_num);
        println!("gender     : {}", gender_num);
        println!("ability    : {}", ability_num);
        println!("------------------")
    }

    pub fn find_seed_from_ivs(&self, ivs: IVs) {
        let calc_time_comp = |ivs_ranges: [&RangeInclusive<u8>; 3]| {
            ivs_ranges
                .iter()
                .map(|r| (*r.end() as u32) - (*r.start() as u32))
                .sum::<u32>()
        };

        let group_1 = [&ivs.hp, &ivs.attack, &ivs.defense];
        let group_2 = [&ivs.speed, &ivs.sp_attack, &ivs.sp_defense];

        let (outer_group, inner_group, forward) =
            if calc_time_comp(group_1) < calc_time_comp(group_2) {
                (group_1, group_2, true)
            } else {
                (group_2, group_1, false)
            };

        for ivs_1 in outer_group[0].clone() {
            for ivs_2 in outer_group[1].clone() {
                for ivs_3 in outer_group[2].clone() {
                    let ivs_rand_1_0 = self.ivs_to_rand([ivs_1, ivs_2, ivs_3]);
                    let ivs_rand_1_1 = (1 << 15) | ivs_rand_1_0;

                    for ivs_rand_2 in 0..=0xffff {
                        for &ivs_rand_1 in &[ivs_rand_1_0, ivs_rand_1_1] {
                            let (ivs_seed_1, ivs_seed_2, ivs) = if forward {
                                let _ivs_seed_1 = self.make_seed_from_rands(ivs_rand_1, ivs_rand_2);
                                let _ivs_seed_2 = self.lcrng.next(_ivs_seed_1);
                                let _ivs = self.rand_to_ivs(self.lcrng.extract_rand(_ivs_seed_2));
                                (_ivs_seed_1, _ivs_seed_2, _ivs)
                            } else {
                                let _ivs_seed_2 = self.make_seed_from_rands(ivs_rand_1, ivs_rand_2);
                                let _ivs_seed_1 = self.lcrng.prev(_ivs_seed_2);
                                let _ivs = self.rand_to_ivs(self.lcrng.extract_rand(_ivs_seed_1));
                                (_ivs_seed_1, _ivs_seed_2, _ivs)
                            };

                            if inner_group
                                .iter()
                                .zip(ivs.iter())
                                .all(|(range, value)| range.contains(value))
                            {
                                self.extract_pokemon_data_from_ivs_seeds(ivs_seed_1, ivs_seed_2);
                            }
                        }
                    }
                }
            }
        }
    }
}
