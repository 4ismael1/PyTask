use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK,
    MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput, VIRTUAL_KEY, keybd_event,
    mouse_event as win_mouse_event,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetSystemMetrics, HHOOK, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, SM_CXSCREEN,
    SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::model::MacroEvent;

const INPUT_SIGNATURE: usize = 0x5A17_B10C;
const MOVE_EVENT_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Default)]
struct RecorderState {
    recording: bool,
    start: Option<Instant>,
    last_move: Option<Instant>,
    events: Vec<MacroEvent>,
    mouse_hook: isize,
    keyboard_hook: isize,
    ignored_vks: Vec<u32>,
}

struct HotkeyState {
    keyboard_hook: isize,
    record_vk: u32,
    play_vk: u32,
    record_down: bool,
    play_down: bool,
    on_record: Option<Arc<dyn Fn() + Send + Sync>>,
    on_play: Option<Arc<dyn Fn() + Send + Sync>>,
}

static RECORDER: OnceLock<Mutex<RecorderState>> = OnceLock::new();
static HOTKEYS: OnceLock<Mutex<HotkeyState>> = OnceLock::new();

pub struct MacroRecorder;

impl MacroRecorder {
    pub fn new() -> Self {
        RECORDER.get_or_init(|| Mutex::new(RecorderState::default()));
        Self
    }

    pub fn start_ignoring_hotkeys(&self, hotkeys: &[&str]) -> Result<(), String> {
        let mutex = RECORDER.get_or_init(|| Mutex::new(RecorderState::default()));
        let mut state = mutex.lock().map_err(|_| "Recorder lock failed")?;
        if state.recording {
            return Ok(());
        }

        let (mouse_hook, keyboard_hook) = unsafe {
            let module = GetModuleHandleW(None).map_err(|err| err.message().to_string())?;
            let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), module, 0)
                .map_err(|err| err.message().to_string())?;
            let keyboard_hook =
                match SetWindowsHookExW(WH_KEYBOARD_LL, Some(record_keyboard_hook_proc), module, 0)
                {
                    Ok(hook) => hook,
                    Err(err) => {
                        let _ = UnhookWindowsHookEx(mouse_hook);
                        return Err(err.message().to_string());
                    }
                };
            (mouse_hook.0 as isize, keyboard_hook.0 as isize)
        };

        state.events.clear();
        state.start = Some(Instant::now());
        state.last_move = None;
        state.mouse_hook = mouse_hook;
        state.keyboard_hook = keyboard_hook;
        state.ignored_vks = hotkeys.iter().filter_map(|key| hotkey_to_vk(key)).collect();
        state.recording = true;

        Ok(())
    }

    pub fn stop(&self) -> Vec<MacroEvent> {
        let Some(mutex) = RECORDER.get() else {
            return Vec::new();
        };
        let Ok(mut state) = mutex.lock() else {
            return Vec::new();
        };
        if !state.recording {
            return Vec::new();
        }

        state.recording = false;
        unsafe {
            if state.mouse_hook != 0 {
                let _ = UnhookWindowsHookEx(HHOOK(state.mouse_hook as _));
                state.mouse_hook = 0;
            }
            if state.keyboard_hook != 0 {
                let _ = UnhookWindowsHookEx(HHOOK(state.keyboard_hook as _));
                state.keyboard_hook = 0;
            }
        }
        state.ignored_vks.clear();
        state.events.clone()
    }
}

impl Drop for MacroRecorder {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

pub struct GlobalHotkeys {
    keyboard_hook: isize,
}

impl GlobalHotkeys {
    pub fn register(
        record_key: &str,
        play_key: &str,
        on_record: impl Fn() + Send + Sync + 'static,
        on_play: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let record_vk = hotkey_to_vk(record_key).ok_or("Unsupported record hotkey")?;
        let play_vk = hotkey_to_vk(play_key).ok_or("Unsupported play hotkey")?;
        let module = unsafe { GetModuleHandleW(None).map_err(|err| err.message().to_string())? };
        let hook = unsafe {
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(hotkey_hook_proc), module, 0)
                .map_err(|err| err.message().to_string())?
        };

