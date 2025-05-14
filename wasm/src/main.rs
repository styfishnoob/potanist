mod constants;
mod modules;
mod types;

use crate::constants::time_sum_map;
use crate::modules::seed_analyzer::SeedAnalyzer;
use crate::modules::seed_checker::SeedChecker;
use crate::modules::seed_finder::SeedFinder;
use crate::types::pokemon_ivs::IVRanges;
use crate::types::types::*;

fn main() {
    // let potanist = Potanist::new();
    let seed_finder = SeedFinder::new();
    let seed_checker = SeedChecker::new();
    let seed_analyzer = SeedAnalyzer::new();

    let iv_ranges = IVRanges {
        hp: 31..=31,
        attack: 31..=31,
        defense: 31..=31,
        speed: 31..=31,
        sp_attack: 31..=31,
        sp_defense: 31..=31,
    };

    let matched_iv_1st_seed_list = seed_finder.find_seed_from_iv_ranges(iv_ranges);

    for iv_1st_seed in matched_iv_1st_seed_list {
        // seed_analyzer.display_pokemon_data_from_iv_1st_seed(iv_1st_seed);
    }
}
