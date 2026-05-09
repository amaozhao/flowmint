# Flowmint UI Notes

The MVP desktop UI is organized around:

- Overview
- Assets
- Projects
- Sync
- Settings

React/Tauri pages and components will call typed API wrappers instead of invoking Tauri commands directly from page code.

Overview includes compact charts for asset mix, project readiness, and agent
target support. These charts use existing local data and do not add a charting
runtime dependency.
