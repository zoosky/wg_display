//! Implementation of the system configuration persistence
use common::models::SystemConfiguration;
use rocket::serde::json::serde_json;

use std::sync::atomic::{AtomicBool, Ordering};

static DB_NAME: &str = "wg_display.db";
lazy_static! {
    static ref DB: sled::Db = sled::open(DB_NAME).expect("Could not open DB");
}

static CONFIG_UPDATED: AtomicBool = AtomicBool::new(false);

pub struct Persistence {}

/// Persists the system configuration.
/// Uses the SLED embedded database.
impl Persistence {
    const DB_KEY: &str = "system_configuration";

    /// Save the system configuration
    /// # Arguments
    /// * `config` - The system configuration to save
    pub fn save_config(config: SystemConfiguration) {
        let serialized = serde_json::to_string(&config).expect("Could not serialize config");
        DB.insert(Persistence::DB_KEY, serialized.as_bytes())
            .expect("Could not save configuration");
        CONFIG_UPDATED.store(true, Ordering::Relaxed);
    }

    /// Load the system configuration
    /// # Returns
    /// The system configuration
    pub fn get_config() -> Option<SystemConfiguration> {
        let config = DB
            .get(Persistence::DB_KEY)
            .expect("FATAL: Could not read DB");
        match config {
            Some(bytes) => {
                let config_str = String::from_utf8(bytes.to_vec())
                    .expect("Could not convert config bytes to string");
                Some(
                    serde_json::from_str(&config_str).expect("Could not deserialize configuration"),
                )
            }
            _ => {
                Persistence::create_default_config();
                Persistence::get_config()
            }
        }
    }

    /// Returns Some system configuration if a new one is available
    /// Can be used for polling updates to the system configuration
    pub fn get_config_change() -> Option<SystemConfiguration> {
        if CONFIG_UPDATED.load(Ordering::Relaxed) {
            CONFIG_UPDATED.store(false, Ordering::Relaxed);
            Some(Persistence::get_config().expect("Could not load config"))
        } else {
            None
        }
    }

    /// Create a default system configuration
    /// This is used on systems that never stored a configuration before
    fn create_default_config() {
        Persistence::save_config(SystemConfiguration::default());
    }
}

#[cfg(test)]
mod tests {
    use common::models::{BaseWidgetConfig, WidgetConfiguration};

    use super::*;

    #[test]
    fn test_persistence() {
        let config = SystemConfiguration {
            background_color: "#FF3A3A".to_string(),
            widget_config: WidgetConfiguration {
                bernaqua_config: BaseWidgetConfig { enabled: true },
                ..Default::default()
            },
        };
        Persistence::save_config(config.clone());
        let read_config = Persistence::get_config();
        assert!(read_config.is_some());
        assert_eq!(config, read_config.unwrap());
    }
}
