import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";

// The shell is an empty work area for now. One line of text shows that the
// interface, the Tauri shell and the core library are all connected.
async function showVersions() {
  const note = document.getElementById("shell-note");
  if (!note) return;
  try {
    const [app, core] = await Promise.all([getVersion(), invoke<string>("core_version")]);
    note.textContent = `Box CAD ${app} · core ${core}`;
  } catch {
    // Opened in a plain browser, where there is no shell to ask.
    note.textContent = "Box CAD (interface only, not running in the app)";
  }
}

void showVersions();
