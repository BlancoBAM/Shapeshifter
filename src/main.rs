use chrono::Local;
use serde::{Deserialize, Serialize};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

slint::include_modules!();

// ─── DE Definitions ────────────────────────────────────────────────────────────

struct DeDefinition {
    name: &'static str,
    meta_package: &'static str,
    fallback_packages: &'static [&'static str],
}

const DE_LIST: &[DeDefinition] = &[
    DeDefinition { name: "Plasma", meta_package: "lilith-plasma", fallback_packages: &["plasma-desktop", "sddm", "systemsettings", "plasma-workspace-wayland"] },
    DeDefinition { name: "XFCE", meta_package: "lilith-xfce", fallback_packages: &["xfce4", "xfce4-session", "xfwm4", "xfce4-panel"] },
    DeDefinition { name: "LXDE", meta_package: "lilith-lxde", fallback_packages: &["lxde-core", "lxsession", "openbox"] },
    DeDefinition { name: "LXQt", meta_package: "lilith-lxqt", fallback_packages: &["lxqt-core", "openbox"] },
    DeDefinition { name: "Budgie", meta_package: "lilith-budgie", fallback_packages: &["budgie-desktop", "budgie-indicator-applet"] },
    DeDefinition { name: "Deepin", meta_package: "lilith-deepin", fallback_packages: &["dde-session-ui", "dde-desktop", "deepin-wm"] },
    DeDefinition { name: "Trinity", meta_package: "lilith-trinity", fallback_packages: &["tde-trinity", "tdm"] },
    DeDefinition { name: "Enlightenment", meta_package: "lilith-enlightenment", fallback_packages: &["enlightenment", "terminology"] },
    DeDefinition { name: "Pantheon", meta_package: "lilith-pantheon", fallback_packages: &["pantheon-shell", "gala", "wingpanel", "plank"] },
    DeDefinition { name: "GNOME", meta_package: "lilith-gnome", fallback_packages: &["gnome-session", "gnome-shell", "gnome-control-center"] },
    DeDefinition { name: "MATE", meta_package: "lilith-mate", fallback_packages: &["mate-desktop-environment-core"] },
    DeDefinition { name: "i3", meta_package: "lilith-i3", fallback_packages: &["i3-wm", "i3status", "i3lock", "dmenu"] },
    DeDefinition { name: "Sway", meta_package: "lilith-sway", fallback_packages: &["sway", "swaylock", "swayidle", "swaybg", "wmenu"] },
];

fn get_de_backup_paths(de: &str) -> Vec<&'static str> {
    match de.to_lowercase().as_str() {
        "plasma" | "kde" => vec![
            ".config/plasma*", ".config/kde*",
            ".local/share/plasma/", ".config/kwinrc",
            ".config/kdeglobals", ".config/plasmarc",
        ],
        "gnome" => vec![
            ".config/gnome*", ".local/share/gnome-shell/",
            ".config/dconf/",
        ],
        "xfce" => vec![".config/xfce4/"],
        "lxde" => vec![".config/lxsession/", ".config/openbox/"],
        "lxqt" => vec![".config/lxqt/", ".config/openbox/"],
        "budgie" => vec![".config/budgie-desktop/"],
        "deepin" => vec![".config/deepin/"],
        "trinity" => vec![".trinity/"],
        "enlightenment" => vec![".e/", ".config/terminology/"],
        "pantheon" => vec![".config/plank/", ".config/wingpanel/"],
        "mate" => vec![".config/mate/"],
        "i3" => vec![".config/i3/", ".config/i3status/"],
        "sway" => vec![".config/sway/", ".config/swaylock/"],
        _ => vec![],
    }
}

fn get_shared_backup_paths() -> Vec<&'static str> {
    vec![
        ".themes/",
        ".icons/",
        ".fonts/",
        ".local/share/fonts/",
        ".config/gtk-3.0/",
        ".config/gtk-4.0/",
    ]
}

// ─── Data Models ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Profile {
    name: String,
    de: String,
    created: String,
    packages: Vec<String>,
    thumbnail: String,
    remote_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppConfig {
    version: i32,
    last_remote_path: String,
    installed_des: Vec<String>,
    last_profile: Option<LastProfile>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LastProfile {
    de: String,
    name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 2,
            last_remote_path: String::new(),
            installed_des: Vec::new(),
            last_profile: None,
        }
    }
}

// ─── Config Helpers ────────────────────────────────────────────────────────────

fn get_shapeshifter_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".config/shapeshifter")
}

fn get_config_path() -> PathBuf {
    get_shapeshifter_dir().join("config.json")
}

