// Cargo.toml dependencies needed:
// [dependencies]
// slint = "1.3"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// chrono = "0.4"
// dirs = "5.0"
// walkdir = "2.4"
// tar = "0.4"
// flate2 = "1.0"
// reqwest = { version = "0.11", features = ["blocking", "multipart"] }
// tokio = { version = "1", features = ["full"] }

// Save this as ui/appwindow.slint in your project directory

/*
import { Button, VerticalBox, HorizontalBox, ListView, LineEdit, CheckBox } from "std-widgets.slint";

export struct ProfileData {
    name: string,
    created: string,
    desktop: string,
    is_remote: bool,
}

export struct DesktopEnvironmentData {
    name: string,
    is_current: bool,
}

export component AppWindow inherits Window {
    title: "Shapeshifter - Profile Manager";
    background: #0a0a0a;
    preferred-width: 900px;
    preferred-height: 600px;

    in-out property <string> remote-url;
    in-out property <[ProfileData]> profiles;
    in-out property <[DesktopEnvironmentData]> available-desktops;
    in-out property <string> status-message;
    in-out property <bool> show-de-switcher: false;
    
    callback save-remote-url(string);
    callback create-profile(string, bool);
    callback apply-profile(int);
    callback switch-desktop(int);

    HorizontalBox {
        // Sidebar
        sidebar := Rectangle {
            width: 250px;
            background: #141414;
            
            VerticalBox {
                padding: 20px;
                spacing: 20px;
                
                Text {
                    text: "SHAPESHIFTER";
                    font-size: 24px;
                    font-weight: 700;
                    color: #ff6b35;
                }
                
                Rectangle {
                    height: 2px;
                    background: #ff6b35;
                }
                
                Button {
                    text: "📋 Profiles";
                    background: #ff6b35;
                    height: 45px;
                    clicked => { 
                        root.show-de-switcher = false;
                    }
                }
                
                Button {
                    text: "🖥️ Switch DE";
                    background: root.show-de-switcher ? #ff6b35 : #1a1a1a;
                    height: 45px;
                    clicked => { 
                        root.show-de-switcher = true;
                    }
                }
                
                Button {
                    text: "⚙️ Settings";
                    background: #1a1a1a;
                    height: 45px;
                    clicked => { settings-panel.visible = !settings-panel.visible; }
                }
                
                Rectangle {
                    // Spacer
                }
                
                Text {
                    text: "v1.0.0";
                    font-size: 12px;
                    color: #666;
                    horizontal-alignment: center;
                }
            }
        }
        
        // Main content area
        VerticalBox {
            padding: 30px;
            spacing: 20px;
            
            // Header
            HorizontalBox {
                spacing: 20px;
                alignment: space-between;
                
                Text {
                    text: root.show-de-switcher ? "Desktop Environments" : "Saved Profiles";
                    font-size: 28px;
                    font-weight: 600;
                    color: #f5f5f5;
                }
                
                if !root.show-de-switcher: Button {
                    text: "+ New Profile";
                    background: #ff6b35;
                    height: 40px;
                    clicked => { create-dialog.visible = true; }
                }
            }
            
            // Desktop Environment Switcher
            if root.show-de-switcher: de-list := Rectangle {
                background: #1a1a1a;
                border-radius: 8px;
                
                VerticalBox {
                    padding: 20px;
                    spacing: 15px;
                    
                    Text {
                        text: "Available Desktop Environments";
                        font-size: 16px;
                        color: #999;
                    }
                    
                    Text {
                        text: "Switch between installed desktop environments while preserving your settings and packages.";
                        font-size: 13px;
                        color: #666;
                        wrap: word-wrap;
                    }
                    
                    Rectangle {
                        height: 1px;
                        background: #2a2a2a;
                    }
                    
                    ListView {
                        for de[index] in available-desktops: Rectangle {
                            height: 70px;
                            background: de-touch.has-hover ? #252525 : transparent;
                            
                            de-touch := TouchArea {
                                HorizontalBox {
                                    padding: 15px;
                                    spacing: 15px;
                                    
                                    VerticalBox {
                                        spacing: 5px;
                                        
                                        HorizontalBox {
                                            spacing: 10px;
                                            
                                            Text {
                                                text: de.name;
                                                font-size: 18px;
                                                font-weight: 600;
                                                color: #f5f5f5;
                                            }
                                            
                                            if de.is-current: Rectangle {
                                                width: 70px;
                                                height: 20px;
                                                background: #4CAF50;
                                                border-radius: 4px;
                                                
                                                Text {
                                                    text: "CURRENT";
                                                    font-size: 10px;
                                                    font-weight: 700;
                                                    color: #0a0a0a;
                                                    horizontal-alignment: center;
                                                    vertical-alignment: center;
                                                }
                                            }
                                        }
                                        
                                        Text {
                                            text: de.is-current ? "Currently active" : "Click to switch (requires logout)";
                                            font-size: 13px;
                                            color: #999;
                                        }
                                    }
                                    
                                    Rectangle {
                                        // Spacer
                                    }
                                    
                                    if !de.is-current: Button {
                                        text: "Switch";
                                        background: #ff6b35;
                                        width: 100px;
                                        clicked => { root.switch-desktop(index); }
                                    }
                                }
                            }
                            
                            Rectangle {
                                y: parent.height - 1px;
                                height: 1px;
                                background: #2a2a2a;
                            }
                        }
                    }
                }
            }
            
            // Profile list
            if !root.show-de-switcher: profile-list := Rectangle {
                background: #1a1a1a;
                border-radius: 8px;
                
                ListView {
                    for profile[index] in profiles: Rectangle {
                        height: 80px;
                        background: touch-area.has-hover ? #252525 : transparent;
                        
                        touch-area := TouchArea {
                            HorizontalBox {
                                padding: 20px;
                                spacing: 15px;
                                
                                VerticalBox {
                                    spacing: 5px;
                                    
                                    HorizontalBox {
                                        spacing: 10px;
                                        
                                        Text {
                                            text: profile.name;
                                            font-size: 18px;
                                            font-weight: 600;
                                            color: #f5f5f5;
                                        }
                                        
                                        if profile.is-remote: Rectangle {
                                            width: 60px;
                                            height: 20px;
                                            background: #ff6b35;
                                            border-radius: 4px;
                                            
                                            Text {
                                                text: "REMOTE";
                                                font-size: 10px;
                                                font-weight: 700;
                                                color: #0a0a0a;
                                                horizontal-alignment: center;
                                                vertical-alignment: center;
                                            }
                                        }
                                    }
                                    
                                    Text {
                                        text: "Created: " + profile.created;
                                        font-size: 13px;
                                        color: #999;
                                    }
                                    
                                    Text {
                                        text: "Desktop: " + profile.desktop;
                                        font-size: 13px;
                                        color: #999;
                                    }
                                }
                                
                                Rectangle {
                                    // Spacer
                                }
                                
                                Button {
                                    text: "Apply";
                                    background: #ff6b35;
                                    width: 100px;
                                    clicked => { root.apply-profile(index); }
                                }
                            }
                        }
                        
                        Rectangle {
                            y: parent.height - 1px;
                            height: 1px;
                            background: #2a2a2a;
                        }
                    }
                }
            }
            
            // Status bar
            if status-message != "": Rectangle {
                height: auto;
                min-height: 40px;
                background: #252525;
                border-radius: 6px;
                
                VerticalBox {
                    padding: 12px;
                    
                    Text {
                        text: status-message;
                        color: #ff6b35;
                        horizontal-alignment: left;
                        wrap: word-wrap;
                    }
                }
            }
        }
    }
    
    // Settings panel overlay
    if settings-panel.visible: Rectangle {
        width: 100%;
        height: 100%;
        background: #000000cc;
        
        settings-panel := Rectangle {
            property <bool> visible: false;
            width: 500px;
            height: 400px;
            background: #1a1a1a;
            border-radius: 12px;
            x: (parent.width - self.width) / 2;
            y: (parent.height - self.height) / 2;
            
            VerticalBox {
                padding: 30px;
                spacing: 20px;
                
                HorizontalBox {
                    Text {
                        text: "Settings";
                        font-size: 24px;
                        font-weight: 600;
                        color: #f5f5f5;
                    }
                    
                    Rectangle {
                        // Spacer
                    }
                    
                    Button {
                        text: "✕";
                        width: 40px;
                        background: transparent;
                        clicked => { settings-panel.visible = false; }
                    }
                }
                
                Rectangle {
                    height: 1px;
                    background: #2a2a2a;
                }
                
                VerticalBox {
                    spacing: 10px;
                    
                    Text {
                        text: "Remote Backup URL";
                        font-size: 14px;
                        color: #999;
                    }
                    
                    remote-input := LineEdit {
                        text: remote-url;
                        placeholder-text: "https://backup.example.com/profiles";
                    }
                    
                    Button {
                        text: "Save Remote URL";
                        background: #ff6b35;
                        clicked => { 
                            root.save-remote-url(remote-input.text);
                            settings-panel.visible = false;
                        }
                    }
                }
            }
        }
    }
    
    // Create profile dialog
    if create-dialog.visible: Rectangle {
        width: 100%;
        height: 100%;
        background: #000000cc;
        
        create-dialog := Rectangle {
            property <bool> visible: false;
            width: 500px;
            height: 350px;
            background: #1a1a1a;
            border-radius: 12px;
            x: (parent.width - self.width) / 2;
            y: (parent.height - self.height) / 2;
            
            VerticalBox {
                padding: 30px;
                spacing: 20px;
                
                Text {
                    text: "Create New Profile";
                    font-size: 24px;
                    font-weight: 600;
                    color: #f5f5f5;
                }
                
                Rectangle {
                    height: 1px;
                    background: #2a2a2a;
                }
                
                VerticalBox {
                    spacing: 10px;
                    
                    Text {
                        text: "Profile Name";
                        font-size: 14px;
                        color: #999;
                    }
                    
                    profile-name := LineEdit {
                        placeholder-text: "My Profile";
                    }
                }
                
                save-remote-check := CheckBox {
                    text: "Save to remote backup";
                }
                
                HorizontalBox {
                    spacing: 10px;
                    
                    Button {
                        text: "Cancel";
                        background: #2a2a2a;
                        clicked => { create-dialog.visible = false; }
                    }
                    
                    Button {
                        text: "Create Profile";
                        background: #ff6b35;
                        clicked => {
                            root.create-profile(profile-name.text, save-remote-check.checked);
                            create-dialog.visible = false;
                            profile-name.text = "";
                        }
                    }
                }
            }
        }
    }
}
*/

