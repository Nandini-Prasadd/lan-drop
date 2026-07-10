use std::{error::Error, fmt, path::Path};

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::transfer::TransferMetadata;
use crate::pairing::identity::DeviceIdentity;

const SCHEMA_VERSION: i32 = 3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub device_name: String,
    pub download_directory: String,
    pub discovery_enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransferHistoryRecord {
    pub metadata: TransferMetadata,
    pub failure_message: Option<String>,
    pub recorded_at: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            device_name: "lan-drop".into(),
            download_directory: String::new(),
            discovery_enabled: true,
        }
    }
}

#[derive(Debug)]
pub enum StorageError {
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    InvalidIdentityKey,
    InvalidTransferMetadata(String),
    UnsupportedSchema(i32),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "Local database error: {error}"),
            Self::Serialization(error) => {
                write!(formatter, "Local data could not be decoded: {error}")
            }
            Self::InvalidIdentityKey => write!(formatter, "Stored device identity is invalid."),
            Self::InvalidTransferMetadata(message) => {
                write!(formatter, "Invalid transfer metadata: {message}")
            }
            Self::UnsupportedSchema(version) => {
                write!(
                    formatter,
                    "This database uses unsupported schema version {version}."
                )
            }
        }
    }
}

impl Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

pub type StorageResult<T> = Result<T, StorageError>;

pub struct LocalStore {
    connection: Connection,
}

impl LocalStore {
    pub fn open(path: impl AsRef<Path>) -> StorageResult<Self> {
        let mut store = Self {
            connection: Connection::open(path)?,
        };
        store.apply_migrations()?;
        Ok(store)
    }

    pub fn open_in_memory() -> StorageResult<Self> {
        let mut store = Self {
            connection: Connection::open_in_memory()?,
        };
        store.apply_migrations()?;
        Ok(store)
    }

