//! Known FreeRTOS API items, embedded from `freertos_api.json`
//!
//! Items are kept as plain strings, parsing into an
//! [Identifier](crate::parse::Identifier) is left to the caller.

use std::sync::LazyLock;

use crate::parse::Identifier;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ApiItem {
    /// The full identifier
    pub ident: String,
    /// Name of the type -- i.e. a variable's type or a function's return type -- if known.
    #[serde(rename = "type")]
    pub ty: Option<String>,
    /// Link to the documentation, if known
    pub docs: Option<String>,
}

/// All known API items
static API_ITEMS: LazyLock<Vec<ApiItem>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../freertos_api.json")).expect("freertos_api.json is valid")
});

/// Find the API item matching `id`
///
/// Matches on the formatted identifier, so e.g. `vTaskDelete` and
/// `vTaskDelete()` refer to the same item.
pub(crate) fn find(id: &Identifier) -> Option<&'static ApiItem> {
    let name = id.to_string();
    API_ITEMS.iter().find(|item| {
        item.ident
            .parse::<Identifier>()
            .is_ok_and(|item_id| item_id.to_string() == name)
    })
}

/// Search the API items matching `needle`, best matches first
///
/// Case insensitive, ranked by where `needle` matches:
/// 1. start of the identifier, e.g. `vtask` -> `vTaskDelete`
/// 2. start of the name (w/o prefix), e.g. `taskdel` -> `vTaskDelete`
/// 3. anywhere in the identifier, e.g. `delete` -> `vTaskDelete`
///
/// Ties are ordered alphabetically.
pub(crate) fn search(needle: &str) -> Vec<&'static ApiItem> {
    // Parens only mark a function, they're not part of the identifier
    let needle = needle.trim().trim_end_matches(['(', ')']).to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }

    let mut ranked: Vec<(u8, &'static ApiItem)> = API_ITEMS
        .iter()
        .filter_map(|item| {
            // Same split as the tokenizer: the prefix runs up to the first capital
            let name = item.ident.trim_start_matches(|c: char| !c.is_uppercase());
            let ident = item.ident.to_lowercase();
            if ident.starts_with(&needle) {
                Some((0, item))
            } else if name.to_lowercase().starts_with(&needle) {
                Some((1, item))
            } else if ident.contains(&needle) {
                Some((2, item))
            } else {
                None
            }
        })
        .collect();
    ranked.sort_by(|(ra, a), (rb, b)| ra.cmp(rb).then_with(|| a.ident.cmp(&b.ident)));
    ranked.into_iter().map(|(_, item)| item).collect()
}

#[cfg(test)]
mod test {
    //! Tests only rely on a few well-known items being present, never on the
    //! full contents or order of `freertos_api.json`, so they keep passing
    //! when it's expanded.

    use super::*;

    /// Identifiers of the search results for `needle`
    fn search_idents(needle: &str) -> Vec<&'static str> {
        search(needle).iter().map(|i| i.ident.as_str()).collect()
    }

    /// Index of `ident` in `idents`, panics if absent
    fn pos(idents: &[&str], ident: &str) -> usize {
        idents
            .iter()
            .position(|i| *i == ident)
            .unwrap_or_else(|| panic!("`{ident}` not in {idents:?}"))
    }

    #[test]
    fn search_ident_prefix() {
        let idents = search_idents("vtaskdel");
        pos(&idents, "vTaskDelay()");
        pos(&idents, "vTaskDelete()");
    }

    #[test]
    fn search_name_prefix() {
        // No Hungarian prefix typed
        let idents = search_idents("taskdel");
        pos(&idents, "vTaskDelete()");
        pos(&idents, "xTaskDelayUntil()");
    }

    #[test]
    fn search_only_returns_matches() {
        for needle in ["task", "delay", "isr", "xtask"] {
            for ident in search_idents(needle) {
                assert!(
                    ident.to_lowercase().contains(needle),
                    "`{ident}` doesn't match `{needle}`"
                );
            }
        }
    }

    #[test]
    fn search_ranks_prefix_over_substring() {
        let idents = search_idents("xtask");
        let substring = idents
            .iter()
            .position(|i| !i.to_lowercase().starts_with("xtask"))
            .unwrap_or(idents.len());

        // All identifier prefix matches come before the first substring match ...
        assert!(
            idents[substring..]
                .iter()
                .all(|i| !i.to_lowercase().starts_with("xtask"))
        );
        // ... and both kinds are present
        assert!(pos(&idents, "xTaskCreate()") < pos(&idents, "uxTaskPriorityGet()"));
    }

    #[test]
    fn search_sorts_ties_alphabetically() {
        let idents = search_idents("vtask"); // All identifier prefix matches
        assert!(idents.is_sorted(), "{idents:?} is not sorted");
    }

    #[test]
    fn search_ignores_case_and_parens() {
        // Shorter identifiers sort first, so an exact match always leads
        assert_eq!(search_idents("VTASKDELETE()").first(), Some(&"vTaskDelete()"));
    }

    #[test]
    fn search_empty_needle() {
        assert!(search("  ").is_empty());
        assert!(search("()").is_empty());
    }

    #[test]
    fn find_ignores_call_parens() {
        let with = find(&"vTaskDelete()".parse().unwrap()).unwrap();
        let without = find(&"vTaskDelete".parse().unwrap()).unwrap();
        assert_eq!(with.ident, "vTaskDelete()");
        assert_eq!(without.ident, "vTaskDelete()");
    }

    #[test]
    fn find_unknown_ident() {
        assert!(find(&"vNotAFreeRtosFunction()".parse().unwrap()).is_none());
    }

    #[test]
    fn api_items_load() {
        assert!(!API_ITEMS.is_empty());
    }

    #[test]
    fn api_items_are_found_by_their_ident() {
        // Every item is suggested by `search`, so it must also parse and be
        // found by `find`, or selecting it would show "Unknown identifier"
        let unfindable: Vec<&str> = API_ITEMS
            .iter()
            .filter(|item| {
                let found = item
                    .ident
                    .parse::<Identifier>()
                    .ok()
                    .and_then(|id| find(&id));
                !found.is_some_and(|found| std::ptr::eq(found, *item))
            })
            .map(|item| item.ident.as_str())
            .collect();
        assert!(unfindable.is_empty(), "can't look up {unfindable:?}");
    }

    #[test]
    fn api_items_contains_known_item() {
        let item = API_ITEMS.iter().find(|i| i.ident == "xTaskCreate()").unwrap();
        assert_eq!(item.ty.as_deref(), Some("BaseType_t"));
    }
}
