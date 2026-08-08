use wasm_bindgen::prelude::*;

use crate::ast::Ident;

mod api;
mod ast;
mod parse;

#[wasm_bindgen]
pub fn search(query: &str, min_len: usize) -> Vec<String> {
    let sanitized_query = query.trim().to_lowercase();
    let mut suggestions = vec![query.to_string()];
    if query.len() >= min_len {
        suggestions.extend(
            api::idents()
                .iter()
                .map(|i| i.ident_str())
                .filter(|s| s.to_lowercase().starts_with(&sanitized_query)) // Filter out different prefixes
                .filter(|s| s.ne(query)) // Filter out exact match
                .map(|s| s.to_string()),
        );
    }
    suggestions
}

fn parse_if_not_known(query: &str) -> Option<Ident> {
    api::idents()
        .iter()
        .find(|i| i.ident_str() == query)
        .cloned()
        .or_else(|| parse::parse_ident(query.trim()))
}

#[wasm_bindgen]
pub fn type_of(query: &str) -> String {
    let ident = parse_if_not_known(query);
    let ty = ident.and_then(|i| i.ty());
    ty.map(|t| t.to_string()).unwrap_or("unknown".to_string())
}

#[wasm_bindgen]
pub fn ident_of(query: &str) -> String {
    let ident = parse_if_not_known(query);
    ident
        .map(|i| i.ident_str())
        .unwrap_or("parsing failed".to_string())
}

#[wasm_bindgen]
pub fn kind_of(query: &str) -> String {
    let ident = parse_if_not_known(query);
    ident
        .map(|i| i.kind_str().to_string())
        .unwrap_or("parsing failed".to_string())
}

#[wasm_bindgen]
pub fn doc_url(query: &str) -> Option<String> {
    None
}
