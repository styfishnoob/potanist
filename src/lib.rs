mod constants;
mod modules;
mod types;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn test() -> String {
    return "[WASM]: OK!".to_string();
}
