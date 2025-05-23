#![allow(dead_code)]

use bincode;
use std::collections::HashMap;

type TimeSumMap = HashMap<u16, Vec<(u8, u8, u8, u8)>>;

pub fn build_time_sum_map() -> TimeSumMap {
    let mut map: TimeSumMap = HashMap::new();

    for month in 1..=12 {
        let max_day = match month {
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };

        for day in 1..=max_day {
            for min in 0..=59 {
                for sec in 0..=59 {
                    let time_sum = (month as u16) * (day as u16) + (min as u16) + (sec as u16);
                    let candidates = map.entry(time_sum).or_insert_with(Vec::new);
                    candidates.push((month, day, min, sec));
                }
            }
        }
    }

    return map;
}

pub fn save_bincode(map: &TimeSumMap, path: &str) {
    let config = bincode::config::standard();
    let encoded = bincode::encode_to_vec(map, config).expect("Failed to encode bincode");
    std::fs::write(path, encoded).expect("Failed to write bincode");
}

pub fn load_bincode(path: &str) -> TimeSumMap {
    let config = bincode::config::standard();
    let encoded = std::fs::read(path).expect("Failed to read bincode");
    let (decoded, _): (TimeSumMap, _) =
        bincode::decode_from_slice(&encoded, config).expect("Failed to decode");
    return decoded;
}