use slint::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};
use std::process::Command;

slint::include_modules!();

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Profile {
    name: String,
    created: String,
    desktop_environment: String,
    packages: Vec<String>,
    config_files: Vec<String>,
    is_remote: bool,
    de_specific_configs: Vec<String>, // Configs that are DE-specific and may conflict
    shared_configs: Vec<String>, // Configs that work across DEs
}

impl Profile {
    fn new(name: String, save_remote: bool) -> Self {
        let de = get_current_desktop_environment();
        let all_configs = get_config_files();
        let conflicts = get_de_config_conflicts(&de);
        
        let (de_specific, shared): (Vec<_>, Vec<_>) = all_configs
            .into_iter()
            .partition(|config| {
                conflicts.iter().any(|conflict| 
                    config.to_lowercase().contains(&conflict.to_lowercase())
                )
            });
        
        Self {
            name,
            created: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            desktop_environment: de,
            packages: get_installed_packages(),
            config_files: Vec::new(), // Deprecated, split into specific types
            is_remote: save_remote,
            de_specific_configs: de_specific,
            shared_configs: shared,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppConfig {
    remote_url: String,
    profiles_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            remote_url: String::new(),
            profiles_dir: home.join(".config/shapeshifter/profiles").to_string_lossy().to_string(),
        }
    }
}

fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".config/shapeshifter/config.json")
}

