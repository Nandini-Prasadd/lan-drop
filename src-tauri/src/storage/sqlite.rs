use std::{error::Error, fmt, path::Path};

use rusqlite::{params, Connection, OptionalExtension};

const SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub device_name: String,
    pub download_directory: String,
    pub discovery_enabled: bool,
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
    UnsupportedSchema(i32),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "Local database error: {error}"),
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

    fn apply_migrations(&mut self) -> StorageResult<()> {
        let current_version = self.schema_version()?;
        if current_version > SCHEMA_VERSION {
            return Err(StorageError::UnsupportedSchema(current_version));
        }

        if current_version == SCHEMA_VERSION {
            return Ok(());
        }

        let transaction = self.connection.transaction()?;
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS app_settings (
                settings_key TEXT PRIMARY KEY NOT NULL,
                device_name TEXT NOT NULL,
                download_directory TEXT NOT NULL,
                discovery_enabled INTEGER NOT NULL CHECK(discovery_enabled IN (0, 1))
            );",
        )?;
        transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        transaction.commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{AppSettings, LocalStore, SCHEMA_VERSION};

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
}
