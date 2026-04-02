# Shapeshifter - Desktop Environment & Profile Manager

A complete desktop environment switcher, profile manager, and system configuration backup tool for **Lilith Linux**, built with Rust and Slint.

## Overview

Shapeshifter unifies the functionality of several desktop configuration tools into a single, native application:

| Integrated Tool | Feature |
|----------------|---------|
| **SaveDesktop** | Full DE backup (themes, icons, fonts, wallpapers, extensions, Flatpak apps + data) |
| **MendingWall** | Multi-DE conflict prevention (icon/cursor themes, scaling, dark/light mode, menu cleanup) |
| **One-Click-Backup** | Simple folder-to-external-location backup |
| **KonSave** | Dotfile/profile save-apply-export-import with shareable archives |
| **KonUI** | Graphical interface for all profile management operations |

## Features

### Profiles
- **Create** profiles that capture your current desktop environment, installed packages, and configuration files
- **Apply** profiles to restore settings, configs, and install missing packages
- **Export** profiles as portable `.tar.gz` archives for sharing or migration
- **Import** profiles from exported archives
- **Delete** profiles you no longer need
- **Remote backup** to HTTP endpoints or local/network paths

### Desktop Environment Switcher
- Detect all installed desktop environments (X11 and Wayland sessions)
- Switch default session with proper `.dmrc` and AccountsService integration
- Automatic config conflict detection and backup before switching
- Config restoration when switching back to a previous DE

### Backup Items
- Granular control over what gets included in profiles:
  - GTK/Icon/Cursor themes
  - Fonts and wallpapers
  - GNOME extensions, KDE Plasma configs, XFCE configs
  - dConf settings dumps
  - Terminal, shell, and editor configurations
  - Flatpak apps and their data
  - Desktop folder and custom paths
- Per-item enable/disable toggles with size estimates

### DE Conflict Prevention
Automatically fix common multi-desktop issues:
- **Theme isolation** — preserve icon, cursor, and GTK themes per-DE
- **Display settings** — keep scaling and dark/light mode separate per-DE
- **Menu cleanup** — remove duplicate entries and hide DE-specific apps from wrong menus
- **Default apps** — set appropriate default applications per-DE
- **Autostart isolation** — keep DE-specific startup apps separate

### Folder Backup
- Add source-to-destination folder backup pairs
- Run backups on demand with timestamped copies
- Enable/disable individual backup jobs
- Support for local, external drive, and network destinations

### Automatic Saving
- Configurable interval-based auto-save of profiles
- Enable/disable with adjustable frequency (1–168 hours)
- Manual "Run Now" trigger

### Settings
- Remote backup target URL or file path
- External backup location for folder backups
- Auto-save configuration
- Profile storage location management

## Installation

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install system dependencies (Debian/Ubuntu)
sudo apt update
sudo apt install -y build-essential libfontconfig1-dev libxcb-render0-dev \
    libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev \
    libgtk-3-dev
```

### Build

```bash
cargo build --release
```

### Run

```bash
./target/release/shapeshifter
```

Or for development:

```bash
cargo run
```

## Project Structure

```
shapeshifter/
├── Cargo.toml           # Rust dependencies
├── build.rs             # Slint build script
├── src/
│   └── main.rs          # Application logic
├── ui/
│   └── appwindow.slint  # Slint UI definition
└── target/              # Build output
```

## Configuration

Shapeshifter stores its configuration at `~/.config/shapeshifter/config.json`:

```json
{
  "remote_url": "",
  "profiles_dir": "/home/user/.config/shapeshifter/profiles",
  "backup_items": [...],
  "conflict_fixes": [...],
  "folder_backups": [],
  "scheduled_save": {
    "enabled": false,
    "interval_hours": 24,
    "last_run": ""
  },
  "external_backup_path": ""
}
```

Profiles are stored at `~/.config/shapeshifter/profiles/<name>/`:
- `profile.json` — Profile metadata and package list
- `configs.tar.gz` — Archived configuration files

## Usage

### Creating a Profile
1. Navigate to **Profiles** in the sidebar
2. Click **+ New Profile**
3. Enter a name and optionally enable remote backup
4. Click **Create Profile**

### Applying a Profile
1. Go to **Profiles**
2. Click **Apply** on the desired profile
3. The app restores configs, installs missing packages, and sets the default DE

### Switching Desktop Environments
1. Go to **Desktop Switcher**
2. Click **Switch** next to your target DE
3. Log out and back in to complete the switch

### Managing Backup Items
1. Go to **Backup Items**
2. Toggle items on/off to control what gets included in profiles
3. Size estimates show how much space each item uses

### Applying Conflict Fixes
1. Go to **DE Conflict Fixes**
2. Enable the fixes you want
3. Click **Apply Selected Fixes**

### Folder Backup
1. Go to **Folder Backup**
2. Enter source and destination paths
3. Click **+ Add** to create a backup job
4. Click **Run** to execute a backup

### Settings
1. Go to **Settings**
2. Configure remote backup URL, auto-save interval, and external backup path

## Desktop Entry

Install the desktop launcher:

```bash
cp shapeshifter.desktop ~/.local/share/applications/
# Edit Exec path to match your installation
```

## Part of Lilith Linux

| Component | Purpose |
|-----------|---------|
| **COSMIC Desktop** | Desktop Environment |
| **Shapeshifter** | DE Switching & Profiles |
| **Tweakers** | System Optimization |
| **Lilim** | AI Assistant |

## License

See LICENSE file for details.
