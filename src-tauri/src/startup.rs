use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartupApp {
    pub id: String,
    pub name: String,
    pub command: String, // This will now hold the CLEAN path, not full command
    pub full_command: String, // New: Holds the full command with args (for tooltip)
    pub enabled: bool,
    pub path: PathBuf,
    pub size: String,
    pub location: String,
    pub publisher: String,
}

#[cfg(target_os = "linux")]
pub fn get_startup_apps() -> Vec<StartupApp> {
    let mut apps = Vec::new();
    if let Some(config_dir) = dirs::config_dir() {
        let autostart_dir = config_dir.join("autostart");
        if autostart_dir.exists() {
            for entry in WalkDir::new(&autostart_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "desktop")
                {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let name = extract_value(&content, "Name")
                            .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
                        let raw_command = extract_value(&content, "Exec").unwrap_or_default();
                        let full_command = raw_command
                            .replace("env GDK_BACKEND=x11 ", "")
                            .replace("env ", "");

                        // Extract clean path (first part of command)
                        let clean_path = full_command
                            .split_whitespace()
                            .next()
                            .unwrap_or(&full_command)
                            .to_string();
                        let size = get_file_size(std::path::Path::new(&clean_path));

                        let hidden = extract_value(&content, "Hidden")
                            .map(|v| v.to_lowercase() == "true")
                            .unwrap_or(false);
                        let x_gnome_enabled = extract_value(&content, "X-GNOME-Autostart-enabled")
                            .map(|v| v.to_lowercase() == "true")
                            .unwrap_or(true);

                        let enabled = !hidden && x_gnome_enabled;

                        apps.push(StartupApp {
                            id: entry.file_name().to_string_lossy().to_string(),
                            name,
                            command: clean_path, // Show clean path
                            full_command,        // Keep full command for tooltip
                            enabled,
                            path: entry.path().to_path_buf(),
                            size,
                            location: "Startup Folder".to_string(),
                            publisher: "Linux Desktop Entry".to_string(),
                        });
                    }
                }
            }
        }
    }
    apps
}

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[cfg(target_os = "windows")]
pub fn get_startup_apps() -> Vec<StartupApp> {
    let mut apps = Vec::new();

    // 1. Check Startup Folder
    if let Some(startup_dir) =
        dirs::data_dir().map(|d| d.join("Microsoft\\Windows\\Start Menu\\Programs\\Startup"))
    {
        if startup_dir.exists() {
            for entry in WalkDir::new(&startup_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().map_or(false, |ext| {
                    ext == "lnk" || ext == "bat" || ext == "cmd" || ext == "exe"
                }) {
                    // For .lnk, we ideally need to resolve target. Without libs, we use the lnk itself or try to guess.
                    // Since we can't easily resolve .lnk without 'lnks' crate or similar (which requires C++ libs sometimes),
                    // we will stick to file size of the shortcut for now OR mark as "Shortcut".
                    // But user wants actual size.
                    // Let's try to be honest: "Shortcut" size is misleading.
                    // If it's a .bat/.cmd/.exe in startup folder, we can get size.

                    let is_shortcut = entry.path().extension().map_or(false, |e| e == "lnk");
                    let size = if is_shortcut {
                        "Shortcut".to_string() // Honest fallback
                    } else {
                        get_file_size(entry.path())
                    };

                    apps.push(StartupApp {
                        id: entry.file_name().to_string_lossy().to_string(),
                        name: entry
                            .file_name()
                            .to_string_lossy()
                            .replace(".lnk", "")
                            .to_string(),
                        command: entry.path().to_string_lossy().to_string(),
                        full_command: entry.path().to_string_lossy().to_string(),
                        enabled: true,
                        path: entry.path().to_path_buf(),
                        size,
                        location: "Startup Folder".to_string(),
                        publisher: "Unknown".to_string(),
                    });
                }
            }
        }
    }

    // 2. Check Registry (HKCU)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        for (name, value) in run_key.enum_values().filter_map(|x| x.ok()) {
            let full_command = value.to_string();
            // Clean path: remove quotes and args
            let clean_path_str = full_command
                .split('"')
                .nth(1)
                .unwrap_or(&full_command)
                .split_whitespace()
                .next()
                .unwrap_or(&full_command)
                .to_string();
            let clean_path = PathBuf::from(&clean_path_str);

            let size = if clean_path.exists() {
                get_file_size(&clean_path)
            } else {
                "Unknown".to_string()
            };

            apps.push(StartupApp {
                id: name.clone(),
                name: name.clone(),
                command: clean_path_str,
                full_command: full_command.clone(),
                enabled: true,
                path: PathBuf::from(format!("REGISTRY::HKCU::{}", name)),
                size,
                location: "Registry (HKCU)".to_string(),
                publisher: "Unknown".to_string(),
            });
        }
    }

    // 3. Check Registry (HKLM)
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(run_key) = hklm.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        for (name, value) in run_key.enum_values().filter_map(|x| x.ok()) {
            let full_command = value.to_string();
            let clean_path_str = full_command
                .split('"')
                .nth(1)
                .unwrap_or(&full_command)
                .split_whitespace()
                .next()
                .unwrap_or(&full_command)
                .to_string();
            let clean_path = PathBuf::from(&clean_path_str);

            let size = if clean_path.exists() {
                get_file_size(&clean_path)
            } else {
                "Unknown".to_string()
            };

            apps.push(StartupApp {
                id: name.clone(),
                name: name.clone(),
                command: clean_path_str,
                full_command: full_command.clone(),
                enabled: true,
                path: PathBuf::from(format!("REGISTRY::HKLM::{}", name)),
                size,
                location: "Registry (HKLM)".to_string(),
                publisher: "System".to_string(),
            });
        }
    }

    apps
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn get_startup_apps() -> Vec<StartupApp> {
    Vec::new()
}