fn load_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)?;
    fs::write(config_path, json)?;
    Ok(())
}

fn get_installed_packages() -> Vec<String> {
    let mut packages = Vec::new();
    
    // Try dpkg for Debian-based systems
    if let Ok(output) = Command::new("dpkg").args(&["--get-selections"]).output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if let Some(pkg) = line.split_whitespace().next() {
                    packages.push(pkg.to_string());
                }
            }
        }
    }
    
    packages
}

fn get_current_desktop_environment() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn get_available_desktop_environments() -> Vec<String> {
    let mut desktops = Vec::new();
    let xsessions_paths = vec![
        "/usr/share/xsessions",
        "/usr/local/share/xsessions",
    ];
    
    for path in xsessions_paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") {
                        // Parse .desktop file to get the actual DE name
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            for line in content.lines() {
                                if line.starts_with("Name=") {
                                    let de_name = line.trim_start_matches("Name=").to_string();
                                    if !desktops.contains(&de_name) {
                                        desktops.push(de_name);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Also check Wayland sessions
    let wayland_paths = vec![
        "/usr/share/wayland-sessions",
        "/usr/local/share/wayland-sessions",
    ];
    
    for path in wayland_paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            for line in content.lines() {
                                if line.starts_with("Name=") {
                                    let de_name = line.trim_start_matches("Name=").to_string();
                                    if !desktops.contains(&de_name) {
                                        desktops.push(de_name);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    desktops.sort();
    desktops
}

fn get_de_exec_name(de_display_name: &str) -> Option<String> {
    let xsessions_paths = vec![
        "/usr/share/xsessions",
        "/usr/local/share/xsessions",
        "/usr/share/wayland-sessions",
        "/usr/local/share/wayland-sessions",
    ];
    
    for path in xsessions_paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let mut name_matches = false;
                    let mut exec_line = None;
                    
                    for line in content.lines() {
                        if line.starts_with("Name=") && line.trim_start_matches("Name=") == de_display_name {
                            name_matches = true;
                        }
                        if line.starts_with("Exec=") {
                            exec_line = Some(line.trim_start_matches("Exec=").to_string());
                        }
                    }
                    
                    if name_matches && exec_line.is_some() {
                        return exec_line;
                    }
                }
            }
        }
    }
    
    None
}

fn get_de_config_conflicts(de_name: &str) -> Vec<String> {
    // Define known conflicting config directories between DEs
    let conflicts: Vec<String> = match de_name.to_lowercase().as_str() {
        name if name.contains("gnome") => vec![
            "kde".to_string(),
            "plasma".to_string(),
            "xfce4".to_string(),
        ],
        name if name.contains("kde") || name.contains("plasma") => vec![
            "gnome".to_string(),
            "xfce4".to_string(),
        ],
        name if name.contains("xfce") => vec![
            "gnome".to_string(),
            "kde".to_string(),
            "plasma".to_string(),
        ],
        name if name.contains("cinnamon") => vec![
            "gnome".to_string(),
            "kde".to_string(),
            "plasma".to_string(),
        ],
        name if name.contains("mate") => vec![
            "gnome".to_string(),
            "kde".to_string(),
            "plasma".to_string(),
        ],
        _ => vec![],
    };
    
    conflicts
}

fn backup_conflicting_configs(target_de: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let config_dir = home.join(".config");
    let backup_dir = home.join(".config/shapeshifter/de_backups");
    
    fs::create_dir_all(&backup_dir)?;
    
    let conflicts = get_de_config_conflicts(target_de);
    
    for conflict in conflicts {
        let source = config_dir.join(&conflict);
        if source.exists() {
            let backup_path = backup_dir.join(&conflict);
            
            // Create timestamped backup
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let backup_with_time = backup_dir.join(format!("{}_{}", conflict, timestamp));
            
            if backup_path.exists() {
                fs::rename(&backup_path, &backup_with_time)?;
            }
            
            // Copy conflicting config to backup
            copy_dir_recursive(&source, &backup_path)?;
            println!("Backed up {} to {:?}", conflict, backup_path);
        }
    }
    
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

fn restore_de_configs(de_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let backup_dir = home.join(".config/shapeshifter/de_backups");
    let config_dir = home.join(".config");
    
    // Determine which config directory to restore based on DE
    let config_to_restore = match de_name.to_lowercase().as_str() {
        name if name.contains("gnome") => Some("gnome"),
        name if name.contains("kde") || name.contains("plasma") => Some("kde"),
        name if name.contains("xfce") => Some("xfce4"),
        name if name.contains("cinnamon") => Some("cinnamon"),
        name if name.contains("mate") => Some("mate"),
        _ => None,
    };
    
    if let Some(config_name) = config_to_restore {
        let backup_path = backup_dir.join(config_name);
        let restore_path = config_dir.join(config_name);
        
        if backup_path.exists() {
            // Remove current config if exists
            if restore_path.exists() {
                fs::remove_dir_all(&restore_path)?;
            }
            
            // Restore from backup
            copy_dir_recursive(&backup_path, &restore_path)?;
            println!("Restored {} config from backup", config_name);
        }
    }
    
    Ok(())
}

fn get_config_files() -> Vec<String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let config_dir = home.join(".config");
    
    let mut configs = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&config_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    configs.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    
    configs
}

fn load_profiles(profiles_dir: &str) -> Vec<Profile> {
    let path = Path::new(profiles_dir);
    let mut profiles = Vec::new();
    
    if !path.exists() {
        return profiles;
    }
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let profile_file = entry.path().join("profile.json");
            if profile_file.exists() {
                if let Ok(content) = fs::read_to_string(profile_file) {
                    if let Ok(profile) = serde_json::from_str::<Profile>(&content) {
                        profiles.push(profile);
                    }
                }
            }
        }
    }
    
    profiles.sort_by(|a, b| b.created.cmp(&a.created));
    profiles
}

