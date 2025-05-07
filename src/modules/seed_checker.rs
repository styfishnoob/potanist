use crate::constants::roaming_pokemon_routes;
use crate::modules::lcrng::LCRng;
use crate::types::types::*;
use std::collections::HashMap;

pub struct SeedChecker {
    lcrng: LCRng,
}

impl SeedChecker {
    pub fn new() -> Self {
        Self { lcrng: LCRng::new() }
    }

    pub fn create_call_response_sequence_map(
        &self,
        seed: Seed,
        roaming_num: u8,
        search_range: u8,
    ) -> HashMap<Seed, Vec<u8>> {
        let mut call_response_sequence_list: HashMap<Seed, Vec<u8>> = HashMap::new();
        let range_start = seed.saturating_sub(search_range as u32);
        let range_end = seed.saturating_add(search_range as u32);
        let seeds_to_search: Vec<Seed> = (range_start..=range_end).collect();

        for seed in seeds_to_search {
            let mut _seed = seed;

            for _ in 0..roaming_num {
                _seed = self.lcrng.next(_seed);
            }

            /*
              とりあえず10回先までの返答内容を表示
              10回で確定できるっぽい？
              この範囲を設定できるようにすることも視野
            */
            for _ in 0..10 {
                _seed = self.lcrng.next(_seed);
                let rand = self.lcrng.extract_rand(_seed);
                let response_type = (rand % 3) as u8;
                call_response_sequence_list
                    .entry(seed)
                    .or_insert_with(Vec::new)
                    .push(response_type);
            }
        }

        return call_response_sequence_list;
    }

    /*
      HashMap<Seed, (Vec<u8>, [bool; 3])>
      [(30,1),  [true,  false, true]] -> ライコウ: 30 ラティ: 1
      [(1),     [false, false, true]] -> ラティ: 1
      [(30,31), [true,  true,  false]] -> ライコウ: 30 エンテイ: 31

      犬　-> ラティの順で処理
      犬: 乱数 % 16 の値がジョウト道路の昇順に対応
      ラ: 乱数 % 25 の値がカントー道路の昇順に対応
    */
    pub fn create_roamers_location_map(
        &self,
        seed: Seed,
        roaming: [bool; 3],
        search_range: u8,
    ) -> HashMap<Seed, (Vec<u8>, [bool; 3])> {
        let mut roamers_location_map: HashMap<Seed, (Vec<u8>, [bool; 3])> = HashMap::new();
        let range_start = seed.saturating_sub(search_range as u32);
        let range_end = seed.saturating_add(search_range as u32);
        let seeds_to_search: Vec<Seed> = (range_start..=range_end).collect();

        for seed in seeds_to_search {
            let mut next_seed = seed;

            for (i, _) in roaming.iter().enumerate() {
                if !roaming[i] {
                    continue;
                }

                next_seed = self.lcrng.next(next_seed);
                let rand = self.lcrng.extract_rand(next_seed);

                match i {
                    0 | 1 => {
                        let route_index = (rand % 16) as usize;
                        roamers_location_map
                            .entry(seed)
                            .or_insert_with(|| (Vec::new(), roaming))
                            .0
                            .push(roaming_pokemon_routes::ROAMING_POKEMON_ROUTE_LIST_JOHTO[route_index]);
                    }

                    2 => {
                        let route_index = (rand % 25) as usize;
                        roamers_location_map
                            .entry(seed)
                            .or_insert_with(|| (Vec::new(), roaming))
                            .0
                            .push(roaming_pokemon_routes::ROAMING_POKEMON_ROUTE_LIST_KANTO[route_index]);
                    }

                    _ => {}
                }
            }
        }

        return roamers_location_map;
    }

    pub fn create_coin_flip_result_list() {}
}

fn display_roamers_location(roamers_location_map: HashMap<Seed, (Vec<u8>, [bool; 3])>) {
    let mut sorted_roamers_location_list: Vec<_> = roamers_location_map.iter().collect();
    sorted_roamers_location_list.sort_by_key(|&(k, _)| k);

    for (seed, value) in sorted_roamers_location_list {
        print!("{:0x}: ", seed);

        let true_indicies: Vec<usize> = value
            .1
            .iter()
            .enumerate()
            .filter_map(|(i, &b)| if b { Some(i) } else { None })
            .collect();

        for i in true_indicies {
            match i {
                0 => print!("ライコウ: {:2} ", value.0[0]),
                1 => print!("エンテイ: {:2} ", value.0[1]),
                2 => print!("ラティ:  {:2} ", value.0[2]),
                _ => {}
            }
        }

        println!("")
    }
}

fn display_call_response_sequence(call_response_sequence_map: HashMap<Seed, Vec<u8>>, person: u8) {
    let mut sorted_call_response_sequence_map: Vec<_> = call_response_sequence_map.iter().collect();
    sorted_call_response_sequence_map.sort_by_key(|&(k, _)| k);

    const ELM_RESPONSE_TYPE: [&str; 3] = ["し", "カ", "菌"];
    const IRWIN_RESPONSE_TYPE: [&str; 3] = ["0", "1", "2"];

    for (seed, vec) in sorted_call_response_sequence_map {
        print!("{:0x}: ", seed);

        for v in vec {
            match person {
                0 => print!("{}", ELM_RESPONSE_TYPE[*v as usize]),
                _ => {}
            }
            print!("{} ", v);
        }

        println!("")
    }
}