fn get_profiles_dir() -> PathBuf {
    get_shapeshifter_dir().join("profiles")
}

fn load_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
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

fn detect_installed_des() -> Vec<String> {
    let mut installed = Vec::new();
    for de_def in DE_LIST {
        if is_de_installed(de_def.name) {
            installed.push(de_def.name.to_string());
        }
    }
    installed
}

fn is_de_installed(de_name: &str) -> bool {
    let session_dirs = vec![
        "/usr/share/xsessions",
        "/usr/local/share/xsessions",
        "/usr/share/wayland-sessions",
        "/usr/local/share/wayland-sessions",
    ];

    let lower = de_name.to_lowercase();

    for dir in &session_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    let name_lower = name.to_lowercase();
                    if name_lower.starts_with(&lower) && name_lower.ends_with(".desktop") {
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[allow(dead_code)]
fn find_de_definition_index(de_name: &str) -> Option<usize> {
    DE_LIST.iter().position(|d| d.name.eq_ignore_ascii_case(de_name))
}

// ─── Profile Operations ────────────────────────────────────────────────────────

fn get_profiles_dir_for_de(de: &str) -> PathBuf {
    get_profiles_dir().join(de.to_lowercase())
}

fn get_profile_dir(de: &str, name: &str) -> PathBuf {
    get_profiles_dir_for_de(de).join(name)
}

fn load_profiles() -> Vec<Profile> {
    let profiles_dir = get_profiles_dir();
    let mut profiles = Vec::new();

    if !profiles_dir.exists() {
        return profiles;
    }

    if let Ok(de_entries) = fs::read_dir(&profiles_dir) {
        for de_entry in de_entries.flatten() {
            let de_path = de_entry.path();
            if !de_path.is_dir() {
                continue;
            }
            let de_name = de_entry.file_name().to_string_lossy().to_string();

            if let Ok(profile_entries) = fs::read_dir(&de_path) {
                for profile_entry in profile_entries.flatten() {
                    let profile_path = profile_entry.path();
                    if !profile_path.is_dir() {
                        continue;
                    }
                    let profile_file = profile_path.join("profile.json");
                    if profile_file.exists() {
                        if let Ok(content) = fs::read_to_string(profile_file) {
                            if let Ok(mut profile) = serde_json::from_str::<Profile>(&content) {
                                profile.de = de_name.clone();
                                profiles.push(profile);
                            }
                        }
                    }
                }
            }
        }
    }

    profiles.sort_by(|a, b| b.created.cmp(&a.created));
    profiles
}

fn save_profile(
    name: &str,
    de: &str,
    thumbnail_path: &str,
    remote_path: &str,
) -> Result<Profile, Box<dyn std::error::Error>> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Profile name cannot be empty".into());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Invalid profile name".into());
    }

    let profile_dir = get_profile_dir(de, name);
    if profile_dir.exists() {
        return Err(format!("Profile '{}' already exists for {}", name, de).into());
    }
    fs::create_dir_all(&profile_dir)?;

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    // Collect and archive DE-specific configs
    let archive_path = profile_dir.join("configs.tar.gz");
    let de_paths = get_de_backup_paths(de);
    let shared_paths = get_shared_backup_paths();

    let tar_file = fs::File::create(&archive_path)?;
    let enc = flate2::write::GzEncoder::new(tar_file, flate2::Compression::default());
    let mut tar_builder = tar::Builder::new(enc);

    let all_paths: Vec<&str> = de_paths.iter().chain(shared_paths.iter()).copied().collect();

    for rel_path in &all_paths {
        let full_path = home.join(rel_path);
        if full_path.exists() {
            if full_path.is_dir() {
                if let Some(name) = full_path.file_name() {
                    let _ = tar_builder.append_dir_all(name, &full_path);
                }
            } else if full_path.is_file() {
                if let Some(name) = full_path.file_name() {
                    if let Ok(mut file) = fs::File::open(&full_path) {
                        let _ = tar_builder.append_file(name, &mut file);
                    }
                }
            }
        }
    }

    // Dconf dump for GNOME
    if de.eq_ignore_ascii_case("gnome") {
        if let Ok(output) = Command::new("dconf").args(["dump", "/"]).output() {
            if output.status.success() {
                let temp_dir = get_shapeshifter_dir().join("temp");
                let _ = fs::create_dir_all(&temp_dir);
                let dconf_file = temp_dir.join("dconf-settings.ini");
                if fs::write(&dconf_file, &output.stdout).is_ok() {
                    let _ = tar_builder.append_file("dconf-settings.ini", &mut fs::File::open(&dconf_file)?);
                }
            }
        }
    }

    tar_builder.finish()?;

    // Handle thumbnail
    let thumbnail_dest = profile_dir.join("thumbnail.png");
    if !thumbnail_path.is_empty() && Path::new(thumbnail_path).exists() {
        let _ = fs::copy(thumbnail_path, &thumbnail_dest);
    }

    let packages = get_installed_packages();

    let profile = Profile {
        name: name.to_string(),
        de: de.to_string(),
        created: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        packages,
        thumbnail: if thumbnail_dest.exists() { "thumbnail.png".to_string() } else { String::new() },
        remote_path: remote_path.to_string(),
    };

    let profile_file = profile_dir.join("profile.json");
    let json = serde_json::to_string_pretty(&profile)?;
    fs::write(profile_file, json)?;

    // Update last_profile in config
    let mut config = load_config();
    config.last_profile = Some(LastProfile {
        de: de.to_string(),
        name: name.to_string(),
    });
    config.last_remote_path = remote_path.to_string();
    let _ = save_config(&config);

    Ok(profile)
}

