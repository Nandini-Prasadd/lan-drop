mod app;
pub mod domain;
pub mod storage;

use app::info::AppInfo;
use domain::transfer::{MetadataValidationResult, TransferMetadata};

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo::current()
}

#[tauri::command]
fn validate_transfer_metadata(metadata: TransferMetadata) -> MetadataValidationResult {
    MetadataValidationResult::from_metadata(&metadata)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app_info,
            validate_transfer_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
