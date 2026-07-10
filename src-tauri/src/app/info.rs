use serde::Serialize;

/// Stable application metadata exposed to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub storage_scope: &'static str,
}

impl AppInfo {
    pub fn current() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            storage_scope: "local-only",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppInfo;

    #[test]
    fn reports_local_only_storage_scope() {
        assert_eq!(AppInfo::current().storage_scope, "local-only");
    }
}
