use wasm_bindgen::prelude::*;

mod api;
mod naming;

#[wasm_bindgen]
pub fn search(query: &str, min_len: usize) -> Vec<String> {
    let sanitized_query = query.trim().to_lowercase();
    let mut suggestions = vec![query.to_string()];
    if query.len() >= min_len {
        suggestions.extend(
            api::names_iter()
                .filter(|name| name.to_lowercase().starts_with(&sanitized_query))
                .map(|name| name.to_string()),
        );
    }
    suggestions
}

#[wasm_bindgen]
pub fn type_of(name: &str) -> String {
    match api::names_iter().find(|&n| n.eq_ignore_ascii_case(name)) {
        Some(_) => api::type_of(name.into()).unwrap(),
        None => naming::prefix_to_ty(name),
    }
}

#[wasm_bindgen]
pub fn doc_url(name: &str) -> Option<String> {
    api::doc_link(name.into())
}
