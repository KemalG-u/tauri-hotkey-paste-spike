> 🧪 **Status: experiment, archived.** Kept for reference; not maintained.

# tauri-hotkey-paste-spike

Two small Tauri v2 spikes for Windows that answer one question: **can a global push-to-talk style hotkey reliably place text into whatever window has focus?** The text is a fixed Turkish test sentence (diacritics, number, currency, percent), so the spikes also check that non-ASCII characters survive the trip. There is no speech recognition in here.

Why it exists: the author was evaluating a dictation-style tool and needed evidence for the last step of such a pipeline (hotkey down = record, hotkey up = insert the transcript) before choosing a shell. The code is the evidence, kept as a reference for anyone weighing the same approach.

## What is in the repository

| Path | What it does |
|---|---|
| `src/main.rs`, `Cargo.toml`, `tauri.conf.json`, `frontend/` | Minimal spike (about 150 lines of Rust). `Ctrl+Alt+D`: press is logged, release runs the clipboard chain: save the clipboard, write the test text, send `Ctrl+V` through `SendInput` after releasing stuck modifiers, wait, restore the clipboard. Hidden window, log file only. |
| `placement-variants/` | Second spike with a small window and two placement methods you can compare: `Ctrl+Alt+D` = clipboard + `Ctrl+V` (with a clipboard-ownership check and a restore check), `Ctrl+Alt+U` = `KEYEVENTF_UNICODE` typing through `enigo` (does not touch the clipboard). Buttons trigger the same methods and show the log. The crate and library are still named `cp3-app` / `cp3_app_lib` (historical name). |

## Quick start

Requirements: Windows 10/11 with WebView2 and a Rust toolchain. The variant pins the GNU toolchain in `placement-variants/src-tauri/rust-toolchain.toml` (see the comment in its `Cargo.toml`); delete that file to use MSVC.

```
# minimal spike (repository root)
cargo run
# log: events.log in the working directory, or the path in SPIKE_TAURI_LOG

# variant with two methods
cd placement-variants
npm install
npx tauri dev
# log: placement-log.jsonl in the working directory, or the path in PLACEMENT_SPIKE_LOG
```

Click into any text field, then hold and release the hotkey. Do not run both spikes at once: they register the same `Ctrl+Alt+D`.

Logs record only the length of the previous clipboard, never its content.

## Limits and what is not verified

- **This release was not compiled or run during preparation.** No Rust toolchain was available on the preparation machine, so there is no test or CI here. The author's own run logs (not published, because they contained clipboard contents) showed registered hotkeys, press/release events and completed paste-and-restore cycles on one Windows machine. Treat that as a single-machine observation, not a guarantee.
- Windows only (`SendInput`, `windows-sys`). One keyboard layout, one machine, no elevated-window or remote-desktop testing.
- The paste is blind: the spike cannot tell whether the target application actually received the text (that would need UI Automation). The clipboard restore uses a fixed delay and can race with slow target applications.
- A synthetic `Ctrl+V` is ignored by some applications and blocked in elevated windows when sent from a non-elevated process.
- The code is a spike: errors are logged, not handled; there is no settings UI and no packaging.

Turkish version: [README.tr.md](README.tr.md)

## License

Apache-2.0, see [LICENSE](LICENSE) and [NOTICE](NOTICE). Rust crates and the Tauri icon template keep their own licenses (see `Cargo.lock` for the dependency list).
