const { invoke } = window.__TAURI__.core;

let statusEl;
let logEl;

async function place(method) {
  statusEl.textContent = await invoke("run_placement", { method });
}

async function showLog() {
  logEl.textContent = await invoke("read_log");
}

window.addEventListener("DOMContentLoaded", () => {
  statusEl = document.querySelector("#status");
  logEl = document.querySelector("#log");
  document.querySelector("#btn-clip").addEventListener("click", () => place("clipboard"));
  document.querySelector("#btn-uni").addEventListener("click", () => place("unicode"));
  document.querySelector("#btn-log").addEventListener("click", showLog);
});
