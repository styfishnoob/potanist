use itertools::iproduct;
use serde::Deserialize;

use super::rand_analyzer::RandAnalyzer;
use super::rng_lc::RngLC;
use super::rng_mt::RngMT;
use super::seed_analyzer::SeedAnalyzer;
use crate::types::iv::*;
use crate::types::seed::*;

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

pub struct ReturnParams {
    pub initial_seed: InitialSeed,
    pub ivs: IVs,
    pub pid: PID,
    pub nature: u8,
    pub gender: u8,
    pub ability: u8,
    pub hidden_power_type: u8,
    pub hidden_power_power: u8,
    pub advances: u16,
    pub time_sum: u16,
    pub hour: u16,
    pub frame_sum: u16,
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
    pub fn search_seeds_from_status(&self, params: SearchParams) -> Vec<ReturnParams> {
        let mut result: Vec<ReturnParams> = Vec::new();

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
                let mut ivs: [u8; 6] = [0; 6];

                let (iv_1st_seed, iv_2nd_iv_group) = if forward {
                    let iv_1st_seed = self.rng_analyzer.rands_to_seed(iv_rand_high, iv_rand_low);
                    let iv_2nd_seed = self.rng_lc.next(iv_1st_seed);
                    let iv_2nd_rand = self.rng_analyzer.extract_rand(iv_2nd_seed);
                    let iv_2nd_iv_group = self.rng_analyzer.rand_to_iv_group(iv_2nd_rand);
                    ivs[0..3].copy_from_slice(&iv_group);
                    ivs[3..6].copy_from_slice(&iv_2nd_iv_group);
                    (iv_1st_seed, iv_2nd_iv_group)
                } else {
                    let iv_2nd_seed = self.rng_analyzer.rands_to_seed(iv_rand_high, iv_rand_low);
                    let iv_1st_seed = self.rng_lc.prev(iv_2nd_seed);
                    let iv_1st_rand = self.rng_analyzer.extract_rand(iv_1st_seed);
                    let iv_2nd_iv_group = self.rng_analyzer.rand_to_iv_group(iv_1st_rand);
                    ivs[0..3].copy_from_slice(&iv_2nd_iv_group);
                    ivs[3..6].copy_from_slice(&iv_group);
                    (iv_1st_seed, iv_2nd_iv_group)
                };

                let ivs_contains_range = larger_group
                    .iter()
                    .zip(iv_2nd_iv_group.iter())
                    .all(|(range, value)| range.contains(value));

                let status = self
                    .seed_analyzer
                    .extract_status(iv_1st_seed, params.tid, params.sid);
                let check_nature = params.nature == -1 || params.nature == status.nature as i16;
                let check_ability = params.ability == -1 || params.ability == status.ability as i16;
                let check_shiny = !params.shiny || status.shiny;

                if ivs_contains_range && check_nature && check_ability && check_shiny {
                    let initial_seed_data = self.search_initial_seed(
                        iv_1st_seed,
                        params.max_advances,
                        params.max_frame_sum,
                    );

                    let ivs = IVs {
                        hp: ivs[0],
                        attack: ivs[1],
                        defense: ivs[2],
                        speed: ivs[3],
                        sp_attack: ivs[4],
                        sp_defense: ivs[5],
                    };

                    if let Some(init_seed_data) = initial_seed_data {
                        result.push(ReturnParams {
                            initial_seed: init_seed_data.0,
                            ivs: ivs,
                            pid: status.pid,
                            nature: status.nature,
                            gender: status.gender,
                            ability: status.ability,
                            hidden_power_type: status.hidden_power_type,
                            hidden_power_power: status.hidden_power_power,
                            advances: init_seed_data.1,
                            time_sum: init_seed_data.2,
                            hour: init_seed_data.3,
                            frame_sum: init_seed_data.4,
                        });
                    }
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
                    let is_shiny = {
                        let tsid_xor = (params.tid ^ params.sid) as u32;
                        let pid_xor = ((pid >> 16) ^ (pid & 0xffff)) as u32;
                        (tsid_xor ^ pid_xor) <= 7
                    };

                    if nature_num == 3 && ability_num == 0 && 127 <= gender_num && is_shiny {
                        continue 'time_sum_loop; // 外側のループへ
                    }
                }
            }
        }
    }

    pub fn search_seeds_from_egg_iv(
        &self,
        params: SearchParams,
        parent_ivs_0: IVs,
        parent_ivs_1: IVs,
    ) -> Vec<IV1stSeed> {
        let mut result: Vec<IV1stSeed> = Vec::new();

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

        let (smaller_group, larger_group, forward) = if complexity_1 <= complexity_2 {
            (iv_range_group_1.clone(), iv_range_group_2.clone(), true)
        } else {
            (iv_range_group_2.clone(), iv_range_group_1.clone(), false)
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
                let mut ivs: [u8; 6] = [0; 6];

                let (iv_1st_seed, iv_2nd_seed, iv_2nd_iv_group) = if forward {
                    let iv_1st_seed = self.rng_analyzer.rands_to_seed(iv_rand_high, iv_rand_low);
                    let iv_2nd_seed = self.rng_lc.next(iv_1st_seed);
                    let iv_2nd_rand = self.rng_analyzer.extract_rand(iv_2nd_seed);
                    let iv_2nd_iv_group = self.rng_analyzer.rand_to_iv_group(iv_2nd_rand);
                    ivs[0..3].copy_from_slice(&iv_group);
                    ivs[3..6].copy_from_slice(&iv_2nd_iv_group);
                    (iv_1st_seed, iv_2nd_seed, iv_2nd_iv_group)
                } else {
                    let iv_2nd_seed = self.rng_analyzer.rands_to_seed(iv_rand_high, iv_rand_low);
                    let iv_1st_seed = self.rng_lc.prev(iv_2nd_seed);
                    let iv_1st_rand = self.rng_analyzer.extract_rand(iv_1st_seed);
                    let iv_2nd_iv_group = self.rng_analyzer.rand_to_iv_group(iv_1st_rand);
                    ivs[0..3].copy_from_slice(&iv_2nd_iv_group);
                    ivs[3..6].copy_from_slice(&iv_group);
                    (iv_1st_seed, iv_2nd_seed, iv_2nd_iv_group)
                };

                let iv_2nd_iv_group_contains_range = larger_group
                    .iter()
                    .zip(iv_2nd_iv_group.iter())
                    .all(|(range, value)| range.contains(value));

                if iv_2nd_iv_group_contains_range {
                    result.push(iv_1st_seed);
                    continue;
                }

                let parent_ivs_0: [IV; 6] = [
                    parent_ivs_0.hp,
                    parent_ivs_0.attack,
                    parent_ivs_0.defense,
                    parent_ivs_0.speed,
                    parent_ivs_0.sp_attack,
                    parent_ivs_0.sp_defense,
                ];

                let parent_ivs_1: [IV; 6] = [
                    parent_ivs_1.hp,
                    parent_ivs_1.attack,
                    parent_ivs_1.defense,
                    parent_ivs_1.speed,
                    parent_ivs_1.sp_attack,
                    parent_ivs_1.sp_defense,
                ];

                // 0: hp | 1: attack | 2: defense | 3: speed | 4: sp_attack | 5: sp_defense
                let mut gene_loci: Vec<u8> = [0, 1, 2, 3, 4, 5].to_vec();
                let parents_ivs = [parent_ivs_0, parent_ivs_1];

                let gene_locus_seed_1 = self.rng_lc.next(iv_2nd_seed);
                let gene_locus_seed_2 = self.rng_lc.next(gene_locus_seed_1);
                let gene_locus_seed_3 = self.rng_lc.next(gene_locus_seed_2);
                let gene_parent_seed_1 = self.rng_lc.next(gene_locus_seed_3);
                let gene_parent_seed_2 = self.rng_lc.next(gene_parent_seed_1);
                let gene_parent_seed_3 = self.rng_lc.next(gene_parent_seed_2);

                let gene_locus_rand_1 = self.rng_analyzer.extract_rand(gene_locus_seed_1);
                let gene_locus_rand_2 = self.rng_analyzer.extract_rand(gene_locus_seed_2);
                let gene_locus_rand_3 = self.rng_analyzer.extract_rand(gene_locus_seed_3);
                let gene_parent_rand_1 = self.rng_analyzer.extract_rand(gene_parent_seed_1);
                let gene_parent_rand_2 = self.rng_analyzer.extract_rand(gene_parent_seed_2);
                let gene_parent_rand_3 = self.rng_analyzer.extract_rand(gene_parent_seed_3);

                let gene_locus_index_1 = (gene_locus_rand_1 % 6) as usize; // gene_loci のインデックス番号
                let gene_locus_index_2 = (gene_locus_rand_2 % 5) as usize; // gene_loci のインデックス番号
                let gene_locus_index_3 = (gene_locus_rand_3 % 4) as usize; // gene_loci のインデックス番号
                let gene_parent_num_1 = (gene_parent_rand_1 % 2) as usize; // 参照する親番号 (0 -> 先親 | 1 -> 後親)
                let gene_parent_num_2 = (gene_parent_rand_2 % 2) as usize; // 参照する親番号 (0 -> 先親 | 1 -> 後親)
                let gene_parent_num_3 = (gene_parent_rand_3 % 2) as usize; // 参照する親番号 (0 -> 先親 | 1 -> 後親)

                let gene_locus_1 = gene_loci[gene_locus_index_1] as usize;
                gene_loci.remove(gene_locus_index_1);

                let gene_locus_2 = gene_loci[gene_locus_index_2] as usize;
                gene_loci.remove(gene_locus_index_2);

                let gene_locus_3 = gene_loci[gene_locus_index_3] as usize;
                gene_loci.remove(gene_locus_index_3);

                ivs[gene_locus_1] = parents_ivs[gene_parent_num_1][gene_locus_1];
                ivs[gene_locus_2] = parents_ivs[gene_parent_num_2][gene_locus_2];
                ivs[gene_locus_3] = parents_ivs[gene_parent_num_3][gene_locus_3];

                let all_ivs_contains_range = [
                    iv_range_group_1[0].clone(),
                    iv_range_group_1[1].clone(),
                    iv_range_group_1[2].clone(),
                    iv_range_group_2[0].clone(),
                    iv_range_group_2[1].clone(),
                    iv_range_group_2[2].clone(),
                ]
                .iter()
                .zip(ivs.iter())
                .all(|(range, value)| range.contains(value));

                if all_ivs_contains_range {
                    result.push(iv_1st_seed);
                }
            }
        }

        return result;
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
        iv_1st_seed: IV1stSeed,
        max_advances: u16,
        max_frame_sum: u16,
    ) -> Option<(InitialSeed, u16, u16, u16, u16)> {
        let mut initial_seed = self.rng_lc.prev(iv_1st_seed); // 一つ目のpidシードが有効かもしれないので、一回だけprevして一つ目が有効かどうか確認する
        let mut advances: u16 = 0; // 消費数

        while (0..=max_advances).contains(&advances) {
            initial_seed = self.rng_lc.prev(initial_seed);
            advances += 1;

            let time_sum = ((initial_seed >> 24) & 0xff) as u16;
            let hour = ((initial_seed >> 16) & 0xff) as u16;
            let frame_sum = (initial_seed & 0xffff) as u16;

            // time_sum の最大値は 12 * 31 + 59 + 59 の 490
            if time_sum > 490 {
                continue;
            }

            // frame_sum の最低値は600で固定する。
            if 24 <= hour {
                continue;
            }

            // +99 はDSで設定できる最大の年
            if (600..=max_frame_sum + 99).contains(&frame_sum) == false {
                continue;
            }

            // 最初 iv_1st -> pid_2nd -> pid_1st と二回prevする必要があるものの、一回しかprevしていないため、一回分消費数を減らす。
            return Some((initial_seed, advances - 1, time_sum, hour, frame_sum));
        }

        return None;
    }
}
