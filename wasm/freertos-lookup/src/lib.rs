use wasm_bindgen::prelude::*;

fn get_api_names() -> impl Iterator<Item = &'static str> {
    [
        "xTaskCreate",
        "vTaskDelay",
        "vTaskDelayUntil",
        "uxTaskPriorityGet",
        "uxTaskPriorityGetFromISR",
        "vTaskSuspend",
    ]
    .into_iter()
}

#[wasm_bindgen]
pub fn search(query: &str, min_len: usize) -> Vec<String> {
    let sanitized_query = query.trim().to_lowercase();
    let mut suggestions = vec![query.to_string()];
    if query.len() >= min_len {
        suggestions.extend(
            get_api_names()
                .filter(|name| name.to_lowercase().starts_with(&sanitized_query))
                .map(|name| name.to_string()),
        );
    }
    suggestions
}