fn restore_profile(de: &str, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let profile_dir = get_profile_dir(de, name);
    if !profile_dir.exists() {
        return Err(format!("Profile '{}' for {} not found", name, de).into());
    }

    let profile_file = profile_dir.join("profile.json");
    if !profile_file.exists() {
        return Err("Profile metadata not found".into());
    }

    let content = fs::read_to_string(&profile_file)?;
    let _profile: Profile = serde_json::from_str(&content)?;

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    // Extract config archive
    let archive_path = profile_dir.join("configs.tar.gz");
    if archive_path.exists() {
        let tar_file = fs::File::open(&archive_path)?;
        let dec = flate2::read::GzDecoder::new(tar_file);
        let mut archive = tar::Archive::new(dec);
        archive.unpack(&home)?;
    }

    // Run DE conflict isolation
    backup_conflicting_configs(de)?;
    restore_de_configs(de)?;

    // Set DE as default session
    set_default_session(de)?;

    // Update last-profile.json
    let last_profile = LastProfile {
        de: de.to_string(),
        name: name.to_string(),
    };
    let last_profile_path = get_shapeshifter_dir().join("last-profile.json");
    if let Some(parent) = last_profile_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&last_profile_path, serde_json::to_string_pretty(&last_profile)?);

    // Update config
    let mut config = load_config();
    config.last_profile = Some(last_profile);
    let _ = save_config(&config);

    Ok(format!("Profile '{}' restored. Log out to switch to {}.", name, de))
}

fn delete_profile(de: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profile_dir = get_profile_dir(de, name);
    if profile_dir.exists() {
        fs::remove_dir_all(profile_dir)?;
    }
    Ok(())
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
    packages
}

fn get_current_desktop_environment() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

// ─── UI Conversion ─────────────────────────────────────────────────────────────

fn build_de_store_items(installed_des: &[String]) -> Vec<DeStoreItem> {
    DE_LIST
        .iter()
        .map(|de_def| {
            let is_installed = installed_des
                .iter()
                .any(|d| d.eq_ignore_ascii_case(de_def.name));
            DeStoreItem {
                name: de_def.name.into(),
                meta_package: de_def.meta_package.into(),
                is_installed,
                is_installing: false,
                install_status: SharedString::default(),
            }
        })
        .collect()
}

fn build_display_items(profiles: &[Profile]) -> Vec<DisplayItem> {
    if profiles.is_empty() {
        return Vec::new();
    }

    // Group profiles by DE, preserving insertion order
    let mut de_groups: Vec<(String, Vec<&Profile>)> = Vec::new();
    let mut seen_des: Vec<String> = Vec::new();

    for p in profiles {
        if !seen_des.iter().any(|d| d.eq_ignore_ascii_case(&p.de)) {
            seen_des.push(p.de.clone());
        }
    }

    for de in &seen_des {
        let group: Vec<&Profile> = profiles.iter().filter(|p| p.de.eq_ignore_ascii_case(de)).collect();
        if !group.is_empty() {
            de_groups.push((de.clone(), group));
        }
    }

    let mut items = Vec::new();
    let mut row_counter = 0;
    let cols = 4;

    for (de_name, group) in &de_groups {
        // Section header
        items.push(DisplayItem {
            item_type: 0,
            header_text: de_name.clone().into(),
            grid_row: row_counter,
            grid_col: 0,
            profile_name: SharedString::default(),
            profile_de: SharedString::default(),
            profile_thumbnail: slint::Image::default(),
            has_thumbnail: false,
            profile_created: SharedString::default(),
            profile_index: -1,
        });
        row_counter += 1;

        // Profile cards in this section
        for (i, p) in group.iter().enumerate() {
            let has_thumbnail = !p.thumbnail.is_empty();
            let profile_thumbnail = if has_thumbnail {
                let path = get_profile_dir(&p.de, &p.name).join(&p.thumbnail);
                if path.exists() {
                    slint::Image::load_from_path(&path).unwrap_or_else(|_| slint::Image::default())
                } else {
                    slint::Image::default()
                }
            } else {
                slint::Image::default()
            };

            items.push(DisplayItem {
                item_type: 1,
                header_text: SharedString::default(),
                grid_row: row_counter + (i / cols) as i32,
                grid_col: (i % cols) as i32,
                profile_name: p.name.clone().into(),
                profile_de: p.de.clone().into(),
                profile_thumbnail,
                has_thumbnail,
                profile_created: p.created.clone().into(),
                profile_index: find_profile_index(profiles, p) as i32,
            });
        }
        row_counter += ((group.len() + cols - 1) / cols) as i32;
    }

    items
}

