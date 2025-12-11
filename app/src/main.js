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
const saveConfigBtn = document.querySelector("#save-config-btn");
const reloadConfigBtn = document.querySelector("#reload-config-btn");

const cfgApiUrl = document.querySelector("#cfg-api-url");
const cfgApiKey = document.querySelector("#cfg-api-key");
const cfgWatchTime = document.querySelector("#cfg-watch-time");
const cfgMediaEnable = document.querySelector("#cfg-media-enable");
const cfgLogEnable = document.querySelector("#cfg-log-enable");

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
    const status = await invoke("cmd_get_status");
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
    const status = await invoke("cmd_start_watcher", { config_path: configInput.value });
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
    await invoke("cmd_stop_watcher");
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

async function reloadConfig() {
  setStatusMessage("Loading config...");
  try {
    const cfg = await invoke("cmd_read_config", { config_path: configInput.value });
    cfgApiUrl.value = cfg.api_url || "";
    cfgApiKey.value = cfg.api_key || "";
    cfgWatchTime.value = cfg.watch_time ?? 5;
    cfgMediaEnable.checked = !!cfg.media_enable;
    cfgLogEnable.checked = !!cfg.log_enable;
    setStatusMessage("Config loaded");
  } catch (error) {
    setStatusMessage(`Load config failed: ${error}`);
  }
}

async function saveConfig() {
  setBusy(true);
  try {
    const cfg = {
      api_url: cfgApiUrl.value.trim(),
      api_key: cfgApiKey.value.trim(),
      watch_time: Number(cfgWatchTime.value || 5),
      media_enable: cfgMediaEnable.checked,
      log_enable: cfgLogEnable.checked,
    };
    await invoke("cmd_write_config", { config_path: configInput.value, cfg });
    setStatusMessage("Config saved");
  } catch (error) {
    setStatusMessage(`Save failed: ${error}`);
  } finally {
    setBusy(false);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  startBtn.addEventListener("click", startWatcher);
  stopBtn.addEventListener("click", stopWatcher);
  refreshBtn.addEventListener("click", refreshStatus);
  saveConfigBtn.addEventListener("click", saveConfig);
  reloadConfigBtn.addEventListener("click", reloadConfig);
  refreshStatus();
  reloadConfig();
  ensurePolling();
});
