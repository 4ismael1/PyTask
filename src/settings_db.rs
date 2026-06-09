use std::fs;
use std::path::PathBuf;

use rusqlite::{Connection, Result, params};

use crate::model::Settings;

pub struct SettingsDatabase {
    conn: Connection,
}

impl SettingsDatabase {
    pub fn open_default() -> Result<Self> {
        let appdata = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let dir = appdata.join("PyTask");
        let _ = fs::create_dir_all(&dir);
        Self::open(dir.join("pytask.db"))
    }

    pub(crate) fn open(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn load_settings(&self) -> Settings {
        let defaults = Settings::default();
        Settings {
            speed_half: self.get_bool("speed_half", defaults.speed_half),
            speed_1x: self.get_bool("speed_1x", defaults.speed_1x),
            speed_2x: self.get_bool("speed_2x", defaults.speed_2x),
            speed_100x: self.get_bool("speed_100x", defaults.speed_100x),
            record_hotkey: normalized_hotkey(
                self.get_string("record_hotkey"),
                &["F6", "F7", "F8", "F9"],
                &defaults.record_hotkey,
            ),
            play_hotkey: normalized_hotkey(
                self.get_string("play_hotkey"),
                &["F5", "F10", "F11", "F12"],
                &defaults.play_hotkey,
            ),
            always_on_top: self.get_bool("always_on_top", defaults.always_on_top),
            show_captions: self.get_bool("show_captions", defaults.show_captions),
            use_sendinput: self.get_bool("use_sendinput", defaults.use_sendinput),
        }
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        self.set_bool("speed_half", settings.speed_half)?;
        self.set_bool("speed_1x", settings.speed_1x)?;
        self.set_bool("speed_2x", settings.speed_2x)?;
        self.set_bool("speed_100x", settings.speed_100x)?;
        self.set_string("record_hotkey", &settings.record_hotkey)?;
        self.set_string("play_hotkey", &settings.play_hotkey)?;
        self.set_bool("always_on_top", settings.always_on_top)?;
        self.set_bool("show_captions", settings.show_captions)?;
        self.set_bool("use_sendinput", settings.use_sendinput)?;
        Ok(())
    }

    fn get_string(&self, key: &str) -> Option<String> {
        self.conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .ok()
    }

    fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get_string(key)
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(default)
    }

    fn set_string(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    fn set_bool(&self, key: &str, value: bool) -> Result<()> {
        self.set_string(key, if value { "true" } else { "false" })
    }
}

fn normalized_hotkey(value: Option<String>, allowed: &[&str], default: &str) -> String {
    let Some(value) = value else {
        return default.to_string();
    };
    let normalized = value.trim().to_ascii_uppercase();
    if allowed.iter().any(|candidate| *candidate == normalized) {
        normalized
    } else {
        default.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsDatabase;
    use crate::model::Settings;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_defaults_when_database_is_empty() {
        let path = temp_path("defaults.db");
        let db = SettingsDatabase::open(path.clone()).unwrap();

        let settings = db.load_settings();
        drop(db);
        let _ = fs::remove_file(path);

        assert_eq!(settings.record_hotkey, "F9");
        assert_eq!(settings.play_hotkey, "F10");
        assert!(settings.show_captions);
        assert!(settings.use_sendinput);
    }

    #[test]
    fn persists_all_user_settings() {
        let path = temp_path("persist.db");
        let db = SettingsDatabase::open(path.clone()).unwrap();
        let settings = Settings {
            speed_half: false,
            speed_1x: true,
            speed_2x: false,
            speed_100x: true,
            record_hotkey: "F8".to_string(),
            play_hotkey: "F12".to_string(),
            always_on_top: true,
            show_captions: false,
            use_sendinput: false,
        };

        db.save_settings(&settings).unwrap();
        let loaded = db.load_settings();
        drop(db);
        let _ = fs::remove_file(path);

        assert!(!loaded.speed_half);
        assert!(loaded.speed_1x);
        assert!(!loaded.speed_2x);
        assert!(loaded.speed_100x);
        assert_eq!(loaded.record_hotkey, "F8");
        assert_eq!(loaded.play_hotkey, "F12");
        assert!(loaded.always_on_top);
        assert!(!loaded.show_captions);
        assert!(!loaded.use_sendinput);
    }

    #[test]
    fn normalizes_loaded_hotkeys_and_falls_back_for_invalid_values() {
        let path = temp_path("normalize-hotkeys.db");
        let db = SettingsDatabase::open(path.clone()).unwrap();

        db.set_string("record_hotkey", " f8 ").unwrap();
        db.set_string("play_hotkey", "f12").unwrap();
        let loaded = db.load_settings();
        assert_eq!(loaded.record_hotkey, "F8");
        assert_eq!(loaded.play_hotkey, "F12");

        db.set_string("record_hotkey", "F5").unwrap();
        db.set_string("play_hotkey", "A").unwrap();
        let loaded = db.load_settings();
        drop(db);
        let _ = fs::remove_file(path);

        assert_eq!(loaded.record_hotkey, "F9");
        assert_eq!(loaded.play_hotkey, "F10");
    }

    fn temp_path(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "pytask-rust-{}-{}-{}",
            std::process::id(),
            suffix,
            name
        ))
    }
}