fn find_profile_index(profiles: &[Profile], target: &Profile) -> usize {
    profiles.iter().position(|p| p.name == target.name && p.de == target.de).unwrap_or(0)
}

// ─── DE Install/Remove (Threaded) ──────────────────────────────────────────────

fn run_apt_install(packages: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("pkexec");
    cmd.arg("apt").arg("install").arg("-y");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                Ok(stdout)
            } else {
                Err(format!("{}\n{}", stderr, stdout))
            }
        }
        Err(e) => Err(format!("Failed to run pkexec: {}", e)),
    }
}

fn run_apt_remove(packages: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("pkexec");
    cmd.arg("apt").arg("remove").arg("-y");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                Ok(stdout)
            } else {
                Err(format!("{}\n{}", stderr, stdout))
            }
        }
        Err(e) => Err(format!("Failed to run pkexec: {}", e)),
    }
}

fn start_install(de_index: usize, ui: &AppWindow) {
    if de_index >= DE_LIST.len() {
        return;
    }
    let de_def = &DE_LIST[de_index];
    let meta_pkg = de_def.meta_package.to_string();
    let fallback: Vec<String> = de_def.fallback_packages.iter().map(|s| s.to_string()).collect();

    // Set installing state
    {
        let model_rc = ui.get_de_store_items();
        let mut items: Vec<DeStoreItem> = (0..model_rc.row_count())
            .filter_map(|i| model_rc.row_data(i))
            .collect();
        if let Some(item) = items.get_mut(de_index) {
            item.is_installing = true;
            item.install_status = "Installing...".into();
        }
        ui.set_de_store_items(ModelRc::new(VecModel::from(items)));
    }

    let ui_weak = ui.as_weak();

    thread::spawn(move || {
        let result = run_apt_install(&[&meta_pkg]);

        let (success, status_text) = match result {
            Ok(_) => (true, "Install complete!".to_string()),
            Err(_e) => {
                // Try fallback packages
                let fallback_refs: Vec<&str> = fallback.iter().map(|s| s.as_str()).collect();
                match run_apt_install(&fallback_refs) {
                    Ok(_) => (true, "Installed with alternative packages".to_string()),
                    Err(fb_err) => (false, format!("Installation failed: {}", fb_err)),
                }
            }
        };

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let model_rc = ui.get_de_store_items();
                let mut items: Vec<DeStoreItem> = (0..model_rc.row_count())
                    .filter_map(|i| model_rc.row_data(i))
                    .collect();
                if let Some(item) = items.get_mut(de_index) {
                    item.is_installing = false;
                    item.is_installed = success;
                    item.install_status = status_text.into();
                }
                ui.set_de_store_items(ModelRc::new(VecModel::from(items)));

                // Refresh installed DEs
                let installed = detect_installed_des();
                let mut config = load_config();
                config.installed_des = installed.clone();
                let _ = save_config(&config);

                // Refresh profiles display
                let profiles = load_profiles();
                ui.set_display_items(ModelRc::new(VecModel::from(build_display_items(&profiles))));

                ui.set_status_message(
                    if success { "Desktop environment installed successfully".into() }
                    else { "Installation failed. See details for more info.".into() }
                );
                ui.set_status_type(if success { "success".into() } else { "error".into() });
            }
        });
    });
}

