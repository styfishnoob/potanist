use std::collections::HashMap;

use crate::{constants::roaming_routes, types::types::Seed};

use super::{rand_analyzer::RandAnalyzer, rng_lc::RngLC, rng_mt::RngMT};

pub struct SeedChecker {
    rng_analyzer: RandAnalyzer,
    rng_lc: RngLC,
}

impl SeedChecker {
    pub fn new() -> Self {
        Self {
            rng_analyzer: RandAnalyzer::new(),
            rng_lc: RngLC::new(),
        }
    }

    /*
        電話をかける候補として、ウツギとジャグラーのマイクが挙がるが、同じシードの場合、電話の内容は違えど使用する乱数は同じのため、
        会話内容の順番としては同じものになる。
        sequence_mapが例えば [0, 0, 1, 0, 0] ならウツギ、マイクそれぞれの対応する0の会話、1の会話がこの順番で返ってくる。
        そのため、この関数はとりあえず返答の内容番号だけ列挙したmapを返却し、どんな内容なのかはウェブ側で処理することにする。
    */
    pub fn create_call_response_sequence_map(
        &self,
        initial_seed: Seed,
        roaming_num: u8,
        search_range: u8,
    ) -> HashMap<Seed, Vec<u8>> {
        let mut call_response_sequence_map: HashMap<Seed, Vec<u8>> = HashMap::new();
        let range_start = initial_seed.saturating_sub(search_range as u32);
        let range_end = initial_seed.saturating_add(search_range as u32);
        let seeds_to_search: Vec<Seed> = (range_start..=range_end).collect();

        for seed in seeds_to_search {
            let mut _seed = seed;

            for _ in 0..roaming_num {
                _seed = self.rng_lc.next(_seed);
            }

            /*
              とりあえず10回先までの返答内容を表示
              10回で確定できるっぽい？
              この範囲を設定できるようにすることも視野
            */
            for _ in 0..10 {
                _seed = self.rng_lc.next(_seed);
                let rand = self.rng_analyzer.extract_rand(_seed);
                let response_type = (rand % 3) as u8;
                call_response_sequence_map
                    .entry(seed)
                    .or_insert_with(Vec::new)
                    .push(response_type);
            }
        }

        return call_response_sequence_map;
    }

    /*
      HashMap<Seed, (Vec<u8>, [bool; 3])>
      [(30,1),  [true,  false, true]]  -> ライコウ: 30 ラティ: 1
      [(1),     [false, false, true]]  -> ラティ: 1
      [(30,31), [true,  true,  false]] -> ライコウ: 30 エンテイ: 31

      犬　-> ラティの順で処理
      犬: 乱数 % 16 の値がジョウト道路の昇順に対応
      ラ: 乱数 % 25 の値がカントー道路の昇順に対応
    */
    pub fn create_roamers_location_map(
        &self,
        initial_seed: Seed,
        roaming: [bool; 3],
        search_range: u8,
    ) -> HashMap<Seed, (Vec<u8>, [bool; 3])> {
        let mut roamers_location_map: HashMap<Seed, (Vec<u8>, [bool; 3])> = HashMap::new();
        let range_start = initial_seed.saturating_sub(search_range as u32);
        let range_end = initial_seed.saturating_add(search_range as u32);
        let seeds_to_search: Vec<Seed> = (range_start..=range_end).collect();

        for seed in seeds_to_search {
            let mut next_seed = seed;

            for (i, _) in roaming.iter().enumerate() {
                if !roaming[i] {
                    continue;
                }

                next_seed = self.rng_lc.next(next_seed);
                let rand = self.rng_analyzer.extract_rand(next_seed);

                match i {
                    0 | 1 => {
                        let route_index = (rand % 16) as usize;
                        roamers_location_map
                            .entry(seed)
                            .or_insert_with(|| (Vec::new(), roaming))
                            .0
                            .push(roaming_routes::ROAMING_POKEMON_ROUTE_LIST_JOHTO[route_index]);
                    }

                    2 => {
                        let route_index = (rand % 25) as usize;
                        roamers_location_map
                            .entry(seed)
                            .or_insert_with(|| (Vec::new(), roaming))
                            .0
                            .push(roaming_routes::ROAMING_POKEMON_ROUTE_LIST_KANTO[route_index]);
                    }

                    _ => {}
                }
            }
        }

        return roamers_location_map;
    }

    /*
        DPPt専用。
        メルセンヌツイスタを使用しているため、野生乱数の場合は乱数消費に影響しない。
        ただし、孵化の場合は影響するので注意！
    */
    pub fn create_coin_flip_result_map(
        &self,
        initial_seed: Seed,
        search_range: u8,
    ) -> HashMap<Seed, Vec<bool>> {
        let mut coin_flip_result_map: HashMap<Seed, Vec<bool>> = HashMap::new();
        let range_start = initial_seed.saturating_sub(search_range as u32);
        let range_end = initial_seed.saturating_add(search_range as u32);
        let seeds_to_search: Vec<Seed> = (range_start..=range_end).collect();

        for seed in seeds_to_search {
            let mut mt = RngMT::new(seed);

            /*
                電話同様、一旦10回先の結果まで表示する
                範囲を指定できるようにもするかも
            */
            for _ in 0..10 {
                let next_seed = mt.next();
                let pid = mt.get_pid(next_seed);
                coin_flip_result_map
                    .entry(seed)
                    .or_insert_with(Vec::new)
                    .push(pid % 2 == 1);
            }
        }

        return coin_flip_result_map;
    }
}