fn get_file_size(path: &std::path::Path) -> String {
    if let Ok(metadata) = fs::metadata(path) {
        let bytes = metadata.len();
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    } else {
        "Unknown".to_string()
    }
}

#[allow(dead_code)]
fn extract_value(content: &str, key: &str) -> Option<String> {
    let key_eq = format!("{}=", key);
    for line in content.lines() {
        if line.starts_with(&key_eq) {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub fn toggle_app(path: PathBuf, enable: bool) -> Result<(), String> {
    // Check if it's a symlink
    let is_symlink = fs::symlink_metadata(&path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut new_lines = Vec::new();
    let mut hidden_found = false;
    let mut gnome_enabled_found = false;

    for line in content.lines() {
        if line.starts_with("Hidden=") {
            new_lines.push(format!("Hidden={}", !enable));
            hidden_found = true;
        } else if line.starts_with("X-GNOME-Autostart-enabled=") {
            new_lines.push(format!("X-GNOME-Autostart-enabled={}", enable));
            gnome_enabled_found = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !hidden_found {
        new_lines.push(format!("Hidden={}", !enable));
    }

    if !gnome_enabled_found {
        new_lines.push(format!("X-GNOME-Autostart-enabled={}", enable));
    }

    // If it was a symlink, remove it first so we can write a regular file
    // This fixes "Permission denied" when trying to write to a symlink pointing to a root-owned file
    if is_symlink {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }

    fs::write(path, new_lines.join("\n")).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn toggle_app(path: PathBuf, enable: bool) -> Result<(), String> {
    let path_str = path.to_string_lossy().to_string();

    // Handle Registry Entries
    if path_str.starts_with("REGISTRY::") {
        // Registry toggling is complex (requires deleting/re-adding value).
        // For now, let's return an error or implement a simple "delete to disable" logic later.
        // Or we can move it to a "RunOnce" or similar, but standard way is deleting.
        // User asked for toggle.
        // Let's just say "Not supported for Registry yet" or implement delete/add.
        // Implementing delete/add requires remembering the command.
        return Err("Toggling Registry apps is not supported yet. Use Delete.".to_string());
    }

    // Handle Folder Entries (Rename logic)
    let new_path = if enable {
        if path.extension().map_or(false, |e| e == "disabled") {
            path.with_extension("")
        } else {
            return Ok(());
        }
    } else {
        let mut p = path.clone().into_os_string();
        p.push(".disabled");
        PathBuf::from(p)
    };

    fs::rename(path, new_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn toggle_app(_path: PathBuf, _enable: bool) -> Result<(), String> {
    Err("Not supported on this OS".to_string())
}

#[cfg(target_os = "linux")]
pub fn create_app(name: String, command: String, description: String) -> Result<(), String> {
    if let Some(config_dir) = dirs::config_dir() {
        let autostart_dir = config_dir.join("autostart");
        if !autostart_dir.exists() {
            fs::create_dir_all(&autostart_dir).map_err(|e| e.to_string())?;
        }

        let safe_name = name
            .replace(" ", "-")
            .replace("/", "-")
            .replace("\\", "-")
            .to_lowercase();
        let filename = format!("{}.desktop", safe_name);
        let path = autostart_dir.join(filename);

        let content = format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec={}\nComment={}\nHidden=false\nX-GNOME-Autostart-enabled=true\n",
            name, command, description
        );

        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Could not find config directory".to_string())
    }
}

#[cfg(target_os = "windows")]
pub fn create_app(name: String, command: String, _description: String) -> Result<(), String> {
    if let Some(startup_dir) =
        dirs::data_dir().map(|d| d.join("Microsoft\\Windows\\Start Menu\\Programs\\Startup"))
    {
        if !startup_dir.exists() {
            fs::create_dir_all(&startup_dir).map_err(|e| e.to_string())?;
        }

        let safe_name = name
            .replace(" ", "-")
            .replace("/", "-")
            .replace("\\", "-")
            .to_lowercase();
        let filename = format!("{}.bat", safe_name);
        let path = startup_dir.join(filename);

        let content = format!("@echo off\nstart \"\" \"{}\"", command);

        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Could not find startup directory".to_string())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn create_app(_name: String, _command: String, _description: String) -> Result<(), String> {
    Err("Not supported on this OS".to_string())
}

pub fn delete_app(path: PathBuf) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let path_str = path.to_string_lossy().to_string();
        if path_str.starts_with("REGISTRY::") {
            // Parse ID from "REGISTRY::HKCU::AppName"
            let parts: Vec<&str> = path_str.split("::").collect();
            if parts.len() == 3 {
                let hive = parts[1];
                let name = parts[2];

                let root = if hive == "HKCU" {
                    HKEY_CURRENT_USER
                } else {
                    HKEY_LOCAL_MACHINE
                };
                let hk = RegKey::predef(root);

                // Use open_subkey_with_flags instead of create_subkey for better control and intent
                // KEY_SET_VALUE is required to delete values
                let key = hk.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_SET_VALUE | KEY_QUERY_VALUE)
                    .map_err(|e| {
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            return "Access Denied: Please run the app as Administrator to delete system items.".to_string();
                        }
                        e.to_string()
                    })?;

                key.delete_value(name).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        return "Access Denied: Please run the app as Administrator to delete system items.".to_string();
                    }
                    e.to_string()
                })?;
                return Ok(());
            }
            return Err("Invalid registry path format".to_string());
        }
    }

    fs::remove_file(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            return "Access Denied: Please run the app as Administrator to delete this file."
                .to_string();
        }
        e.to_string()
    })
}