fn start_remove(de_index: usize, ui: &AppWindow) {
    if de_index >= DE_LIST.len() {
        return;
    }
    let de_def = &DE_LIST[de_index];
    let meta_pkg = de_def.meta_package.to_string();
    let fallback: Vec<String> = de_def.fallback_packages.iter().map(|s| s.to_string()).collect();

    // Set removing state
    {
        let model_rc = ui.get_de_store_items();
        let mut items: Vec<DeStoreItem> = (0..model_rc.row_count())
            .filter_map(|i| model_rc.row_data(i))
            .collect();
        if let Some(item) = items.get_mut(de_index) {
            item.is_installing = true;
            item.install_status = "Removing...".into();
        }
        ui.set_de_store_items(ModelRc::new(VecModel::from(items)));
    }

    let ui_weak = ui.as_weak();

    thread::spawn(move || {
        // Try meta package first, then fallback
        let mut all_pkgs = vec![meta_pkg.as_str()];
        all_pkgs.extend(fallback.iter().map(|s| s.as_str()));

        let result = run_apt_remove(&all_pkgs);
        let (success, status_text) = match result {
            Ok(_) => (true, "Removal complete".to_string()),
            Err(e) => (false, format!("Removal failed: {}", e)),
        };

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let model_rc = ui.get_de_store_items();
                let mut items: Vec<DeStoreItem> = (0..model_rc.row_count())
                    .filter_map(|i| model_rc.row_data(i))
                    .collect();
                if let Some(item) = items.get_mut(de_index) {
                    item.is_installing = false;
                    if success {
                        item.is_installed = false;
                    }
                    item.install_status = status_text.into();
                }
                ui.set_de_store_items(ModelRc::new(VecModel::from(items)));

                let installed = detect_installed_des();
                let mut config = load_config();
                config.installed_des = installed.clone();
                let _ = save_config(&config);

                let profiles = load_profiles();
                ui.set_display_items(ModelRc::new(VecModel::from(build_display_items(&profiles))));

                ui.set_status_message(
                    if success { "Desktop environment removed".into() }
                    else { "Removal failed.".into() }
                );
                ui.set_status_type(if success { "success".into() } else { "error".into() });
            }
        });
    });
}

// ─── File Browser ──────────────────────────────────────────────────────────────

fn browse_file() -> String {
    let output = Command::new("zenity")
        .args([
            "--file-selection",
            "--file-filter=Images | *.png *.jpg *.jpeg *.webp",
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                String::new()
            }
        }
        Err(_) => String::new(),
    }
}

// ─── Conflict Isolation (kept from v1) ─────────────────────────────────────────

fn get_de_config_conflicts(de_name: &str) -> Vec<String> {
    let lower = de_name.to_lowercase();
    let mut conflicts = Vec::new();
    if lower.contains("gnome") {
        conflicts.extend_from_slice(&["kde".into(), "plasma".into(), "xfce4".into()]);
    } else if lower.contains("kde") || lower.contains("plasma") {
        conflicts.extend_from_slice(&["gnome".into(), "xfce4".into()]);
    } else if lower.contains("xfce") {
        conflicts.extend_from_slice(&["gnome".into(), "kde".into(), "plasma".into()]);
    } else if lower.contains("cinnamon") {
        conflicts.extend_from_slice(&["gnome".into(), "kde".into(), "plasma".into()]);
    } else if lower.contains("mate") {
        conflicts.extend_from_slice(&["gnome".into(), "kde".into(), "plasma".into()]);
    } else if lower.contains("cosmic") {
        conflicts.extend_from_slice(&["gnome".into(), "kde".into(), "plasma".into(), "xfce4".into(), "cinnamon".into()]);
    }
    conflicts
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
    let backup_dir = get_shapeshifter_dir().join("de_backups");
    fs::create_dir_all(&backup_dir)?;

    let conflicts = get_de_config_conflicts(target_de);
    for conflict in conflicts {
        let source = config_dir.join(&conflict);
        if source.exists() {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let backup_path = backup_dir.join(format!("{}_{}", conflict, timestamp));
            copy_dir_recursive(&source, &backup_path)?;
        }
    }
    Ok(())
}

