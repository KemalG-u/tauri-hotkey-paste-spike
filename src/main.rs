// Tauri v2 spike: global push-to-talk style hotkey + clipboard-first text placement chain.
// Design: PRESSED = recording would start (dictation), RELEASED = transcript ready -> place text.
// The release-triggered paste sends Ctrl+V only after releasing any stuck modifier keys.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::OpenOptions;
use std::io::Write as _;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
};

const TEST_TEXT: &str =
    "Pijamalı hasta yağız şoföre çabucak güvendi — ığdır İĞŞÇÖÜ ıspanak 3,5 TL %18";
const DEFAULT_LOG_PATH: &str = "events.log";

/// Log file location: `SPIKE_TAURI_LOG` if set, otherwise `events.log` in the working directory.
fn log_path() -> String {
    std::env::var("SPIKE_TAURI_LOG").unwrap_or_else(|_| DEFAULT_LOG_PATH.to_string())
}

static PRESS_AT: Mutex<Option<Instant>> = Mutex::new(None);

fn now_ms() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}.{:03}", d.as_secs(), d.subsec_millis())
}

fn log_line(s: &str) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path()) {
        let _ = writeln!(f, "{}", s);
    }
}

fn ki(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// First release every modifier (they may be stuck), then send a clean Ctrl+V.
fn send_ctrl_v() {
    const VK_SHIFT: u16 = 0x10;
    const VK_CONTROL: u16 = 0x11;
    const VK_MENU: u16 = 0x12; // ALT
    const VK_LWIN: u16 = 0x5B;
    const VK_V: u16 = 0x56;
    let inputs = [
        ki(VK_MENU, true),
        ki(VK_SHIFT, true),
        ki(VK_LWIN, true),
        ki(VK_CONTROL, true),
        ki(VK_CONTROL, false),
        ki(VK_V, false),
        ki(VK_V, true),
        ki(VK_CONTROL, true),
    ];
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

/// Placement chain: SAVE clipboard -> write text -> Ctrl+V -> wait -> RESTORE clipboard.
fn placement_chain() {
    let t0 = Instant::now();
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            log_line(&format!("[{}] CHAIN_ERR clipboard_new {e}", now_ms()));
            return;
        }
    };
    let saved = clipboard.get_text().unwrap_or_default();
    // Only the length is logged: the saved clipboard may hold private data.
    log_line(&format!("[{}] CHAIN saved_clipboard_chars={}", now_ms(), saved.chars().count()));

    if let Err(e) = clipboard.set_text(TEST_TEXT.to_string()) {
        log_line(&format!("[{}] CHAIN_ERR set_text {e}", now_ms()));
        return;
    }
    std::thread::sleep(Duration::from_millis(40));

    send_ctrl_v();
    let t_to_paste = t0.elapsed().as_millis();

    // Wait until the target application has read the clipboard, THEN restore the old contents.
    std::thread::sleep(Duration::from_millis(150));
    if let Err(e) = clipboard.set_text(saved.clone()) {
        log_line(&format!("[{}] CHAIN_ERR restore {e}", now_ms()));
    }
    let t_total = t0.elapsed().as_millis();
    log_line(&format!(
        "[{}] CHAIN_DONE t_to_paste_ms={} t_total_ms={} restored_chars={}",
        now_ms(),
        t_to_paste,
        t_total,
        saved.chars().count()
    ));
}

fn main() {
    log_line(&format!("[{}] APP_STARTED", now_ms()));
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, _shortcut, event| match event.state() {
                    ShortcutState::Pressed => {
                        *PRESS_AT.lock().unwrap() = Some(Instant::now());
                        log_line(&format!("[{}] PRESSED", now_ms()));
                    }
                    ShortcutState::Released => {
                        let held = PRESS_AT
                            .lock()
                            .unwrap()
                            .take()
                            .map(|t| t.elapsed().as_millis())
                            .unwrap_or(0);
                        log_line(&format!("[{}] RELEASED held_ms={}", now_ms(), held));
                        placement_chain();
                    }
                })
                .build(),
        )
        .setup(|app| {
            let sc = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyD);
            app.global_shortcut().register(sc)?;
            log_line(&format!("[{}] SHORTCUT_REGISTERED ctrl+alt+d", now_ms()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("tauri run failed");
}