        let mutex = HOTKEYS.get_or_init(|| {
            Mutex::new(HotkeyState {
                keyboard_hook: 0,
                record_vk,
                play_vk,
                record_down: false,
                play_down: false,
                on_record: None,
                on_play: None,
            })
        });
        let mut state = match mutex.lock() {
            Ok(state) => state,
            Err(_) => {
                unsafe {
                    let _ = UnhookWindowsHookEx(hook);
                }
                return Err("Hotkey lock failed".to_string());
            }
        };
        if state.keyboard_hook != 0 {
            unsafe {
                let _ = UnhookWindowsHookEx(HHOOK(state.keyboard_hook as _));
            }
        }
        *state = HotkeyState {
            keyboard_hook: hook.0 as isize,
            record_vk,
            play_vk,
            record_down: false,
            play_down: false,
            on_record: Some(Arc::new(on_record)),
            on_play: Some(Arc::new(on_play)),
        };
        Ok(Self {
            keyboard_hook: hook.0 as isize,
        })
    }
}

impl Drop for GlobalHotkeys {
    fn drop(&mut self) {
        if let Some(mutex) = HOTKEYS.get()
            && let Ok(mut state) = mutex.lock()
            && state.keyboard_hook == self.keyboard_hook
            && state.keyboard_hook != 0
        {
            unsafe {
                let _ = UnhookWindowsHookEx(HHOOK(state.keyboard_hook as _));
            }
            state.keyboard_hook = 0;
        }
    }
}

pub struct MacroPlayer {
    metrics: ScreenMetrics,
    cancel: Arc<AtomicBool>,
}

