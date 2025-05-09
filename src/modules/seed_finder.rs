use itertools::iproduct;

use crate::modules::rng_lc::RngLC;
use crate::modules::seed_analyzer::SeedAnalyzer;
use crate::types::pokemon_ivs::IVRanges;
use crate::types::types::*;

pub struct SeedFinder {
    rng_lc: RngLC,
    seed_analyzer: SeedAnalyzer,
}

impl SeedFinder {
    pub fn new() -> Self {
        Self {
            rng_lc: RngLC::new(),
            seed_analyzer: SeedAnalyzer::new(),
        }
    }

    pub fn find_seed_from_iv_ranges(&self, iv_ranges: IVRanges) -> Vec<Seed> {
        let mut matched_iv_1st_seed_list = Vec::<Seed>::new();
        let iv_range_group_1 = [&iv_ranges.hp, &iv_ranges.attack, &iv_ranges.defense];
        let iv_range_group_2 = [&iv_ranges.speed, &iv_ranges.sp_attack, &iv_ranges.sp_defense];

        let iv_range_group_1_combination_size = iv_range_group_1
            .iter()
            .map(|range| (*range.end() as u16 - *range.start() as u16))
            .sum::<u16>();

        let iv_range_group_2_combination_size = iv_range_group_2
            .iter()
            .map(|range| (*range.end() as u16 - *range.start() as u16))
            .sum::<u16>();

        let (smaller_group, larger_group, forward) =
            if iv_range_group_1_combination_size < iv_range_group_2_combination_size {
                (iv_range_group_1, iv_range_group_2, true)
            } else {
                (iv_range_group_2, iv_range_group_1, false)
            };

        for (iv_range_1, iv_range_2, iv_range_3) in iproduct!(
            smaller_group[0].clone(),
            smaller_group[1].clone(),
            smaller_group[2].clone()
        ) {
            let iv_smaller_group_rand_high_msb_0 = self
                .seed_analyzer
                .iv_group_to_rand([iv_range_1, iv_range_2, iv_range_3]);

            let iv_smaller_group_rand_high_msb_1 = (1 << 15) | iv_smaller_group_rand_high_msb_0;

            for (iv_smaller_group_rand_high, iv_smaller_group_rand_low) in iproduct!(
                [iv_smaller_group_rand_high_msb_0, iv_smaller_group_rand_high_msb_1],
                0..=0xffff
            ) {
                let (iv_1st_seed, ivs) = if forward {
                    let iv_1st_seed = self
                        .seed_analyzer
                        .rands_to_seed(iv_smaller_group_rand_high, iv_smaller_group_rand_low);

                    let iv_2nd_seed = self.rng_lc.next(iv_1st_seed);
                    let ivs = self.seed_analyzer.rand_to_ivs(self.rng_lc.extract_rand(iv_2nd_seed));
                    (iv_1st_seed, ivs)
                } else {
                    let iv_2nd_seed = self
                        .seed_analyzer
                        .rands_to_seed(iv_smaller_group_rand_high, iv_smaller_group_rand_low);

                    let iv_1st_seed = self.rng_lc.prev(iv_2nd_seed);
                    let ivs = self.seed_analyzer.rand_to_ivs(self.rng_lc.extract_rand(iv_1st_seed));
                    (iv_1st_seed, ivs)
                };

                if larger_group
                    .iter()
                    .zip(ivs.iter())
                    .all(|(range, value)| range.contains(value))
                {
                    matched_iv_1st_seed_list.push(iv_1st_seed)
                }
            }
        }

        return matched_iv_1st_seed_list;
    }

    pub fn find_seed_from_pid(&self, pid: Pid) -> Vec<Seed> {
        let mut matched_pid_1st_seed_list: Vec<Seed> = Vec::new();
        let pid_1st_rand_high = (pid & 0xffff) as Rand;
        let pid_2nd_rand_high = (pid >> 16) as Rand;

        for pid_1st_rand_low in 0..=0xffff {
            let pid_1st_seed = self.seed_analyzer.rands_to_seed(pid_1st_rand_high, pid_1st_rand_low);
            let pid_2nd_seed = self.rng_lc.next(pid_1st_seed);

            if pid_2nd_rand_high == self.rng_lc.extract_rand(pid_2nd_seed) {
                matched_pid_1st_seed_list.push(pid_1st_seed);
            }
        }

        return matched_pid_1st_seed_list;
    }

    pub fn find_seed_from_boot_time(&self, seed: Seed, max_advances: u16, min_waiting_frame: u16) -> Seed {
        let mut _seed = self.rng_lc.next(seed); // 引数で与えられたシードが有効かもしれないので、一度nextしてチェックする
        let mut advances = 0; // 消費数
        let mut found = false;

        while found == false && advances < max_advances {
            _seed = self.rng_lc.prev(_seed);

            /*
              seed: 0x12345678
              12   -> time_sum      (month * day + minute + second)
              34   -> hour          (ここが24以上だと、いつ起動してもこのシードにならないのでcontinue)
              5678 -> waiting_frame (frame + (year - 2000)
              ソフト選択からつづきからまで約14秒かかり、その内空白時間が約4秒のため、つづきからを押すまでに最速でも10秒はかかる。
              よってframeが600Fくらいはないと間に合わないことになる。
            */
            let hour = ((_seed >> 16) & 0xff) as u8;
            let frame_sum = (_seed & 0xffff) as u16;
            advances += 1;

            if 24 <= hour || min_waiting_frame < frame_sum {
                continue;
            }

            found = true;
        }

        return _seed;
    }
}