fn create_profile(name: String, save_remote: bool, config: &AppConfig) -> Result<Profile, Box<dyn std::error::Error>> {
    let profile = Profile::new(name.clone(), save_remote);
    
    // Save locally
    let profile_dir = Path::new(&config.profiles_dir).join(&name);
    fs::create_dir_all(&profile_dir)?;
    
    let profile_file = profile_dir.join("profile.json");
    let json = serde_json::to_string_pretty(&profile)?;
    fs::write(profile_file, json)?;
    
    // Save remotely if requested
    if save_remote && !config.remote_url.is_empty() {
        upload_profile(&profile, &config.remote_url)?;
    }
    
    Ok(profile)
}

fn upload_profile(profile: &Profile, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Simplified upload - in production, use reqwest with proper authentication
    println!("Uploading profile {} to {}", profile.name, url);
    // This is a placeholder - implement actual HTTP upload based on your server
    Ok(())
}

fn apply_profile(profile: &Profile) -> Result<(), Box<dyn std::error::Error>> {
    let current_de = get_current_desktop_environment();
    
    // If switching desktop environments
    if current_de != profile.desktop_environment {
        println!("Switching from {} to {}", current_de, profile.desktop_environment);
        
        // Backup any conflicting configs before switching
        backup_conflicting_configs(&profile.desktop_environment)?;
        
        // Restore configs for the target DE if we have them backed up
        restore_de_configs(&profile.desktop_environment)?;
        
        // Write the new session to .dmrc for display manager
        set_default_session(&profile.desktop_environment)?;
        
        println!("Desktop environment will switch to {} on next login", profile.desktop_environment);
    }
    
    // Show package differences (non-conflicting packages)
    let current_packages = get_installed_packages();
    let to_install: Vec<_> = profile.packages.iter()
        .filter(|p| !current_packages.contains(p))
        .collect();
    
    if !to_install.is_empty() {
        println!("Packages to install ({} packages):", to_install.len());
        println!("Run: sudo apt install {}", to_install.join(" "));
    }
    
    Ok(())
}

