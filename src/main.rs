#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread;

use image::imageops::FilterType;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BeginPaint, BitBlt, CLIP_DEFAULT_PRECIS, ClientToScreen,
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen, CreateSolidBrush,
    DIB_RGB_COLORS, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject, DrawTextW,
    EndPaint, FF_DONTCARE, FW_BOLD, FW_NORMAL, FillRect, HDC, HFONT, HGDIOBJ, InvalidateRect,
    OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID, SRCCOPY, SelectObject, SetBkMode, SetTextColor,
    StretchDIBits, TRANSPARENT, UpdateWindow,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_HIDEREADONLY,
    OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForSystem, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, SetFocus, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, AppendMenuW, BS_DEFPUSHBUTTON, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW,
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW,
    ES_AUTOHSCROLL, GWLP_USERDATA, GetMessageW, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect,
    GetWindowTextW, HICON, HMENU, HWND_NOTOPMOST, HWND_TOPMOST, IDC_ARROW, IMAGE_ICON, IsWindow,
    KillTimer, LoadCursorW, LoadImageW, MB_ICONINFORMATION, MB_OK, MENU_ITEM_FLAGS, MF_CHECKED,
    MF_POPUP, MF_SEPARATOR, MF_STRING, MSG, MessageBoxW, PostMessageW, PostQuitMessage,
    RegisterClassW, SM_CXICON, SM_CXSCREEN, SM_CXSMICON, SM_CYICON, SM_CYSCREEN, SM_CYSMICON,
    SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SendMessageW, SetForegroundWindow, SetProcessDPIAware,
    SetTimer, SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow, TPM_LEFTALIGN,
    TPM_TOPALIGN, TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP,
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_NCCREATE, WM_PAINT, WM_SETFONT, WM_SETICON, WM_TIMER, WNDCLASSW, WS_BORDER,
    WS_CAPTION, WS_CHILD, WS_EX_DLGMODALFRAME, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU,
    WS_TABSTOP, WS_VISIBLE,
};
use windows::core::{PCWSTR, PWSTR, w};

mod engine;
mod macro_io;
mod model;
mod settings_db;

use engine::{GlobalHotkeys, MacroPlayer, MacroRecorder};
use model::{MacroEvent, Settings};
use settings_db::SettingsDatabase;

const BASE_CLIENT_W: i32 = 340;
const BASE_CLIENT_H: i32 = 96;
const BASE_STATUS_Y: i32 = 74;
const BASE_BUTTON_W: i32 = 58;
const BASE_BUTTON_H: i32 = 62;
const BASE_BUTTON_Y: i32 = 7;
const BASE_BUTTON_GAP: i32 = 7;
const BASE_BUTTON_X: i32 = 11;
const BASE_ICON_SIZE: i32 = 26;
const BASE_STATUS_ICON_SIZE: i32 = 14;
const UI_DPI_SCALE_WEIGHT: f64 = 0.5;
const WM_HOTKEY_RECORD: u32 = WM_APP + 1;
const WM_HOTKEY_PLAY: u32 = WM_APP + 2;
const WM_PLAYBACK_FINISHED: u32 = WM_APP + 3;
const STATUS_TIMER_ID: usize = 1;
const STATUS_TEMP_MS: u32 = 2500;
const STATUS_WARNING_MS: u32 = 4000;
const ID_DIALOG_OK: usize = 1;
const ID_DIALOG_CANCEL: usize = 2;
const ICON_SMALL_ID: usize = 0;
const ICON_BIG_ID: usize = 1;
const APP_ICON_RESOURCE_ID: usize = 1;
const APP_LOGO_ICON_INDEX: usize = 5;
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

static UI_SCALE: OnceLock<f64> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonKind {
    Open,
    Save,
    Rec,
    Play,
    Prefs,
}

#[derive(Clone, Copy)]
enum HotkeyTarget {
    Record,
    Play,
}

struct ToolbarButton {
    kind: ButtonKind,
    label: &'static str,
    icon: usize,
    enabled: bool,
    split: bool,
}

struct IconAsset {
    rgba: Vec<u8>,
}

struct NumberDialogState {
    title: &'static str,
    label1: &'static str,
    suffix1: Option<&'static str>,
    label2: Option<&'static str>,
    suffix2: Option<&'static str>,
    value1: String,
    value2: String,
    edit1: HWND,
    edit2: HWND,
    font: HFONT,
    width: i32,
    height: i32,
    result: Option<(String, String)>,
}

struct AppState {
    buttons: Vec<ToolbarButton>,
    icons: Vec<IconAsset>,
    hover: Option<ButtonKind>,
    pressed: Option<ButtonKind>,
    tracking_leave: bool,
    status: String,
    db: Option<SettingsDatabase>,
    settings: Settings,
    recorder: MacroRecorder,
    player: MacroPlayer,
    hotkeys: Option<GlobalHotkeys>,
    current_events: Vec<MacroEvent>,
    current_file: Option<PathBuf>,
    play_speed: f64,
    custom_speed: f64,
    playback_loops: i32,
    interval_mode: bool,
    interval_seconds: i32,
    is_recording: bool,
    is_playing: bool,
    playback_stop_requested: bool,
    playback_generation: u64,
}

impl AppState {
    fn new() -> Self {
        let (db, settings, status) = match SettingsDatabase::open_default() {
            Ok(db) => {
                let settings = db.load_settings();
                let status = default_status(&settings);
                (Some(db), settings, status)
            }
            Err(err) => {
                let settings = Settings::default();
                let status = with_prefs_open_warning(default_status(&settings), &err.to_string());
                (None, settings, status)
            }
        };
        let icons = vec![
            load_png(include_bytes!("../assets/icons/abrir-documento.png")),
            load_png(include_bytes!("../assets/icons/Save.png")),
            load_png(include_bytes!("../assets/icons/boton-detener.png")),
            load_png(include_bytes!("../assets/icons/Play.png")),
            load_png(include_bytes!("../assets/icons/preferencias.png")),
            load_png(include_bytes!("../assets/icons/pytask-logo.png")),
        ];

        Self {
            buttons: vec![
                ToolbarButton {
                    kind: ButtonKind::Open,
                    label: "Open",
                    icon: 0,
                    enabled: true,
                    split: false,
                },
                ToolbarButton {
                    kind: ButtonKind::Save,
                    label: "Save",
                    icon: 1,
                    enabled: false,
                    split: false,
                },
                ToolbarButton {
                    kind: ButtonKind::Rec,
                    label: "Rec",
                    icon: 2,
                    enabled: true,
                    split: false,
                },
                ToolbarButton {
                    kind: ButtonKind::Play,
                    label: "Play",
                    icon: 3,
                    enabled: false,
                    split: false,
                },
                ToolbarButton {
                    kind: ButtonKind::Prefs,
                    label: "Prefs",
                    icon: 4,
                    enabled: true,
                    split: false,
                },
            ],
            icons,
            hover: None,
            pressed: None,
            tracking_leave: false,
            status,
            db,
            settings,
            recorder: MacroRecorder::new(),
            player: MacroPlayer::new(),
            hotkeys: None,
            current_events: Vec::new(),
            current_file: None,
            play_speed: 1.0,
            custom_speed: 8.0,
            playback_loops: 1,
            interval_mode: false,
            interval_seconds: 5,
            is_recording: false,
            is_playing: false,
            playback_stop_requested: false,
            playback_generation: 0,
        }
    }

    fn button_rect(index: usize) -> RECT {
        let left = button_x() + index as i32 * (button_w() + button_gap());
        RECT {
            left,
            top: button_y(),
            right: left + button_w(),
            bottom: button_y() + button_h(),
        }
    }

    fn hit_test(&self, x: i32, y: i32) -> Option<ButtonKind> {
        self.buttons.iter().enumerate().find_map(|(index, button)| {
            let rect = Self::button_rect(index);
            (x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom && button.enabled)
                .then_some(button.kind)
        })
    }

    fn init_hotkeys(&mut self, hwnd: HWND) -> Result<(), String> {
        let record_key = self.settings.record_hotkey.clone();
        let play_key = self.settings.play_hotkey.clone();
        let hwnd_value = hwnd.0 as isize;
        let hotkeys = GlobalHotkeys::register(
            &record_key,
            &play_key,
            move || {
                let hwnd = HWND(hwnd_value as *mut c_void);
                let _ = unsafe { PostMessageW(hwnd, WM_HOTKEY_RECORD, WPARAM(0), LPARAM(0)) };
            },
            move || {
                let hwnd = HWND(hwnd_value as *mut c_void);
                let _ = unsafe { PostMessageW(hwnd, WM_HOTKEY_PLAY, WPARAM(0), LPARAM(0)) };
            },
        )?;
        self.hotkeys = Some(hotkeys);
        Ok(())
    }

    fn save_settings(&self) -> Result<(), String> {
        if let Some(db) = &self.db {
            db.save_settings(&self.settings)
                .map_err(|err| err.to_string())?;
        }
        Ok(())
    }

    fn set_button_enabled(&mut self, kind: ButtonKind, enabled: bool) {
        if let Some(button) = self.buttons.iter_mut().find(|button| button.kind == kind) {
            button.enabled = enabled;
        }
    }

    fn refresh_buttons(&mut self) {
        let has_macro = !self.current_events.is_empty();
        self.set_button_enabled(ButtonKind::Save, has_macro && !self.is_playing);
        self.set_button_enabled(ButtonKind::Play, has_macro && !self.is_recording);
        self.set_button_enabled(ButtonKind::Rec, !self.is_playing || self.is_recording);
    }
}