impl MacroPlayer {
    pub fn new() -> Self {
        Self {
            metrics: ScreenMetrics::new(),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn prepare_playback(&self) {
        self.cancel.store(false, Ordering::SeqCst);
    }

    pub fn playback_instance(&self) -> Self {
        Self {
            metrics: ScreenMetrics::new(),
            cancel: Arc::clone(&self.cancel),
        }
    }

    pub fn stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn play_blocking(
        &self,
        events: Vec<MacroEvent>,
        speed: f64,
        loops: i32,
        interval_mode: bool,
        interval_seconds: i32,
        use_sendinput: bool,
    ) -> bool {
        if events.is_empty() {
            return true;
        }
        let speed = if speed <= 0.0 { 0.01 } else { speed };
        let infinite = loops == 0;
        let mut done = 0;

        while !self.cancel.load(Ordering::SeqCst) {
            if !self.play_once(&events, speed, use_sendinput) {
                return false;
            }
            done += 1;
            if !infinite && done >= loops.max(1) {
                break;
            }
            let delay = if interval_mode {
                Duration::from_secs(interval_seconds.max(0) as u64)
            } else {
                Duration::from_millis(50)
            };
            if !self.sleep_cancelable(delay) {
                return false;
            }
        }
        !self.cancel.load(Ordering::SeqCst)
    }

    fn play_once(&self, events: &[MacroEvent], speed: f64, use_sendinput: bool) -> bool {
        let start = Instant::now();
        for event in events {
            if self.cancel.load(Ordering::SeqCst) {
                return false;
            }
            let target = Duration::from_secs_f64(event.timestamp / speed);
            let elapsed = start.elapsed();
            if target > elapsed && !self.sleep_cancelable(target - elapsed) {
                return false;
            }
            self.execute_event(event, use_sendinput);
        }
        true
    }

    fn sleep_cancelable(&self, duration: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < duration {
            if self.cancel.load(Ordering::SeqCst) {
                return false;
            }
            thread::sleep(Duration::from_millis(10));
        }
        true
    }

    fn execute_event(&self, event: &MacroEvent, use_sendinput: bool) {
        match event.event_type.as_str() {
            "mouse_move" => {
                if let (Some(x), Some(y)) = (event.x, event.y) {
                    self.send_mouse_move(x, y, use_sendinput);
                }
            }
            "mouse_click" => {
                if let (Some(x), Some(y), Some(button), Some(pressed)) =
                    (event.x, event.y, event.button.as_deref(), event.pressed)
                {
                    self.send_mouse_click(x, y, button, pressed, use_sendinput);
                }
            }
            "mouse_scroll" => {
                if let (Some(x), Some(y), Some(dy)) = (event.x, event.y, event.dy) {
                    self.send_mouse_scroll(x, y, dy, use_sendinput);
                }
            }
            "key_press" | "key_release" => {
                if let Some(vk) = event.vk_code {
                    self.send_key(vk as u16, event.event_type == "key_press", use_sendinput);
                }
            }
            _ => {}
        }
    }

    fn normalize(&self, x: i32, y: i32) -> (i32, i32) {
        normalize_point(x, y, self.metrics)
    }

    fn send_mouse_move(&self, x: i32, y: i32, use_sendinput: bool) {
        let (nx, ny) = self.normalize(x, y);
        let flags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        if use_sendinput {
            send_inputs(&[mouse_input(nx, ny, flags, 0)]);
        } else {
            unsafe {
                win_mouse_event(flags, nx, ny, 0, INPUT_SIGNATURE);
            }
        }
    }

    fn send_mouse_click(&self, x: i32, y: i32, button: &str, pressed: bool, use_sendinput: bool) {
        let (nx, ny) = self.normalize(x, y);
        let flag = match button.to_ascii_lowercase().as_str() {
            "right" => {
                if pressed {
                    MOUSEEVENTF_RIGHTDOWN
                } else {
                    MOUSEEVENTF_RIGHTUP
                }
            }
            "middle" => {
                if pressed {
                    MOUSEEVENTF_MIDDLEDOWN
                } else {
                    MOUSEEVENTF_MIDDLEUP
                }
            }
            _ => {
                if pressed {
                    MOUSEEVENTF_LEFTDOWN
                } else {
                    MOUSEEVENTF_LEFTUP
                }
            }
        };
        let move_flags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        let click_flags = flag | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        if use_sendinput {
            send_inputs(&[
                mouse_input(nx, ny, move_flags, 0),
                mouse_input(nx, ny, click_flags, 0),
            ]);
        } else {
            unsafe {
                win_mouse_event(move_flags, nx, ny, 0, INPUT_SIGNATURE);
                win_mouse_event(click_flags, nx, ny, 0, INPUT_SIGNATURE);
            }
        }
    }

    fn send_mouse_scroll(&self, x: i32, y: i32, dy: i32, use_sendinput: bool) {
        let (nx, ny) = self.normalize(x, y);
        let wheel_delta = dy * 120;
        let move_flags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        if use_sendinput {
            send_inputs(&[
                mouse_input(nx, ny, move_flags, 0),
                mouse_input(0, 0, MOUSEEVENTF_WHEEL, wheel_delta as u32),
            ]);
        } else {
            unsafe {
                win_mouse_event(move_flags, nx, ny, 0, INPUT_SIGNATURE);
                win_mouse_event(MOUSEEVENTF_WHEEL, 0, 0, wheel_delta, INPUT_SIGNATURE);
            }
        }
    }

    fn send_key(&self, vk: u16, pressed: bool, use_sendinput: bool) {
        let flags = if pressed {
            Default::default()
        } else {
            KEYEVENTF_KEYUP
        };
        if !use_sendinput {
            unsafe {
                keybd_event(vk as u8, 0, flags, INPUT_SIGNATURE);
            }
            return;
        }

        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk),
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: INPUT_SIGNATURE,
                },
            },
        };
        send_inputs(&[input]);
    }
}

