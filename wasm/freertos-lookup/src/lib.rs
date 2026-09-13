use wasm_bindgen::prelude::*;

use crate::parse::Identifier;

mod api;
mod ast;
mod parse;
mod tokens;

#[wasm_bindgen]
pub fn search(query: &str, min_len: usize) -> Vec<String> {
    let mut suggestions = vec![query.to_string()];
    if query.len() >= min_len {
        // TODO: needs an Identifier -> String formatter (the old `ident_str`)
        todo!("Identifier -> String formatting not yet implemented by the new parser");
    }
    suggestions
}

fn parse_if_not_known(query: &str) -> Option<Identifier> {
    api::idents()
        .iter()
        .find(|_i| {
            // TODO: needs an Identifier -> String formatter (the old `ident_str`)
            todo!("Identifier -> String formatting not yet implemented by the new parser")
        })
        .cloned()
        .or_else(|| query.trim().parse::<Identifier>().ok())
}

#[wasm_bindgen]
pub fn type_of(query: &str) -> String {
    let _ident = parse_if_not_known(query);
    // TODO: needs Identifier::ty() and a Type -> String formatter
    todo!("Identifier -> Type formatting not yet implemented by the new parser")
}

#[wasm_bindgen]
pub fn ident_of(query: &str) -> String {
    let _ident = parse_if_not_known(query);
    // TODO: needs an Identifier -> String formatter (the old `ident_str`)
    todo!("Identifier -> String formatting not yet implemented by the new parser")
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
