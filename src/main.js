const { invoke } = window.__TAURI__.core;
const { Window } = window.__TAURI__.window;
const appWindow = Window.getCurrent();

// Window Controls
document.getElementById('titlebar-minimize').addEventListener('click', () => appWindow.minimize());
document.getElementById('titlebar-maximize').addEventListener('click', async () => {
  await appWindow.toggleMaximize();
  // Fix for window potentially losing focus/disappearing on Linux after unmaximize
  setTimeout(() => appWindow.setFocus(), 50);
});
document.getElementById('titlebar-close').addEventListener('click', () => appWindow.close());

// Theme Logic
const themeToggleBtn = document.getElementById('theme-toggle');
const savedTheme = localStorage.getItem('theme');

// Only apply AMOLED mode if explicitly saved, default is light mode
if (savedTheme === 'amoled') {
  document.body.classList.add('amoled-mode');
  themeToggleBtn.textContent = '☀️';
} else {
  // Ensure we start in default mode
  document.body.classList.remove('amoled-mode');
  themeToggleBtn.textContent = '🌙';
}

themeToggleBtn.onclick = () => {
  document.body.classList.toggle('amoled-mode');
  const isAmoled = document.body.classList.contains('amoled-mode');
  localStorage.setItem('theme', isAmoled ? 'amoled' : 'default');
  themeToggleBtn.textContent = isAmoled ? '☀️' : '🌙';
};

// Manual Drag Implementation for Linux/Wayland compatibility
const titlebar = document.querySelector('.titlebar');

titlebar.addEventListener('mousedown', (e) => {
  if (e.target.closest('.titlebar-button')) return;
  if (e.button === 0 && e.detail === 1) { // Left click only, single click (prevent drag on double click)
    appWindow.startDragging();
  }
});

titlebar.addEventListener('dblclick', async (e) => {
  if (e.target.closest('.titlebar-button')) return;
  await appWindow.toggleMaximize();
  setTimeout(() => appWindow.setFocus(), 50);
});

const appListEl = document.getElementById("app-list");
const addBtn = document.getElementById("add-btn");
const modal = document.getElementById("modal");
const cancelBtn = document.getElementById("cancel-btn");
const addForm = document.getElementById("add-form");

async function loadApps() {
  try {
    const apps = await invoke("get_apps");
    renderApps(apps);
  } catch (error) {
    console.error("Failed to load apps:", error);
  }
}

function renderApps(apps) {
  appListEl.innerHTML = "";
  apps.forEach((app) => {
    const card = document.createElement("div");
    card.className = "app-card";

    const info = document.createElement("div");
    info.className = "app-info";

    const header = document.createElement("div");
    header.className = "app-header";

    const name = document.createElement("div");
    name.className = "app-name";
    name.textContent = app.name;

    header.appendChild(name);

    // Meta tags container
    const meta = document.createElement("div");
    meta.className = "app-meta";

    // Location Badge
    const locationBadge = document.createElement("span");
    locationBadge.className = "meta-badge location";
    locationBadge.textContent = app.location || "Startup Folder";
    locationBadge.title = "Startup Location";

    // Size Badge
    const sizeBadge = document.createElement("span");
    sizeBadge.className = "meta-badge size";
    sizeBadge.textContent = app.size || "Unknown";
    sizeBadge.title = "File Size";

    // Publisher Badge (Optional, only show if not unknown/system)
    if (app.publisher && app.publisher !== "Unknown" && app.publisher !== "Linux Desktop Entry") {
      const pubBadge = document.createElement("span");
      pubBadge.className = "meta-badge publisher";
      pubBadge.textContent = app.publisher;
      meta.appendChild(pubBadge);
    }

    meta.appendChild(locationBadge);
    meta.appendChild(sizeBadge);

    // Command (Path) - Make it smaller/subtle
    const command = document.createElement("div");
    command.className = "app-command";
    command.textContent = app.command; // Shows clean path (e.g., /usr/bin/ulauncher)
    command.title = app.full_command || app.command; // Tooltip shows full command with args

    info.appendChild(header);
    info.appendChild(meta);
    info.appendChild(command);

    const actions = document.createElement("div");
    actions.className = "app-actions";

    // Toggle Switch
    const switchLabel = document.createElement("label");
    switchLabel.className = "switch";

    const input = document.createElement("input");
    input.type = "checkbox";
    input.checked = app.enabled;
    input.onchange = () => toggleApp(app.path, input.checked);

    const slider = document.createElement("span");
    slider.className = "slider";

    switchLabel.appendChild(input);
    switchLabel.appendChild(slider);

    // Delete Button
    const deleteBtn = document.createElement("button");
    deleteBtn.className = "delete-btn";
    deleteBtn.innerHTML = `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>`;
    deleteBtn.onclick = () => deleteApp(app.path);

    actions.appendChild(switchLabel);
    actions.appendChild(deleteBtn);

    card.appendChild(info);
    card.appendChild(actions);

    appListEl.appendChild(card);
  });
}

// Tab Switching
const tabs = document.querySelectorAll('.tab-btn');
const views = document.querySelectorAll('.view-content');

tabs.forEach(tab => {
  tab.addEventListener('click', () => {
    tabs.forEach(t => t.classList.remove('active'));
    views.forEach(v => v.style.display = 'none');

    tab.classList.add('active');
    const target = tab.dataset.tab;
    document.getElementById(`${target}-view`).style.display = 'block';

    if (target === 'services') {
      loadServices();
    }
  });
});

