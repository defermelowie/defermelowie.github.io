use crate::naming::{Base, NonStdInt, Sign, Type};

/// Get an iterator of all (known) FreeRTOS API function names
pub fn names_iter() -> impl Iterator<Item = &'static str> {
    API_ITEMS.iter().map(|i| i.name)
}

/// Retrieve an API item based on its name
fn find_by_name(name: String) -> Option<ApiItem> {
    API_ITEMS.iter().find(|&i| i.name == name).cloned()
}

/// Get documentation url for the given item
pub fn doc_link(name: String) -> Option<String> {
    find_by_name(name).map(|i| i.doc_link())
}

pub fn type_of(name: String) -> Option<String> {
    find_by_name(name).map(|i| i.return_ty().to_string())
}

#[derive(Clone)]
struct ApiItem {
    name: &'static str,
    doc: &'static str,
    ty: Type,
}

impl ApiItem {
    const DOC_BASE_URL: &'static str = "https://www.freertos.org/Documentation";

    /// Get the name of this API item
    const fn name(&self) -> &'static str {
        self.name
    }

    /// Get the documentation of this API item
    fn doc_link(&self) -> String {
        format!(
            "{}/{}",
            Self::DOC_BASE_URL.trim_end_matches('/'),
            self.doc.trim_start_matches('/')
        )
    }

    /// Get the return type of this API item
    fn return_ty(&self) -> Type {
        self.ty.clone()
    }
}

const API_ITEMS: [ApiItem; 5] = [
    // Task creation
    ApiItem {
        name: "xTaskCreate",
        doc: "02-Kernel/04-API-references/01-Task-creation/01-xTaskCreate",
        ty: NonStdInt::BaseType_t(),
    },
    ApiItem {
        name: "xTaskCreateStatic",
        doc: "02-Kernel/04-API-references/01-Task-creation/02-xTaskCreateStatic",
        ty: NonStdInt::TaskHandle_t(),
    },
    ApiItem {
        name: "vTaskDelete",
        doc: "02-Kernel/04-API-references/01-Task-creation/03-vTaskDelete",
        ty: Type::Value(Sign::Signed(Base::Void)),
    },
    // Task control
    ApiItem {
        name: "vTaskDelay",
        doc: "02-Kernel/04-API-references/02-Task-control/01-vTaskDelay",
        ty: Type::Value(Sign::Signed(Base::Void)),
    },
    ApiItem {
        name: "xTaskDelayUntil",
        doc: "02-Kernel/04-API-references/02-Task-control/02-xTaskDelayUntil",
        ty: NonStdInt::BaseType_t(),
    },
];