fn set_default_session(de_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let dmrc_path = home.join(".dmrc");
    
    // Get the exec name for the DE
    if let Some(exec_name) = get_de_exec_name(de_name) {
        // Extract just the session name (first word of exec command)
        let session_name = exec_name.split_whitespace().next().unwrap_or(de_name);
        
        let dmrc_content = format!(
            "[Desktop]\nSession={}\n",
            session_name
        );
        
        fs::write(&dmrc_path, dmrc_content)?;
        
        // Also try to write to accountsservice (requires root, so may fail)
        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        let accountsservice_path = format!("/var/lib/AccountsService/users/{}", username);
        
        if Path::new(&accountsservice_path).exists() {
            // Try to update it, but don't fail if we can't (need sudo)
            let _ = Command::new("sudo")
                .args(&[
                    "sed",
                    "-i",
                    &format!("s/^XSession=.*/XSession={}/", session_name),
                    &accountsservice_path,
                ])
                .output();
        }
        
        println!("Set default session to: {}", session_name);
    }
    
    Ok(())
}

fn switch_de_now(de_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Check if we can switch without logout (limited cases)
    let current_de = get_current_desktop_environment();
    
    if current_de == de_name {
        return Ok("Already running this desktop environment".to_string());
    }
    
    // For most cases, we need to logout and log back in
    set_default_session(de_name)?;
    
    // Provide instructions for immediate switch
    let message = format!(
        "Desktop environment will switch to {} on next login.\n\
        To switch now:\n\
        1. Save your work\n\
        2. Log out (or run: loginctl terminate-user $USER)\n\
        3. Log back in\n\n\
        Your settings and packages will be preserved.",
        de_name
    );
    
    Ok(message)
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let config = load_config();
    
    // Initialize UI
    ui.set_remote_url(config.remote_url.clone().into());
    
    // Get available desktop environments
    let available_des = get_available_desktop_environments();
    let de_models: Vec<DesktopEnvironmentData> = available_des.iter().map(|de| {
        DesktopEnvironmentData {
            name: de.clone().into(),
            is_current: de == &get_current_desktop_environment(),
        }
    }).collect();
    
    let de_model = VecModel::from(de_models);
    ui.set_available_desktops(ModelRc::new(de_model));
    
    // Load and display profiles
    let profiles = load_profiles(&config.profiles_dir);
    let profile_models: Vec<ProfileData> = profiles.iter().map(|p| ProfileData {
        name: p.name.clone().into(),
        created: p.created.clone().into(),
        desktop: p.desktop_environment.clone().into(),
        is_remote: p.is_remote,
    }).collect();
    
    let model = VecModel::from(profile_models);
    ui.set_profiles(ModelRc::new(model));
    
    // Handle save remote URL
    let ui_weak = ui.as_weak();
    ui.on_save_remote_url(move |url| {
        let mut config = load_config();
        config.remote_url = url.to_string();
        if save_config(&config).is_ok() {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_message("Remote URL saved successfully".into());
            }
        }
    });
    
    // Handle create profile
    let ui_weak = ui.as_weak();
    ui.on_create_profile(move |name, save_remote| {
        let config = load_config();
        match create_profile(name.to_string(), save_remote, &config) {
            Ok(profile) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(format!("Profile '{}' created successfully", profile.name).into());
                    
                    // Reload profiles
                    let profiles = load_profiles(&config.profiles_dir);
                    let profile_models: Vec<ProfileData> = profiles.iter().map(|p| ProfileData {
                        name: p.name.clone().into(),
                        created: p.created.clone().into(),
                        desktop: p.desktop_environment.clone().into(),
                        is_remote: p.is_remote,
                    }).collect();
                    
                    let model = VecModel::from(profile_models);
                    ui.set_profiles(ModelRc::new(model));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(format!("Error creating profile: {}", e).into());
                }
            }
        }
    });
    
    // Handle apply profile
    let ui_weak = ui.as_weak();
    ui.on_apply_profile(move |index| {
        let config = load_config();
        let profiles = load_profiles(&config.profiles_dir);
        
        if let Some(profile) = profiles.get(index as usize) {
            match apply_profile(profile) {
                Ok(_) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        let msg = if profile.desktop_environment != get_current_desktop_environment() {
                            format!(
                                "Profile '{}' applied. Desktop will switch to {} on next login. \
                                Save your work and log out to complete the switch.",
                                profile.name, profile.desktop_environment
                            )
                        } else {
                            format!("Profile '{}' applied successfully", profile.name)
                        };
                        ui.set_status_message(msg.into());
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(format!("Error applying profile: {}", e).into());
                    }
                }
            }
        }
    });
    
    // Handle direct DE switch
    let ui_weak = ui.as_weak();
    ui.on_switch_desktop(move |index| {
        let available = get_available_desktop_environments();
        if let Some(de_name) = available.get(index as usize) {
            match switch_de_now(de_name) {
                Ok(msg) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(msg.into());
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(format!("Error switching desktop: {}", e).into());
                    }
                }
            }
        }
    });
    
    ui.run()
}
