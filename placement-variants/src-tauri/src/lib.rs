// Tauri spike: global hotkeys + text placement into the focused window.
// No speech recognition here; only the shell, the placement methods and
// Turkish-character integrity.

use std::fs::OpenOptions;
use std::io::Write as _;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde_json::json;

// Turkish integrity test text: diacritics + number/currency/percent.
const TEST_TEXT: &str =
    "Pijamalı hasta yağız şoföre çabucak güvendi — ığdır İĞŞÇÖÜ ıspanak 3,5 TL %18";

/// Log file location: `PLACEMENT_SPIKE_LOG` if set, otherwise
/// `placement-log.jsonl` in the working directory.
fn log_path() -> String {
    std::env::var("PLACEMENT_SPIKE_LOG").unwrap_or_else(|_| "placement-log.jsonl".to_string())
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn append_log(v: serde_json::Value) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path()) {
        let _ = writeln!(f, "{}", v);
    }
}

// Clipboard access on Windows can be briefly locked -> retry a few times.
fn read_clip() -> Option<String> {
    for _ in 0..12 {
        match Clipboard::new().and_then(|mut c| c.get_text()) {
            Ok(s) => return Some(s),
            Err(_) => thread::sleep(Duration::from_millis(20)),
        }
    }
    None
}

fn write_clip(s: &str) -> bool {
    for _ in 0..12 {
        match Clipboard::new().and_then(|mut c| c.set_text(s.to_owned())) {
            Ok(_) => return true,
            Err(_) => thread::sleep(Duration::from_millis(20)),
        }
    }
    false
}

// Release modifier keys left over from the trigger (Ctrl+Alt); otherwise
// Ctrl+Alt+V may be sent instead of Ctrl+V. Releasing a key that is not held is harmless.
fn clear_modifiers(enigo: &mut Enigo) {
    let _ = enigo.key(Key::Alt, Direction::Release);
    let _ = enigo.key(Key::Control, Direction::Release);
    let _ = enigo.key(Key::Shift, Direction::Release);
    let _ = enigo.key(Key::Meta, Direction::Release);
}

// Method 1: clipboard + Ctrl+V, paste, verify, restore the previous clipboard.
fn placement_clipboard(method: &str) {
    let started = now_ms();
    let previous = read_clip().unwrap_or_default();
    let wrote = write_clip(TEST_TEXT);
    thread::sleep(Duration::from_millis(40)); // let the clipboard write settle

    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            append_log(json!({"ts": started, "method": method, "error": format!("enigo init: {e:?}")}));
            return;
        }
    };
    clear_modifiers(&mut enigo);
    thread::sleep(Duration::from_millis(40));

    // Ctrl+V (an ASCII letter maps to a VK through VkKeyScan, so the shortcut works)
    let _ = enigo.key(Key::Control, Direction::Press);
    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
    let _ = enigo.key(Key::Control, Direction::Release);

    // Verification: safe delay + clipboard-ownership check.
    // (A real "content changed" check would need UI Automation. In this spike:
    //  is the clipboard still OUR text before restoring? That guards against races.)
    thread::sleep(Duration::from_millis(160));
    let clip_after_paste = read_clip().unwrap_or_default();
    let clip_owned = clip_after_paste == TEST_TEXT; // no other app interfered

    // Restore
    let restored = write_clip(&previous);
    thread::sleep(Duration::from_millis(30));
    let clip_final = read_clip().unwrap_or_default();
    let restore_ok = clip_final == previous;

    append_log(json!({
        "ts": started,
        "method": method,
        "wrote_clip": wrote,
        "clip_owned_after_paste": clip_owned,
        "previous_clip_chars": previous.chars().count(),
        "restore_wrote": restored,
        "restore_ok": restore_ok,
        "test_text": TEST_TEXT,
    }));
}

// Method 2: SendInput + KEYEVENTF_UNICODE (does not touch the clipboard; fallback path).
fn placement_unicode(method: &str) {
    let started = now_ms();
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            append_log(json!({"ts": started, "method": method, "error": format!("enigo init: {e:?}")}));
            return;
        }
    };
    clear_modifiers(&mut enigo);
    thread::sleep(Duration::from_millis(40));
    let res = enigo.text(TEST_TEXT);
    append_log(json!({
        "ts": started,
        "method": method,
        "text_result": format!("{res:?}"),
        "test_text": TEST_TEXT,
    }));
}

// Run on a separate thread when triggered: it avoids blocking the hotkey handler
// and gives the trigger keys time to be released (key-up).
fn spawn_placement(method: &'static str) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        match method {
            "unicode" => placement_unicode(method),
            _ => placement_clipboard(method),
        }
    });
}

#[tauri::command]
fn run_placement(method: String) -> String {
    let m: &'static str = if method == "unicode" { "unicode" } else { "clipboard" };
    spawn_placement(m);
    format!("triggered: {m}")
}

#[tauri::command]
fn read_log() -> String {
    std::fs::read_to_string(log_path()).unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                let sc_clip = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyD);
                let sc_uni = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyU);
                let sc_clip_h = sc_clip.clone();
                let sc_uni_h = sc_uni.clone();

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, shortcut, event| {
                            let st = if event.state == ShortcutState::Pressed { "pressed" } else { "released" };
                            let which = if shortcut == &sc_clip_h { "clip(Ctrl+Alt+D)" }
                                        else if shortcut == &sc_uni_h { "uni(Ctrl+Alt+U)" }
                                        else { "?" };
                            append_log(json!({"ts": now_ms(), "event": "hotkey_state", "which": which, "state": st}));
                            // Dictation pattern: place the text on RELEASE (push-to-talk).
                            // Both methods run when the hotkey is released.
                            if event.state == ShortcutState::Released {
                                if shortcut == &sc_clip_h { spawn_placement("clipboard"); }
                                else if shortcut == &sc_uni_h { spawn_placement("unicode"); }
                            }
                        })
                        .build(),
                )?;

                let reg_clip = app.global_shortcut().register(sc_clip);
                let reg_uni = app.global_shortcut().register(sc_uni);
                append_log(json!({"ts": now_ms(), "event": "app_started",
                    "shortcuts": ["Ctrl+Alt+D=clipboard", "Ctrl+Alt+U=unicode"],
                    "reg_clip_ok": reg_clip.is_ok(), "reg_uni_ok": reg_uni.is_ok(),
                    "reg_clip_err": reg_clip.err().map(|e| format!("{e:?}")),
                    "reg_uni_err": reg_uni.err().map(|e| format!("{e:?}"))}));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![run_placement, read_log])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