fn normalize_point(x: i32, y: i32, metrics: ScreenMetrics) -> (i32, i32) {
    if metrics.width <= 1 || metrics.height <= 1 {
        return (0, 0);
    }
    let nx = (((x - metrics.left) as f64) * metrics.x_normalizer)
        .round()
        .clamp(0.0, 65535.0) as i32;
    let ny = (((y - metrics.top) as f64) * metrics.y_normalizer)
        .round()
        .clamp(0.0, 65535.0) as i32;
    (nx, ny)
}

#[derive(Clone, Copy)]
struct ScreenMetrics {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    x_normalizer: f64,
    y_normalizer: f64,
}

impl ScreenMetrics {
    fn new() -> Self {
        unsafe {
            let mut left = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let mut top = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let mut width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let mut height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
            if width <= 0 || height <= 0 {
                left = 0;
                top = 0;
                width = GetSystemMetrics(SM_CXSCREEN);
                height = GetSystemMetrics(SM_CYSCREEN);
            }
            Self {
                left,
                top,
                width,
                height,
                x_normalizer: 65535.0 / (width - 1).max(1) as f64,
                y_normalizer: 65535.0 / (height - 1).max(1) as f64,
            }
        }
    }
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut next_hook = HHOOK::default();
    if code >= 0
        && let Some(mutex) = RECORDER.get()
        && let Ok(mut state) = mutex.lock()
    {
        next_hook = HHOOK(state.mouse_hook as _);
        if state.recording {
            let hook = *(lparam.0 as *const MSLLHOOKSTRUCT);
            if hook.dwExtraInfo != INPUT_SIGNATURE {
                let now = Instant::now();
                let timestamp = state
                    .start
                    .map(|start| start.elapsed().as_secs_f64())
                    .unwrap_or_default();
                match wparam.0 as u32 {
                    WM_MOUSEMOVE => {
                        if state
                            .last_move
                            .is_none_or(|last| now.duration_since(last) >= MOVE_EVENT_INTERVAL)
                        {
                            state.events.push(mouse_event(
                                "mouse_move",
                                hook.pt.x,
                                hook.pt.y,
                                None,
                                None,
                                timestamp,
                            ));
                            state.last_move = Some(now);
                        }
                    }
                    WM_LBUTTONDOWN => push_click(&mut state, &hook, "left", true, timestamp),
                    WM_LBUTTONUP => push_click(&mut state, &hook, "left", false, timestamp),
                    WM_RBUTTONDOWN => push_click(&mut state, &hook, "right", true, timestamp),
                    WM_RBUTTONUP => push_click(&mut state, &hook, "right", false, timestamp),
                    WM_MBUTTONDOWN => push_click(&mut state, &hook, "middle", true, timestamp),
                    WM_MBUTTONUP => push_click(&mut state, &hook, "middle", false, timestamp),
                    WM_MOUSEWHEEL => {
                        let delta = ((hook.mouseData >> 16) & 0xffff) as i16 as i32;
                        state.events.push(MacroEvent {
                            event_type: "mouse_scroll".to_string(),
                            x: Some(hook.pt.x),
                            y: Some(hook.pt.y),
                            dx: Some(0),
                            dy: Some(delta / 120),
                            timestamp,
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
        }
    }
    CallNextHookEx(next_hook, code, wparam, lparam)
}

unsafe extern "system" fn record_keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mut next_hook = HHOOK::default();
    if code >= 0
        && let Some(mutex) = RECORDER.get()
        && let Ok(mut state) = mutex.lock()
    {
        next_hook = HHOOK(state.keyboard_hook as _);
        if state.recording {
            let hook = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            if hook.dwExtraInfo != INPUT_SIGNATURE
                && !is_ignored_vk(hook.vkCode, &state.ignored_vks)
            {
                let msg = wparam.0 as u32;
                let timestamp = state
                    .start
                    .map(|start| start.elapsed().as_secs_f64())
                    .unwrap_or_default();
                let pressed = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
                state.events.push(MacroEvent {
                    event_type: if pressed { "key_press" } else { "key_release" }.to_string(),
                    key: Some(key_name(hook.vkCode)),
                    vk_code: Some(hook.vkCode),
                    timestamp,
                    ..Default::default()
                });
            }
        }
    }
    CallNextHookEx(next_hook, code, wparam, lparam)
}

fn is_ignored_vk(vk_code: u32, ignored_vks: &[u32]) -> bool {
    ignored_vks.contains(&vk_code)
}

unsafe extern "system" fn hotkey_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut callback_to_run: Option<Arc<dyn Fn() + Send + Sync>> = None;
    let mut next_hook = HHOOK::default();
    if code >= 0
        && let Some(mutex) = HOTKEYS.get()
        && let Ok(mut state) = mutex.lock()
    {
        next_hook = HHOOK(state.keyboard_hook as _);
        let hook = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        if hook.dwExtraInfo != INPUT_SIGNATURE {
            let msg = wparam.0 as u32;
            let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
            if hook.vkCode == state.record_vk {
                if down && !state.record_down {
                    state.record_down = true;
                    if let Some(callback) = &state.on_record {
                        callback_to_run = Some(Arc::clone(callback));
                    }
                } else if up {
                    state.record_down = false;
                }
            } else if hook.vkCode == state.play_vk {
                if down && !state.play_down {
                    state.play_down = true;
                    if let Some(callback) = &state.on_play {
                        callback_to_run = Some(Arc::clone(callback));
                    }
                } else if up {
                    state.play_down = false;
                }
            }
        }
    }
    if let Some(callback) = callback_to_run {
        callback();
    }
    CallNextHookEx(next_hook, code, wparam, lparam)
}

fn push_click(
    state: &mut RecorderState,
    hook: &MSLLHOOKSTRUCT,
    button: &str,
    pressed: bool,
    timestamp: f64,
) {
    state.events.push(mouse_event(
        "mouse_click",
        hook.pt.x,
        hook.pt.y,
        Some(button),
        Some(pressed),
        timestamp,
    ));
}

fn mouse_event(
    kind: &str,
    x: i32,
    y: i32,
    button: Option<&str>,
    pressed: Option<bool>,
    timestamp: f64,
) -> MacroEvent {
    MacroEvent {
        event_type: kind.to_string(),
        x: Some(x),
        y: Some(y),
        button: button.map(ToString::to_string),
        pressed,
        timestamp,
        ..Default::default()
    }
}

fn mouse_input(dx: i32, dy: i32, flags: impl Into<_MouseFlags>, mouse_data: u32) -> INPUT {
    let flags = flags.into().0;
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: mouse_data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INPUT_SIGNATURE,
            },
        },
    }
}

