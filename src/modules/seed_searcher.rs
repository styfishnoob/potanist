use itertools::iproduct;
use serde::Deserialize;
use std::ops::RangeInclusive;

use super::rand_analyzer::RandAnalyzer;
use super::rng_lc::RngLC;
use super::rng_mt::RngMT;
use super::seed_analyzer::SeedAnalyzer;
use crate::types::iv::*;
use crate::types::status::Status;
use crate::types::types::*;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    pub iv_ranges: IVRanges,
    pub nature: i16,
    pub ability: i16,
    pub shiny: bool,
    pub tid: Rand,
    pub sid: Rand,
    pub max_advances: u16,
    pub max_frame_sum: u16, // 最低値は600で固定
}

pub struct SeedSearcher {
    rng_analyzer: RandAnalyzer,
    rng_lc: RngLC,
    seed_analyzer: SeedAnalyzer,
}

impl SeedSearcher {
    pub fn new() -> Self {
        Self {
            rng_analyzer: RandAnalyzer::new(),
            rng_lc: RngLC::new(),
            seed_analyzer: SeedAnalyzer::new(),
        }
    }

    /*
        個体値/性格/特性/めざパ-タイプ/めざパ-威力/色違い(TID/SID)
        これらのパラメータから目的のシードを探索する。
        この処理で求められるシードは個体値の一つ目のシードであり、初期シードではないので注意。
    */
    pub fn search_seeds_from_status(&self, params: SearchParams) -> Vec<Seed> {
        let mut result: Vec<Seed> = Vec::new();

        let iv_range_group_1 = [
            params.iv_ranges.hp.clone(),
            params.iv_ranges.attack.clone(),
            params.iv_ranges.defense.clone(),
        ];
        let iv_range_group_2 = [
            params.iv_ranges.speed.clone(),
            params.iv_ranges.sp_attack.clone(),
            params.iv_ranges.sp_defense.clone(),
        ];

        let calc_complexity = |iv_ranges: &[IVRange; 3]| {
            iv_ranges
                .iter()
                .fold(1, |acc, range| acc * (range.end() - range.start()))
        };

        let complexity_1 = calc_complexity(&iv_range_group_1);
        let complexity_2 = calc_complexity(&iv_range_group_2);

        let (smaller_group, larger_group, forward) = if complexity_1 < complexity_2 {
            (iv_range_group_1, iv_range_group_2, true)
        } else {
            (iv_range_group_2, iv_range_group_1, false)
        };

        for (iv_1, iv_2, iv_3) in iproduct!(
            smaller_group[0].clone(),
            smaller_group[1].clone(),
            smaller_group[2].clone()
        ) {
            let iv_group: IVGroup = [iv_1, iv_2, iv_3];
            let iv_rand_high_msb_0 = self.rng_analyzer.iv_group_to_rand(iv_group);
            let iv_rand_high_msb_1 = (1 << 15) | iv_rand_high_msb_0;
            let iv_rand_high_group = [iv_rand_high_msb_0, iv_rand_high_msb_1];

            for (iv_rand_high, iv_rand_low) in iproduct!(iv_rand_high_group, 0..=0xffff) {
                let (iv_1st_seed, iv_2nd_iv_group) = if forward {
                    let iv_1st_seed = self.rng_analyzer.rands_to_seed(iv_rand_high, iv_rand_low);
                    let iv_2nd_seed = self.rng_lc.next(iv_1st_seed);
                    let iv_2nd_rand = self.rng_analyzer.extract_rand(iv_2nd_seed);
                    let iv_2nd_iv_group = self.rng_analyzer.rand_to_iv_group(iv_2nd_rand);
                    (iv_1st_seed, iv_2nd_iv_group)
                } else {
                    let iv_2nd_seed = self.rng_analyzer.rands_to_seed(iv_rand_high, iv_rand_low);
                    let iv_1st_seed = self.rng_lc.prev(iv_2nd_seed);
                    let iv_1st_rand = self.rng_analyzer.extract_rand(iv_1st_seed);
                    let iv_2nd_iv_group = self.rng_analyzer.rand_to_iv_group(iv_1st_rand);
                    (iv_1st_seed, iv_2nd_iv_group)
                };

                let ivs_contains_range = larger_group
                    .iter()
                    .zip(iv_2nd_iv_group.iter())
                    .all(|(range, value)| range.contains(value));

                let status = self.seed_analyzer.extract_status(iv_1st_seed);
                let check_nature = params.nature == -1 || params.nature == status.nature;
                let check_ability = params.ability == -1 || params.ability == status.ability;
                let check_shiny = !params.shiny || is_shiny(params.tid, params.sid, status.pid);

                if ivs_contains_range && check_nature && check_ability && check_shiny {
                    result.push(iv_1st_seed);
                }
            }
        }

        return result;
    }

