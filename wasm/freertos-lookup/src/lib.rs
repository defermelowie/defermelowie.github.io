use wasm_bindgen::prelude::*;

use crate::parse::Identifier;

// mod api;
mod parse;
mod tokens;

#[wasm_bindgen]
pub fn search(query: &str, min_len: usize) -> Vec<String> {
    // FixMe: currently does only echo the search query, should suggest existing fn names instead
    let mut suggestions = vec![query.to_string()];
    suggestions
}

/// Everything known about a looked up identifier
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

impl From<Identifier> for Info {
    fn from(id: Identifier) -> Self {
        Self {
            name: id.to_string(),
            kind: id.kind_str().to_string(),
            ty: id.ty().map(|ty| ty.to_string()),
            // ToDo: fill from the API items once `api` is re-enabled
            doc: None,
        }
    }
}

/// Look up an identifier, returns `None` if it can't be parsed
#[wasm_bindgen]
pub fn lookup(query: &str) -> Option<Info> {
    query.trim().parse::<Identifier>().ok().map(Info::from)
}