struct _MouseFlags(windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS);

impl From<windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS> for _MouseFlags {
    fn from(value: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS) -> Self {
        Self(value)
    }
}

fn send_inputs(inputs: &[INPUT]) {
    unsafe {
        let _ = SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

pub fn hotkey_to_vk(key: &str) -> Option<u32> {
    match key.trim().to_ascii_uppercase().as_str() {
        "F5" => Some(0x74),
        "F6" => Some(0x75),
        "F7" => Some(0x76),
        "F8" => Some(0x77),
        "F9" => Some(0x78),
        "F10" => Some(0x79),
        "F11" => Some(0x7A),
        "F12" => Some(0x7B),
        _ => None,
    }
}

fn key_name(vk: u32) -> String {
    match vk {
        0x20 => "space".to_string(),
        0x0D => "enter".to_string(),
        0x09 => "tab".to_string(),
        0x08 => "backspace".to_string(),
        0x1B => "esc".to_string(),
        0x2E => "delete".to_string(),
        0x10 => "shift".to_string(),
        0x11 => "ctrl".to_string(),
        0x12 => "alt".to_string(),
        0x25 => "left".to_string(),
        0x26 => "up".to_string(),
        0x27 => "right".to_string(),
        0x28 => "down".to_string(),
        0x70..=0x7B => format!("f{}", vk - 0x6F),
        0x41..=0x5A => (vk as u8 as char).to_ascii_lowercase().to_string(),
        0x30..=0x39 => (vk as u8 as char).to_string(),
        _ => vk.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MacroPlayer, ScreenMetrics, hotkey_to_vk, is_ignored_vk, key_name, normalize_point,
    };
    use crate::model::MacroEvent;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn maps_supported_function_hotkeys() {
        assert_eq!(hotkey_to_vk("F5"), Some(0x74));
        assert_eq!(hotkey_to_vk("f9"), Some(0x78));
        assert_eq!(hotkey_to_vk(" F12 "), Some(0x7B));
        assert_eq!(hotkey_to_vk("F13"), None);
    }

    #[test]
    fn ignores_configured_control_hotkeys_while_recording() {
        let ignored = ["F9", "F10"]
            .iter()
            .filter_map(|key| hotkey_to_vk(key))
            .collect::<Vec<_>>();

        assert!(is_ignored_vk(0x78, &ignored));
        assert!(is_ignored_vk(0x79, &ignored));
        assert!(!is_ignored_vk(0x41, &ignored));
    }

    #[test]
    fn formats_recorded_key_names_like_csharp() {
        assert_eq!(key_name(0x20), "space");
        assert_eq!(key_name(0x41), "a");
        assert_eq!(key_name(0x70), "f1");
        assert_eq!(key_name(0x7B), "f12");
        assert_eq!(key_name(0xBA), "186");
    }

    #[test]
    fn normalizes_points_across_virtual_screen() {
        let metrics = ScreenMetrics {
            left: 0,
            top: 0,
            width: 1920,
            height: 1080,
            x_normalizer: 65535.0 / 1919.0,
            y_normalizer: 65535.0 / 1079.0,
        };

        assert_eq!(metrics.width, 1920);
        assert_eq!(metrics.height, 1080);
        assert_eq!(normalize_point(0, 0, metrics), (0, 0));
        assert_eq!(normalize_point(1919, 1079, metrics), (65535, 65535));
        let center = normalize_point(960, 540, metrics);
        assert!((32760..=32810).contains(&center.0));
        assert!((32760..=32840).contains(&center.1));
    }

    #[test]
    fn normalizes_points_on_negative_virtual_screen_origin() {
        let metrics = ScreenMetrics {
            left: -1920,
            top: -1080,
            width: 3840,
            height: 2160,
            x_normalizer: 65535.0 / 3839.0,
            y_normalizer: 65535.0 / 2159.0,
        };

        assert_eq!(normalize_point(-1920, -1080, metrics), (0, 0));
        assert_eq!(normalize_point(1919, 1079, metrics), (65535, 65535));
        let center = normalize_point(0, 0, metrics);
        assert!((32750..=32820).contains(&center.0));
        assert!((32750..=32840).contains(&center.1));
    }

    #[test]
    fn playback_stop_cancels_shared_playback_instance_quickly() {
        let player = MacroPlayer::new();
        player.prepare_playback();
        let playback = player.playback_instance();
        let events = vec![MacroEvent {
            event_type: "noop".to_string(),
            timestamp: 5.0,
            ..Default::default()
        }];

        let started = Instant::now();
        let handle = thread::spawn(move || playback.play_blocking(events, 1.0, 1, false, 0, true));
        thread::sleep(Duration::from_millis(40));
        player.stop();

        assert!(!handle.join().unwrap());
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn prepare_playback_clears_previous_stop_request() {
        let player = MacroPlayer::new();
        let events = vec![MacroEvent {
            event_type: "noop".to_string(),
            timestamp: 0.0,
            ..Default::default()
        }];

        player.stop();
        assert!(
            !player
                .playback_instance()
                .play_blocking(events.clone(), 1.0, 1, false, 0, true)
        );

        player.prepare_playback();
        assert!(
            player
                .playback_instance()
                .play_blocking(events, 1.0, 1, false, 0, true)
        );
    }
}
