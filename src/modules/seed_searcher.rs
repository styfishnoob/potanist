use std::ops::RangeInclusive;

use itertools::iproduct;

use super::rand_analyzer::RandAnalyzer;
use super::rng_lc::RngLC;
use super::seed_analyzer::SeedAnalyzer;
use crate::types::iv::*;
use crate::types::status::Status;
use crate::types::types::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchSeedParams {
    pub iv_ranges: IVRanges,
    pub nature: i16,
    pub ability: i16,
    pub hidden_power_type: i16,
    pub hidden_power_power: IVRange, // 30 ~ 70
    pub shiny: bool,
    pub tid: Rand,
    pub sid: Rand,
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
    pub fn search_seed(&self, search_seed_params: SearchSeedParams) -> Vec<Seed> {
        let mut matched_iv_1st_seed_vec: Vec<Seed> = Vec::new();
        let (smaller_group, larger_group, forward) =
            compare_range_group_by_complexity(&search_seed_params);

        for (iv_range_1, iv_range_2, iv_range_3) in iproduct!(
            smaller_group[0].clone(),
            smaller_group[1].clone(),
            smaller_group[2].clone()
        ) {
            let iv_group: IVGroup = [iv_range_1, iv_range_2, iv_range_3];
            let iv_smaller_group_rand_high_msb_0 = self.rng_analyzer.iv_group_to_rand(iv_group);
            let iv_smaller_group_rand_high_msb_1 = (1 << 15) | iv_smaller_group_rand_high_msb_0;

            for (iv_smaller_group_rand_high, iv_smaller_group_rand_low) in iproduct!(
                [
                    iv_smaller_group_rand_high_msb_0,
                    iv_smaller_group_rand_high_msb_1
                ],
                0..=0xffff
            ) {
                let (iv_1st_seed, iv_2nd_iv_group) = derive_iv_1st_seed_and_iv_2nd_iv_group(
                    iv_smaller_group_rand_high,
                    iv_smaller_group_rand_low,
                    forward,
                    &self.rng_analyzer,
                    &self.rng_lc,
                );

                let ivs_contains_range = larger_group
                    .iter()
                    .zip(iv_2nd_iv_group.iter())
                    .all(|(range, value)| range.contains(value));

                let extracted_status = self.seed_analyzer.extract_status(iv_1st_seed);
                let check_status = check_status(&extracted_status, &search_seed_params);

                if ivs_contains_range && check_status {
                    matched_iv_1st_seed_vec.push(iv_1st_seed)
                }
            }
        }

        return matched_iv_1st_seed_vec;
    }

    /*
      search_seedで求めたシードから、初期シードを求める。
      シードが以下の値に一致しない(hourの部分が78で、24を超えているなど)場合、いつ起動しても到達不可能のため、
      到達可能な初期シードを求める。

      seed: 0x12345678
      12   -> time_sum      (month * day + minute + second)
      34   -> hour          (ここが24以上だと、いつ起動してもこのシードにならないのでcontinue)
      5678 -> waiting_frame (frame + (year - 2000)

      ※ ソフト選択からつづきからまで約14秒かかり、その内空白時間が約4秒(DPPt/HGSSによって異なる)のため、つづきからを押すまでに最速でも10秒はかかる。
        よってframeが600F(10秒)くらいはないと間に合わないことになる。

    */
    pub fn search_initial_seed(
        &self,
        iv_1st_seed: Seed,
        advance_range: RangeInclusive<u16>,
        frame_sum_range: RangeInclusive<u16>,
    ) -> Option<(Seed, u16)> {
        let mut initial_seed = self.rng_lc.prev(iv_1st_seed); // 一つ目のpidシードが有効かもしれないので、一回だけprevして一つ目が有効かどうか確認する
        let mut advances: u16 = 0; // 消費数
        let mut found = false;

        while found == false && advance_range.contains(&advances) {
            initial_seed = self.rng_lc.prev(initial_seed);
            advances += 1;

            let hour = ((initial_seed >> 16) & 0xff) as u8;
            let frame_sum = (initial_seed & 0xffff) as u16;

            if 24 <= hour || !frame_sum_range.contains(&frame_sum) {
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

/*
  (IVRangeGroup, IVRangeGroup, bool) -> (smaller_group, larget_group, forward)
  forwardは、smaller_groupがiv_1stだった場合とiv_2ndだった場合で処理順が変わるため、こちらを返却する。
*/
fn compare_range_group_by_complexity(
    search_seed_params: &SearchSeedParams,
) -> (IVRangeGroup, IVRangeGroup, bool) {
    let iv_range_group_1 = [
        search_seed_params.iv_ranges.hp.clone(),
        search_seed_params.iv_ranges.attack.clone(),
        search_seed_params.iv_ranges.defense.clone(),
    ];
    let iv_range_group_2 = [
        search_seed_params.iv_ranges.speed.clone(),
        search_seed_params.iv_ranges.sp_attack.clone(),
        search_seed_params.iv_ranges.sp_defense.clone(),
    ];

    let complexity_1 = iv_range_group_1
        .iter()
        .fold(1, |acc, range| acc * (range.end() - range.start()));

    let compolexity_2 = iv_range_group_2
        .iter()
        .fold(1, |acc, range| acc * (range.end() - range.start()));

    if complexity_1 < compolexity_2 {
        (iv_range_group_1, iv_range_group_2, true)
    } else {
        (iv_range_group_2, iv_range_group_1, false)
    }
}

fn derive_iv_1st_seed_and_iv_2nd_iv_group(
    iv_smaller_group_rand_high: Rand,
    iv_smaller_group_rand_low: Rand,
    forward: bool,
    rng_analyzer: &RandAnalyzer,
    rng_lc: &RngLC,
) -> (Seed, IVGroup) {
    if forward {
        let iv_1st_seed =
            rng_analyzer.rands_to_seed(iv_smaller_group_rand_high, iv_smaller_group_rand_low);
        let iv_2nd_seed = rng_lc.next(iv_1st_seed);
        let iv_2nd_iv_group = rng_analyzer.rand_to_iv_group(rng_analyzer.extract_rand(iv_2nd_seed));
        (iv_1st_seed, iv_2nd_iv_group)
    } else {
        let iv_2nd_seed =
            rng_analyzer.rands_to_seed(iv_smaller_group_rand_high, iv_smaller_group_rand_low);
        let iv_1st_seed = rng_lc.prev(iv_2nd_seed);
        let iv_2nd_iv_group = rng_analyzer.rand_to_iv_group(rng_analyzer.extract_rand(iv_1st_seed));
        (iv_1st_seed, iv_2nd_iv_group)
    }
}

fn check_status(extracted_status: &Status, search_seed_params: &SearchSeedParams) -> bool {
    let check_nature =
        search_seed_params.nature == -1 || search_seed_params.nature == extracted_status.nature;

    let check_ability =
        search_seed_params.ability == -1 || search_seed_params.ability == extracted_status.ability;

    let check_hidden_power_type = search_seed_params.hidden_power_type == -1
        || search_seed_params.hidden_power_type == extracted_status.hidden_power_type;

    let check_hidden_power_power = search_seed_params.hidden_power_type == -1
        || search_seed_params
            .hidden_power_power
            .contains(&extracted_status.hidden_power_power);

    let check_shiny = !search_seed_params.shiny || {
        let tsid_xor = (search_seed_params.tid ^ search_seed_params.sid) as u32;
        let pid_xor = ((extracted_status.pid >> 16) ^ (extracted_status.pid & 0xffff)) as u32;
        (tsid_xor ^ pid_xor) <= 7
    };

    return check_nature
        && check_ability
        && check_hidden_power_type
        && check_hidden_power_power
        && check_shiny;
}
