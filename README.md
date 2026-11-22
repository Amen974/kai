# kai

[![cargo test](https://github.com/Amen974/kai/actions/workflows/cargo-test.yml/badge.svg)](https://github.com/Amen974/kai/actions/workflows/cargo-test.yml)

# kai

A local-first desktop AI chat application that streams responses from a locally running Ollama server and stores conversations in SQLite.

![alt text](public/design.png)

## Features

- Streaming chat with Ollama, including separate rendering of model "thinking" output.
- Message editing that forks a new branch and regenerates the response, with a branch navigator to switch between variants.
- Resend the last response and cancel an in-flight generation.
- Conversation history with server-side SQLite FTS5 search over titles and user messages.
- Model catalog synced from Ollama (`/api/tags` and `/api/show`), cached in SQLite, with per-model family, parameter size, quantization, disk size and vision support.
- Model picker and model download panel with live pull progress.
- Assistant messages rendered as Markdown with syntax highlighting, copy support and collapsible thinking blocks.
- Conversation titles generated automatically after the first message.
- Session recovery on launch and persisted branch selection.
- Keyboard-driven UI with a frameless window.

## Tech stack

| Layer | Technology |
| --- | --- |
| UI | React 19.1, TypeScript 6.0, Tailwind CSS 4.3, Zustand 5.0 |
| Build | Vite 8.0, `@vitejs/plugin-react` 6.0 |
| Desktop shell | Tauri 2 (`@tauri-apps/api` 2, `@tauri-apps/cli` 2) |
| Backend | Rust (edition 2021), tokio, reqwest, thiserror |
| Storage | SQLite via `rusqlite` 0.40 (bundled) with `rusqlite_migration` 2.6 |
| Markdown | `react-markdown` 10.1, `remark-gfm`, `rehype-highlight` + `highlight.js` 11 |
| LLM runtime | Ollama at `http://localhost:11434` |

## Getting started

### Prerequisites

- Node.js and npm.
- A Rust toolchain, plus the system dependencies required by Tauri 2 for your platform.
- [Ollama](https://ollama.com) running locally on port `11434`, with at least one model pulled (for example `ollama pull qwen3:4b`).

Ollama is **not** bundled with kai and no model is shipped with it. The app looks for Ollama on `127.0.0.1:11434` and reports whether it is missing or simply not running; models can be pulled from the model download panel (`ctrl+d`) or with `ollama pull <model>`.

### Install and run

```bash
git clone https://github.com/Amen974/kai.git
cd kai
npm install
npm run tauri dev
```

The Vite dev server runs on port `1420` and is started automatically by `tauri dev`. Running `npm run dev` alone serves only the frontend, so Tauri commands are unavailable in that mode.

### Environment variables

None. The Ollama base URL is fixed in `src-tauri/src/ollama.rs`, and the app needs no configuration file.

## Available scripts

| Script | Description |
| --- | --- |
| `npm run dev` | Start the Vite dev server on port 1420 (frontend only). |
| `npm run build` | Typecheck with `tsc` and build the production frontend bundle with Vite. |
| `npm run tauri build` | Build the production desktop binary and the platform installers. |

## Building installers

```bash
npm ci
npm run tauri build
```

Generated bundles:

| Platform | Output |
| --- | --- |
| Windows | `src-tauri/target/release/bundle/nsis/kai_<version>_x64-setup.exe`, `src-tauri/target/release/bundle/msi/kai_<version>_x64_en-US.msi` |
| Linux | `src-tauri/target/release/bundle/appimage/kai_<version>_amd64.AppImage`, `src-tauri/target/release/bundle/deb/kai_<version>_amd64.deb` |

Linux packaging needs the Tauri system dependencies (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`); build on Ubuntu 22.04 so the AppImage and `.deb` stay compatible with older distributions. To build just the two Linux formats, use `npm run tauri build -- --bundles appimage,deb`.

The Windows NSIS installer installs per user, so no administrator rights are required. kai keeps its database in the per-user app data directory (`%APPDATA%\com.aamen.kai\kai.db` on Windows, `~/.local/share/com.aamen.kai/kai.db` on Linux); the install directory holds only the binary and the bundled frontend assets.

## Releases

Pushing a tag such as `v1.0.0` runs `.github/workflows/release.yml`, which builds the Windows (`.exe`, `.msi`) and Linux (`.AppImage`, `.deb`) installers and attaches them to a draft GitHub Release. Keep the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` identical to the tag before pushing it.

## Project structure

```text
src/           React frontend: components, pages, Zustand stores, keymap, Tauri event listener
src-tauri/     Rust backend: Tauri commands, services, Ollama client, SQLite repository and migrations
public/        Static assets (logo and background images)
index.html     Vite entry HTML
vite.config.ts Vite config with the fixed dev port used by Tauri
```

## Author

Amen Allah Arfaoui — [github.com/Amen974](https://github.com/Amen974)
