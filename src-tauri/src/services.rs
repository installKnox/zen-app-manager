#[cfg(target_os = "linux")]
use std::process::Command;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub state: String,
}

#[allow(dead_code)]
fn is_flatpak() -> bool {
    std::path::Path::new("/.flatpak-info").exists()
}

#[tauri::command]
#[cfg(target_os = "linux")]
pub fn get_system_services() -> Result<Vec<Service>, String> {
    let (program, args) = if is_flatpak() {
        ("flatpak-spawn", vec!["--host", "systemctl", "list-unit-files", "--type=service", "--no-pager", "--no-legend"])
    } else {
        ("/usr/bin/systemctl", vec!["list-unit-files", "--type=service", "--no-pager", "--no-legend"])
    };

    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut services = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].to_string();
            let state = parts[1].to_string();
            
            if state == "enabled" || state == "disabled" {
                services.push(Service {
                    name,
                    state,
                });
            }
        }
    }

    services.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(services)
}

#[tauri::command]
#[cfg(target_os = "windows")]
pub fn get_system_services() -> Result<Vec<Service>, String> {
    // Windows services support can be added later via 'sc' command
    Ok(Vec::new()) 
}

#[tauri::command]
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn get_system_services() -> Result<Vec<Service>, String> {
    Ok(Vec::new())
}

#[tauri::command]
#[cfg(target_os = "linux")]
pub fn toggle_service(name: String, enable: bool) -> Result<(), String> {
    let action = if enable { "enable" } else { "disable" };
    
    let (program, args) = if is_flatpak() {
        ("flatpak-spawn", vec!["--host", "pkexec", "systemctl", action, &name])
    } else {
        ("pkexec", vec!["/usr/bin/systemctl", action, &name])
    };

    // Use pkexec to ask for password securely via GUI
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
#[cfg(not(target_os = "linux"))]
pub fn toggle_service(_name: String, _enable: bool) -> Result<(), String> {
    Err("Service management is currently only supported on Linux".to_string())
}
