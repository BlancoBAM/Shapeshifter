use chrono::Local;
use serde::{Deserialize, Serialize};
use slint::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

slint::include_modules!();

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Profile {
    name: String,
    created: String,
    desktop_environment: String,
    packages: Vec<String>,
    is_remote: bool,
    backup_items: Vec<String>,
    config_archive: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BackupItemConfig {
    name: String,
    path: String,
    enabled: bool,
    category: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ConflictFixConfig {
    name: String,
    description: String,
    enabled: bool,
    category: String,
    apply_fn: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FolderBackupEntry {
    label: String,
    source_path: String,
    dest_path: String,
    enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ScheduledSaveConfig {
    enabled: bool,
    interval_hours: u32,
    last_run: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppConfig {
    remote_url: String,
    profiles_dir: String,
    backup_items: Vec<BackupItemConfig>,
    conflict_fixes: Vec<ConflictFixConfig>,
    folder_backups: Vec<FolderBackupEntry>,
    scheduled_save: ScheduledSaveConfig,
    external_backup_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            remote_url: String::new(),
            profiles_dir: home
                .join(".config/shapeshifter/profiles")
                .to_string_lossy()
                .to_string(),
            backup_items: default_backup_items(),
            conflict_fixes: default_conflict_fixes(),
            folder_backups: Vec::new(),
            scheduled_save: ScheduledSaveConfig {
                enabled: false,
                interval_hours: 24,
                last_run: String::new(),
            },
            external_backup_path: String::new(),
        }
    }
}

fn default_backup_items() -> Vec<BackupItemConfig> {
    vec![
        BackupItemConfig {
            name: "GTK Themes".to_string(),
            path: "$HOME/.themes, $HOME/.local/share/themes".to_string(),
            enabled: true,
            category: "Themes".to_string(),
        },
        BackupItemConfig {
            name: "Icon Themes".to_string(),
            path: "$HOME/.icons, $HOME/.local/share/icons".to_string(),
            enabled: true,
            category: "Themes".to_string(),
        },
        BackupItemConfig {
            name: "Cursor Themes".to_string(),
            path: "$HOME/.local/share/icons".to_string(),
            enabled: true,
            category: "Themes".to_string(),
        },
        BackupItemConfig {
            name: "Fonts".to_string(),
            path: "$HOME/.local/share/fonts, $HOME/.fonts".to_string(),
            enabled: true,
            category: "Appearance".to_string(),
        },
        BackupItemConfig {
            name: "Wallpapers".to_string(),
            path: "$HOME/Pictures/Wallpapers, $HOME/.local/share/wallpapers".to_string(),
            enabled: false,
            category: "Appearance".to_string(),
        },
        BackupItemConfig {
            name: "GNOME Extensions".to_string(),
            path: "$HOME/.local/share/gnome-shell/extensions".to_string(),
            enabled: false,
            category: "Extensions".to_string(),
        },
        BackupItemConfig {
            name: "KDE Plasma Configs".to_string(),
            path: "$HOME/.config/kdeglobals, $HOME/.config/plasma-org.kde.plasma.desktop-appletsrc"
                .to_string(),
            enabled: false,
            category: "DE Config".to_string(),
        },
BackupItemConfig {
            name: "GNOME/DConf Settings".to_string(),
            path: "$HOME/.config/dconf".to_string(),
            enabled: false,
            category: "DE Config".to_string(),
        },
        BackupItemConfig {
            name: "XFCE Config".to_string(),
            path: "$HOME/.config/xfce4".to_string(),
            enabled: false,
            category: "DE Config".to_string(),
        },
        BackupItemConfig {
            name: "Terminal Config".to_string(),
            path: "$HOME/.config/alacritty, $HOME/.config/kitty, $HOME/.config/foot".to_string(),
            enabled: true,
            category: "Apps".to_string(),
        },
        BackupItemConfig {
            name: "Shell Config".to_string(),
            path: "$HOME/.bashrc, $HOME/.zshrc, $HOME/.profile".to_string(),
            enabled: true,
            category: "Apps".to_string(),
        },
        BackupItemConfig {
            name: "Editor Config".to_string(),
            path: "$HOME/.config/nvim, $HOME/.config/Code, $HOME/.vscode".to_string(),
            enabled: false,
            category: "Apps".to_string(),
        },
        BackupItemConfig {
            name: "Flatpak Apps & Data".to_string(),
            path: "$HOME/.var/app".to_string(),
            enabled: false,
            category: "Apps".to_string(),
        },
        BackupItemConfig {
            name: "Desktop Folder".to_string(),
            path: "$HOME/Desktop".to_string(),
            enabled: false,
            category: "Files".to_string(),
        },
        BackupItemConfig {
            name: "Custom Folders".to_string(),
            path: "".to_string(),
            enabled: false,
            category: "Files".to_string(),
        },
    ]
}

fn default_conflict_fixes() -> Vec<ConflictFixConfig> {
    vec![
        ConflictFixConfig {
            name: "Preserve Icon Theme Per-DE".to_string(),
            description: "Store and restore icon theme settings separately for each desktop environment to prevent theme mixing".to_string(),
            enabled: true,
            category: "Themes".to_string(),
            apply_fn: "icon_theme_isolation".to_string(),
        },
        ConflictFixConfig {
            name: "Preserve Cursor Theme Per-DE".to_string(),
            description: "Keep cursor theme settings isolated between desktop environments".to_string(),
            enabled: true,
            category: "Themes".to_string(),
            apply_fn: "cursor_theme_isolation".to_string(),
        },
        ConflictFixConfig {
            name: "Preserve GTK Theme Per-DE".to_string(),
            description: "Store GTK theme settings per-DE so switching back restores the correct theme".to_string(),
            enabled: true,
            category: "Themes".to_string(),
            apply_fn: "gtk_theme_isolation".to_string(),
        },
        ConflictFixConfig {
            name: "Preserve Scaling Per-DE".to_string(),
            description: "Keep display scaling factors separate for each desktop environment".to_string(),
            enabled: true,
            category: "Display".to_string(),
            apply_fn: "scaling_isolation".to_string(),
        },
        ConflictFixConfig {
            name: "Preserve Dark/Light Mode Per-DE".to_string(),
            description: "Store color scheme preference (dark/light) separately for each DE".to_string(),
            enabled: true,
            category: "Display".to_string(),
            apply_fn: "color_scheme_isolation".to_string(),
        },
        ConflictFixConfig {
            name: "Clean Duplicate Menu Entries".to_string(),
            description: "Remove duplicate application entries from menus caused by multiple DEs (terminals, file managers, etc.)".to_string(),
            enabled: true,
            category: "Menus".to_string(),
            apply_fn: "menu_cleanup".to_string(),
        },
        ConflictFixConfig {
            name: "Set Default Apps Per-DE".to_string(),
            description: "Configure default applications (terminal, file manager, editor) appropriate to each DE".to_string(),
            enabled: false,
            category: "Menus".to_string(),
            apply_fn: "default_apps_per_de".to_string(),
        },
        ConflictFixConfig {
            name: "Fix Stale KDE Entries in GNOME".to_string(),
            description: "Hide KDE-specific applications from appearing in GNOME menus after switching".to_string(),
            enabled: true,
            category: "Menus".to_string(),
            apply_fn: "hide_kde_in_gnome".to_string(),
        },
        ConflictFixConfig {
            name: "Fix Stale GNOME Entries in KDE".to_string(),
            description: "Hide GNOME-specific applications from appearing in KDE menus after switching".to_string(),
            enabled: true,
            category: "Menus".to_string(),
            apply_fn: "hide_gnome_in_kde".to_string(),
        },
        ConflictFixConfig {
            name: "Preserve XDG Autostart Per-DE".to_string(),
            description: "Keep autostart entries separate so DE-specific startup apps don't interfere".to_string(),
            enabled: false,
            category: "Startup".to_string(),
            apply_fn: "autostart_isolation".to_string(),
        },
    ]
}

fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".config/shapeshifter/config.json")
}

fn load_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                if config.backup_items.is_empty() {
                    config.backup_items = default_backup_items();
                }
                if config.conflict_fixes.is_empty() {
                    config.conflict_fixes = default_conflict_fixes();
                }
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

fn normalize_remote_target(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Remote target cannot be empty".into());
    }
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
        || trimmed.starts_with('/')
    {
        return Ok(trimmed.to_string());
    }
    Err("Remote target must be an http(s) URL, file:// URL, or absolute path".into())
}

