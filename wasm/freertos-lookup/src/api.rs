use crate::{ast, parse};

use std::{borrow::Cow, sync::LazyLock};

#[derive(Clone)]
struct ApiItem {
    ident: ast::Ident,
    docs: Cow<'static, String>,
}

impl<'de> serde::Deserialize<'de> for ApiItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize the JSON object into a generic JSON value
        let value = serde_json::Value::deserialize(deserializer)?;

        // Extract the "ident" field
        let mut ident = value
            .get("ident")
            .and_then(|v| v.as_str())
            .and_then(|s| parse::parse_ident(s))
            .ok_or_else(|| serde::de::Error::custom("Missing 'ident' field"))?;

        // Fill custom return type if provided
        let name = value
            .get("returns")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(name) = name {
            ident.fill_custom_type(name.leak());
        }

        // Extract the "docs" field
        let docs = value
            .get("docs")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .map(|s| Cow::Owned(s))
            .ok_or_else(|| serde::de::Error::custom("Missing or invalid 'docs' field"))?;

        // Return the result
        Ok(ApiItem { ident, docs })
    }
}

const API_ITEMS: LazyLock<Vec<ApiItem>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../freertos_api.json")).unwrap_or_else(|err| {
        eprintln!("Failed to parse FreeRTOS API items: {}", err);
        Vec::new()
    })
});

pub fn idents() -> Vec<ast::Ident> {
    API_ITEMS.iter().map(|i| i.ident.clone()).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::*;

    #[test]
    fn idents_contains_first() {
        let idents = dbg!(idents());

        let mut expected = parse::parse_ident("xTaskCreate").unwrap();
        expected.fill_custom_type("BaseType_t");
        assert!(idents.contains(&expected));
    }
}