    /*
        time_sum が 0..=0xff な理由は、ffから値が折り返すため。
        0(0x0), 1(0x1), ... , 255(0xff), 256(0x100), 257(0x101)
        で、time_sum は実質 & 0xff されるため、256(0x100) は 0x00 になり、
        256(0x00) = 0, 257(0x101) = 1, 258(0x102) = 2 ... と、256を折り返しにそれぞれ　0, 1, 2, ...　に対応していく。
        なので、0 から 255 まで回すだけでよく、time_sumを表示する際は、0x00 と 0x00 の左端に 1 を足し、256(0x100) を表示させればいい。
        ただし、time_sum の最大値は 12 * 31 + 59 + 59 の 490(0x1ea) で、それに対応する 234(0xea) 以降は実現不可能な値になるため注意。
    */
    pub fn search_seeds_from_egg_pid(&self, params: SearchParams) {
        'time_sum_loop: for time_sum in 0..=0xff {
            for (hour, frame_sum) in iproduct!(0..=23, 600..=params.max_frame_sum) {
                let initial_seed =
                    (time_sum as Seed) << 24 | (hour as Seed) << 12 | frame_sum as Seed;
                let mut rng_mt = RngMT::new(initial_seed);

                for advances in 0..=params.max_advances {
                    let next_seed = rng_mt.next();
                    let pid = {
                        let k0 = (next_seed / 0x800) ^ next_seed;
                        let k1 = (k0.wrapping_mul(0x80) & 0x9d2c5680) ^ k0;
                        let k2 = (k1.wrapping_mul(0x8000) & 0xefc60000) ^ k1;
                        (k2 / 0x40000) ^ k2
                    };

                    let nature_num = (pid % 25) as i16;
                    let gender_num = (pid & 0xff) as i16;
                    let ability_num = (pid & 1) as i16;
                    let is_shiny = is_shiny(params.tid, params.sid, pid);

                    if nature_num == 3 && ability_num == 0 && 127 <= gender_num && is_shiny {
                        continue 'time_sum_loop; // 外側のループへ
                    }
                }
            }
        }
    }

    /*
      search_seedで求めたシードから、初期シードを求める。
      シードが以下の値に一致しない(hourの部分が78で、24を超えているなど)場合、いつ起動しても到達不可能のため、
      到達可能な初期シードを求める。

      seed: 0x12345678
      12   -> time_sum      (month * day + minute + second)
      34   -> hour          (ここが24以上だと、いつ起動してもこのシードにならないのでcontinue)
      5678 -> frame_sum (frame + (year - 2000)

      ※ ソフト選択からつづきからまで約14秒かかり、その内空白時間が約4秒(DPPt/HGSSによって異なる)のため、つづきからを押すまでに最速でも10秒はかかる。
        よってframeが600F(10秒)くらいはないと間に合わないことになる。

    */
    pub fn search_initial_seed(
        &self,
        iv_1st_seed: Seed,
        max_advances: u16,
        max_frame_sum: u16,
    ) -> Option<(Seed, u16)> {
        let mut initial_seed = self.rng_lc.prev(iv_1st_seed); // 一つ目のpidシードが有効かもしれないので、一回だけprevして一つ目が有効かどうか確認する
        let mut advances: u16 = 0; // 消費数
        let mut found = false;

        while found == false && (0..=max_advances).contains(&advances) {
            initial_seed = self.rng_lc.prev(initial_seed);
            advances += 1;

            let hour = ((initial_seed >> 16) & 0xff) as u8;
            let frame_sum = (initial_seed & 0xffff) as u16;

            // frame_sum の最低値は600で固定する。
            if 24 <= hour || !(600..=max_frame_sum).contains(&frame_sum) {
                continue;
            }

            found = true;
        }

        // 最初 iv_1st -> pid_2nd -> pid_1st と二回prevする必要があるものの、一回しかprevしていないため、一回分消費数を減らす。
        if found {
            return Some((initial_seed, advances - 1));
        } else {
            return None;
        }
    }
}

fn is_shiny(tid: Rand, sid: Rand, pid: Pid) -> bool {
    let tsid_xor = (tid ^ sid) as u32;
    let pid_xor = ((pid >> 16) ^ (pid & 0xffff)) as u32;
    (tsid_xor ^ pid_xor) <= 7
}
