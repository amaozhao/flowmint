# Flowmint Development Setup

## Linux Desktop Prerequisites

Flowmint uses Tauri 2 for the desktop app. On Deepin/Debian/Ubuntu-like systems,
install the native desktop development packages before running `tauri dev`:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  file \
  libayatana-appindicator3-dev \
  libdbus-1-dev \
  libgtk-3-dev \
  libjavascriptcoregtk-4.1-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  pkg-config \
  wget
```

Verify the `pkg-config` entries that Tauri's Linux dependencies need:

```bash
pkg-config --exists dbus-1
pkg-config --exists gdk-3.0
pkg-config --exists gtk+-3.0
pkg-config --exists javascriptcoregtk-4.1
pkg-config --exists webkit2gtk-4.1
```

Each command should exit with status `0`.

## Run The Desktop App

The real desktop development entrypoint is:

```bash
npm --workspace apps/desktop run tauri dev
```

This command starts Vite as an internal frontend server, then compiles and opens
the Tauri desktop window. Running `npm --workspace apps/desktop run dev` alone is
only a browser preview and is not the desktop app.
