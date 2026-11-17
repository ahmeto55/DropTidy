# DropTidy

A production-quality Windows desktop app that safely organizes your Downloads
folder — built with **Tauri v2**, **Rust**, **React**, **TypeScript**, and
**Tailwind CSS v3**.

## Features

- **Safe detection** — finds your Downloads folder via `dirs`; you can pick a
  different folder. Only top-level files are scanned; sub-folders,
  `desktop.ini`, and active downloads (`.crdownload`, `.part`, `.tmp`,
  `.opdownload`) are ignored.
- **Smart categories** — Installers, Archives, Documents, Images, Media, Code,
  and Other (case-insensitive extension matching).
- **SHA-256 duplicates** — same-size files are compared with a cheap 4 KB
  partial hash first, then a full hash, and duplicates are flagged in the
  preview.
- **Safe moves** — category folders are created only when needed, existing
  names get ` (1)`, ` (2)`, … suffixes instead of being overwritten, and
  cross-drive moves fall back to copy + delete.
- **Auto-Organize (watch mode)** — a debounced background watcher sorts new
  files automatically. Files are only moved once they have been quiet for the
  configured delay (2.5 s by default) and can be opened without a Windows file
  lock. The setting and delay persist across restarts, and every automatic move
  is recorded in SQLite so **Undo Last Sort** still restores it.
- **System tray** — open the window, toggle Auto-Organize, or quit from the tray
  icon. The UI refreshes automatically whenever the watcher sorts a file.
- **Undo** — every batch is recorded in a bundled SQLite database in the app
  data directory. Undo restores files and removes empty category folders.
- **Dry-run dashboard** — dark-mode UI with summary cards, category / duplicate
  filters, bulk selection, and a full destination preview.

## Development

```bash
npm install
npm run dev          # Vite dev server on :1420
```

In another terminal:

```bash
cd src-tauri
cargo tauri dev      # requires the Tauri CLI
```

## Verification builds

```bash
npm run build        # TypeScript check + Vite production bundle
cd src-tauri && cargo check
```

## Local installer builds

`bundle.createUpdaterArtifacts` is disabled in `src-tauri/tauri.conf.json`, so
NSIS / MSI bundles can be produced locally without updater signing keys:

```bash
cd src-tauri && cargo tauri build
```

## Architecture

```
src-tauri/src/
  scanner.rs     folder detection, filtering, categorization, hash duplicates
  organizer.rs   conflict-safe moves, cross-drive fallback, undo
  watcher.rs     debounced notify watcher, lock checks, auto-sort events
  db.rs          SQLite batch history and persisted settings
  lib.rs         #[tauri::command] handlers, tray icon, app builder
  main.rs        Windows GUI entry point
src/
  App.tsx        dashboard state and action handlers
  components/    TopBar, SummaryCards, FilterBar, FileTable, Toast, badges
  lib/api.ts     typed invoke wrappers and native folder picker
  types.ts       shared TypeScript models
```
