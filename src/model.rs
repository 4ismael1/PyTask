use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MacroEvent {
    #[serde(rename = "type", alias = "Type")]
    pub event_type: String,
    #[serde(alias = "Timestamp")]
    pub timestamp: f64,
    #[serde(alias = "X")]
    pub x: Option<i32>,
    #[serde(alias = "Y")]
    pub y: Option<i32>,
    #[serde(alias = "Button")]
    pub button: Option<String>,
    #[serde(alias = "Pressed")]
    pub pressed: Option<bool>,
    #[serde(alias = "Dx")]
    pub dx: Option<i32>,
    #[serde(alias = "Dy")]
    pub dy: Option<i32>,
    #[serde(alias = "Key")]
    pub key: Option<String>,
    #[serde(rename = "vkCode", alias = "VkCode", alias = "vk_code")]
    pub vk_code: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MacroFile {
    #[serde(alias = "Events")]
    pub events: Vec<MacroEvent>,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub speed_half: bool,
    pub speed_1x: bool,
    pub speed_2x: bool,
    pub speed_100x: bool,
    pub record_hotkey: String,
    pub play_hotkey: String,
    pub always_on_top: bool,
    pub show_captions: bool,
    pub use_sendinput: bool,
}

#[cfg(test)]
mod tests {
    use super::{MacroEvent, MacroFile};

    #[test]
    fn macro_file_round_trips_csharp_shape() {
        let json = r#"{"events":[{"type":"key_press","timestamp":0.25,"key":"a","vkCode":65},{"type":"mouse_click","timestamp":0.5,"x":10,"y":20,"button":"left","pressed":true}]}"#;
        let parsed: MacroFile = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.events.len(), 2);
        assert_eq!(parsed.events[0].event_type, "key_press");
        assert_eq!(parsed.events[0].vk_code, Some(65));
        assert_eq!(parsed.events[1].button.as_deref(), Some("left"));

        let encoded = serde_json::to_string(&MacroFile {
            events: vec![MacroEvent {
                event_type: "key_release".to_string(),
                timestamp: 1.0,
                key: Some("a".to_string()),
                vk_code: Some(65),
                ..Default::default()
            }],
        })
        .unwrap();
        assert!(encoded.contains(r#""vkCode":65"#));
        assert!(encoded.contains(r#""type":"key_release""#));
        assert!(encoded.contains(r#""x":null"#));
        assert!(encoded.contains(r#""button":null"#));
    }

    #[test]
    fn macro_file_accepts_case_variants_like_csharp() {
        let json = r#"{"Events":[{"Type":"key_press","Timestamp":0.25,"Key":"A","VkCode":65},{"Type":"mouse_scroll","Timestamp":0.5,"X":10,"Y":20,"Dx":0,"Dy":-1}]}"#;
        let parsed: MacroFile = serde_json::from_str(json).unwrap();

        assert_eq!(parsed.events.len(), 2);
        assert_eq!(parsed.events[0].event_type, "key_press");
        assert_eq!(parsed.events[0].timestamp, 0.25);
        assert_eq!(parsed.events[0].key.as_deref(), Some("A"));
        assert_eq!(parsed.events[0].vk_code, Some(65));
        assert_eq!(parsed.events[1].event_type, "mouse_scroll");
        assert_eq!(parsed.events[1].x, Some(10));
        assert_eq!(parsed.events[1].dy, Some(-1));
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            speed_half: true,
            speed_1x: true,
            speed_2x: true,
            speed_100x: true,
            record_hotkey: "F9".to_string(),
            play_hotkey: "F10".to_string(),
            always_on_top: false,
            show_captions: true,
            use_sendinput: true,
        }
    }
}
