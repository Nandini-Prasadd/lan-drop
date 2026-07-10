mod app;
pub mod discovery;
pub mod domain;
pub mod pairing;
pub mod storage;

use std::{
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use app::info::AppInfo;
use discovery::{DiscoveryService, ManualPeerAddress};
use domain::transfer::{MetadataValidationResult, TransferMetadata};
use qrcode::{render::svg, QrCode};
use serde::Serialize;
use storage::sqlite::LocalStore;
use tauri::{Manager, State};

struct AppState {
    store: Mutex<LocalStore>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PairingInvitation {
    code: String,
    expires_at_epoch_seconds: u64,
    device_name: String,
    public_key: String,
    qr_svg: String,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo::current()
}

#[tauri::command]
fn validate_transfer_metadata(metadata: TransferMetadata) -> MetadataValidationResult {
    MetadataValidationResult::from_metadata(&metadata)
}

#[tauri::command]
fn create_pairing_invitation(state: State<AppState>) -> Result<PairingInvitation, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "Local storage is unavailable.".to_owned())?;
    let identity = store
        .load_or_create_device_identity()
        .map_err(|error| error.to_string())?;
    let settings = store.get_settings().map_err(|error| error.to_string())?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock is invalid.".to_owned())?
        .as_secs();
    let code = pairing::short_code::ShortPairingCode::generate(now, 300);
    let public_key = identity
        .public_key()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let payload = serde_json::json!({
        "version": 1,
        "code": code.value,
        "expiresAtEpochSeconds": code.expires_at_epoch_seconds,
        "deviceName": settings.device_name.clone(),
        "publicKey": public_key.clone(),
    });
    let qr_svg = QrCode::new(payload.to_string())
        .map_err(|error| format!("Could not create a pairing QR code: {error}"))?
        .render::<svg::Color>()
        .min_dimensions(256, 256)
        .build();

    Ok(PairingInvitation {
        code: payload["code"].as_str().unwrap_or_default().to_owned(),
        expires_at_epoch_seconds: code.expires_at_epoch_seconds,
        device_name: settings.device_name,
        public_key,
        qr_svg,
    })
}

#[tauri::command]
fn validate_manual_peer_address(address: String) -> Result<ManualPeerAddress, String> {
    ManualPeerAddress::parse(&address).map_err(|error| error.to_string())
}

#[tauri::command]
fn advertise_local_peer(state: State<AppState>) -> Result<(), String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "Local storage is unavailable.".to_owned())?;
    let identity = store
        .load_or_create_device_identity()
        .map_err(|error| error.to_string())?;
    let settings = store.get_settings().map_err(|error| error.to_string())?;
    DiscoveryService::advertise(&settings.device_name, &identity.public_key())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn discover_local_peers() -> Result<Vec<discovery::DiscoveredPeer>, String> {
    DiscoveryService::listen_once().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_directory)?;
            let store = LocalStore::open(data_directory.join("lan-drop.sqlite"))?;
            app.manage(AppState {
                store: Mutex::new(store),
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app_info,
            validate_transfer_metadata,
            create_pairing_invitation,
            validate_manual_peer_address,
            advertise_local_peer,
            discover_local_peers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