async function loadServices() {
  const serviceListEl = document.getElementById("service-list");
  serviceListEl.innerHTML = "<p>Loading services...</p>";

  try {
    const services = await invoke("get_system_services");
    renderServices(services);
  } catch (error) {
    serviceListEl.innerHTML = `<p style="color: red;">Error: ${error}</p>`;
  }
}

function renderServices(services) {
  const serviceListEl = document.getElementById("service-list");
  serviceListEl.innerHTML = "";

  const CHUNK_SIZE = 50;
  let index = 0;

  function renderChunk() {
    const chunk = services.slice(index, index + CHUNK_SIZE);
    const fragment = document.createDocumentFragment();

    chunk.forEach(service => {
      const card = document.createElement("div");
      card.className = "app-card";

      const info = document.createElement("div");
      info.className = "app-info";

      const name = document.createElement("div");
      name.className = "app-name";
      name.textContent = service.name;

      const meta = document.createElement("div");
      meta.className = "app-meta";

      const stateBadge = document.createElement("span");
      stateBadge.className = "meta-badge";
      stateBadge.textContent = service.state;
      stateBadge.style.color = service.state === 'enabled' ? '#22c55e' : '#94a3b8';

      meta.appendChild(stateBadge);

      info.appendChild(name);
      info.appendChild(meta);

      const actions = document.createElement("div");
      actions.className = "app-actions";

      const switchLabel = document.createElement("label");
      switchLabel.className = "switch";

      const input = document.createElement("input");
      input.type = "checkbox";
      input.checked = service.state === 'enabled';
      input.onchange = async () => {
        try {
          await invoke("toggle_service", { name: service.name, enable: input.checked });
          // Refresh to confirm state
          loadServices();
        } catch (err) {
          alert("Failed to toggle service: " + err);
          input.checked = !input.checked; // Revert
        }
      };

      const slider = document.createElement("span");
      slider.className = "slider";

      switchLabel.appendChild(input);
      switchLabel.appendChild(slider);

      actions.appendChild(switchLabel);
      card.appendChild(info);
      card.appendChild(actions);

      fragment.appendChild(card);
    });

    serviceListEl.appendChild(fragment);
    index += CHUNK_SIZE;

    if (index < services.length) {
      requestAnimationFrame(renderChunk);
    }
  }

  renderChunk();
}

async function toggleApp(path, enabled) {
  try {
    await invoke("toggle_app", { path, enable: enabled });
  } catch (error) {
    console.error("Failed to toggle app:", error);
    loadApps(); // Revert UI on error
  }
}

const confirmModal = document.getElementById("confirm-modal");
const confirmYesBtn = document.getElementById("confirm-yes-btn");
const confirmCancelBtn = document.getElementById("confirm-cancel-btn");
let appToDelete = null;

async function deleteApp(path) {
  appToDelete = path;
  confirmModal.classList.add("active");
}

confirmCancelBtn.onclick = () => {
  confirmModal.classList.remove("active");
  appToDelete = null;
};

confirmYesBtn.onclick = async () => {
  if (appToDelete) {
    try {
      await invoke("delete_app", { path: appToDelete });
      loadApps();
    } catch (error) {
      console.error("Failed to delete app:", error);
      alert("Failed to delete app: " + error);
    }
    confirmModal.classList.remove("active");
    appToDelete = null;
  }
};

addBtn.onclick = () => {
  modal.classList.add("active");
};

const helpBtn = document.getElementById("help-btn");
const helpContent = document.getElementById("help-content");

helpBtn.onclick = () => {
  helpContent.classList.toggle("active");
};

cancelBtn.onclick = () => {
  modal.classList.remove("active");
};

addForm.onsubmit = async (e) => {
  e.preventDefault();
  const name = document.getElementById("app-name").value;
  const command = document.getElementById("app-command").value;
  const description = document.getElementById("app-desc").value;

  try {
    await invoke("create_app", { name, command, description });
    modal.classList.remove("active");
    addForm.reset();
    loadApps();
  } catch (error) {
    console.error("Failed to create app:", error);
    alert("Failed to create app: " + error);
  }
};

// Close modal on outside click
window.onclick = (event) => {
  if (event.target == modal) {
    modal.classList.remove("active");
  }
  if (event.target == confirmModal) {
    confirmModal.classList.remove("active");
    appToDelete = null;
  }
};

loadApps();

// File Picker Logic - Optional for users who want to browse
const browseBtn = document.getElementById("browse-btn");
if (browseBtn) {
  browseBtn.onclick = async () => {
    try {
      // Use Tauri's dialog plugin with correct parameters
      const selected = await invoke('plugin:dialog|open', {
        options: {
          multiple: false,
          filters: [{
            name: 'Applications',
            extensions: ['exe', 'lnk', 'sh', 'desktop', 'AppImage', 'bat', 'cmd']
          }]
        }
      });

      if (selected) {
        const commandInput = document.getElementById("app-command");
        const nameInput = document.getElementById("app-name");

        commandInput.value = selected;

        // Auto-fill name if empty
        if (!nameInput.value) {
          const filename = selected.split(/[\\/]/).pop();
          const name = filename.split('.').slice(0, -1).join('.') || filename;
          nameInput.value = name.charAt(0).toUpperCase() + name.slice(1);
        }
      }
    } catch (error) {
      alert("Error: " + error);
    }
  };
}
