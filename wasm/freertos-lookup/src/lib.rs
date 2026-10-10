use wasm_bindgen::prelude::*;

use crate::api::ApiItem;
use crate::parse::Identifier;

mod api;
mod parse;
mod tokens;

/// Suggest known identifiers for `query`
///
/// Returns `query` itself, followed by at most `max_suggestions` known
/// identifiers once `query` is at least `min_len` long.
#[wasm_bindgen]
pub fn search(query: &str, min_len: usize, max_suggestions: usize) -> Vec<String> {
    // Echo the query itself as the first suggestion
    let mut suggestions = vec![query.to_string()];
    if query.trim().len() >= min_len {
        suggestions.extend(
            api::search(query)
                .into_iter()
                .map(|item| item.ident.clone())
                .filter(|ident| ident != query.trim()) // Filter out exact match
                .take(max_suggestions),
        );
    }
    suggestions
}

/// Everything known about a looked up identifier
///
/// NOTE: optional fields that are `None` read as `undefined` in JS, so check
/// for that before using them.
#[wasm_bindgen(getter_with_clone)]
pub struct Info {
    /// The full identifier
    pub name: String,
    /// Kind of the identifier -- i.e. variable, function or macro
    pub kind: String,
    /// The (return) type, `None` for macros and privileged functions
    pub ty: Option<String>,
    /// Link to the documentation, if known
    pub doc: Option<String>,
}

impl Info {
    /// Combine a parsed identifier with what the API items know about it
    fn new(id: &Identifier, item: Option<&ApiItem>) -> Self {
        Self {
            name: id.to_string(),
            kind: id.kind_str().to_string(),
            // Prefer the documented return type, it may be a better alias
            // (e.g. `BaseType_t`) than what the prefix tells
            ty: item
                .and_then(|i| i.ty.clone())
                .or_else(|| id.ty().map(|ty| ty.to_string())),
            doc: item.and_then(|i| i.docs.clone()),
        }
    }
}

/// Look up an identifier, returns `None` if it can't be parsed
#[wasm_bindgen]
pub fn lookup(query: &str) -> Option<Info> {
    let id = query.trim().parse::<Identifier>().ok()?;
    Some(Info::new(&id, api::find(&id)))
}
