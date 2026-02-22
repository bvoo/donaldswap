let ws = null;
let config = null;
let state = null;
let windows = [];

function connectWS() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

  ws.onmessage = (event) => {
    state = JSON.parse(event.data);
    updateStateDisplay();
  };

  ws.onclose = () => {
    setTimeout(connectWS, 1000);
  };
}

async function fetchConfig() {
  const res = await fetch("/api/config");
  config = await res.json();
  renderGameList();
  renderSettings();
}

async function fetchState() {
  const res = await fetch("/api/state");
  state = await res.json();
  updateStateDisplay();
}

async function fetchWindows() {
  const res = await fetch("/api/windows");
  windows = await res.json();
  renderWindowPicker();
}

function updateStateDisplay() {
  if (!state) return;

  const statusBadge = document.getElementById("status-badge");
  const statusText = document.getElementById("status-text");
  const currentGame = document.getElementById("current-game");
  const timeSince = document.getElementById("time-since");
  const nextSwap = document.getElementById("next-swap");
  const swapCount = document.getElementById("swap-count");

  if (state.is_paused) {
    statusText.textContent = "Paused";
    statusBadge.className = "status-badge paused";
  } else if (state.current_game) {
    statusText.textContent = "Active";
    statusBadge.className = "status-badge active";
  } else {
    statusText.textContent = "Waiting";
    statusBadge.className = "status-badge waiting";
  }

  currentGame.textContent = state.current_game || "None";

  if (
    state.time_since_swap_seconds !== null &&
    state.time_since_swap_seconds !== undefined
  ) {
    timeSince.textContent = formatTime(Math.abs(state.time_since_swap_seconds));
  } else {
    timeSince.textContent = "--:--";
  }

  if (
    state.time_until_swap_seconds !== null &&
    state.time_until_swap_seconds !== undefined
  ) {
    if (config && config.hide_next_swap) {
      nextSwap.textContent = "REDACTED";
    } else {
      const seconds = Math.max(0, state.time_until_swap_seconds);
      nextSwap.textContent = formatTime(seconds);
    }
  } else {
    nextSwap.textContent = "--:--";
  }

  swapCount.textContent = state.swap_count || 0;

  renderHistory();
}

function renderHistory() {
  const container = document.getElementById("swap-history");
  if (!state || !state.history || state.history.length === 0) {
    container.innerHTML =
      '<div class="empty-state" style="padding: 1.5rem;">No history yet</div>';
    return;
  }

  container.innerHTML = state.history
    .map(
      (item) => `
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1.25rem; border-bottom: 1px solid var(--border);">
            <div style="font-weight: 500; font-size: 0.875rem;">${escapeHtml(item.game_name)}</div>
            <div style="font-family: var(--mono); font-size: 0.8125rem; color: var(--muted);">${formatTime(item.duration_seconds)}</div>
        </div>
    `,
    )
    .join("");
}