    pub fn schema_version(&self) -> StorageResult<i32> {
        Ok(self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))?)
    }

    pub fn get_settings(&self) -> StorageResult<AppSettings> {
        let settings = self
            .connection
            .query_row(
                "SELECT device_name, download_directory, discovery_enabled FROM app_settings WHERE settings_key = 'primary'",
                [],
                |row| {
                    Ok(AppSettings {
                        device_name: row.get(0)?,
                        download_directory: row.get(1)?,
                        discovery_enabled: row.get::<_, i64>(2)? != 0,
                    })
                },
            )
            .optional()?;

        Ok(settings.unwrap_or_default())
    }

    pub fn save_settings(&self, settings: &AppSettings) -> StorageResult<()> {
        self.connection.execute(
            "INSERT INTO app_settings (settings_key, device_name, download_directory, discovery_enabled)
             VALUES ('primary', ?1, ?2, ?3)
             ON CONFLICT(settings_key) DO UPDATE SET
               device_name = excluded.device_name,
               download_directory = excluded.download_directory,
               discovery_enabled = excluded.discovery_enabled",
            params![
                settings.device_name,
                settings.download_directory,
                i64::from(settings.discovery_enabled),
            ],
        )?;

        Ok(())
    }

    pub fn record_transfer(&self, record: &TransferHistoryRecord) -> StorageResult<()> {
        record
            .metadata
            .validate()
            .map_err(|error| StorageError::InvalidTransferMetadata(error.message))?;

        self.connection.execute(
            "INSERT INTO transfer_history (transfer_id, metadata_json, failure_message, recorded_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(transfer_id) DO UPDATE SET
               metadata_json = excluded.metadata_json,
               failure_message = excluded.failure_message,
               recorded_at = excluded.recorded_at",
            params![
                record.metadata.id,
                serde_json::to_string(&record.metadata)?,
                record.failure_message,
                record.recorded_at,
            ],
        )?;

        Ok(())
    }

    pub fn list_transfer_history(&self) -> StorageResult<Vec<TransferHistoryRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT metadata_json, failure_message, recorded_at
             FROM transfer_history
             ORDER BY recorded_at DESC, transfer_id DESC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let raw_records = rows.collect::<rusqlite::Result<Vec<_>>>()?;

        raw_records
            .into_iter()
            .map(|(metadata_json, failure_message, recorded_at)| {
                Ok(TransferHistoryRecord {
                    metadata: serde_json::from_str(&metadata_json)?,
                    failure_message,
                    recorded_at,
                })
            })
            .collect()
    }

    pub fn load_or_create_device_identity(&self) -> StorageResult<DeviceIdentity> {
        let stored_key = self
            .connection
            .query_row(
                "SELECT secret_key FROM device_identity WHERE identity_key = 'primary'",
                [],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?;

        if let Some(stored_key) = stored_key {
            let secret_key: [u8; 32] = stored_key
                .try_into()
                .map_err(|_| StorageError::InvalidIdentityKey)?;
            return Ok(DeviceIdentity::from_secret_key(secret_key));
        }

        let identity = DeviceIdentity::generate();
        self.connection.execute(
            "INSERT INTO device_identity (identity_key, secret_key) VALUES ('primary', ?1)",
            params![identity.secret_key_bytes().to_vec()],
        )?;

        Ok(identity)
    }

    fn apply_migrations(&mut self) -> StorageResult<()> {
        let current_version = self.schema_version()?;
        if current_version > SCHEMA_VERSION {
            return Err(StorageError::UnsupportedSchema(current_version));
        }

        if current_version == SCHEMA_VERSION {
            return Ok(());
        }

        let transaction = self.connection.transaction()?;
        if current_version < 1 {
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS app_settings (
                settings_key TEXT PRIMARY KEY NOT NULL,
                device_name TEXT NOT NULL,
                download_directory TEXT NOT NULL,
                discovery_enabled INTEGER NOT NULL CHECK(discovery_enabled IN (0, 1))
            );",
            )?;
            transaction.pragma_update(None, "user_version", 1)?;
        }

        if current_version < 2 {
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS transfer_history (
                    transfer_id TEXT PRIMARY KEY NOT NULL,
                    metadata_json TEXT NOT NULL,
                    failure_message TEXT,
                    recorded_at TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS transfer_history_recorded_at
                    ON transfer_history(recorded_at DESC);",
            )?;
            transaction.pragma_update(None, "user_version", 2)?;
        }

        if current_version < 3 {
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS device_identity (
                    identity_key TEXT PRIMARY KEY NOT NULL,
                    secret_key BLOB NOT NULL
                );",
            )?;
            transaction.pragma_update(None, "user_version", 3)?;
        }
        transaction.commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::transfer::{
        FileMetadata, PeerMetadata, TransferDirection, TransferMetadata, TransferState,
    };

    use super::{AppSettings, LocalStore, StorageError, TransferHistoryRecord, SCHEMA_VERSION};

    fn transfer_record(id: &str, recorded_at: &str) -> TransferHistoryRecord {
        TransferHistoryRecord {
            metadata: TransferMetadata {
                id: id.into(),
                direction: TransferDirection::Outgoing,
                peer: PeerMetadata {
                    id: "peer-01".into(),
                    display_name: "Nandini's laptop".into(),
                },
                file: FileMetadata {
                    name: "notes.pdf".into(),
                    size_bytes: 1_024,
                    sha256: Some("a".repeat(64)),
                },
                state: TransferState::Completed,
                created_at: recorded_at.into(),
            },
            failure_message: None,
            recorded_at: recorded_at.into(),
        }
    }

    #[test]
    fn initializes_the_current_schema() {
        let store = LocalStore::open_in_memory().unwrap();

        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn uses_safe_default_settings_before_the_first_save() {
        let store = LocalStore::open_in_memory().unwrap();

        assert_eq!(store.get_settings().unwrap(), AppSettings::default());
    }

    #[test]
    fn persists_each_settings_field_locally() {
        let store = LocalStore::open_in_memory().unwrap();
        let settings = AppSettings {
            device_name: "Office laptop".into(),
            download_directory: "D:/Private Downloads".into(),
            discovery_enabled: false,
        };

        store.save_settings(&settings).unwrap();

        assert_eq!(store.get_settings().unwrap(), settings);
    }

    #[test]
    fn round_trips_history_records_in_reverse_chronological_order() {
        let store = LocalStore::open_in_memory().unwrap();
        let earlier = transfer_record("transfer-01", "2026-07-10T10:00:00Z");
        let later = transfer_record("transfer-02", "2026-07-10T10:01:00Z");

        store.record_transfer(&earlier).unwrap();
        store.record_transfer(&later).unwrap();

        assert_eq!(store.list_transfer_history().unwrap(), vec![later, earlier]);
    }

    #[test]
    fn rejects_history_records_with_unsafe_metadata() {
        let store = LocalStore::open_in_memory().unwrap();
        let mut record = transfer_record("transfer-01", "2026-07-10T10:00:00Z");
        record.metadata.file.name = "../secret.txt".into();

        assert!(matches!(
            store.record_transfer(&record),
            Err(StorageError::InvalidTransferMetadata(_))
        ));
    }

    #[test]
    fn retains_a_single_local_device_identity() {
        let store = LocalStore::open_in_memory().unwrap();
        let first = store.load_or_create_device_identity().unwrap();
        let second = store.load_or_create_device_identity().unwrap();

        assert_eq!(first.public_key(), second.public_key());
    }
}