fn resolve_path(input: &str) -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let mut results = Vec::new();

    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let resolved = part
            .replace("$HOME", &home.to_string_lossy())
            .replace("$CONFIG_DIR", &home.join(".config").to_string_lossy())
            .replace("$SHARE_DIR", &home.join(".local/share").to_string_lossy());
        results.push(PathBuf::from(&resolved));
    }
    results
}

fn estimate_path_size(paths: &[PathBuf]) -> String {
    let mut total: u64 = 0;
    for path in paths {
        if path.exists() {
            total += dir_size(path);
        }
    }
    if total == 0 {
        return "Not found".to_string();
    }
    if total < 1024 {
        return std::format!("{} B", total);
    }
    if total < 1024 * 1024 {
        return std::format!("{} KB", total / 1024);
    }
    if total < 1024 * 1024 * 1024 {
        return std::format!("{} MB", total / (1024 * 1024));
    }
    std::format!("{:.1} GB", total as f64 / (1024.0 * 1024.0 * 1024.0))
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ty) = entry.file_type() {
                if ty.is_dir() {
                    total += dir_size(&entry.path());
                } else if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
    }
    total
}

fn get_installed_packages() -> Vec<String> {
    let mut packages = Vec::new();

    if let Ok(output) = Command::new("dpkg").args(["--get-selections"]).output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if let Some(pkg) = line.split_whitespace().next() {
                    if !pkg.starts_with('#') {
                        packages.push(pkg.to_string());
                    }
                }
            }
        }
    }

    if packages.is_empty() {
        if let Ok(output) = Command::new("flatpak").args(["list", "--app"]).output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if let Some(pkg) = line.split_whitespace().next() {
                        packages.push(pkg.to_string());
                    }
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

#[derive(Debug, Clone)]
struct DesktopEnvironment {
    name: String,
    session_id: String,
    is_current: bool,
}

fn get_available_desktop_environments() -> Vec<DesktopEnvironment> {
    let mut desktops: HashMap<String, String> = HashMap::new();
    let current = get_current_desktop_environment();

    let session_paths = vec![
        "/usr/share/xsessions",
        "/usr/local/share/xsessions",
        "/usr/share/wayland-sessions",
        "/usr/local/share/wayland-sessions",
    ];

    for path in session_paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().map_or(false, |ext| ext == "desktop") {
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        let mut name = None;
                        let mut no_display = false;

                        for line in content.lines() {
                            if line.starts_with("Name=") {
                                name = Some(line.trim_start_matches("Name=").to_string());
                            }
                            if line == "NoDisplay=true" {
                                no_display = true;
                            }
                            if name.is_some() && no_display {
                                break;
                            }
                        }

                        if let Some(de_name) = name {
                            if !no_display {
                                let session_id = file_path
                                    .file_stem()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                desktops.entry(de_name).or_insert(session_id);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut result: Vec<DesktopEnvironment> = desktops
        .into_iter()
        .map(|(name, session_id)| DesktopEnvironment {
            is_current: current.to_lowercase().contains(&name.to_lowercase())
                || session_id.to_lowercase() == current.to_lowercase(),
            name,
            session_id,
        })
        .collect();

    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

fn get_de_exec_name(de_display_name: &str) -> Option<String> {
    let session_paths = vec![
        "/usr/share/xsessions",
        "/usr/local/share/xsessions",
        "/usr/share/wayland-sessions",
        "/usr/local/share/wayland-sessions",
    ];

    for path in session_paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let mut name_matches = false;
                    let mut exec_line = None;

                    for line in content.lines() {
                        if line.starts_with("Name=")
                            && line.trim_start_matches("Name=") == de_display_name
                        {
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
    let lower = de_name.to_lowercase();
    if lower.contains("gnome") {
        vec![
            "kde".to_string(),
            "plasma".to_string(),
            "xfce4".to_string(),
            "cosmic".to_string(),
        ]
    } else if lower.contains("kde") || lower.contains("plasma") {
        vec![
            "gnome".to_string(),
            "xfce4".to_string(),
            "cosmic".to_string(),
        ]
    } else if lower.contains("xfce") {
        vec![
            "gnome".to_string(),
            "kde".to_string(),
            "plasma".to_string(),
            "cosmic".to_string(),
        ]
    } else if lower.contains("cinnamon") {
        vec![
            "gnome".to_string(),
            "kde".to_string(),
            "plasma".to_string(),
            "cosmic".to_string(),
        ]
    } else if lower.contains("mate") {
        vec![
            "gnome".to_string(),
            "kde".to_string(),
            "plasma".to_string(),
            "cosmic".to_string(),
        ]
    } else if lower.contains("cosmic") {
        vec![
            "gnome".to_string(),
            "kde".to_string(),
            "plasma".to_string(),
            "xfce4".to_string(),
            "cinnamon".to_string(),
        ]
    } else {
        vec![]
    }
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

fn backup_conflicting_configs(target_de: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let config_dir = home.join(".config");
    let backup_dir = home.join(".config/shapeshifter/de_backups");
    fs::create_dir_all(&backup_dir)?;

    let conflicts = get_de_config_conflicts(target_de);
    for conflict in conflicts {
        let source = config_dir.join(&conflict);
        if source.exists() {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let backup_path = backup_dir.join(std::format!("{}_{}", conflict, timestamp));
            copy_dir_recursive(&source, &backup_path)?;
        }
    }
    Ok(())
}

fn restore_de_configs(de_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let backup_dir = home.join(".config/shapeshifter/de_backups");
    let config_dir = home.join(".config");

    let config_to_restore = match de_name.to_lowercase().as_str() {
        name if name.contains("gnome") => Some("gnome"),
        name if name.contains("kde") || name.contains("plasma") => Some("kde"),
        name if name.contains("xfce") => Some("xfce4"),
        name if name.contains("cinnamon") => Some("cinnamon"),
        name if name.contains("mate") => Some("mate"),
        name if name.contains("cosmic") => Some("cosmic"),
        _ => None,
    };

    if let Some(config_name) = config_to_restore {
        let entries = fs::read_dir(&backup_dir)?;
        let mut latest_backup: Option<(PathBuf, std::time::SystemTime)> = None;

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(&std::format!("{}_", config_name)) {
                if let Ok(meta) = entry.metadata() {
                    if let Some((_, latest_time)) = latest_backup {
                        if meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                            > latest_time
                        {
                            latest_backup = Some((
                                entry.path(),
                                meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                            ));
                        }
                    } else {
                        latest_backup = Some((
                            entry.path(),
                            meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                        ));
                    }
                }
            }
        }

        if let Some((backup_path, _)) = latest_backup {
            let restore_path = config_dir.join(config_name);
            if restore_path.exists() {
                fs::remove_dir_all(&restore_path)?;
            }
            copy_dir_recursive(&backup_path, &restore_path)?;
        }
    }
    Ok(())
}

fn set_default_session(de_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let dmrc_path = home.join(".dmrc");

    if let Some(exec_name) = get_de_exec_name(de_name) {
        let session_name = exec_name.split_whitespace().next().unwrap_or(de_name);
        let dmrc_content = std::format!("[Desktop]\nSession={}\n", session_name);
        fs::write(&dmrc_path, dmrc_content)?;

        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        let accountsservice_path = std::format!("/var/lib/AccountsService/users/{}", username);

        if Path::new(&accountsservice_path).exists() {
            // Try pkexec first (GUI-friendly), fall back to sudo, then direct write
            let sed_cmd = std::format!("s/^XSession=.*/XSession={}/", session_name);
            let result = Command::new("pkexec")
                .args(["sed", "-i", &sed_cmd, &accountsservice_path])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            if result.is_err() || !result.unwrap().success() {
                // Fallback: try sudo
                let sudo_result = Command::new("sudo")
                    .args(["sed", "-i", &sed_cmd, &accountsservice_path])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                if sudo_result.is_err() || !sudo_result.unwrap().success() {
                    // Last resort: try direct write if we have permission
                    if let Ok(content) = fs::read_to_string(&accountsservice_path) {
                        let modified = content.lines()
                            .map(|line| {
                                if line.starts_with("XSession=") {
                                    std::format!("XSession={}", session_name)
                                } else {
                                    line.to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        let _ = fs::write(&accountsservice_path, modified);
                    }
                }
            }
        }
    }
    Ok(())
}

fn collect_config_files(config: &AppConfig) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    for item in &config.backup_items {
        if !item.enabled {
            continue;
        }

        // Special handling for dConf settings (dconf dump)
        if item.path.starts_with("dconf://") || item.path == "dconf dump /" {
            let dconf_output = Command::new("dconf")
                .args(["dump", "/"])
                .output();
            if let Ok(output) = dconf_output {
                if output.status.success() {
                    let temp_dir = home.join(".config/shapeshifter/temp");
                    let _ = fs::create_dir_all(&temp_dir);
                    let dconf_file = temp_dir.join("dconf-settings.ini");
                    if fs::write(&dconf_file, &output.stdout).is_ok() {
                        files.push(dconf_file.to_path_buf());
                    }
                }
            }
            continue;
        }

        for path in resolve_path(&item.path) {
            if path.exists() {
                files.push(path);
            }
        }
    }

    // Also include GNOME extension version-specific directories
    let gnome_ext_base = home.join(".local/share/gnome-shell/extensions");
    if gnome_ext_base.exists() {
        if let Ok(entries) = fs::read_dir(&gnome_ext_base) {
            for entry in entries.flatten() {
                let ext_path = entry.path();
                if ext_path.is_dir() {
                    // Check for version subdirectory (e.g., extension@author/40/, 3.38/, etc.)
                    if let Ok(sub_entries) = fs::read_dir(&ext_path) {
                        for sub_entry in sub_entries.flatten() {
                            if sub_entry.file_type().map_or(false, |t| t.is_dir()) {
                                // This is a version subdirectory, the parent already covers it
                            }
                        }
                    }
                }
            }
        }
    }

    // Always include common dotfiles
    let dotfiles = vec![
        ".bashrc",
        ".zshrc",
        ".profile",
        ".Xresources",
        ".xsettingsd",
    ];
    for dotfile in dotfiles {
        let p = home.join(dotfile);
        if p.exists() && !files.contains(&p) {
            files.push(p);
        }
    }

    files
}

fn create_profile_archive(
    profile_name: &str,
    config: &AppConfig,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let profile_dir = Path::new(&config.profiles_dir).join(profile_name);
    fs::create_dir_all(&profile_dir)?;

    let archive_path = profile_dir.join("configs.tar.gz");
    let config_files = collect_config_files(config);

    if !config_files.is_empty() {
        let tar_file = fs::File::create(&archive_path)?;
        let enc = flate2::write::GzEncoder::new(tar_file, flate2::Compression::default());
        let mut tar_builder = tar::Builder::new(enc);

        for src_path in &config_files {
            if src_path.is_dir() {
                if let Some(name) = src_path.file_name() {
                    tar_builder.append_dir_all(name, src_path)?;
                }
            } else if src_path.is_file() {
                if let Some(name) = src_path.file_name() {
                    let mut file = fs::File::open(src_path)?;
                    tar_builder.append_file(name, &mut file)?;
                }
            }
        }

        tar_builder.finish()?;
    }

    Ok(archive_path)
}

fn load_profiles(profiles_dir: &str) -> Vec<Profile> {
    let path = Path::new(profiles_dir);
    let mut profiles = Vec::new();

    if !path.exists() {
        return profiles;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.file_type().map_or(false, |t| t.is_dir()) {
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
    }

    profiles.sort_by(|a, b| b.created.cmp(&a.created));
    profiles
}

fn create_profile(
    name: String,
    save_remote: bool,
    config: &AppConfig,
) -> Result<Profile, Box<dyn std::error::Error>> {
    if name.trim().is_empty() {
        return Err("Profile name cannot be empty".into());
    }

    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Profile name cannot contain path separators".into());
    }

    let profile_dir = Path::new(&config.profiles_dir).join(&name);
    if profile_dir.exists() {
        return Err(std::format!("Profile '{}' already exists", name).into());
    }

    let packages = get_installed_packages();

    let _ = create_profile_archive(&name, config);

    let profile = Profile {
        name: name.clone(),
        created: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        desktop_environment: get_current_desktop_environment(),
        packages,
        is_remote: save_remote,
        backup_items: config
            .backup_items
            .iter()
            .filter(|i| i.enabled)
            .map(|i| i.name.clone())
            .collect(),
        config_archive: "configs.tar.gz".to_string(),
    };

    fs::create_dir_all(&profile_dir)?;
    let profile_file = profile_dir.join("profile.json");
    let json = serde_json::to_string_pretty(&profile)?;
    fs::write(profile_file, json)?;

    if save_remote && !config.remote_url.trim().is_empty() {
        let _ = upload_profile(&profile, &config.remote_url);
    }

    Ok(profile)
}

fn rename_profile(
    old_name: &str,
    new_name: &str,
    config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if new_name.trim().is_empty() {
        return Err("Profile name cannot be empty".into());
    }
    if new_name.contains('/') || new_name.contains('\\') || new_name.contains("..") {
        return Err("Profile name cannot contain path separators".into());
    }

    let old_dir = Path::new(&config.profiles_dir).join(old_name);
    let new_dir = Path::new(&config.profiles_dir).join(new_name);

    if !old_dir.exists() {
        return Err(std::format!("Profile '{}' not found", old_name).into());
    }
    if new_dir.exists() {
        return Err(std::format!("Profile '{}' already exists", new_name).into());
    }

    fs::rename(&old_dir, &new_dir)?;

    // Update the profile.json name field
    let profile_file = new_dir.join("profile.json");
    if let Ok(content) = fs::read_to_string(&profile_file) {
        if let Ok(mut profile) = serde_json::from_str::<Profile>(&content) {
            profile.name = new_name.to_string();
            let json = serde_json::to_string_pretty(&profile)?;
            fs::write(&profile_file, json)?;
        }
    }

    Ok(())
}

fn update_profile_remote_status(
    profile_name: &str,
    is_remote: bool,
    config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let profile_dir = Path::new(&config.profiles_dir).join(profile_name);
    let profile_file = profile_dir.join("profile.json");

    if let Ok(content) = fs::read_to_string(&profile_file) {
        if let Ok(mut profile) = serde_json::from_str::<Profile>(&content) {
            profile.is_remote = is_remote;
            let json = serde_json::to_string_pretty(&profile)?;
            fs::write(&profile_file, json)?;
        }
    }

    Ok(())
}

fn upload_profile(profile: &Profile, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let normalized = normalize_remote_target(url)?;

    if normalized.starts_with("file://") || normalized.starts_with('/') {
        let base_path = if let Some(path) = normalized.strip_prefix("file://") {
            PathBuf::from(path)
        } else {
            PathBuf::from(&normalized)
        };
        fs::create_dir_all(&base_path)?;
        let file_name = std::format!(
            "{}-{}.json",
            profile.name,
            profile.created.replace([' ', ':'], "_")
        );
        let remote_file = base_path.join(file_name);
        fs::write(remote_file, serde_json::to_string_pretty(profile)?)?;
        return Ok(());
    }

    let client = reqwest::blocking::Client::new();
    let body = serde_json::to_string_pretty(profile)?;
    let response = client
        .post(&normalized)
        .header("content-type", "application/json")
        .body(body)
        .send()?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(std::format!(
            "Remote server rejected profile upload: {}",
            response.status()
        )
        .into())
    }
}

fn apply_profile(
    profile: &Profile,
    config: &AppConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let current_de = get_current_desktop_environment();
    let mut messages = Vec::new();

    if current_de != profile.desktop_environment {
        backup_conflicting_configs(&profile.desktop_environment)?;
        restore_de_configs(&profile.desktop_environment)?;
        set_default_session(&profile.desktop_environment)?;
        messages.push(std::format!(
            "Desktop will switch to {} on next login",
            profile.desktop_environment
        ));
    }

    let profile_dir = Path::new(&config.profiles_dir).join(&profile.name);
    let archive_path = profile_dir.join("configs.tar.gz");
    if archive_path.exists() {
        let tar_file = fs::File::open(&archive_path)?;
        let dec = flate2::read::GzDecoder::new(tar_file);
        let mut archive = tar::Archive::new(dec);
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        archive.unpack(&home)?;
        messages.push("Configuration files restored".to_string());
    }

    let current_packages = get_installed_packages();
    let to_install: Vec<_> = profile
        .packages
        .iter()
        .filter(|p| !current_packages.contains(p))
        .collect();

    if !to_install.is_empty() {
        let pkg_list: Vec<&str> = to_install.iter().map(|s| s.as_str()).collect();
        let mut install_failed = false;
        for chunk in pkg_list.chunks(100) {
            let mut cmd = Command::new("sudo");
            cmd.arg("apt").arg("install").arg("-y").arg("-qq");
            for pkg in chunk {
                cmd.arg(pkg);
            }
            match cmd.output() {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        eprintln!("apt install failed: {}\n{}", stderr, stdout);
                        install_failed = true;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to run apt install: {}", e);
                    install_failed = true;
                }
            }
        }
        if install_failed {
            messages.push(std::format!(
                "Attempted to install {} packages (some may have failed)",
                to_install.len()
            ));
        } else {
            messages.push(std::format!(
                "Installed {} missing packages",
                to_install.len()
            ));
        }
    }

    if messages.is_empty() {
        messages.push("Profile applied successfully".to_string());
    }

    Ok(messages.join(". "))
}

fn delete_profile(name: &str, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let profile_dir = Path::new(&config.profiles_dir).join(name);
    if profile_dir.exists() {
        fs::remove_dir_all(profile_dir)?;
    }
    Ok(())
}

fn export_profile(name: &str, config: &AppConfig) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let profile_dir = Path::new(&config.profiles_dir).join(name);
    if !profile_dir.exists() {
        return Err(std::format!("Profile directory not found: {:?}", profile_dir).into());
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let tarball_path = home.join(std::format!("{}.tar.gz", name));

    let tar_file = fs::File::create(&tarball_path)?;
    let enc = flate2::write::GzEncoder::new(tar_file, flate2::Compression::default());
    let mut tar_builder = tar::Builder::new(enc);

    tar_builder.append_dir_all(name, &profile_dir)?;
    tar_builder.finish()?;

    Ok(tarball_path)
}

fn import_profile(
    tarball_path: &str,
    config: &AppConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let path = Path::new(tarball_path);
    if !path.exists() {
        return Err(std::format!("Archive not found: {}", tarball_path).into());
    }

    let tar_file = fs::File::open(path)?;
    let dec = flate2::read::GzDecoder::new(tar_file);
    let mut archive = tar::Archive::new(dec);

    let profiles_dir = Path::new(&config.profiles_dir);
    fs::create_dir_all(profiles_dir)?;
    archive.unpack(profiles_dir)?;

    let profile_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported")
        .trim_end_matches(".tar")
        .to_string();

    Ok(profile_name)
}

fn switch_de_now(de_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let current_de = get_current_desktop_environment();
    if current_de == de_name {
        return Ok("Already running this desktop environment".to_string());
    }

    set_default_session(de_name)?;

    Ok(std::format!(
        "Desktop environment will switch to {} on next login.\nTo switch now: save your work, log out, and log back in.",
        de_name
    ))
}

fn apply_conflict_fixes(config: &AppConfig) -> Result<String, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let current_de = get_current_desktop_environment();
    let mut applied = Vec::new();

    for fix in &config.conflict_fixes {
        if !fix.enabled {
            continue;
        }

        match fix.apply_fn.as_str() {
            "icon_theme_isolation" => {
                let de_dir = home.join(std::format!(
                    ".config/shapeshifter/de_settings/{}/icon-theme",
                    current_de.to_lowercase()
                ));
                fs::create_dir_all(&de_dir)?;
                let gtk_settings = home.join(".config/gtk-3.0/settings.ini");
                if gtk_settings.exists() {
                    fs::copy(&gtk_settings, de_dir.join("gtk-settings.ini"))?;
                    applied.push(fix.name.clone());
                }
            }
            "cursor_theme_isolation" => {
                let de_dir = home.join(std::format!(
                    ".config/shapeshifter/de_settings/{}/cursor-theme",
                    current_de.to_lowercase()
                ));
                fs::create_dir_all(&de_dir)?;
                let index_theme = home.join(".icons/default/index.theme");
                if index_theme.exists() {
                    fs::copy(&index_theme, de_dir.join("index.theme"))?;
                    applied.push(fix.name.clone());
                }
            }
            "gtk_theme_isolation" => {
                let de_dir = home.join(std::format!(
                    ".config/shapeshifter/de_settings/{}/gtk-theme",
                    current_de.to_lowercase()
                ));
                fs::create_dir_all(&de_dir)?;
                let gtk_settings = home.join(".config/gtk-3.0/settings.ini");
                if gtk_settings.exists() {
                    fs::copy(&gtk_settings, de_dir.join("gtk-settings.ini"))?;
                    applied.push(fix.name.clone());
                }
            }
            "scaling_isolation" => {
                let de_dir = home.join(std::format!(
                    ".config/shapeshifter/de_settings/{}/display",
                    current_de.to_lowercase()
                ));
                fs::create_dir_all(&de_dir)?;
                let monitors_xml = home.join(".config/monitors.xml");
                if monitors_xml.exists() {
                    fs::copy(&monitors_xml, de_dir.join("monitors.xml"))?;
                    applied.push(fix.name.clone());
                }
            }
            "color_scheme_isolation" => {
                let de_dir = home.join(std::format!(
                    ".config/shapeshifter/de_settings/{}/color-scheme",
                    current_de.to_lowercase()
                ));
                fs::create_dir_all(&de_dir)?;
                let gtk_settings = home.join(".config/gtk-3.0/settings.ini");
                if gtk_settings.exists() {
                    fs::copy(&gtk_settings, de_dir.join("gtk-settings.ini"))?;
                    applied.push(fix.name.clone());
                }
            }
            "menu_cleanup" => {
                let autostart_dir = home.join(".config/autostart");
                if autostart_dir.exists() {
                    let entries = fs::read_dir(&autostart_dir)?;
                    for entry in entries.flatten() {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if content.contains("OnlyShowIn=") && !content.contains(&current_de) {
                                let hidden_path = entry.path().with_extension("desktop.hidden");
                                fs::rename(entry.path(), hidden_path)?;
                            }
                        }
                    }
                    applied.push(fix.name.clone());
                }
            }
            "default_apps_per_de" => {
                let mimeapps = home.join(".config/mimeapps.list");
                let defaults = match current_de.to_lowercase().as_str() {
                    name if name.contains("gnome") => Some("[Default Applications]\norg.gnome.Terminal.desktop;\norg.gnome.Nautilus.desktop;\norg.gnome.TextEditor.desktop;\n"),
                    name if name.contains("kde") || name.contains("plasma") => Some("[Default Applications]\norg.kde.konsole.desktop;\norg.kde.dolphin.desktop;\norg.kde.kate.desktop;\n"),
                    name if name.contains("xfce") => Some("[Default Applications]\nxfce4-terminal.desktop;\nThunar.desktop;\nmousepad.desktop;\n"),
                    _ => None,
                };
                if let Some(default_content) = defaults {
                    fs::write(&mimeapps, default_content)?;
                    applied.push(fix.name.clone());
                }
            }
            "hide_kde_in_gnome" => {
                hide_apps_by_keyword(&home, &["kde", "plasma", "org.kde"], &current_de)?;
                applied.push(fix.name.clone());
            }
            "hide_gnome_in_kde" => {
                hide_apps_by_keyword(&home, &["gnome", "org.gnome"], &current_de)?;
                applied.push(fix.name.clone());
            }
            "autostart_isolation" => {
                let de_autostart = home.join(std::format!(
                    ".config/shapeshifter/de_settings/{}/autostart",
                    current_de.to_lowercase()
                ));
                let user_autostart = home.join(".config/autostart");
                if user_autostart.exists() {
                    fs::create_dir_all(&de_autostart)?;
                    copy_dir_recursive(&user_autostart, &de_autostart)?;
                    applied.push(fix.name.clone());
                }
            }
            _ => {}
        }
    }

    if applied.is_empty() {
        Ok("No conflict fixes were enabled".to_string())
    } else {
        Ok(std::format!(
            "Applied {} fixes: {}",
            applied.len(),
            applied.join(", ")
        ))
    }
}

fn hide_apps_by_keyword(
    home: &PathBuf,
    keywords: &[&str],
    current_de: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let local_apps = home
        .join(".local/share/applications")
        .to_string_lossy()
        .to_string();
    let app_dirs: Vec<&str> = vec![
        "/usr/share/applications",
        "/usr/local/share/applications",
        &local_apps,
    ];

    for dir in app_dirs {
        let dir_path = Path::new(&dir);
        if !dir_path.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext != "desktop" {
                        continue;
                    }
                } else {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let lower_content = content.to_lowercase();
                    let matches = keywords.iter().any(|kw| lower_content.contains(kw));
                    let only_shows_in = content.contains("OnlyShowIn=");
                    let not_shows_in = content.contains("NotShowIn=");

                    if matches && !only_shows_in && !not_shows_in {
                        let mut modified = content.clone();
                        modified.push_str(&std::format!("\nNotShowIn={};\n", current_de));
                        fs::write(entry.path(), modified)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn run_folder_backup(source: &str, dest: &str) -> Result<String, Box<dyn std::error::Error>> {
    let src = Path::new(source);
    let dst = Path::new(dest);

    if !src.exists() {
        return Err(std::format!("Source path does not exist: {}", source).into());
    }

    fs::create_dir_all(dst)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let dest_path = dst.join(src.file_name().unwrap_or(std::ffi::OsStr::new("backup")));
    let final_dest = if dest_path.exists() {
        let renamed = dst.join(std::format!(
            "{}_{}",
            src.file_name()
                .unwrap_or(std::ffi::OsStr::new("backup"))
                .to_string_lossy(),
            timestamp
        ));
        copy_dir_recursive(src, &renamed)?;
        renamed
    } else {
        copy_dir_recursive(src, &dest_path)?;
        dest_path
    };

    Ok(std::format!(
        "Backed up {} to {}",
        source,
        final_dest.display()
    ))
}

fn profile_to_ui(p: &Profile) -> ProfileData {
    ProfileData {
        name: p.name.clone().into(),
        created: p.created.clone().into(),
        desktop: p.desktop_environment.clone().into(),
        is_remote: p.is_remote,
        package_count: p.packages.len() as i32,
        config_count: p.backup_items.len() as i32,
    }
}

fn de_to_ui(de: &DesktopEnvironment) -> DesktopEnvironmentData {
    DesktopEnvironmentData {
        name: de.name.clone().into(),
        is_current: de.is_current,
        session_id: de.session_id.clone().into(),
    }
}

fn backup_item_to_ui(item: &BackupItemConfig, _index: usize) -> BackupItemData {
    let paths = resolve_path(&item.path);
    let size = estimate_path_size(&paths);
    BackupItemData {
        name: item.name.clone().into(),
        path: item.path.clone().into(),
        enabled: item.enabled,
        category: item.category.clone().into(),
        size_estimate: size.into(),
    }
}

fn conflict_fix_to_ui(fix: &ConflictFixConfig) -> ConflictFixData {
    ConflictFixData {
        name: fix.name.clone().into(),
        description: fix.description.clone().into(),
        enabled: fix.enabled,
        category: fix.category.clone().into(),
    }
}

fn folder_backup_to_ui(fb: &FolderBackupEntry) -> FolderBackupData {
    FolderBackupData {
        label: fb.label.clone().into(),
        source_path: fb.source_path.clone().into(),
        dest_path: fb.dest_path.clone().into(),
        enabled: fb.enabled,
    }
}

fn scheduled_save_to_ui(config: &AppConfig) -> ScheduledSaveData {
    ScheduledSaveData {
        label: "Auto-Save Profile".to_string().into(),
        interval: std::format!("{} hours", config.scheduled_save.interval_hours).into(),
        enabled: config.scheduled_save.enabled,
        last_run: if config.scheduled_save.last_run.is_empty() {
            "Never".to_string().into()
        } else {
            config.scheduled_save.last_run.clone().into()
        },
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let config = load_config();

    let current_de = get_current_desktop_environment();
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    ui.set_current_de(current_de.clone().into());
    ui.set_current_user(username.into());
    ui.set_remote_url(config.remote_url.clone().into());
    ui.set_external_backup_path(config.external_backup_path.clone().into());
    ui.set_auto_save_enabled(config.scheduled_save.enabled);
    ui.set_auto_save_interval(config.scheduled_save.interval_hours as i32);
    ui.set_application_version(env!("CARGO_PKG_VERSION").into());

    let available_des = get_available_desktop_environments();
    let de_models: Vec<DesktopEnvironmentData> = available_des.iter().map(de_to_ui).collect();
    ui.set_available_desktops(ModelRc::new(VecModel::from(de_models)));

    let profiles = load_profiles(&config.profiles_dir);
    let profile_models: Vec<ProfileData> = profiles.iter().map(profile_to_ui).collect();
    ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));

    let backup_items: Vec<BackupItemData> = config
        .backup_items
        .iter()
        .enumerate()
        .map(|(i, item)| backup_item_to_ui(item, i))
        .collect();
    ui.set_backup_items(ModelRc::new(VecModel::from(backup_items)));

    let conflict_fixes: Vec<ConflictFixData> = config
        .conflict_fixes
        .iter()
        .map(conflict_fix_to_ui)
        .collect();
    ui.set_conflict_fixes(ModelRc::new(VecModel::from(conflict_fixes)));

    let folder_backups: Vec<FolderBackupData> = config
        .folder_backups
        .iter()
        .map(folder_backup_to_ui)
        .collect();
    ui.set_folder_backups(ModelRc::new(VecModel::from(folder_backups)));

    let scheduled = scheduled_save_to_ui(&config);
    ui.set_scheduled_saves(ModelRc::new(VecModel::from(vec![scheduled])));

    let ui_weak = ui.as_weak();
    ui.on_save_remote_url(move |url| {
        if let Some(ui) = ui_weak.upgrade() {
            match normalize_remote_target(&url.to_string()) {
                Ok(normalized) => {
                    let mut config = load_config();
                    config.remote_url = normalized.clone();
                    match save_config(&config) {
                        Ok(()) => {
                            ui.set_remote_url(normalized.into());
                            ui.set_status_message("Remote target saved successfully".into());
                            ui.set_status_type("success".into());
                        }
                        Err(error) => {
                            ui.set_status_message(
                                std::format!("Failed to save remote target: {}", error).into(),
                            );
                            ui.set_status_type("error".into());
                        }
                    }
                }
                Err(error) => {
                    ui.set_status_message(std::format!("Invalid remote target: {}", error).into());
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_create_profile(move |name, save_remote| {
        let config = load_config();
        match create_profile(name.to_string(), save_remote, &config) {
            Ok(profile) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Profile '{}' created successfully", profile.name).into(),
                    );
                    ui.set_status_type("success".into());
                    let profiles = load_profiles(&config.profiles_dir);
                    let profile_models: Vec<ProfileData> =
                        profiles.iter().map(profile_to_ui).collect();
                    ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(std::format!("Error creating profile: {}", e).into());
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_apply_profile(move |index| {
        let config = load_config();
        let profiles = load_profiles(&config.profiles_dir);
        if let Some(profile) = profiles.get(index as usize) {
            match apply_profile(profile, &config) {
                Ok(msg) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(msg.into());
                        ui.set_status_type("success".into());
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(std::format!("Error applying profile: {}", e).into());
                        ui.set_status_type("error".into());
                    }
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_delete_profile(move |index| {
        let config = load_config();
        let profiles = load_profiles(&config.profiles_dir);
        if let Some(profile) = profiles.get(index as usize) {
            match delete_profile(&profile.name, &config) {
                Ok(()) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(
                            std::format!("Profile '{}' deleted", profile.name).into(),
                        );
                        ui.set_status_type("success".into());
                        let profiles = load_profiles(&config.profiles_dir);
                        let profile_models: Vec<ProfileData> =
                            profiles.iter().map(profile_to_ui).collect();
                        ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(std::format!("Error deleting profile: {}", e).into());
                        ui.set_status_type("error".into());
                    }
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_export_profile(move |index| {
        let config = load_config();
        let profiles = load_profiles(&config.profiles_dir);
        if let Some(profile) = profiles.get(index as usize) {
            match export_profile(&profile.name, &config) {
                Ok(path) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(
                            std::format!("Profile exported to {}", path.display()).into(),
                        );
                        ui.set_status_type("success".into());
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(
                            std::format!("Error exporting profile: {}", e).into(),
                        );
                        ui.set_status_type("error".into());
                    }
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_import_profile(move |path| {
        let config = load_config();
        match import_profile(&path.to_string(), &config) {
            Ok(name) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Profile '{}' imported successfully", name).into(),
                    );
                    ui.set_status_type("success".into());
                    let profiles = load_profiles(&config.profiles_dir);
                    let profile_models: Vec<ProfileData> =
                        profiles.iter().map(profile_to_ui).collect();
                    ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(std::format!("Error importing profile: {}", e).into());
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_switch_desktop(move |index| {
        let available = get_available_desktop_environments();
        if let Some(de) = available.get(index as usize) {
            match switch_de_now(&de.name) {
                Ok(msg) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(msg.into());
                        ui.set_status_type("info".into());
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(
                            std::format!("Error switching desktop: {}", e).into(),
                        );
                        ui.set_status_type("error".into());
                    }
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_toggle_backup_item(move |index, enabled| {
        let mut config = load_config();
        if let Some(item) = config.backup_items.get_mut(index as usize) {
            item.enabled = enabled;
            if let Err(e) = save_config(&config) {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Failed to save backup items config: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
                return;
            }
            if let Some(ui) = ui_weak.upgrade() {
                let backup_items: Vec<BackupItemData> = config
                    .backup_items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| backup_item_to_ui(item, i))
                    .collect();
                ui.set_backup_items(ModelRc::new(VecModel::from(backup_items)));
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_toggle_conflict_fix(move |index, enabled| {
        let mut config = load_config();
        if let Some(fix) = config.conflict_fixes.get_mut(index as usize) {
            fix.enabled = enabled;
            if let Err(e) = save_config(&config) {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Failed to save conflict fixes config: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
                return;
            }
            if let Some(ui) = ui_weak.upgrade() {
                let conflict_fixes: Vec<ConflictFixData> = config
                    .conflict_fixes
                    .iter()
                    .map(conflict_fix_to_ui)
                    .collect();
                ui.set_conflict_fixes(ModelRc::new(VecModel::from(conflict_fixes)));
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_add_folder_backup(move |source, dest| {
        let mut config = load_config();
        let label = Path::new(&source.to_string())
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Backup".to_string());
        config.folder_backups.push(FolderBackupEntry {
            label,
            source_path: source.to_string(),
            dest_path: dest.to_string(),
            enabled: true,
        });
        if let Err(e) = save_config(&config) {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_message(
                    std::format!("Failed to save folder backup config: {}", e).into(),
                );
                ui.set_status_type("error".into());
            }
            return;
        }
        if let Some(ui) = ui_weak.upgrade() {
            let folder_backups: Vec<FolderBackupData> = config
                .folder_backups
                .iter()
                .map(folder_backup_to_ui)
                .collect();
            ui.set_folder_backups(ModelRc::new(VecModel::from(folder_backups)));
            ui.set_status_message("Folder backup added".into());
            ui.set_status_type("success".into());
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_remove_folder_backup(move |index| {
        let mut config = load_config();
        if (index as usize) < config.folder_backups.len() {
            config.folder_backups.remove(index as usize);
            if let Err(e) = save_config(&config) {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Failed to save folder backup config: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
                return;
            }
            if let Some(ui) = ui_weak.upgrade() {
                let folder_backups: Vec<FolderBackupData> = config
                    .folder_backups
                    .iter()
                    .map(folder_backup_to_ui)
                    .collect();
                ui.set_folder_backups(ModelRc::new(VecModel::from(folder_backups)));
                ui.set_status_message("Folder backup removed".into());
                ui.set_status_type("success".into());
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_toggle_folder_backup(move |index, enabled| {
        let mut config = load_config();
        if let Some(fb) = config.folder_backups.get_mut(index as usize) {
            fb.enabled = enabled;
            if let Err(e) = save_config(&config) {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Failed to save folder backup config: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
                return;
            }
            if let Some(ui) = ui_weak.upgrade() {
                let folder_backups: Vec<FolderBackupData> = config
                    .folder_backups
                    .iter()
                    .map(folder_backup_to_ui)
                    .collect();
                ui.set_folder_backups(ModelRc::new(VecModel::from(folder_backups)));
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_run_folder_backup(move |index| {
        let config = load_config();
        if let Some(fb) = config.folder_backups.get(index as usize) {
            match run_folder_backup(&fb.source_path, &fb.dest_path) {
                Ok(msg) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(msg.into());
                        ui.set_status_type("success".into());
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(std::format!("Backup failed: {}", e).into());
                        ui.set_status_type("error".into());
                    }
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_toggle_scheduled_save(move |index, enabled| {
        let mut config = load_config();
        if index == 0 {
            config.scheduled_save.enabled = enabled;
            if let Err(e) = save_config(&config) {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Failed to save auto-save settings: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
                return;
            }
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_auto_save_enabled(enabled);
                let scheduled = scheduled_save_to_ui(&config);
                ui.set_scheduled_saves(ModelRc::new(VecModel::from(vec![scheduled])));
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_run_now_scheduled_save(move |_index| {
        let config = load_config();
        match create_profile(
            std::format!("Auto-Save {}", Local::now().format("%Y-%m-%d %H:%M")),
            false,
            &config,
        ) {
            Ok(profile) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Auto-saved profile: {}", profile.name).into(),
                    );
                    ui.set_status_type("success".into());
                    let profiles = load_profiles(&config.profiles_dir);
                    let profile_models: Vec<ProfileData> =
                        profiles.iter().map(profile_to_ui).collect();
                    ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(std::format!("Auto-save failed: {}", e).into());
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_save_auto_save_settings(move |enabled, interval| {
        let mut config = load_config();
        config.scheduled_save.enabled = enabled;
        config.scheduled_save.interval_hours = interval as u32;
        match save_config(&config) {
            Ok(()) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message("Auto-save settings saved".into());
                    ui.set_status_type("success".into());
                    let scheduled = scheduled_save_to_ui(&config);
                    ui.set_scheduled_saves(ModelRc::new(VecModel::from(vec![scheduled])));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(std::format!("Failed to save settings: {}", e).into());
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_save_external_backup_path(move |path| {
        let mut config = load_config();
        config.external_backup_path = path.to_string();
        match save_config(&config) {
            Ok(()) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_external_backup_path(path);
                    ui.set_status_message("External backup path saved".into());
                    ui.set_status_type("success".into());
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(std::format!("Failed to save path: {}", e).into());
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_apply_conflict_fixes(move || {
        let config = load_config();
        match apply_conflict_fixes(&config) {
            Ok(msg) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(msg.into());
                    ui.set_status_type("success".into());
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(std::format!("Error applying fixes: {}", e).into());
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_refresh_all(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let available_des = get_available_desktop_environments();
            let de_models: Vec<DesktopEnvironmentData> =
                available_des.iter().map(de_to_ui).collect();
            ui.set_available_desktops(ModelRc::new(VecModel::from(de_models)));

            let profiles = load_profiles(&load_config().profiles_dir);
            let profile_models: Vec<ProfileData> = profiles.iter().map(profile_to_ui).collect();
            ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));

            ui.set_status_message("Refreshed desktop environments and profiles".into());
            ui.set_status_type("success".into());
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_edit_profile(move |index| {
        let config = load_config();
        let profiles = load_profiles(&config.profiles_dir);
        if let Some(profile) = profiles.get(index as usize) {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_edit_profile_name(profile.name.clone().into());
                ui.set_edit_profile_remote(profile.is_remote);
                ui.set_show_edit_dialog(true);
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_save_edit_profile(move |old_name, new_name, save_remote| {
        let config = load_config();
        match rename_profile(&old_name.to_string(), &new_name.to_string(), &config) {
            Ok(()) => {
                if let Err(e) = update_profile_remote_status(
                    &new_name.to_string(),
                    save_remote,
                    &config,
                ) {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(
                            std::format!("Failed to update remote status: {}", e).into(),
                        );
                        ui.set_status_type("error".into());
                    }
                }
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Profile '{}' updated successfully", new_name).into(),
                    );
                    ui.set_status_type("success".into());
                    ui.set_show_edit_dialog(false);
                    let profiles = load_profiles(&config.profiles_dir);
                    let profile_models: Vec<ProfileData> =
                        profiles.iter().map(profile_to_ui).collect();
                    ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Error editing profile: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    // Auto-save background thread
    let auto_save_ui = ui.as_weak();
    let auto_save_config = Arc::new(Mutex::new(config.clone()));
    let auto_save_handle = {
        let config_ref = Arc::clone(&auto_save_config);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(60)); // Check every minute

                let cfg = {
                    let config_guard = config_ref.lock().unwrap();
                    if !config_guard.scheduled_save.enabled {
                        continue;
                    }
                    config_guard.clone()
                };

                let interval_secs = cfg.scheduled_save.interval_hours as u64 * 3600;
                if interval_secs == 0 {
                    continue;
                }

                // Check if enough time has passed since last run
                let last_run = if cfg.scheduled_save.last_run.is_empty() {
                    0
                } else {
                    match chrono::DateTime::parse_from_str(
                        &cfg.scheduled_save.last_run,
                        "%Y-%m-%d %H:%M:%S",
                    ) {
                        Ok(dt) => dt.timestamp() as u64,
                        Err(_) => 0,
                    }
                };

                let now = Local::now().timestamp() as u64;
                if now.saturating_sub(last_run) < interval_secs {
                    continue;
                }

                // Time to auto-save
                let profile_name = std::format!(
                    "Auto-Save {}",
                    Local::now().format("%Y-%m-%d %H:%M")
                );
                if let Ok(_profile) = create_profile(profile_name.clone(), false, &cfg) {
                    // Update last run time
                    let mut config_guard = config_ref.lock().unwrap();
                    config_guard.scheduled_save.last_run =
                        Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    let _ = save_config(&config_guard);

                    // Update UI
                    if let Some(ui) = auto_save_ui.upgrade() {
                        let scheduled = scheduled_save_to_ui(&config_guard);
                        ui.set_scheduled_saves(ModelRc::new(VecModel::from(vec![scheduled])));
                        ui.set_status_message(
                            std::format!("Auto-saved profile: {}", profile_name).into(),
                        );
                        ui.set_status_type("success".into());
                        let profiles = load_profiles(&config_guard.profiles_dir);
                        let profile_models: Vec<ProfileData> =
                            profiles.iter().map(profile_to_ui).collect();
                        ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));
                    }
                }
            }
        })
    };

    // Update config when auto-save settings change
    let auto_save_config_clone = Arc::clone(&auto_save_config);
    let ui_weak = ui.as_weak();
    ui.on_save_auto_save_settings(move |enabled, interval| {
        let mut config = load_config();
        config.scheduled_save.enabled = enabled;
        config.scheduled_save.interval_hours = interval as u32;
        match save_config(&config) {
            Ok(()) => {
                // Update the shared config for the auto-save thread
                let mut config_guard = auto_save_config_clone.lock().unwrap();
                *config_guard = config.clone();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message("Auto-save settings saved".into());
                    ui.set_status_type("success".into());
                    let scheduled = scheduled_save_to_ui(&config);
                    ui.set_scheduled_saves(ModelRc::new(VecModel::from(vec![scheduled])));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Failed to save settings: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_save_external_backup_path(move |path| {
        let mut config = load_config();
        config.external_backup_path = path.to_string();
        match save_config(&config) {
            Ok(()) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_external_backup_path(path);
                    ui.set_status_message("External backup path saved".into());
                    ui.set_status_type("success".into());
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Failed to save path: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_apply_conflict_fixes(move || {
        let config = load_config();
        match apply_conflict_fixes(&config) {
            Ok(msg) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(msg.into());
                    ui.set_status_type("success".into());
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Error applying fixes: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_refresh_all(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let available_des = get_available_desktop_environments();
            let de_models: Vec<DesktopEnvironmentData> =
                available_des.iter().map(de_to_ui).collect();
            ui.set_available_desktops(ModelRc::new(VecModel::from(de_models)));

            let profiles = load_profiles(&load_config().profiles_dir);
            let profile_models: Vec<ProfileData> = profiles.iter().map(profile_to_ui).collect();
            ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));

            ui.set_status_message("Refreshed desktop environments and profiles".into());
            ui.set_status_type("success".into());
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_edit_profile(move |index| {
        let config = load_config();
        let profiles = load_profiles(&config.profiles_dir);
        if let Some(profile) = profiles.get(index as usize) {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_edit_profile_name(profile.name.clone().into());
                ui.set_edit_profile_remote(profile.is_remote);
                ui.set_show_edit_dialog(true);
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_save_edit_profile(move |old_name, new_name, save_remote| {
        let config = load_config();
        match rename_profile(&old_name.to_string(), &new_name.to_string(), &config) {
            Ok(()) => {
                if let Err(e) = update_profile_remote_status(
                    &new_name.to_string(),
                    save_remote,
                    &config,
                ) {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(
                            std::format!("Failed to update remote status: {}", e).into(),
                        );
                        ui.set_status_type("error".into());
                    }
                }
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Profile '{}' updated successfully", new_name).into(),
                    );
                    ui.set_status_type("success".into());
                    ui.set_show_edit_dialog(false);
                    let profiles = load_profiles(&config.profiles_dir);
                    let profile_models: Vec<ProfileData> =
                        profiles.iter().map(profile_to_ui).collect();
                    ui.set_profiles(ModelRc::new(VecModel::from(profile_models)));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(
                        std::format!("Error editing profile: {}", e).into(),
                    );
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    // Detach auto-save thread so it doesn't block UI shutdown
    drop(auto_save_handle);

    ui.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_config(profiles_dir: &str) -> AppConfig {
        AppConfig {
            remote_url: String::new(),
            profiles_dir: profiles_dir.to_string(),
            backup_items: default_backup_items(),
            conflict_fixes: default_conflict_fixes(),
            folder_backups: Vec::new(),
            scheduled_save: ScheduledSaveConfig {
                enabled: false,
                interval_hours: 24,
                last_run: String::new(),
            },
            external_backup_path: String::new(),
        }
    }

    #[test]
    fn test_resolve_path_expands_home() {
        let home = dirs::home_dir().unwrap();
        let result = resolve_path("$HOME/.bashrc");
        assert!(!result.is_empty());
        assert!(result[0].starts_with(&home));
    }

    #[test]
    fn test_resolve_path_comma_separated() {
        let result = resolve_path("$HOME/.bashrc, $HOME/.zshrc");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_resolve_path_empty_input() {
        let result = resolve_path("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_path_custom_variables() {
        let result = resolve_path("$CONFIG_DIR/test, $SHARE_DIR/app");
        assert_eq!(result.len(), 2);
        assert!(result[0].to_string_lossy().contains(".config"));
        assert!(result[1].to_string_lossy().contains(".local/share"));
    }

    #[test]
    fn test_normalize_remote_target_http() {
        let result = normalize_remote_target("https://backup.example.com/profiles");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://backup.example.com/profiles");
    }

    #[test]
    fn test_normalize_remote_target_file() {
        let result = normalize_remote_target("file:///mnt/backup");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "file:///mnt/backup");
    }

    #[test]
    fn test_normalize_remote_target_absolute_path() {
        let result = normalize_remote_target("/mnt/backup");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/mnt/backup");
    }

    #[test]
    fn test_normalize_remote_target_empty_fails() {
        let result = normalize_remote_target("");
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_remote_target_invalid_scheme() {
        let result = normalize_remote_target("ftp://server.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_profile_name_validation() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(temp_dir.path().to_str().unwrap());

        // Empty name should fail
        let result = create_profile("".to_string(), false, &config);
        assert!(result.is_err());

        // Name with path separator should fail
        let result = create_profile("my/profile".to_string(), false, &config);
        assert!(result.is_err());

        // Name with backslash should fail
        let result = create_profile("my\\profile".to_string(), false, &config);
        assert!(result.is_err());

        // Name with .. should fail
        let result = create_profile("../parent".to_string(), false, &config);
        assert!(result.is_err());

        // Valid name should succeed
        let result = create_profile("My Test Profile".to_string(), false, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_and_load_profiles() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(temp_dir.path().to_str().unwrap());

        let profile = create_profile("test-profile".to_string(), false, &config).unwrap();
        assert_eq!(profile.name, "test-profile");
        assert_eq!(profile.desktop_environment, get_current_desktop_environment());

        let loaded = load_profiles(&config.profiles_dir);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "test-profile");
    }

    #[test]
    fn test_delete_profile() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(temp_dir.path().to_str().unwrap());

        create_profile("to-delete".to_string(), false, &config).unwrap();
        let loaded = load_profiles(&config.profiles_dir);
        assert_eq!(loaded.len(), 1);

        delete_profile("to-delete", &config).unwrap();
        let loaded_after = load_profiles(&config.profiles_dir);
        assert!(loaded_after.is_empty());
    }

    #[test]
    fn test_export_and_import_profile() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(temp_dir.path().to_str().unwrap());

        create_profile("export-test".to_string(), false, &config).unwrap();
        let export_path = export_profile("export-test", &config).unwrap();
        assert!(export_path.exists());

        // Delete original, then import
        delete_profile("export-test", &config).unwrap();
        let import_name = import_profile(export_path.to_str().unwrap(), &config).unwrap();
        assert_eq!(import_name, "export-test");

        let loaded = load_profiles(&config.profiles_dir);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "export-test");
    }

    #[test]
    fn test_rename_profile() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(temp_dir.path().to_str().unwrap());

        create_profile("old-name".to_string(), false, &config).unwrap();
        rename_profile("old-name", "new-name", &config).unwrap();

        let loaded = load_profiles(&config.profiles_dir);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "new-name");
    }

    #[test]
    fn test_rename_profile_invalid_name() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(temp_dir.path().to_str().unwrap());
        create_profile("valid-name".to_string(), false, &config).unwrap();

        // Empty name should fail
        let result = rename_profile("valid-name", "", &config);
        assert!(result.is_err());

        // Path separator should fail
        let result = rename_profile("valid-name", "bad/name", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_profile_remote_status() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(temp_dir.path().to_str().unwrap());

        create_profile("remote-test".to_string(), false, &config).unwrap();
        update_profile_remote_status("remote-test", true, &config).unwrap();

        let loaded = load_profiles(&config.profiles_dir);
        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].is_remote);
    }

    #[test]
    fn test_get_de_config_conflicts_gnome() {
        let conflicts = get_de_config_conflicts("GNOME");
        assert!(conflicts.contains(&"kde".to_string()));
        assert!(conflicts.contains(&"plasma".to_string()));
        assert!(conflicts.contains(&"xfce4".to_string()));
    }

    #[test]
    fn test_get_de_config_conflicts_kde() {
        let conflicts = get_de_config_conflicts("KDE Plasma");
        assert!(conflicts.contains(&"gnome".to_string()));
        assert!(conflicts.contains(&"xfce4".to_string()));
    }

    #[test]
    fn test_get_de_config_conflicts_unknown() {
        let conflicts = get_de_config_conflicts("UnknownDE");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_create_profile_duplicate_fails() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(temp_dir.path().to_str().unwrap());

        create_profile("dup-test".to_string(), false, &config).unwrap();
        let result = create_profile("dup-test".to_string(), false, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_backup_items_have_categories() {
        let items = default_backup_items();
        for item in &items {
            assert!(!item.category.is_empty(), "Backup item '{}' has empty category", item.name);
        }
    }

    #[test]
    fn test_default_conflict_fixes_have_categories() {
        let fixes = default_conflict_fixes();
        for fix in &fixes {
            assert!(!fix.category.is_empty(), "Conflict fix '{}' has empty category", fix.name);
            assert!(!fix.apply_fn.is_empty(), "Conflict fix '{}' has no apply function", fix.name);
        }
    }

    #[test]
    fn test_profile_to_ui_conversion() {
        let profile = Profile {
            name: "test".to_string(),
            created: "2024-01-01 00:00:00".to_string(),
            desktop_environment: "GNOME".to_string(),
            packages: vec!["pkg1".to_string(), "pkg2".to_string()],
            is_remote: true,
            backup_items: vec!["item1".to_string()],
            config_archive: "configs.tar.gz".to_string(),
        };

        let ui = profile_to_ui(&profile);
        assert_eq!(ui.name, "test");
        assert_eq!(ui.package_count, 2);
        assert_eq!(ui.config_count, 1);
        assert!(ui.is_remote);
    }
}
