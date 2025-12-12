const { invoke } = window.__TAURI__.core;

const processEl = document.querySelector("#process");
const mediaTitleEl = document.querySelector("#media-title");
const mediaArtistEl = document.querySelector("#media-artist");
const runningEl = document.querySelector("#running");
const lastReportEl = document.querySelector("#last-report");
const logArea = document.querySelector("#log-area");
const sessionTag = document.querySelector("#session-tag");
const localeSelect = document.querySelector("#locale-select");

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
let t = {};

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
    runningEl.textContent = status.running ? t.running || "Running" : t.stopped || "Stopped";
    processEl.textContent = status.last_process || "—";
    mediaTitleEl.textContent = status.last_media_title || "—";
    mediaArtistEl.textContent = status.last_media_artist || "—";
    lastReportEl.textContent = status.last_report_at || "—";
    sessionTag.textContent = status.session ? `${t.sessionTag || "Session"}: ${status.session}` : `${t.sessionTag || "Session"}: —`;
    setStatusMessage(status.running ? t.watcherStarted || "Watcher active" : t.watcherStopped || "Watcher idle");
  } catch (error) {
    setStatusMessage(`Failed to fetch status: ${error}`);
  }
}

async function startWatcher() {
  setBusy(true);
  try {
    const status = await invoke("cmd_start_watcher", { config_path: configInput.value });
    setStatusMessage(t.watcherStarted || "Watcher started");
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
    setStatusMessage(t.watcherStopped || "Watcher stopped");
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
  setStatusMessage(t.loadingConfig || "Loading config...");
  try {
    const cfg = await invoke("cmd_read_config", { config_path: configInput.value });
    cfgApiUrl.value = cfg.api_url || "";
    cfgApiKey.value = cfg.api_key || "";
    cfgWatchTime.value = cfg.watch_time ?? 5;
    cfgMediaEnable.checked = !!cfg.media_enable;
    cfgLogEnable.checked = !!cfg.log_enable;
    setStatusMessage(t.configLoaded || "Config loaded");
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
    setStatusMessage(t.configSaved || "Config saved");
  } catch (error) {
    setStatusMessage(`Save failed: ${error}`);
  } finally {
    setBusy(false);
  }
}

async function loadLocale(locale) {
  try {
    const resp = await fetch(`/i18n/${locale}.json`);
    t = await resp.json();
    applyLocale();
  } catch (err) {
    console.error("Failed to load locale", err);
  }
}

function applyLocale() {
  document.title = t.title || "Progress Reporter";
  const eyebrow = document.querySelector(".eyebrow");
  const heroTitle = document.querySelector(".hero h1");
  if (eyebrow) eyebrow.textContent = t.title || "Progress Reporter";
  if (heroTitle) heroTitle.textContent = t.subtitle || "";

  const lblConfig = document.querySelector("label[for='config-path']");
  if (lblConfig) lblConfig.textContent = t.configFile || "Config file";
  const lblApiUrl = document.querySelector("label[for='cfg-api-url']");
  if (lblApiUrl) lblApiUrl.textContent = t.apiUrl || "API_URL";
  const lblApiKey = document.querySelector("label[for='cfg-api-key']");
  if (lblApiKey) lblApiKey.textContent = t.apiKey || "API_KEY";
  const lblWatch = document.querySelector("label[for='cfg-watch-time']");
  if (lblWatch) lblWatch.textContent = t.watchTime || "WATCH_TIME";
  const lblMedia = document.querySelector("label[for='cfg-media-enable']");
  if (lblMedia) lblMedia.textContent = t.mediaEnable || "MEDIA_ENABLE";
  const lblLog = document.querySelector("label[for='cfg-log-enable']");
  if (lblLog) lblLog.textContent = t.logEnable || "LOG_ENABLE";

  const btnSave = document.querySelector("#save-config-btn");
  if (btnSave) btnSave.textContent = t.saveConfig || "Save config";
  const btnReload = document.querySelector("#reload-config-btn");
  if (btnReload) btnReload.textContent = t.reloadConfig || "Reload";
  const btnStart = document.querySelector("#start-btn");
  if (btnStart) btnStart.textContent = t.start || "Start";
  const btnStop = document.querySelector("#stop-btn");
  if (btnStop) btnStop.textContent = t.stop || "Stop";
  const hint = document.querySelector(".hint");
  if (hint) hint.textContent = t.hint || "";
}

window.addEventListener("DOMContentLoaded", () => {
  startBtn.addEventListener("click", startWatcher);
  stopBtn.addEventListener("click", stopWatcher);
  refreshBtn.addEventListener("click", refreshStatus);
  saveConfigBtn.addEventListener("click", saveConfig);
  reloadConfigBtn.addEventListener("click", reloadConfig);
  localeSelect.addEventListener("change", (e) => loadLocale(e.target.value));
  loadLocale(localeSelect.value || "en");
  refreshStatus();
  reloadConfig();
  ensurePolling();
});
