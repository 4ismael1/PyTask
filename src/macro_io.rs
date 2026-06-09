use std::fs;
use std::io;
use std::path::Path;

use crate::model::{MacroEvent, MacroFile};

pub fn load_macro(path: &Path) -> io::Result<Vec<MacroEvent>> {
    let json = fs::read_to_string(path)?;
    let file = serde_json::from_str::<MacroFile>(&json)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(file.events)
}

pub fn save_macro(path: &Path, events: &[MacroEvent]) -> io::Result<()> {
    let file = MacroFile {
        events: events.to_vec(),
    };
    let json = serde_json::to_string_pretty(&file)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::{load_macro, save_macro};
    use crate::model::MacroEvent;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn saves_and_loads_macro_file() {
        let path = temp_path("roundtrip.macro");
        let events = vec![
            MacroEvent {
                event_type: "mouse_move".to_string(),
                timestamp: 0.0,
                x: Some(120),
                y: Some(240),
                ..Default::default()
            },
            MacroEvent {
                event_type: "key_press".to_string(),
                timestamp: 0.15,
                key: Some("F9".to_string()),
                vk_code: Some(120),
                ..Default::default()
            },
        ];

        save_macro(&path, &events).unwrap();
        let loaded = load_macro(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].event_type, "mouse_move");
        assert_eq!(loaded[0].x, Some(120));
        assert_eq!(loaded[1].vk_code, Some(120));
    }

    #[test]
    fn rejects_invalid_macro_json() {
        let path = temp_path("invalid.macro");
        fs::write(&path, "{not-json").unwrap();

        let err = load_macro(&path).unwrap_err();
        let _ = fs::remove_file(&path);

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn loads_manual_validation_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("open-save-smoke.macro");

        let events = load_macro(&path).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "mouse_move");
        assert_eq!(events[0].x, Some(100));
        assert_eq!(events[0].y, Some(100));
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
