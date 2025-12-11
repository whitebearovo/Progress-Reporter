const { invoke } = window.__TAURI__.core;

const processEl = document.querySelector("#process");
const mediaTitleEl = document.querySelector("#media-title");
const mediaArtistEl = document.querySelector("#media-artist");
const runningEl = document.querySelector("#running");
const lastReportEl = document.querySelector("#last-report");
const logArea = document.querySelector("#log-area");
const sessionTag = document.querySelector("#session-tag");

const startBtn = document.querySelector("#start-btn");
const stopBtn = document.querySelector("#stop-btn");
const refreshBtn = document.querySelector("#refresh-btn");
const configInput = document.querySelector("#config-path");

let polling;

function setBusy(isBusy) {
  startBtn.disabled = isBusy;
  stopBtn.disabled = isBusy;
  refreshBtn.disabled = isBusy;
}

function setStatusMessage(message) {
  logArea.textContent = message;
}

async function refreshStatus() {
  try {
    const status = await invoke("get_status");
    runningEl.textContent = status.running ? "Running" : "Stopped";
    processEl.textContent = status.last_process || "—";
    mediaTitleEl.textContent = status.last_media_title || "—";
    mediaArtistEl.textContent = status.last_media_artist || "—";
    lastReportEl.textContent = status.last_report_at || "—";
    sessionTag.textContent = status.session ? `Session: ${status.session}` : "Session: —";
    setStatusMessage(status.running ? "Watcher active" : "Watcher idle");
  } catch (error) {
    setStatusMessage(`Failed to fetch status: ${error}`);
  }
}

async function startWatcher() {
  setBusy(true);
  try {
    const status = await invoke("start_watcher", { config_path: configInput.value });
    setStatusMessage(`Watcher started with ${status.config_path || "config"}`);
  } catch (error) {
    setStatusMessage(`Start failed: ${error}`);
  } finally {
    setBusy(false);
    refreshStatus();
  }
}

async function stopWatcher() {
  setBusy(true);
  try {
    await invoke("stop_watcher");
    setStatusMessage("Watcher stopped");
  } catch (error) {
    setStatusMessage(`Stop failed: ${error}`);
  } finally {
    setBusy(false);
    refreshStatus();
  }
}

function ensurePolling() {
  if (polling) return;
  polling = setInterval(refreshStatus, 5000);
}

window.addEventListener("DOMContentLoaded", () => {
  startBtn.addEventListener("click", startWatcher);
  stopBtn.addEventListener("click", stopWatcher);
  refreshBtn.addEventListener("click", refreshStatus);
  refreshStatus();
  ensurePolling();
});