fn load_png(bytes: &[u8]) -> IconAsset {
    let image = image::load_from_memory(bytes)
        .expect("embedded PNG icon should decode")
        .resize_exact(icon_size() as u32, icon_size() as u32, FilterType::Lanczos3)
        .to_rgba8();

    IconAsset {
        rgba: image.into_raw(),
    }
}

fn main_window_style() -> WINDOW_STYLE {
    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX
}

unsafe fn load_window_icon(instance: HINSTANCE, width: i32, height: i32) -> Option<HICON> {
    unsafe {
        LoadImageW(
            instance,
            PCWSTR(APP_ICON_RESOURCE_ID as *const u16),
            IMAGE_ICON,
            width,
            height,
            Default::default(),
        )
        .ok()
        .map(|handle| HICON(handle.0))
    }
}

fn main() -> windows::core::Result<()> {
    if std::env::args().any(|arg| arg == "--smoke-test") {
        if let Err(err) = run_smoke_test() {
            eprintln!("PyTask smoke test failed: {err}");
            std::process::exit(1);
        }
        return Ok(());
    }

    unsafe {
        make_process_dpi_aware();
        init_ui_scale();
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("PyTaskRustWindow");
        let app_instance: HINSTANCE = instance.into();
        let small_icon = load_window_icon(
            app_instance,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
        );
        let big_icon = load_window_icon(
            app_instance,
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CYICON),
        );

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hIcon: big_icon.unwrap_or_default(),
            hInstance: app_instance,
            lpszClassName: class_name,
            lpfnWndProc: Some(window_proc),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };
        RegisterClassW(&wc);

        let style = main_window_style();
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: client_w(),
            bottom: client_h(),
        };
        AdjustWindowRectEx(&mut rect, style, false, WINDOW_EX_STYLE::default())?;
        let window_w = rect.right - rect.left;
        let window_h = rect.bottom - rect.top;
        let window_x = ((GetSystemMetrics(SM_CXSCREEN) - window_w) / 2).max(0);
        let window_y = ((GetSystemMetrics(SM_CYSCREEN) - window_h) / 2).max(0);

        let state = Box::new(AppState::new());
        let state_ptr = Box::into_raw(state);
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("PyTask"),
            style,
            window_x,
            window_y,
            window_w,
            window_h,
            HWND::default(),
            HMENU::default(),
            instance,
            Some(state_ptr.cast::<c_void>()),
        )?;
        if let Some(icon) = small_icon {
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_SMALL_ID),
                LPARAM(icon.0 as isize),
            );
        }
        if let Some(icon) = big_icon {
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_BIG_ID),
                LPARAM(icon.0 as isize),
            );
        }
        if let Err(err) = (*state_ptr).init_hotkeys(hwnd) {
            (*state_ptr).status = format!("Error al registrar hotkeys: {}", err);
        }
        apply_always_on_top(hwnd, (*state_ptr).settings.always_on_top);

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

fn run_smoke_test() -> Result<(), String> {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("open-save-smoke.macro");
    let events = macro_io::load_macro(&fixture).map_err(|err| err.to_string())?;
    if events.len() != 1 {
        return Err(format!("expected 1 fixture event, got {}", events.len()));
    }

    let out_path = std::env::temp_dir().join(format!(
        "pytask-rust-smoke-{}-{}.macro",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|err| err.to_string())?
            .as_nanos()
    ));
    macro_io::save_macro(&out_path, &events).map_err(|err| err.to_string())?;
    let loaded = macro_io::load_macro(&out_path).map_err(|err| err.to_string())?;
    let _ = std::fs::remove_file(&out_path);

    if loaded.len() != events.len() {
        return Err("saved macro did not round-trip".to_string());
    }
    if loaded[0].event_type != "mouse_move" || loaded[0].x != Some(100) || loaded[0].y != Some(100)
    {
        return Err("saved macro contents changed".to_string());
    }
    run_settings_smoke_test()?;
    Ok(())
}

fn run_settings_smoke_test() -> Result<(), String> {
    let db_path = std::env::temp_dir().join(format!(
        "pytask-rust-smoke-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|err| err.to_string())?
            .as_nanos()
    ));

    let db = SettingsDatabase::open(db_path.clone()).map_err(|err| err.to_string())?;
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
    db.save_settings(&settings).map_err(|err| err.to_string())?;
    let loaded = db.load_settings();
    drop(db);
    let _ = std::fs::remove_file(&db_path);

    if loaded.record_hotkey != "F8"
        || loaded.play_hotkey != "F12"
        || !loaded.always_on_top
        || loaded.show_captions
        || loaded.use_sendinput
    {
        return Err("settings did not round-trip".to_string());
    }

    Ok(())
}

unsafe fn make_process_dpi_aware() {
    if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).is_err() {
        let _ = SetProcessDPIAware();
    }
}

unsafe fn init_ui_scale() {
    let dpi = GetDpiForSystem().max(96);
    let _ = UI_SCALE.set(ui_scale_for_dpi(dpi));
}

fn ui_scale_for_dpi(dpi: u32) -> f64 {
    let dpi_scale = (dpi.max(96) as f64 / 96.0).clamp(1.0, 3.0);
    1.0 + (dpi_scale - 1.0) * UI_DPI_SCALE_WEIGHT
}

fn ui(value: i32) -> i32 {
    ((value as f64) * UI_SCALE.get().copied().unwrap_or(1.0)).round() as i32
}

fn client_w() -> i32 {
    ui(BASE_CLIENT_W)
}

fn client_h() -> i32 {
    ui(BASE_CLIENT_H)
}

fn status_y() -> i32 {
    ui(BASE_STATUS_Y)
}

fn button_w() -> i32 {
    ui(BASE_BUTTON_W)
}

fn button_h() -> i32 {
    ui(BASE_BUTTON_H)
}

fn button_y() -> i32 {
    ui(BASE_BUTTON_Y)
}

fn button_gap() -> i32 {
    ui(BASE_BUTTON_GAP)
}

fn button_x() -> i32 {
    ui(BASE_BUTTON_X)
}