fn restore_de_configs(de_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let backup_dir = get_shapeshifter_dir().join("de_backups");
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
        if !backup_dir.exists() {
            return Ok(());
        }
        let entries = fs::read_dir(&backup_dir)?;
        let mut latest_backup: Option<(PathBuf, std::time::SystemTime)> = None;

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(&format!("{}_", config_name)) {
                if let Ok(meta) = entry.metadata() {
                    let mod_time = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    match latest_backup {
                        Some((_, ref latest_time)) if mod_time > *latest_time => {
                            latest_backup = Some((entry.path(), mod_time));
                        }
                        None => {
                            latest_backup = Some((entry.path(), mod_time));
                        }
                        _ => {}
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

// ─── Session Management ────────────────────────────────────────────────────────

fn get_de_exec_name(de_display_name: &str) -> Option<String> {
    let session_paths = vec![
        "/usr/share/xsessions",
        "/usr/local/share/xsessions",
        "/usr/share/wayland-sessions",
        "/usr/local/share/wayland-sessions",
    ];

    for path in &session_paths {
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

fn set_default_session(de_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let dmrc_path = home.join(".dmrc");

    if let Some(exec_name) = get_de_exec_name(de_name) {
        let session_name = exec_name.split_whitespace().next().unwrap_or(de_name);
        let dmrc_content = format!("[Desktop]\nSession={}\n", session_name);
        fs::write(&dmrc_path, dmrc_content)?;

        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        let accountsservice_path = format!("/var/lib/AccountsService/users/{}", username);

        if Path::new(&accountsservice_path).exists() {
            let sed_cmd = format!("s/^XSession=.*/XSession={}/", session_name);
            let _ = Command::new("pkexec")
                .args(["sed", "-i", &sed_cmd, &accountsservice_path])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
    Ok(())
}

#[allow(dead_code)]
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

// ─── CLI: Session Restore ──────────────────────────────────────────────────────

fn restore_session() {
    let current_de = get_current_desktop_environment();
    let shapeshifter_dir = get_shapeshifter_dir();
    let last_profile_path = shapeshifter_dir.join("last-profile.json");

    if !last_profile_path.exists() {
        eprintln!("Shapeshifter: No last profile found, nothing to restore");
        return;
    }

    let content = match fs::read_to_string(&last_profile_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Shapeshifter: Failed to read last profile: {}", e);
            return;
        }
    };

    let last_profile: LastProfile = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Shapeshifter: Failed to parse last profile: {}", e);
            return;
        }
    };

    // Determine which profile to restore
    let (target_de, target_name) = if current_de.eq_ignore_ascii_case(&last_profile.de) {
        (last_profile.de.clone(), last_profile.name.clone())
    } else {
        // Current DE differs: look for most recent profile for current DE
        let profiles = load_profiles();
        let matching: Vec<&Profile> = profiles.iter()
            .filter(|p| p.de.eq_ignore_ascii_case(&current_de))
            .collect();

        if matching.is_empty() {
            eprintln!("Shapeshifter: No profile found for {}", current_de);
            return;
        }
        (matching[0].de.clone(), matching[0].name.clone())
    };

    let isolation_log = get_shapeshifter_dir().join("isolation.log");
    let _ = fs::write(&isolation_log, format!("Restoring {} / {} for {}\n", target_de, target_name, current_de));

    match restore_profile(&target_de, &target_name) {
        Ok(msg) => {
            let _ = fs::write(&isolation_log, format!("Success: {}\n", msg));
        }
        Err(e) => {
            let _ = fs::write(&isolation_log, format!("Failed: {}\n", e));
            eprintln!("Shapeshifter: Restore failed: {}", e);
        }
    }
}

// ─── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), slint::PlatformError> {
    // Check for CLI mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--restore-session" {
        restore_session();
        return Ok(());
    }

    let ui = AppWindow::new()?;
    let config = load_config();
    let installed = if config.installed_des.is_empty() {
        detect_installed_des()
    } else {
        config.installed_des.clone()
    };

    // Set initial UI state
    let current_de = get_current_desktop_environment();
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    ui.set_current_de(current_de.into());
    ui.set_current_user(username.into());
    ui.set_application_version(env!("CARGO_PKG_VERSION").into());
    ui.set_last_remote_path(config.last_remote_path.clone().into());

    // Initialize DE Store
    let de_items = build_de_store_items(&installed);
    ui.set_de_store_items(ModelRc::new(VecModel::from(de_items)));

    // Initialize Profiles
    let profiles = load_profiles();
    ui.set_display_items(ModelRc::new(VecModel::from(build_display_items(&profiles))));

    // ── Callbacks ──────────────────────────────────────────────────────────────

    let ui_weak = ui.as_weak();
    ui.on_install_de(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            start_install(index as usize, &ui);
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_remove_de(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            start_remove(index as usize, &ui);
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_confirm_install_de(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let index = ui.get_confirm_index();
            if index >= 0 {
                start_install(index as usize, &ui);
                ui.set_confirm_index(-1);
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_confirm_remove_de(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let index = ui.get_confirm_index();
            if index >= 0 {
                start_remove(index as usize, &ui);
                ui.set_confirm_index(-1);
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_save_profile(move |name, thumbnail_path, remote_path| {
        let config = load_config();
        let de = config.last_profile.clone().map(|lp| lp.de).unwrap_or_else(|| get_current_desktop_environment());

        match save_profile(&name, &de, &thumbnail_path, &remote_path) {
            Ok(profile) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(format!("Profile '{}' saved for {}", profile.name, profile.de).into());
                    ui.set_status_type("success".into());

                    let profiles = load_profiles();
                    ui.set_display_items(ModelRc::new(VecModel::from(build_display_items(&profiles))));
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_message(format!("Error saving profile: {}", e).into());
                    ui.set_status_type("error".into());
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_restore_profile(move |index| {
        let profiles = load_profiles();
        if let Some(profile) = profiles.get(index as usize) {
            let de = profile.de.clone();
            let name = profile.name.clone();
            match restore_profile(&de, &name) {
                Ok(msg) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(msg.into());
                        ui.set_status_type("success".into());
                    }
                }
                Err(e) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_message(format!("Error restoring profile: {}", e).into());
                        ui.set_status_type("error".into());
                    }
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_confirm_restore_profile(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let index = ui.get_confirm_index();
            if index >= 0 {
                let profiles = load_profiles();
                if let Some(profile) = profiles.get(index as usize) {
                    let de = profile.de.clone();
                    let name = profile.name.clone();
                    match restore_profile(&de, &name) {
                        Ok(msg) => {
                            ui.set_status_message(msg.into());
                            ui.set_status_type("success".into());
                        }
                        Err(e) => {
                            ui.set_status_message(format!("Error restoring profile: {}", e).into());
                            ui.set_status_type("error".into());
                        }
                    }
                }
                ui.set_confirm_index(-1);
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_confirm_delete_profile(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let index = ui.get_confirm_index();
            if index >= 0 {
                let profiles = load_profiles();
                if let Some(profile) = profiles.get(index as usize) {
                    let de = profile.de.clone();
                    let name = profile.name.clone();
                    match delete_profile(&de, &name) {
                        Ok(()) => {
                            ui.set_status_message(format!("Profile '{}' deleted", name).into());
                            ui.set_status_type("success".into());
                            let refreshed = load_profiles();
                            ui.set_display_items(ModelRc::new(VecModel::from(build_display_items(&refreshed))));
                        }
                        Err(e) => {
                            ui.set_status_message(format!("Error deleting profile: {}", e).into());
                            ui.set_status_type("error".into());
                        }
                    }
                }
                ui.set_confirm_index(-1);
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_browse_thumbnail(move || {
        let path = browse_file();
        if let Some(ui) = ui_weak.upgrade() {
            if !path.is_empty() {
                // Write the selected path back into the pending-thumbnail-path property
                // which the Save dialog's LineEdit reads from
                ui.set_pending_thumbnail_path(path.clone().into());
                ui.set_status_message(format!("Thumbnail selected: {}", path).into());
                ui.set_status_type("info".into());
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_refresh_all(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let installed = detect_installed_des();
            let mut config = load_config();
            config.installed_des = installed.clone();
            let _ = save_config(&config);

            let de_items = build_de_store_items(&installed);
            ui.set_de_store_items(ModelRc::new(VecModel::from(de_items)));

            let profiles = load_profiles();
            ui.set_display_items(ModelRc::new(VecModel::from(build_display_items(&profiles))));

            ui.set_status_message("Refreshed desktop environments and profiles".into());
            ui.set_status_type("success".into());
        }
    });

    ui.run()
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn setup_test_env() -> (tempfile::TempDir, PathBuf, std::sync::MutexGuard<'static, ()>) {
        let guard = TEST_LOCK.lock().unwrap();
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        std::env::set_var("HOME", home.to_str().unwrap());
        std::env::set_var("XDG_CURRENT_DESKTOP", "TestDE");
        (tmp, home, guard)
    }

    #[test]
    fn test_de_definitions_complete() {
        assert_eq!(DE_LIST.len(), 13);
        let names: Vec<&str> = DE_LIST.iter().map(|d| d.name).collect();
        assert!(names.contains(&"Plasma"));
        assert!(names.contains(&"GNOME"));
        assert!(names.contains(&"XFCE"));
        assert!(names.contains(&"Sway"));
        assert!(names.contains(&"i3"));
    }

    #[test]
    fn test_de_backup_paths_not_empty() {
        for de_def in DE_LIST {
            let paths = get_de_backup_paths(de_def.name);
            assert!(!paths.is_empty(), "DE '{}' has no backup paths", de_def.name);
        }
    }

    #[test]
    fn test_shared_backup_paths_not_empty() {
        let paths = get_shared_backup_paths();
        assert!(!paths.is_empty());
        assert!(paths.contains(&".themes/"));
        assert!(paths.contains(&".icons/"));
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.version, 2);
        assert!(config.last_remote_path.is_empty());
        assert!(config.installed_des.is_empty());
        assert!(config.last_profile.is_none());
    }

    #[test]
    fn test_save_and_load_profile() {
        let (_tmp, _home, _guard) = setup_test_env();

        let profile = save_profile("Test Profile", "Plasma", "", "").unwrap();
        assert_eq!(profile.name, "Test Profile");
        assert_eq!(profile.de, "Plasma");
        assert!(!profile.created.is_empty());

        let loaded = load_profiles();
        assert!(!loaded.is_empty());
        assert_eq!(loaded[0].name, "Test Profile");
    }

    #[test]
    fn test_save_profile_duplicate_fails() {
        let (_tmp, _home, _guard) = setup_test_env();

        save_profile("Dup", "Plasma", "", "").unwrap();
        let result = save_profile("Dup", "Plasma", "", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_profile_invalid_name() {
        let (_tmp, _home, _guard) = setup_test_env();

        let result = save_profile("", "Plasma", "", "");
        assert!(result.is_err());

        let result = save_profile("test/path", "Plasma", "", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_profile() {
        let (_tmp, _home, _guard) = setup_test_env();

        save_profile("ToDelete", "XFCE", "", "").unwrap();
        let loaded_before = load_profiles();
        assert_eq!(loaded_before.len(), 1);

        delete_profile("xfce", "ToDelete").unwrap();
        let loaded_after = load_profiles();
        assert!(loaded_after.is_empty());
    }

    #[test]
    fn test_load_profiles_empty() {
        let (_tmp, _home, _guard) = setup_test_env();
        let profiles = load_profiles();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_build_de_store_items() {
        let items = build_de_store_items(&["Plasma".to_string(), "XFCE".to_string()]);
        assert_eq!(items.len(), 13);

        let plasma = items.iter().find(|i| i.name == "Plasma").unwrap();
        assert!(plasma.is_installed);

        let gnome = items.iter().find(|i| i.name == "GNOME").unwrap();
        assert!(!gnome.is_installed);
    }

    #[test]
    fn test_build_display_items() {
        let profiles = vec![
            Profile {
                name: "Dark Mode".to_string(),
                de: "Plasma".to_string(),
                created: "2026-01-01 12:00:00".to_string(),
                packages: vec![],
                thumbnail: String::new(),
                remote_path: String::new(),
            },
            Profile {
                name: "Light Mode".to_string(),
                de: "Plasma".to_string(),
                created: "2026-01-02 12:00:00".to_string(),
                packages: vec![],
                thumbnail: String::new(),
                remote_path: String::new(),
            },
            Profile {
                name: "Minimal".to_string(),
                de: "XFCE".to_string(),
                created: "2026-01-03 12:00:00".to_string(),
                packages: vec![],
                thumbnail: String::new(),
                remote_path: String::new(),
            },
        ];

        let items = build_display_items(&profiles);
        // Plasma header + 2 Plasma cards + XFCE header + 1 XFCE card = 5 items
        assert_eq!(items.len(), 5);

        assert_eq!(items[0].item_type, 0); // Header
        assert_eq!(items[0].header_text, "Plasma");
        assert_eq!(items[1].item_type, 1); // Profile card
        assert_eq!(items[1].profile_name, "Dark Mode");
        assert_eq!(items[2].profile_name, "Light Mode");
        assert_eq!(items[3].item_type, 0); // Header
        assert_eq!(items[3].header_text, "XFCE");
    }

    #[test]
    fn test_build_display_items_empty() {
        let items = build_display_items(&[]);
        assert!(items.is_empty());
    }

    #[test]
    fn test_normalize_remote_target() {
        assert!(normalize_remote_target("https://example.com").is_ok());
        assert!(normalize_remote_target("file:///mnt/backup").is_ok());
        assert!(normalize_remote_target("/mnt/backup").is_ok());
        assert!(normalize_remote_target("").is_err());
        assert!(normalize_remote_target("ftp://bad").is_err());
    }

    #[test]
    fn test_find_de_definition_index() {
        assert_eq!(find_de_definition_index("Plasma"), Some(0));
        assert_eq!(find_de_definition_index("plasma"), Some(0));
        assert_eq!(find_de_definition_index("GNOME"), Some(9));
        assert_eq!(find_de_definition_index("Unknown"), None);
    }
}
