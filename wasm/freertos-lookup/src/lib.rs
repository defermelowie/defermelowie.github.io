use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn search(name: &str) {
    alert(&format!("Hello, {}!", name));
}
