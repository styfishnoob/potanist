use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

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

pub fn save_json(map: &TimeSumMap, path: &str) {
    let file = File::create(path).expect("Failed to create JSON file");
    serde_json::to_writer_pretty(file, map).expect("Failed to write JSON");
}

pub fn save_bincode(map: &TimeSumMap, path: &str) {
    let encoded = bincode::serialize(map).expect("Failed to encode");
    std::fs::write(path, encoded).expect("Failed to write bincode");
}

pub fn load_json(path: &str) -> TimeSumMap {
    let file = File::open(path).expect("Failed to open JSON file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Failed to parse JSON")
}

pub fn load_bincode(path: &str) -> TimeSumMap {
    let encoded = std::fs::read(path).expect("Failed to read bincode");
    bincode::deserialize(&encoded).expect("Failed to decode")
}
