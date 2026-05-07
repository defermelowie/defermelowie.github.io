use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn search(query: &str) -> Vec<String> {
    // For now, return a static list of suggestions
    // In a real application, you would implement a more sophisticated suggestion system
    let suggestions = vec![
        format!("Suggestion 1 for {}", query),
        format!("Suggestion 2 for {}", query),
        format!("Suggestion 3 for {}", query),
    ];
    suggestions
}
