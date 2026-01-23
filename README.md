# Shapeshifter - Complete Setup Guide

Follow these steps in order to build and run Shapeshifter on your Debian Linux system.

---

## Step 1: Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

## Step 2: Create the Project

```bash
cargo new shapeshifter
cd shapeshifter
```

---

## Step 3: Create the UI Directory

```bash
mkdir ui
```

---

## Step 4: Copy the Slint UI File

Copy the **entire contents** from the "appwindow.slint - UI Definition" artifact into a new file:

```bash
nano ui/appwindow.slint
# or use your preferred text editor
```

Paste the complete Slint code and save the file.

---

## Step 5: Copy the Rust Source Code

Copy the **entire contents** from the "Shapeshifter - Profile Manager" artifact into:

```bash
nano src/main.rs
# or use your preferred text editor
```

Replace the default "Hello, world!" code with the complete Shapeshifter Rust code and save.

---

## Step 6: Create build.rs

Create a new file in the project root (same level as Cargo.toml):

```bash
nano build.rs
```

Add this content:

```rust
fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();
}
```

Save the file.

---

## Step 7: Update Cargo.toml

Replace the contents of `Cargo.toml` with:

```toml
[package]
name = "shapeshifter"
version = "1.0.0"
edition = "2021"

[dependencies]
slint = "1.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
dirs = "5.0"
walkdir = "2.4"
tar = "0.4"
flate2 = "1.0"
reqwest = { version = "0.11", features = ["blocking", "multipart"] }
tokio = { version = "1", features = ["full"] }

[build-dependencies]
slint-build = "1.3"
```

---

## Step 8: Install System Dependencies

Slint requires some system libraries on Debian:

```bash
sudo apt update
sudo apt install -y build-essential libfontconfig1-dev libxcb-render0-dev \
    libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev \
    libgtk-3-dev
```

---

## Step 9: Build the Project

```bash
cargo build --release
```

This will take several minutes the first time as it downloads and compiles dependencies.

---

## Step 10: Run Shapeshifter

```bash
./target/release/shapeshifter
```

Or for development/debugging:

```bash
cargo run
```

---

## Your Project Structure Should Look Like This:

```
shapeshifter/
├── Cargo.toml
├── build.rs
├── src/
│   └── main.rs          (Rust code from artifact)
├── ui/
│   └── appwindow.slint  (Slint UI code from artifact)
└── target/              (created by cargo)
```

---

## Using Shapeshifter

### Create Your First Profile:
1. Click **"+ New Profile"**
2. Enter a name (e.g., "My GNOME Setup")
3. Optionally check "Save to remote backup"
4. Click **"Create Profile"**

### Switch Desktop Environments:
1. Click **"🖥️ Switch DE"** in the sidebar
2. Click **"Switch"** on your desired desktop environment
3. Log out and log back in to complete the switch

### Apply a Saved Profile:
1. Click **"📋 Profiles"** in the sidebar
2. Click **"Apply"** next to any saved profile
3. Follow the instructions in the status bar

### Configure Remote Backup:
1. Click **"⚙️ Settings"**
2. Enter your backup URL (e.g., `https://backup.example.com/profiles`)
3. Click **"Save Remote URL"**

---

## Troubleshooting

**"command not found: cargo"**
- Run: `source $HOME/.cargo/env`
- Or restart your terminal

**Build errors about missing libraries**
- Make sure you installed all system dependencies in Step 8

**UI doesn't appear**
- Check that `ui/appwindow.slint` exists and has the correct content
- Check that `build.rs` exists in the project root

**Permission errors when switching DE**
- The app modifies `~/.dmrc` which doesn't require sudo
- Modifying `/var/lib/AccountsService/` requires sudo (app handles this gracefully)

---

## Optional: Create Desktop Launcher

Create `~/.local/share/applications/shapeshifter.desktop`:

```ini
[Desktop Entry]
Name=Shapeshifter
Comment=Desktop Environment and Profile Manager
Exec=/path/to/shapeshifter/target/release/shapeshifter
Icon=preferences-desktop
Terminal=false
Type=Application
Categories=System;Settings;
```

Replace `/path/to/shapeshifter` with your actual project path.