fn icon_size() -> i32 {
    ui(BASE_ICON_SIZE)
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if msg == WM_NCCREATE {
            let createstruct = lparam.0 as *const CREATESTRUCTW;
            let state = (*createstruct).lpCreateParams as *mut AppState;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
        if state_ptr.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        let state = &mut *state_ptr;

        match msg {
            WM_CREATE => LRESULT(0),
            WM_ERASEBKGND => LRESULT(1),
            WM_PAINT => {
                paint(hwnd, state);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                let x = loword(lparam.0 as u32) as i16 as i32;
                let y = hiword(lparam.0 as u32) as i16 as i32;
                let next_hover = state.hit_test(x, y);
                if state.hover != next_hover {
                    state.hover = next_hover;
                    let _ = InvalidateRect(hwnd, None, false);
                }
                if !state.tracking_leave {
                    let mut tme = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    let _ = TrackMouseEvent(&mut tme);
                    state.tracking_leave = true;
                }
                LRESULT(0)
            }
            WM_MOUSELEAVE => {
                state.hover = None;
                state.tracking_leave = false;
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let x = loword(lparam.0 as u32) as i16 as i32;
                let y = hiword(lparam.0 as u32) as i16 as i32;
                state.pressed = state.hit_test(x, y);
                let _ = InvalidateRect(hwnd, None, false);
                let _ = UpdateWindow(hwnd);
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let x = loword(lparam.0 as u32) as i16 as i32;
                let y = hiword(lparam.0 as u32) as i16 as i32;
                let released = state.hit_test(x, y);
                let pressed = state.pressed;
                state.pressed = None;
                let _ = InvalidateRect(hwnd, None, false);
                let _ = UpdateWindow(hwnd);
                if released == pressed
                    && let Some(kind) = released
                {
                    handle_click(hwnd, state, kind);
                    let _ = InvalidateRect(hwnd, None, false);
                    let _ = UpdateWindow(hwnd);
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let command_id = loword(wparam.0 as u32) as usize;
                handle_menu_command(hwnd, state, command_id);
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_TIMER => {
                if wparam.0 == STATUS_TIMER_ID {
                    cancel_status_restore(hwnd);
                    if restore_idle_status(state) {
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                    return LRESULT(0);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_HOTKEY_RECORD => {
                toggle_recording(hwnd, state);
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_HOTKEY_PLAY => {
                toggle_playback(hwnd, state);
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_PLAYBACK_FINISHED => {
                let completed = wparam.0 != 0;
                let generation = lparam.0 as u64;
                if generation == state.playback_generation {
                    state.is_playing = false;
                    state.playback_stop_requested = false;
                    if completed {
                        cancel_status_restore(hwnd);
                        state.status = default_status(&state.settings);
                    } else {
                        show_temporary_status(hwnd, state, "Detenido".to_string());
                    }
                    state.refresh_buttons();
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_DESTROY => {
                cancel_status_restore(hwnd);
                if state.is_playing {
                    state.player.stop();
                }
                if state.is_recording {
                    let _ = state.recorder.stop();
                }
                let _ = Box::from_raw(state_ptr);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn handle_click(hwnd: HWND, state: &mut AppState, kind: ButtonKind) {
    match kind {
        ButtonKind::Open => {
            open_macro(hwnd, state);
        }
        ButtonKind::Rec => {
            toggle_recording(hwnd, state);
        }
        ButtonKind::Prefs => {
            show_prefs_menu(hwnd, state);
        }
        ButtonKind::Save => {
            save_macro(hwnd, state);
        }
        ButtonKind::Play => {
            toggle_playback(hwnd, state);
        }
    }
}

unsafe fn show_prefs_menu(hwnd: HWND, state: &AppState) {
    let menu = CreatePopupMenu().expect("popup menu");

    let reproduction_menu = CreatePopupMenu().expect("reproduction menu");

    let speed_menu = CreatePopupMenu().expect("speed menu");
    append(speed_menu, checked(state.play_speed, 0.5), 101, "0.5x");
    append(speed_menu, checked(state.play_speed, 1.0), 102, "1x");
    append(speed_menu, checked(state.play_speed, 2.0), 103, "2x");
    append(speed_menu, checked(state.play_speed, 100.0), 104, "100x");
    let _ = AppendMenuW(speed_menu, MF_SEPARATOR, 0, PCWSTR::null());
    append(speed_menu, MF_STRING, 105, "Personalizar...");

    let mode_menu = CreatePopupMenu().expect("mode menu");
    append(mode_menu, loop_checked(state, 1), 201, "Una vez");
    append(mode_menu, loop_checked(state, 0), 202, "Infinito");
    let _ = AppendMenuW(mode_menu, MF_SEPARATOR, 0, PCWSTR::null());
    append(
        mode_menu,
        if state.playback_loops > 1 {
            MF_CHECKED | MF_STRING
        } else {
            MF_STRING
        },
        203,
        &format!(
            "Personalizado{}",
            if state.playback_loops > 1 {
                format!(" ({} veces)", state.playback_loops)
            } else {
                "...".to_string()
            }
        ),
    );

    let interval_menu = CreatePopupMenu().expect("interval menu");
    append(
        interval_menu,
        if !state.interval_mode {
            MF_CHECKED | MF_STRING
        } else {
            MF_STRING
        },
        301,
        "Sin intervalo",
    );
    append(interval_menu, interval_checked(state, 1), 302, "1 segundo");
    append(interval_menu, interval_checked(state, 5), 303, "5 segundos");
    append(
        interval_menu,
        interval_checked(state, 10),
        304,
        "10 segundos",
    );
    let _ = AppendMenuW(interval_menu, MF_SEPARATOR, 0, PCWSTR::null());
    append(interval_menu, MF_STRING, 305, "Personalizar...");

    let record_menu = CreatePopupMenu().expect("record menu");
    append(
        record_menu,
        hotkey_checked(&state.settings.record_hotkey, "F6"),
        401,
        "F6",
    );
    append(
        record_menu,
        hotkey_checked(&state.settings.record_hotkey, "F7"),
        402,
        "F7",
    );
    append(
        record_menu,
        hotkey_checked(&state.settings.record_hotkey, "F8"),
        403,
        "F8",
    );
    append(
        record_menu,
        hotkey_checked(&state.settings.record_hotkey, "F9"),
        404,
        "F9",
    );

    let play_menu = CreatePopupMenu().expect("play menu");
    append(
        play_menu,
        hotkey_checked(&state.settings.play_hotkey, "F5"),
        501,
        "F5",
    );
    append(
        play_menu,
        hotkey_checked(&state.settings.play_hotkey, "F10"),
        502,
        "F10",
    );
    append(
        play_menu,
        hotkey_checked(&state.settings.play_hotkey, "F11"),
        503,
        "F11",
    );
    append(
        play_menu,
        hotkey_checked(&state.settings.play_hotkey, "F12"),
        504,
        "F12",
    );

    append_submenu(reproduction_menu, speed_menu, "Velocidad");
    let _ = AppendMenuW(reproduction_menu, MF_SEPARATOR, 0, PCWSTR::null());
    append_submenu(reproduction_menu, mode_menu, "Modo");
    let _ = AppendMenuW(reproduction_menu, MF_SEPARATOR, 0, PCWSTR::null());
    append_submenu(reproduction_menu, interval_menu, "Intervalo");

    append_submenu(menu, reproduction_menu, "Reproduccion");
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    append_submenu(
        menu,
        record_menu,
        &format!("Tecla Grabar ({})", state.settings.record_hotkey),
    );
    append_submenu(
        menu,
        play_menu,
        &format!("Tecla Reproducir ({})", state.settings.play_hotkey),
    );
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    append(
        menu,
        bool_checked(state.settings.always_on_top),
        601,
        "Siempre Visible",
    );
    append(
        menu,
        bool_checked(state.settings.show_captions),
        602,
        "Barra de Estado",
    );
    append(
        menu,
        bool_checked(state.settings.use_sendinput),
        603,
        "Modo Juegos (SendInput)",
    );
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    append(menu, MF_STRING, 701, &format!("PyTask v{APP_VERSION}"));

    let prefs_rect = AppState::button_rect(4);
    let mut point = POINT {
        x: prefs_rect.left,
        y: prefs_rect.bottom + 2,
    };
    let _ = ClientToScreen(hwnd, &mut point);

    let _ = TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_TOPALIGN,
        point.x,
        point.y,
        0,
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);
}

unsafe fn append(menu: HMENU, flags: MENU_ITEM_FLAGS, id: usize, text: &str) {
    let text = wide(text);
    let _ = AppendMenuW(menu, flags, id, PCWSTR(text.as_ptr()));
}

unsafe fn append_submenu(menu: HMENU, submenu: HMENU, text: &str) {
    let text = wide(text);
    let _ = AppendMenuW(
        menu,
        MF_POPUP | MF_STRING,
        submenu.0 as usize,
        PCWSTR(text.as_ptr()),
    );
}

fn checked(value: f64, expected: f64) -> MENU_ITEM_FLAGS {
    if (value - expected).abs() < 0.01 {
        MF_CHECKED | MF_STRING
    } else {
        MF_STRING
    }
}

fn loop_checked(state: &AppState, expected: i32) -> MENU_ITEM_FLAGS {
    if state.playback_loops == expected {
        MF_CHECKED | MF_STRING
    } else {
        MF_STRING
    }
}

fn interval_checked(state: &AppState, seconds: i32) -> MENU_ITEM_FLAGS {
    if state.interval_mode && state.interval_seconds == seconds {
        MF_CHECKED | MF_STRING
    } else {
        MF_STRING
    }
}

fn hotkey_checked(current: &str, expected: &str) -> MENU_ITEM_FLAGS {
    if current.eq_ignore_ascii_case(expected) {
        MF_CHECKED | MF_STRING
    } else {
        MF_STRING
    }
}

fn bool_checked(value: bool) -> MENU_ITEM_FLAGS {
    if value {
        MF_CHECKED | MF_STRING
    } else {
        MF_STRING
    }
}

fn handle_menu_command(hwnd: HWND, state: &mut AppState, command_id: usize) {
    match command_id {
        101 => {
            set_speed(state, 0.5);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        102 => {
            set_speed(state, 1.0);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        103 => {
            set_speed(state, 2.0);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        104 => {
            set_speed(state, 100.0);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        105 => {
            if let Some(speed) = prompt_custom_speed(hwnd, state.custom_speed) {
                state.custom_speed = speed;
                state.play_speed = speed;
                show_temporary_status(
                    hwnd,
                    state,
                    format!("Velocidad personalizada: {}x", trim_number(speed)),
                );
            }
        }
        201 => {
            set_playback_loops(state, 1);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        202 => {
            set_playback_loops(state, 0);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        203 => {
            if let Some(loops) = prompt_playback_repetitions(hwnd, state.playback_loops) {
                set_playback_loops(state, loops);
                schedule_status_restore(hwnd, STATUS_TEMP_MS);
            }
        }
        301 => {
            set_interval_mode(state, 0, state.playback_loops);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        302 => {
            set_quick_interval(state, 1);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        303 => {
            set_quick_interval(state, 5);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        304 => {
            set_quick_interval(state, 10);
            schedule_status_restore(hwnd, STATUS_TEMP_MS);
        }
        305 => {
            if let Some((seconds, loops)) =
                prompt_interval_settings(hwnd, state.interval_seconds.max(1), state.playback_loops)
            {
                set_interval_mode(state, seconds, loops);
                schedule_status_restore(hwnd, STATUS_TEMP_MS);
            }
        }
        401 => set_record_hotkey(hwnd, state, "F6"),
        402 => set_record_hotkey(hwnd, state, "F7"),
        403 => set_record_hotkey(hwnd, state, "F8"),
        404 => set_record_hotkey(hwnd, state, "F9"),
        501 => set_play_hotkey(hwnd, state, "F5"),
        502 => set_play_hotkey(hwnd, state, "F10"),
        503 => set_play_hotkey(hwnd, state, "F11"),
        504 => set_play_hotkey(hwnd, state, "F12"),
        601 => {
            state.settings.always_on_top = !state.settings.always_on_top;
            apply_always_on_top(hwnd, state.settings.always_on_top);
            let status = format!(
                "Siempre visible: {}",
                if state.settings.always_on_top {
                    "ON"
                } else {
                    "OFF"
                }
            );
            let status = with_save_warning(status, state.save_settings());
            show_temporary_unless_error(hwnd, state, status);
        }
        602 => {
            state.settings.show_captions = !state.settings.show_captions;
            let status = format!(
                "Barra de Estado: {}",
                if state.settings.show_captions {
                    "ON"
                } else {
                    "OFF"
                }
            );
            let status = with_save_warning(status, state.save_settings());
            show_temporary_unless_error(hwnd, state, status);
        }
        603 => {
            state.settings.use_sendinput = !state.settings.use_sendinput;
            let status = if state.settings.use_sendinput {
                "Modo Juegos activado (SendInput)".to_string()
            } else {
                "Modo Normal activado".to_string()
            };
            let status = with_save_warning(status, state.save_settings());
            show_temporary_unless_error(hwnd, state, status);
        }
        701 => show_about(hwnd),
        _ => {}
    }
}

fn set_speed(state: &mut AppState, speed: f64) {
    state.play_speed = speed;
    state.status = format!("Velocidad: {}x", speed);
}

fn set_playback_loops(state: &mut AppState, loops: i32) {
    state.playback_loops = loops.clamp(0, 10000);
    state.status = playback_mode_summary(state);
}

fn set_quick_interval(state: &mut AppState, seconds: i32) {
    set_interval_mode(state, seconds, state.playback_loops);
}

fn set_interval_mode(state: &mut AppState, seconds: i32, loops: i32) {
    let seconds = seconds.clamp(0, 3600);
    let loops = loops.clamp(0, 10000);
    state.interval_mode = seconds > 0;
    state.interval_seconds = seconds;
    state.playback_loops = loops;

    state.status = playback_mode_summary(state);
}

fn playback_mode_summary(state: &AppState) -> String {
    let loop_text = match state.playback_loops {
        0 => "Reproduccion infinita".to_string(),
        1 => "Reproducir 1 vez".to_string(),
        loops => format!("Reproducir {} veces", loops),
    };

    if state.interval_mode && state.playback_loops != 1 {
        format!("Modo: {} cada {}s", loop_text, state.interval_seconds)
    } else if state.interval_mode {
        format!(
            "Modo: {} (intervalo {}s guardado)",
            loop_text, state.interval_seconds
        )
    } else {
        format!("Modo: {} sin intervalo", loop_text)
    }
}

fn prompt_custom_speed(hwnd: HWND, current: f64) -> Option<f64> {
    let result = prompt_number_dialog(
        hwnd,
        NumberDialogState {
            title: "Velocidad Personalizada",
            label1: "Multiplicador",
            suffix1: Some("x"),
            label2: None,
            suffix2: None,
            value1: trim_number(current),
            value2: String::new(),
            edit1: HWND::default(),
            edit2: HWND::default(),
            font: HFONT::default(),
            width: 340,
            height: 140,
            result: None,
        },
    )?;

    match parse_decimal(&result.0).filter(|value| (0.1..=1000.0).contains(value)) {
        Some(speed) => Some(speed),
        None => {
            show_message(
                hwnd,
                "Velocidad Personalizada",
                "Ingresa una velocidad entre 0.1 y 1000.",
            );
            None
        }
    }
}

fn prompt_playback_repetitions(hwnd: HWND, current_loops: i32) -> Option<i32> {
    let result = prompt_number_dialog(
        hwnd,
        NumberDialogState {
            title: "Repeticiones",
            label1: "Cantidad de repeticiones",
            suffix1: Some("veces"),
            label2: None,
            suffix2: None,
            value1: current_loops.max(2).clamp(2, 10000).to_string(),
            value2: String::new(),
            edit1: HWND::default(),
            edit2: HWND::default(),
            font: HFONT::default(),
            width: 340,
            height: 140,
            result: None,
        },
    )?;

    match result
        .0
        .trim()
        .parse::<i32>()
        .ok()
        .filter(|loops| (2..=10000).contains(loops))
    {
        Some(loops) => Some(loops),
        None => {
            show_message(
                hwnd,
                "Repeticiones",
                "Ingresa una cantidad entre 2 y 10000.",
            );
            None
        }
    }
}

fn prompt_interval_settings(
    hwnd: HWND,
    current_seconds: i32,
    current_loops: i32,
) -> Option<(i32, i32)> {
    let result = prompt_number_dialog(
        hwnd,
        NumberDialogState {
            title: "Intervalo de Reproduccion",
            label1: "Pausa entre ejecuciones",
            suffix1: Some("segundos"),
            label2: Some("Repeticiones"),
            suffix2: Some("0 = infinito"),
            value1: current_seconds.clamp(1, 3600).to_string(),
            value2: current_loops.clamp(0, 10000).to_string(),
            edit1: HWND::default(),
            edit2: HWND::default(),
            font: HFONT::default(),
            width: 370,
            height: 205,
            result: None,
        },
    )?;

    let seconds = result.0.trim().parse::<i32>().ok();
    let loops = result.1.trim().parse::<i32>().ok();
    match (seconds, loops) {
        (Some(seconds), Some(loops))
            if (1..=3600).contains(&seconds) && (0..=10000).contains(&loops) =>
        {
            Some((seconds, loops))
        }
        _ => {
            show_message(
                hwnd,
                "Intervalo de Reproduccion",
                "Usa segundos entre 1 y 3600 y repeticiones entre 0 y 10000.",
            );
            None
        }
    }
}

fn prompt_number_dialog(parent: HWND, mut state: NumberDialogState) -> Option<(String, String)> {
    unsafe {
        let instance = GetModuleHandleW(None).ok()?;
        let class_name = w!("PyTaskNumberDialog");
        let cursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let wc = WNDCLASSW {
            hCursor: cursor,
            hInstance: HINSTANCE(instance.0),
            lpszClassName: class_name,
            lpfnWndProc: Some(number_dialog_proc),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(16 as _),
            ..Default::default()
        };
        let _ = RegisterClassW(&wc);

        let ex_style = WS_EX_DLGMODALFRAME;
        let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE;
        let mut window_rect = RECT {
            left: 0,
            top: 0,
            right: state.width,
            bottom: state.height,
        };
        AdjustWindowRectEx(&mut window_rect, style, false, ex_style).ok()?;
        let window_w = window_rect.right - window_rect.left;
        let window_h = window_rect.bottom - window_rect.top;

        let mut parent_rect = RECT::default();
        let _ = GetWindowRect(parent, &mut parent_rect);
        let x = parent_rect.left + ((parent_rect.right - parent_rect.left) - window_w) / 2;
        let y = parent_rect.top + ((parent_rect.bottom - parent_rect.top) - window_h) / 2;
        let title = wide(state.title);
        let state_ptr = &mut state as *mut NumberDialogState;
        let hwnd = CreateWindowExW(
            ex_style,
            class_name,
            PCWSTR(title.as_ptr()),
            style,
            x,
            y,
            window_w,
            window_h,
            parent,
            HMENU::default(),
            instance,
            Some(state_ptr.cast::<c_void>()),
        )
        .ok()?;

        let _ = EnableWindow(parent, false);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let mut msg = MSG::default();
        while IsWindow(hwnd).as_bool() && GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = EnableWindow(parent, true);
        let _ = SetForegroundWindow(parent);
    }

    state.result
}

unsafe extern "system" fn number_dialog_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCCREATE {
        let createstruct = lparam.0 as *const CREATESTRUCTW;
        let state = (*createstruct).lpCreateParams as *mut NumberDialogState;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NumberDialogState;
    if state_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let state = &mut *state_ptr;

    match msg {
        WM_CREATE => {
            create_number_dialog_controls(hwnd, state);
            LRESULT(0)
        }
        WM_COMMAND => {
            match loword(wparam.0 as u32) as usize {
                ID_DIALOG_OK => {
                    state.result = Some((window_text(state.edit1), window_text(state.edit2)));
                    let _ = DestroyWindow(hwnd);
                }
                ID_DIALOG_CANCEL => {
                    let _ = DestroyWindow(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            if !state.font.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(state.font.0));
                state.font = HFONT::default();
            }
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn create_number_dialog_controls(hwnd: HWND, state: &mut NumberDialogState) {
    let instance = GetModuleHandleW(None).unwrap_or_default();
    let edit_style =
        WS_CHILD | WS_VISIBLE | WS_BORDER | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32);
    let button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP;
    let margin = 22;
    let field_w = 132;
    let suffix_x = margin + field_w + 10;
    let button_y = state.height - 40;

    state.font = dialog_font(-15, false);

    let first_label_y = 20;
    let first_edit_y = first_label_y + 22;
    let first_label = create_static(
        hwnd,
        instance,
        state.label1,
        margin,
        first_label_y,
        state.width - 44,
        18,
    );
    set_control_font(first_label, state.font);
    state.edit1 = create_edit(
        hwnd,
        instance,
        &state.value1,
        margin,
        first_edit_y,
        field_w,
        26,
        edit_style,
    );
    set_control_font(state.edit1, state.font);
    if let Some(suffix) = state.suffix1 {
        let suffix_hwnd =
            create_static(hwnd, instance, suffix, suffix_x, first_edit_y + 4, 190, 18);
        set_control_font(suffix_hwnd, state.font);
    }

    if let Some(label2) = state.label2 {
        let second_label_y = first_edit_y + 50;
        let second_edit_y = second_label_y + 22;
        let label_hwnd = create_static(
            hwnd,
            instance,
            label2,
            margin,
            second_label_y,
            state.width - 44,
            18,
        );
        set_control_font(label_hwnd, state.font);
        state.edit2 = create_edit(
            hwnd,
            instance,
            &state.value2,
            margin,
            second_edit_y,
            field_w,
            26,
            edit_style,
        );
        set_control_font(state.edit2, state.font);
        if let Some(suffix) = state.suffix2 {
            let suffix_hwnd =
                create_static(hwnd, instance, suffix, suffix_x, second_edit_y + 4, 190, 18);
            set_control_font(suffix_hwnd, state.font);
        }
    }

    let ok = create_button(
        hwnd,
        instance,
        "Aceptar",
        ID_DIALOG_OK,
        state.width - 178,
        button_y,
        78,
        26,
        button_style | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
    );
    set_control_font(ok, state.font);
    let cancel = create_button(
        hwnd,
        instance,
        "Cancelar",
        ID_DIALOG_CANCEL,
        state.width - 92,
        button_y,
        78,
        26,
        button_style,
    );
    set_control_font(cancel, state.font);
    let _ = SetFocus(state.edit1);
}

unsafe fn dialog_font(height: i32, bold: bool) -> HFONT {
    CreateFontW(
        height,
        0,
        0,
        0,
        if bold {
            FW_BOLD.0 as i32
        } else {
            FW_NORMAL.0 as i32
        },
        0,
        0,
        0,
        0,
        OUT_DEFAULT_PRECIS.0 as u32,
        CLIP_DEFAULT_PRECIS.0 as u32,
        0,
        FF_DONTCARE.0 as u32,
        w!("Segoe UI"),
    )
}

unsafe fn set_control_font(hwnd: HWND, font: HFONT) {
    if !hwnd.is_invalid() && !font.is_invalid() {
        let _ = SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    }
}

unsafe fn create_static(
    parent: HWND,
    instance: windows::Win32::Foundation::HMODULE,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> HWND {
    let text = wide(text);
    CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("STATIC"),
        PCWSTR(text.as_ptr()),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        w,
        h,
        parent,
        HMENU::default(),
        instance,
        None,
    )
    .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_edit(
    parent: HWND,
    instance: windows::Win32::Foundation::HMODULE,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    style: WINDOW_STYLE,
) -> HWND {
    let text = wide(text);
    CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("EDIT"),
        PCWSTR(text.as_ptr()),
        style,
        x,
        y,
        w,
        h,
        parent,
        HMENU::default(),
        instance,
        None,
    )
    .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_button(
    parent: HWND,
    instance: windows::Win32::Foundation::HMODULE,
    text: &str,
    id: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    style: WINDOW_STYLE,
) -> HWND {
    let text = wide(text);
    CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("BUTTON"),
        PCWSTR(text.as_ptr()),
        style,
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as *mut c_void),
        instance,
        None,
    )
    .unwrap_or_default()
}

fn window_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..len.max(0) as usize])
}

fn parse_decimal(value: &str) -> Option<f64> {
    value.trim().replace(',', ".").parse::<f64>().ok()
}

fn trim_number(value: f64) -> String {
    let mut text = format!("{:.2}", value);
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

fn show_about(hwnd: HWND) {
    let text = format!(
        "PyTask\nVersion {APP_VERSION} (Rust + Win32 UI)\n\nAutomatizacion Avanzada de Macros\n\n- Modo Juegos activado (SendInput)\n- Compatible con aplicaciones exigentes\n- Grabacion y reproduccion de macros\n- Hotkeys globales configurables\n\nGitHub: @4ismael1"
    );
    show_message(hwnd, "Acerca de PyTask", &text);
}

fn show_message(hwnd: HWND, title: &str, text: &str) {
    let title = wide(title);
    let text = wide(text);
    let _ = unsafe {
        MessageBoxW(
            hwnd,
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        )
    };
}

fn set_record_hotkey(hwnd: HWND, state: &mut AppState, key: &str) {
    let old = state.settings.record_hotkey.clone();
    state.settings.record_hotkey = key.to_string();
    let result = state.init_hotkeys(hwnd);
    let success = result.is_ok();
    finish_hotkey_change(state, HotkeyTarget::Record, old, key, result);
    if success && !state.status.contains("Error") {
        schedule_status_restore(hwnd, STATUS_TEMP_MS);
    }
}

fn set_play_hotkey(hwnd: HWND, state: &mut AppState, key: &str) {
    let old = state.settings.play_hotkey.clone();
    state.settings.play_hotkey = key.to_string();
    let result = state.init_hotkeys(hwnd);
    let success = result.is_ok();
    finish_hotkey_change(state, HotkeyTarget::Play, old, key, result);
    if success && !state.status.contains("Error") {
        schedule_status_restore(hwnd, STATUS_TEMP_MS);
    }
}

fn finish_hotkey_change(
    state: &mut AppState,
    target: HotkeyTarget,
    old: String,
    key: &str,
    result: Result<(), String>,
) {
    match result {
        Ok(()) => {
            state.status = match target {
                HotkeyTarget::Record => format!("Tecla de grabacion cambiada: {} -> {}", old, key),
                HotkeyTarget::Play => {
                    format!("Tecla de reproduccion cambiada: {} -> {}", old, key)
                }
            };
            state.status = with_save_warning(state.status.clone(), state.save_settings());
        }
        Err(err) => {
            match target {
                HotkeyTarget::Record => state.settings.record_hotkey = old,
                HotkeyTarget::Play => state.settings.play_hotkey = old,
            }
            let status = format!("Error al registrar hotkeys: {}", err);
            state.status = with_save_warning(status, state.save_settings());
        }
    }
}

fn with_save_warning(status: String, result: Result<(), String>) -> String {
    match result {
        Ok(()) => status,
        Err(err) => format!("{} | Error al guardar prefs: {}", status, err),
    }
}

fn with_prefs_open_warning(status: String, err: &str) -> String {
    format!("{} | Error al abrir prefs: {}", status, err)
}

fn set_persistent_status(hwnd: HWND, state: &mut AppState, status: String) {
    cancel_status_restore(hwnd);
    state.status = status;
}

fn show_temporary_status(hwnd: HWND, state: &mut AppState, status: String) {
    state.status = status;
    schedule_status_restore(hwnd, STATUS_TEMP_MS);
}

fn show_temporary_warning(hwnd: HWND, state: &mut AppState, status: String) {
    state.status = status;
    schedule_status_restore(hwnd, STATUS_WARNING_MS);
}

fn show_temporary_unless_error(hwnd: HWND, state: &mut AppState, status: String) {
    if status.contains("Error") {
        set_persistent_status(hwnd, state, status);
    } else {
        show_temporary_status(hwnd, state, status);
    }
}

fn schedule_status_restore(hwnd: HWND, delay_ms: u32) {
    unsafe {
        let _ = KillTimer(hwnd, STATUS_TIMER_ID);
        let _ = SetTimer(hwnd, STATUS_TIMER_ID, delay_ms, None);
    }
}

fn cancel_status_restore(hwnd: HWND) {
    let _ = unsafe { KillTimer(hwnd, STATUS_TIMER_ID) };
}

fn restore_idle_status(state: &mut AppState) -> bool {
    if state.is_recording || state.is_playing {
        return false;
    }
    state.status = default_status(&state.settings);
    true
}

fn recording_status(settings: &Settings) -> String {
    format!(
        "Grabando... (Presiona {} para detener)",
        settings.record_hotkey
    )
}

fn open_macro(hwnd: HWND, state: &mut AppState) {
    if let Some(path) = open_macro_dialog(hwnd)
        && load_macro_from_path(state, &path)
    {
        set_title(hwnd, Some(&path));
        schedule_status_restore(hwnd, STATUS_TEMP_MS);
    }
}

fn load_macro_from_path(state: &mut AppState, path: &Path) -> bool {
    match macro_io::load_macro(path) {
        Ok(events) if !events.is_empty() => {
            state.current_events = events;
            state.current_file = Some(path.to_path_buf());
            state.status = format!("Cargado: {} eventos", state.current_events.len());
            state.refresh_buttons();
            true
        }
        Ok(_) => {
            state.status = "El archivo seleccionado no contiene eventos.".to_string();
            false
        }
        Err(err) => {
            state.status = format!("Error al cargar macro: {}", err);
            false
        }
    }
}

fn save_macro(hwnd: HWND, state: &mut AppState) {
    if state.current_events.is_empty() {
        show_temporary_warning(hwnd, state, "No hay macro para guardar".to_string());
        return;
    }

    let path = if let Some(path) = &state.current_file {
        path.clone()
    } else if save_needs_dialog(state) {
        if let Some(path) = save_macro_dialog(hwnd) {
            path
        } else {
            return;
        }
    } else {
        return;
    };

    if save_macro_to_path(state, &path) {
        set_title(hwnd, Some(&path));
        schedule_status_restore(hwnd, STATUS_TEMP_MS);
    }
}

fn save_needs_dialog(state: &AppState) -> bool {
    !state.current_events.is_empty() && state.current_file.is_none()
}

fn save_macro_to_path(state: &mut AppState, path: &Path) -> bool {
    if state.current_events.is_empty() {
        state.status = "No hay macro para guardar".to_string();
        return false;
    }

    match macro_io::save_macro(path, &state.current_events) {
        Ok(()) => {
            state.current_file = Some(path.to_path_buf());
            state.status = format!("Guardado: {}", display_name(path));
            true
        }
        Err(err) => {
            state.status = format!("Error al guardar: {}", err);
            false
        }
    }
}

fn toggle_recording(hwnd: HWND, state: &mut AppState) {
    if !state.is_recording {
        if state.is_playing {
            show_temporary_warning(
                hwnd,
                state,
                "No se puede grabar mientras se reproduce".to_string(),
            );
            return;
        }
        match state
            .recorder
            .start_ignoring_hotkeys(&[&state.settings.record_hotkey, &state.settings.play_hotkey])
        {
            Ok(()) => {
                state.is_recording = true;
                state.current_file = None;
                state.current_events.clear();
                let status = recording_status(&state.settings);
                set_persistent_status(hwnd, state, status);
                state.refresh_buttons();
                set_title(hwnd, None);
            }
            Err(err) => {
                set_persistent_status(hwnd, state, format!("Error al iniciar grabacion: {}", err));
            }
        }
    } else {
        let events = state.recorder.stop();
        state.is_recording = false;
        state.current_events = events;
        if state.current_events.is_empty() {
            show_temporary_warning(hwnd, state, "No se grabaron eventos".to_string());
        } else {
            show_temporary_status(
                hwnd,
                state,
                format!("Grabados {} eventos", state.current_events.len()),
            );
        }
        state.refresh_buttons();
    }
}

fn toggle_playback(hwnd: HWND, state: &mut AppState) {
    if state.is_recording {
        show_temporary_warning(
            hwnd,
            state,
            "No se puede reproducir mientras se graba".to_string(),
        );
        return;
    }
    if state.current_events.is_empty() {
        show_temporary_warning(hwnd, state, "No hay macro cargada".to_string());
        return;
    }
    if state.is_playing {
        if state.playback_stop_requested {
            state.status = "Detenido".to_string();
            return;
        }
        state.player.stop();
        state.playback_stop_requested = true;
        set_persistent_status(hwnd, state, "Detenido".to_string());
        state.refresh_buttons();
        return;
    }

    state.is_playing = true;
    state.playback_stop_requested = false;
    state.playback_generation = state.playback_generation.wrapping_add(1);
    let playback_generation = state.playback_generation;
    state.refresh_buttons();
    let events = state.current_events.clone();
    let speed = state.play_speed;
    let loops = state.playback_loops;
    let interval_mode = state.interval_mode;
    let interval_seconds = state.interval_seconds;
    let use_sendinput = state.settings.use_sendinput;
    let hotkey = state.settings.play_hotkey.clone();
    let status = playback_status(state, &hotkey);
    set_persistent_status(hwnd, state, status);
    state.player.prepare_playback();
    let player = state.player.playback_instance();

    let hwnd_value = hwnd.0 as isize;
    thread::spawn(move || {
        let completed = player.play_blocking(
            events,
            speed,
            loops,
            interval_mode,
            interval_seconds,
            use_sendinput,
        );
        let hwnd = HWND(hwnd_value as *mut c_void);
        let _ = unsafe {
            PostMessageW(
                hwnd,
                WM_PLAYBACK_FINISHED,
                WPARAM(usize::from(completed)),
                LPARAM(playback_generation as isize),
            )
        };
    });
}

fn playback_status(state: &AppState, hotkey: &str) -> String {
    let infinite = state.playback_loops == 0;
    let mode = if state.interval_mode {
        if infinite {
            format!("INTERVALO {}s (INFINITO)", state.interval_seconds)
        } else {
            format!(
                "INTERVALO {}s ({} veces)",
                state.interval_seconds, state.playback_loops
            )
        }
    } else if infinite {
        "INFINITO (sin pausa)".to_string()
    } else if state.playback_loops == 1 {
        "UNA VEZ".to_string()
    } else {
        format!("{} VECES (sin pausa)", state.playback_loops)
    };
    format!("{} a {}x ({}=Detener)", mode, state.play_speed, hotkey)
}

fn default_status(settings: &Settings) -> String {
    format!(
        "Listo | Grabar: {} | Reproducir: {}",
        settings.record_hotkey, settings.play_hotkey
    )
}

fn set_title(hwnd: HWND, path: Option<&Path>) {
    let title = path
        .map(|path| format!("PyTask - {}", display_name(path)))
        .unwrap_or_else(|| "PyTask".to_string());
    let title = wide(&title);
    let _ = unsafe { SetWindowTextW(hwnd, PCWSTR(title.as_ptr())) };
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("macro.macro")
        .to_string()
}

fn open_macro_dialog(hwnd: HWND) -> Option<PathBuf> {
    file_dialog(hwnd, false)
}

fn save_macro_dialog(hwnd: HWND) -> Option<PathBuf> {
    file_dialog(hwnd, true)
}

fn file_dialog(hwnd: HWND, save: bool) -> Option<PathBuf> {
    const MAX_PATH_BUF: usize = 1024;
    let mut file = [0u16; MAX_PATH_BUF];
    if save {
        let suggested = wide("macro.macro");
        file[..suggested.len().min(MAX_PATH_BUF)]
            .copy_from_slice(&suggested[..suggested.len().min(MAX_PATH_BUF)]);
    }
    let filter = wide("Archivos de Macro (*.macro)\0*.macro\0Todos los Archivos\0*.*\0");
    let title = wide(if save { "Guardar Macro" } else { "Abrir Macro" });
    let def_ext = wide("macro");
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: hwnd,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(file.as_mut_ptr()),
        nMaxFile: MAX_PATH_BUF as u32,
        lpstrTitle: PCWSTR(title.as_ptr()),
        lpstrDefExt: PCWSTR(def_ext.as_ptr()),
        Flags: OFN_EXPLORER
            | OFN_PATHMUSTEXIST
            | OFN_NOCHANGEDIR
            | OFN_HIDEREADONLY
            | if save {
                OFN_OVERWRITEPROMPT
            } else {
                OFN_FILEMUSTEXIST
            },
        ..Default::default()
    };

    let ok = unsafe {
        if save {
            GetSaveFileNameW(&mut ofn).as_bool()
        } else {
            GetOpenFileNameW(&mut ofn).as_bool()
        }
    };
    if !ok {
        return None;
    }
    let len = file.iter().position(|ch| *ch == 0).unwrap_or(file.len());
    Some(PathBuf::from(String::from_utf16_lossy(&file[..len])))
}

unsafe fn paint(hwnd: HWND, state: &AppState) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);

    let mem_dc = CreateCompatibleDC(hdc);
    let mem_bitmap = CreateCompatibleBitmap(hdc, client_w(), client_h());
    let old_bitmap = SelectObject(mem_dc, HGDIOBJ(mem_bitmap.0));

    draw_background(mem_dc);
    draw_buttons(mem_dc, state);
    if state.settings.show_captions {
        draw_status(mem_dc, state);
    }

    let _ = BitBlt(hdc, 0, 0, client_w(), client_h(), mem_dc, 0, 0, SRCCOPY);

    SelectObject(mem_dc, old_bitmap);
    let _ = DeleteObject(HGDIOBJ(mem_bitmap.0));
    let _ = DeleteDC(mem_dc);
    let _ = EndPaint(hwnd, &ps);
}

unsafe fn draw_background(hdc: HDC) {
    fill_rect(
        hdc,
        RECT {
            left: 0,
            top: 0,
            right: client_w(),
            bottom: client_h(),
        },
        rgb(0xff, 0xff, 0xff),
    );
}

unsafe fn draw_buttons(hdc: HDC, state: &AppState) {
    for (index, button) in state.buttons.iter().enumerate() {
        let rect = AppState::button_rect(index);
        let hovered = state.hover == Some(button.kind) && button.enabled;
        let pressed = state.pressed == Some(button.kind) && button.enabled;
        let active = matches!(button.kind, ButtonKind::Rec) && state.is_recording
            || matches!(button.kind, ButtonKind::Play) && state.is_playing;
        let label = if matches!(button.kind, ButtonKind::Play) && state.is_playing {
            "Stop"
        } else {
            button.label
        };
        let icon = if matches!(button.kind, ButtonKind::Play) && state.is_playing {
            2
        } else {
            button.icon
        };
        let bg = if !button.enabled {
            rgb(0xf8, 0xf8, 0xf8)
        } else if active {
            rgb(0xff, 0x44, 0x44)
        } else if pressed {
            rgb(0xcc, 0xe4, 0xf7)
        } else if hovered {
            rgb(0xe8, 0xf4, 0xff)
        } else {
            rgb(0xff, 0xff, 0xff)
        };
        let border = if active {
            rgb(0xcc, 0x00, 0x00)
        } else if hovered && button.enabled {
            rgb(0x00, 0x78, 0xd7)
        } else {
            rgb(0xd0, 0xd0, 0xd0)
        };
        let text = if !button.enabled {
            rgb(0xaa, 0xaa, 0xaa)
        } else if active {
            rgb(0xff, 0xff, 0xff)
        } else if hovered || pressed {
            rgb(0x00, 0x78, 0xd7)
        } else {
            rgb(0x33, 0x33, 0x33)
        };

        rounded_rect(hdc, rect, bg, border, 2);

        if button.split {
            let divider_x = rect.right - 13;
            fill_rect(
                hdc,
                RECT {
                    left: divider_x,
                    top: rect.top,
                    right: divider_x + 1,
                    bottom: rect.bottom,
                },
                rgb(0xd0, 0xd0, 0xd0),
            );
            draw_arrow(hdc, divider_x + 4, rect.top + 31, button.enabled);
        }

        let icon_size = icon_size();
        let icon_x =
            rect.left + (button_w() - icon_size) / 2 - if button.split { ui(4) } else { 0 };
        let icon_y = rect.top + ui(10);
        draw_icon(
            hdc,
            &state.icons[icon],
            icon_x,
            icon_y,
            icon_size,
            bg,
            if button.enabled { 1.0 } else { 0.42 },
        );

        draw_label(
            hdc,
            label,
            RECT {
                left: rect.left + ui(2),
                top: rect.top + ui(43),
                right: rect.right - if button.split { ui(13) } else { ui(2) },
                bottom: rect.bottom - ui(5),
            },
            text,
            true,
        );
    }
}

unsafe fn draw_status(hdc: HDC, state: &AppState) {
    fill_rect(
        hdc,
        RECT {
            left: 0,
            top: status_y(),
            right: client_w(),
            bottom: status_y() + 1,
        },
        rgb(0xd0, 0xd0, 0xd0),
    );
    let logo_size = ui(BASE_STATUS_ICON_SIZE);
    draw_icon(
        hdc,
        &state.icons[APP_LOGO_ICON_INDEX],
        ui(5),
        status_y() + ui(3),
        logo_size,
        rgb(0xff, 0xff, 0xff),
        1.0,
    );
    draw_label(
        hdc,
        &state.status,
        RECT {
            left: ui(24),
            top: status_y() + ui(2),
            right: client_w() - ui(5),
            bottom: client_h(),
        },
        rgb(0x33, 0x33, 0x33),
        false,
    );
}

fn apply_always_on_top(hwnd: HWND, enabled: bool) {
    let insert_after = if enabled {
        HWND_TOPMOST
    } else {
        HWND_NOTOPMOST
    };
    let _ = unsafe { SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE) };
}

unsafe fn draw_label(hdc: HDC, text: &str, mut rect: RECT, color: COLORREF, bold: bool) {
    let font = CreateFontW(
        -ui(11),
        0,
        0,
        0,
        if bold {
            FW_BOLD.0 as i32
        } else {
            FW_NORMAL.0 as i32
        },
        0,
        0,
        0,
        0,
        OUT_DEFAULT_PRECIS.0 as u32,
        CLIP_DEFAULT_PRECIS.0 as u32,
        0,
        FF_DONTCARE.0 as u32,
        w!("Segoe UI"),
    );
    let old = SelectObject(hdc, HGDIOBJ(font.0));
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, color);

    let mut wide_text = wide(text);
    let flags = if bold {
        DT_CENTER | DT_SINGLELINE | DT_VCENTER
    } else {
        DT_SINGLELINE | DT_VCENTER
    };
    DrawTextW(hdc, &mut wide_text, &mut rect, flags);
    SelectObject(hdc, old);
    let _ = DeleteObject(HGDIOBJ(font.0));
}

unsafe fn draw_icon(
    hdc: HDC,
    icon: &IconAsset,
    x: i32,
    y: i32,
    size: i32,
    bg: COLORREF,
    opacity: f32,
) {
    let bg_r = (bg.0 & 0xff) as f32;
    let bg_g = ((bg.0 >> 8) & 0xff) as f32;
    let bg_b = ((bg.0 >> 16) & 0xff) as f32;
    let mut dib = Vec::with_capacity((size * size * 4) as usize);

    for pixel in icon.rgba.chunks_exact(4) {
        let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        let alpha = (a as f32 / 255.0) * opacity;
        let out_r = (r as f32 * alpha + bg_r * (1.0 - alpha)).round() as u8;
        let out_g = (g as f32 * alpha + bg_g * (1.0 - alpha)).round() as u8;
        let out_b = (b as f32 * alpha + bg_b * (1.0 - alpha)).round() as u8;
        dib.extend_from_slice(&[out_b, out_g, out_r, 0]);
    }

    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    StretchDIBits(
        hdc,
        x,
        y,
        size,
        size,
        0,
        0,
        size,
        size,
        Some(dib.as_ptr().cast::<c_void>()),
        &info,
        DIB_RGB_COLORS,
        SRCCOPY,
    );
}

unsafe fn draw_arrow(hdc: HDC, x: i32, y: i32, enabled: bool) {
    let color = if enabled {
        rgb(0x66, 0x66, 0x66)
    } else {
        rgb(0xc8, 0xc8, 0xc8)
    };
    fill_rect(
        hdc,
        RECT {
            left: x,
            top: y,
            right: x + 7,
            bottom: y + 1,
        },
        color,
    );
    fill_rect(
        hdc,
        RECT {
            left: x + 1,
            top: y + 1,
            right: x + 6,
            bottom: y + 2,
        },
        color,
    );
    fill_rect(
        hdc,
        RECT {
            left: x + 2,
            top: y + 2,
            right: x + 5,
            bottom: y + 3,
        },
        color,
    );
}

unsafe fn rounded_rect(hdc: HDC, rect: RECT, bg: COLORREF, border: COLORREF, border_width: i32) {
    let brush = CreateSolidBrush(bg);
    let pen = CreatePen(PS_SOLID, border_width, border);
    let old_brush = SelectObject(hdc, HGDIOBJ(brush.0));
    let old_pen = SelectObject(hdc, HGDIOBJ(pen.0));
    let _ = windows::Win32::Graphics::Gdi::RoundRect(
        hdc,
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
        ui(8),
        ui(8),
    );
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    let _ = DeleteObject(HGDIOBJ(pen.0));
    let _ = DeleteObject(HGDIOBJ(brush.0));
}

unsafe fn fill_rect(hdc: HDC, rect: RECT, color: COLORREF) {
    let brush = CreateSolidBrush(color);
    FillRect(hdc, &rect, brush);
    let _ = DeleteObject(HGDIOBJ(brush.0));
}

fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | ((g as u32) << 8) | ((b as u32) << 16))
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn loword(value: u32) -> u32 {
    value & 0xffff
}

fn hiword(value: u32) -> u32 {
    (value >> 16) & 0xffff
}

#[cfg(test)]
mod tests {
    use super::{
        AppState, ButtonKind, HotkeyTarget, IconAsset, ToolbarButton, bool_checked, default_status,
        finish_hotkey_change, handle_menu_command, hotkey_checked, interval_checked,
        load_macro_from_path, loop_checked, main_window_style, parse_decimal, playback_status,
        recording_status, restore_idle_status, save_macro_to_path, save_needs_dialog,
        set_interval_mode, set_playback_loops, set_quick_interval, set_speed, trim_number,
        ui_scale_for_dpi, with_prefs_open_warning, with_save_warning,
    };
    use crate::engine::{MacroPlayer, MacroRecorder};
    use crate::model::{MacroEvent, Settings};
    use crate::settings_db::SettingsDatabase;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        MENU_ITEM_FLAGS, MF_CHECKED, WINDOW_STYLE, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
    };

    fn test_state() -> AppState {
        AppState {
            buttons: vec![
                ToolbarButton {
                    kind: ButtonKind::Open,
                    label: "Open",
                    icon: 0,
                    enabled: true,
                    split: false,
                },
                ToolbarButton {
                    kind: ButtonKind::Save,
                    label: "Save",
                    icon: 1,
                    enabled: false,
                    split: false,
                },
                ToolbarButton {
                    kind: ButtonKind::Rec,
                    label: "Rec",
                    icon: 2,
                    enabled: true,
                    split: false,
                },
                ToolbarButton {
                    kind: ButtonKind::Play,
                    label: "Play",
                    icon: 3,
                    enabled: false,
                    split: false,
                },
                ToolbarButton {
                    kind: ButtonKind::Prefs,
                    label: "Prefs",
                    icon: 4,
                    enabled: true,
                    split: false,
                },
            ],
            icons: vec![IconAsset { rgba: Vec::new() }],
            hover: None,
            pressed: None,
            tracking_leave: false,
            status: String::new(),
            db: None,
            settings: Settings::default(),
            recorder: MacroRecorder::new(),
            player: MacroPlayer::new(),
            hotkeys: None,
            current_events: Vec::new(),
            current_file: None,
            play_speed: 1.0,
            custom_speed: 8.0,
            playback_loops: 1,
            interval_mode: false,
            interval_seconds: 5,
            is_recording: false,
            is_playing: false,
            playback_stop_requested: false,
            playback_generation: 0,
        }
    }

    fn button_enabled(state: &AppState, kind: ButtonKind) -> bool {
        state
            .buttons
            .iter()
            .find(|button| button.kind == kind)
            .map(|button| button.enabled)
            .unwrap()
    }

    fn is_checked(flags: MENU_ITEM_FLAGS) -> bool {
        flags & MF_CHECKED == MF_CHECKED
    }

    fn temp_path(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "pytask-rust-ui-{}-{}-{}",
            std::process::id(),
            suffix,
            name
        ))
    }

    #[test]
    fn parses_decimal_values_for_native_dialogs() {
        assert_eq!(parse_decimal("8"), Some(8.0));
        assert_eq!(parse_decimal("0.5"), Some(0.5));
        assert_eq!(parse_decimal("1,25"), Some(1.25));
        assert_eq!(parse_decimal("abc"), None);
    }

    #[test]
    fn trims_display_numbers_without_losing_decimals() {
        assert_eq!(trim_number(8.0), "8");
        assert_eq!(trim_number(0.5), "0.5");
        assert_eq!(trim_number(1.25), "1.25");
    }

    #[test]
    fn keeps_user_approved_compact_dpi_scaling() {
        assert_eq!(ui_scale_for_dpi(96), 1.0);
        assert!((ui_scale_for_dpi(120) - 1.125).abs() < f64::EPSILON);
        assert!((ui_scale_for_dpi(144) - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn main_window_style_keeps_native_minimize_without_maximize() {
        let style = main_window_style();

        assert_eq!(style & WS_CAPTION, WS_CAPTION);
        assert_eq!(style & WS_MINIMIZEBOX, WS_MINIMIZEBOX);
        assert_eq!(style & WS_MAXIMIZEBOX, WINDOW_STYLE(0));
    }

    #[test]
    fn refresh_buttons_tracks_macro_recording_and_playback_state() {
        let mut state = test_state();

        state.refresh_buttons();
        assert!(!button_enabled(&state, ButtonKind::Save));
        assert!(!button_enabled(&state, ButtonKind::Play));
        assert!(button_enabled(&state, ButtonKind::Rec));

        state.current_events.push(MacroEvent {
            event_type: "noop".to_string(),
            ..Default::default()
        });
        state.refresh_buttons();
        assert!(button_enabled(&state, ButtonKind::Save));
        assert!(button_enabled(&state, ButtonKind::Play));
        assert!(button_enabled(&state, ButtonKind::Rec));

        state.is_recording = true;
        state.refresh_buttons();
        assert!(!button_enabled(&state, ButtonKind::Play));
        assert!(button_enabled(&state, ButtonKind::Rec));

        state.is_recording = false;
        state.is_playing = true;
        state.refresh_buttons();
        assert!(!button_enabled(&state, ButtonKind::Save));
        assert!(button_enabled(&state, ButtonKind::Play));
        assert!(!button_enabled(&state, ButtonKind::Rec));
    }

    #[test]
    fn preference_helpers_match_csharp_status_and_modes() {
        let mut state = test_state();

        set_speed(&mut state, 2.0);
        assert_eq!(state.play_speed, 2.0);
        assert_eq!(state.status, "Velocidad: 2x");

        set_interval_mode(&mut state, 5, 0);
        assert!(state.interval_mode);
        assert_eq!(state.interval_seconds, 5);
        assert_eq!(state.playback_loops, 0);
        assert_eq!(state.status, "Modo: Reproduccion infinita cada 5s");

        let current_loops = state.playback_loops;
        set_interval_mode(&mut state, 0, current_loops);
        assert!(!state.interval_mode);
        assert_eq!(state.playback_loops, 0);
        assert_eq!(state.status, "Modo: Reproduccion infinita sin intervalo");

        set_playback_loops(&mut state, 3);
        assert_eq!(state.playback_loops, 3);
        assert_eq!(state.status, "Modo: Reproducir 3 veces sin intervalo");
        set_quick_interval(&mut state, 1);
        assert!(state.interval_mode);
        assert_eq!(state.playback_loops, 3);
        assert_eq!(state.status, "Modo: Reproducir 3 veces cada 1s");

        set_playback_loops(&mut state, 0);
        set_quick_interval(&mut state, 10);
        assert_eq!(state.playback_loops, 0);
        assert_eq!(state.status, "Modo: Reproduccion infinita cada 10s");
    }

    #[test]
    fn preference_menu_checkmarks_track_current_state() {
        let mut state = test_state();
        state.settings.record_hotkey = "F8".to_string();
        state.settings.play_hotkey = "F12".to_string();
        state.settings.always_on_top = true;
        state.settings.show_captions = false;
        state.settings.use_sendinput = true;
        state.interval_mode = true;
        state.interval_seconds = 5;

        assert!(is_checked(hotkey_checked(
            &state.settings.record_hotkey,
            "F8"
        )));
        assert!(!is_checked(hotkey_checked(
            &state.settings.record_hotkey,
            "F9"
        )));
        assert!(is_checked(hotkey_checked(
            &state.settings.play_hotkey,
            "F12"
        )));
        assert!(is_checked(interval_checked(&state, 5)));
        assert!(!is_checked(interval_checked(&state, 10)));
        assert!(is_checked(loop_checked(&state, 1)));
        assert!(!is_checked(loop_checked(&state, 0)));
        state.playback_loops = 4;
        assert!(is_checked(loop_checked(&state, 4)));
        assert!(is_checked(bool_checked(state.settings.always_on_top)));
        assert!(!is_checked(bool_checked(state.settings.show_captions)));
        assert!(is_checked(bool_checked(state.settings.use_sendinput)));
    }

    #[test]
    fn prefs_toggle_commands_persist_boolean_options() {
        let path = temp_path("prefs-toggles.db");
        let mut state = test_state();
        state.db = Some(SettingsDatabase::open(path.clone()).unwrap());

        handle_menu_command(HWND::default(), &mut state, 602);
        assert!(!state.settings.show_captions);
        assert_eq!(state.status, "Barra de Estado: OFF");

        handle_menu_command(HWND::default(), &mut state, 603);
        assert!(!state.settings.use_sendinput);
        assert_eq!(state.status, "Modo Normal activado");

        let loaded = state.db.as_ref().unwrap().load_settings();
        drop(state);
        let _ = fs::remove_file(path);

        assert!(!loaded.show_captions);
        assert!(!loaded.use_sendinput);
        assert_eq!(loaded.record_hotkey, "F9");
        assert_eq!(loaded.play_hotkey, "F10");
    }

    #[test]
    fn save_warning_preserves_status_and_reports_persistence_errors() {
        assert_eq!(with_save_warning("Listo".to_string(), Ok(())), "Listo");
        assert_eq!(
            with_save_warning("Listo".to_string(), Err("disk full".to_string())),
            "Listo | Error al guardar prefs: disk full"
        );
        assert_eq!(
            with_prefs_open_warning("Listo".to_string(), "database locked"),
            "Listo | Error al abrir prefs: database locked"
        );
    }

    #[test]
    fn status_strings_use_configured_hotkeys_and_playback_mode() {
        let mut state = test_state();
        state.settings.record_hotkey = "F8".to_string();
        state.settings.play_hotkey = "F12".to_string();
        assert_eq!(
            default_status(&state.settings),
            "Listo | Grabar: F8 | Reproducir: F12"
        );

        state.play_speed = 100.0;
        state.playback_loops = 3;
        assert_eq!(
            playback_status(&state, "F12"),
            "3 VECES (sin pausa) a 100x (F12=Detener)"
        );

        state.interval_mode = true;
        state.interval_seconds = 10;
        assert_eq!(
            playback_status(&state, "F12"),
            "INTERVALO 10s (3 veces) a 100x (F12=Detener)"
        );
    }

    #[test]
    fn restoring_temporary_status_returns_to_updated_idle_status() {
        let mut state = test_state();
        state.settings.record_hotkey = "F8".to_string();
        state.settings.play_hotkey = "F12".to_string();
        state.status = "Tecla de grabacion cambiada: F9 -> F8".to_string();

        assert!(restore_idle_status(&mut state));

        assert_eq!(state.status, "Listo | Grabar: F8 | Reproducir: F12");
    }

    #[test]
    fn restoring_temporary_status_does_not_override_active_modes() {
        let mut state = test_state();
        state.is_recording = true;
        state.status = recording_status(&state.settings);
        assert!(!restore_idle_status(&mut state));
        assert_eq!(state.status, "Grabando... (Presiona F9 para detener)");

        state.is_recording = false;
        state.is_playing = true;
        state.status = playback_status(&state, "F10");
        let playing_status = state.status.clone();
        assert!(!restore_idle_status(&mut state));
        assert_eq!(state.status, playing_status);
    }

    #[test]
    fn hotkey_change_success_updates_status_and_keeps_new_key() {
        let mut state = test_state();
        let old = state.settings.record_hotkey.clone();
        state.settings.record_hotkey = "F8".to_string();

        finish_hotkey_change(&mut state, HotkeyTarget::Record, old, "F8", Ok(()));

        assert_eq!(state.settings.record_hotkey, "F8");
        assert_eq!(state.status, "Tecla de grabacion cambiada: F9 -> F8");
    }

    #[test]
    fn hotkey_change_success_persists_to_settings_database() {
        let path = temp_path("hotkey-change.db");
        let mut state = test_state();
        state.db = Some(SettingsDatabase::open(path.clone()).unwrap());

        let old = state.settings.play_hotkey.clone();
        state.settings.play_hotkey = "F12".to_string();
        finish_hotkey_change(&mut state, HotkeyTarget::Play, old, "F12", Ok(()));

        let loaded = state.db.as_ref().unwrap().load_settings();
        drop(state);
        let _ = fs::remove_file(path);

        assert_eq!(loaded.play_hotkey, "F12");
        assert_eq!(loaded.record_hotkey, "F9");
    }

    #[test]
    fn hotkey_change_failure_rolls_back_key() {
        let mut state = test_state();
        let old = state.settings.play_hotkey.clone();
        state.settings.play_hotkey = "F12".to_string();

        finish_hotkey_change(
            &mut state,
            HotkeyTarget::Play,
            old,
            "F12",
            Err("hook unavailable".to_string()),
        );

        assert_eq!(state.settings.play_hotkey, "F10");
        assert_eq!(state.status, "Error al registrar hotkeys: hook unavailable");
    }

    #[test]
    fn open_save_helpers_update_state_like_dialog_flow() {
        let save_path = temp_path("saved.macro");
        let mut state = test_state();
        state.current_events = vec![MacroEvent {
            event_type: "mouse_move".to_string(),
            timestamp: 0.25,
            x: Some(100),
            y: Some(200),
            ..Default::default()
        }];

        assert!(save_macro_to_path(&mut state, &save_path));
        assert_eq!(state.current_file.as_deref(), Some(save_path.as_path()));
        assert_eq!(
            state.status,
            format!(
                "Guardado: {}",
                save_path.file_name().unwrap().to_string_lossy()
            )
        );

        let mut loaded_state = test_state();
        assert!(load_macro_from_path(&mut loaded_state, &save_path));
        let _ = fs::remove_file(&save_path);

        assert_eq!(loaded_state.current_events.len(), 1);
        assert_eq!(loaded_state.current_events[0].x, Some(100));
        assert_eq!(
            loaded_state.current_file.as_deref(),
            Some(save_path.as_path())
        );
        assert_eq!(loaded_state.status, "Cargado: 1 eventos");
        assert!(button_enabled(&loaded_state, ButtonKind::Save));
        assert!(button_enabled(&loaded_state, ButtonKind::Play));
    }

    #[test]
    fn save_prompts_only_for_new_macro_without_existing_path() {
        let mut state = test_state();
        assert!(!save_needs_dialog(&state));

        state.current_events.push(MacroEvent {
            event_type: "mouse_move".to_string(),
            ..Default::default()
        });
        assert!(save_needs_dialog(&state));

        state.current_file = Some(temp_path("existing.macro"));
        assert!(!save_needs_dialog(&state));
    }

    #[test]
    fn open_helper_rejects_empty_and_invalid_macro_files() {
        let empty_path = temp_path("empty.macro");
        let invalid_path = temp_path("invalid.macro");
        fs::write(&empty_path, r#"{"events":[]}"#).unwrap();
        fs::write(&invalid_path, "{not-json").unwrap();

        let mut state = test_state();
        assert!(!load_macro_from_path(&mut state, &empty_path));
        assert_eq!(state.status, "El archivo seleccionado no contiene eventos.");
        assert!(state.current_events.is_empty());

        assert!(!load_macro_from_path(&mut state, &invalid_path));
        assert!(state.status.starts_with("Error al cargar macro:"));
        assert!(state.current_events.is_empty());

        let _ = fs::remove_file(&empty_path);
        let _ = fs::remove_file(&invalid_path);
    }

    #[test]
    fn save_helper_refuses_empty_macro() {
        let path = temp_path("empty-save.macro");
        let mut state = test_state();

        assert!(!save_macro_to_path(&mut state, &path));
        assert_eq!(state.status, "No hay macro para guardar");
        assert!(state.current_file.is_none());
        assert!(!path.exists());
    }
}