function formatTime(seconds) {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

function renderGameList() {
  const container = document.getElementById("game-list");
  if (!config || !config.games || config.games.length === 0) {
    container.innerHTML = '<div class="empty-state">No games configured</div>';
    return;
  }

  container.innerHTML = config.games
    .map(
      (game, index) => `
        <div class="list-item" data-index="${index}">
            <div class="list-item-info">
                <input type="text" 
                       class="editable-title" 
                       value="${escapeHtml(game.display_name)}" 
                       onblur="updateGameTitle(${index}, this.value)"
                       onkeydown="if(event.key === 'Enter') this.blur()"
                       title="Click to edit display name">
                <div class="list-item-sub">${escapeHtml(game.exe_name)}</div>
                
                <div style="margin-top: 0.5rem; display: flex; align-items: center; gap: 0.5rem;">
                    <span style="font-size: 0.75rem; color: var(--muted);">OBS Scene:</span>
                    <input type="text" 
                           placeholder="Scene name (empty = ignore)"
                           value="${escapeHtml(game.obs_scene || '')}"
                           onblur="updateGameScene(${index}, this.value)"
                           onkeydown="if(event.key === 'Enter') this.blur()"
                           style="padding: 0.25rem 0.5rem; font-size: 0.75rem; width: 200px; border: 1px solid var(--border); background: transparent; color: var(--fg); border-radius: 4px;">
                </div>
            </div>
            <div class="list-item-actions">
                <div class="toggles-row">
                    <label class="checkbox-container">
                        <input type="checkbox" ${game.send_esc_on_leave ? "checked" : ""} 
                            onchange="updateGame(${index}, 'send_esc_on_leave', this.checked)">
                        <span class="checkmark"></span>
                        ESC on Leave
                    </label>
                    <label class="checkbox-container">
                        <input type="checkbox" ${game.send_esc_on_enter ? "checked" : ""} 
                            onchange="updateGame(${index}, 'send_esc_on_enter', this.checked)">
                        <span class="checkmark"></span>
                        ESC on Enter
                    </label>
                    <label class="checkbox-container">
                        <input type="checkbox" ${game.enabled ? "checked" : ""} 
                            onchange="updateGame(${index}, 'enabled', this.checked)">
                        <span class="checkmark"></span>
                        Enabled
                    </label>
                    <button class="btn btn-small" onclick="removeGame(${index})">Remove</button>
                </div>
            </div>
        </div>
    `,
    )
    .join("");
}

function renderSettings() {
  if (!config) return;

  document.getElementById("min-swap").value = config.min_swap_minutes;
  document.getElementById("max-swap").value = config.max_swap_minutes;
  document.getElementById("auto-swap").checked = config.auto_swap_enabled;
  document.getElementById("hide-next-swap").checked = config.hide_next_swap;
  document.getElementById("obs-host").value = config.obs_ws_host;
  document.getElementById("obs-port").value = config.obs_ws_port;
  document.getElementById("obs-password").value = config.obs_ws_password || "";
}

function renderWindowPicker() {
  const container = document.getElementById("window-picker");
  if (!windows.length) {
    container.innerHTML = '<div class="empty-state">No processes found</div>';
    return;
  }

  const uniqueExes = [...new Map(windows.map((w) => [w.exe_name, w])).values()];

  container.innerHTML = uniqueExes
    .map(
      (w) => `
        <div class="list-item">
            <div class="list-item-info">
                <div class="list-item-title">${escapeHtml(w.title)}</div>
                <div class="list-item-sub">${escapeHtml(w.exe_name)}</div>
            </div>
            <div class="list-item-actions">
                <button class="btn btn-small" onclick="addGame('${escapeHtml(w.exe_name)}', '${escapeHtml(w.title)}')">Add</button>
            </div>
        </div>
    `,
    )
    .join("");
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

async function updateGame(index, field, value) {
  config.games[index][field] = value;
  await saveConfig();
}

async function updateGameTitle(index, newTitle) {
  if (newTitle.trim() === "") return;
  config.games[index].display_name = newTitle.trim();
  await saveConfig();
}

async function updateGameScene(index, newScene) {
  config.games[index].obs_scene = newScene.trim() === "" ? null : newScene.trim();
  await saveConfig();
}

async function removeGame(index) {
  config.games.splice(index, 1);
  await saveConfig();
  renderGameList();
}

async function addGame(exeName, title) {
  if (
    config.games.some((g) => g.exe_name.toLowerCase() === exeName.toLowerCase())
  ) {
    alert("Process already tracked");
    return;
  }

  config.games.push({
    exe_name: exeName,
    display_name: title || exeName.replace(".exe", ""),
    send_esc_on_leave: true,
    send_esc_on_enter: true,
    enabled: true,
  });

  await saveConfig();
  renderGameList();
}

async function saveConfig() {
  await fetch("/api/config", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(config),
  });
}

async function updateSettings() {
  config.min_swap_minutes =
    parseInt(document.getElementById("min-swap").value) || 5;
  config.max_swap_minutes =
    parseInt(document.getElementById("max-swap").value) || 15;
  config.auto_swap_enabled = document.getElementById("auto-swap").checked;
  config.hide_next_swap = document.getElementById("hide-next-swap").checked;
  await saveConfig();
  updateStateDisplay();
}

async function forceSwap() {
  await fetch("/api/swap", { method: "POST" });
}

async function pauseSwapper() {
  await fetch("/api/pause", { method: "POST" });
}

async function resumeSwapper() {
  await fetch("/api/resume", { method: "POST" });
}

document.getElementById("settings-form").addEventListener("submit", (e) => {
  e.preventDefault();
  updateSettings();
});

document.getElementById("obs-form").addEventListener("submit", (e) => {
  e.preventDefault();
  config.obs_ws_host = document.getElementById("obs-host").value.trim() || "localhost";
  config.obs_ws_port = parseInt(document.getElementById("obs-port").value) || 4455;
  config.obs_ws_password = document.getElementById("obs-password").value.trim() || null;
  saveConfig().then(() => alert("OBS Settings Saved"));
});

fetchConfig();
fetchState();
fetchWindows();
connectWS();

setInterval(fetchState, 1000);
