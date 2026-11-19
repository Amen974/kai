# PROJECT_STATE

## Overview
`kai` is a local-first desktop AI chat app: Tauri 2 shell, React 19 + TypeScript + Tailwind 4 frontend, Rust backend, SQLite storage, and a local Ollama server (`http://localhost:11434`) for generation.

## Stack
- Frontend: React 19, TypeScript (strict), Vite 8, Tailwind 4, Zustand 5, react-markdown + rehype-highlight.
- Backend: Tauri 2, tokio, reqwest, rusqlite (bundled) + rusqlite_migration, thiserror.
- Storage: SQLite file `kai.db` in the app data dir, WAL mode, foreign keys on.

## Commands (verification)
Frontend (project root):
```
npx tsc --noEmit      # typecheck
npm run build         # typecheck + production bundle (tsc && vite build)
npm run dev           # Vite dev server on port 1420
npm run tauri dev     # desktop app (runs npm run dev first)
```
Backend (`src-tauri/`):
```
cargo check           # typecheck
cargo build           # build
```
Tests (`src-tauri/`):
```
cargo test            # all backend unit tests
cargo test <name>     # single test
```
Covered: `database/repository.rs` (branch loading, head pointers, variants, FTS5 search and its triggers, model catalog reads/writes and the single-active constraint) against an in-memory SQLite database with the real migrations, via the shared `database/test_db.rs` helper; `services/chat_service.rs` line buffer and NDJSON parsing, persisting user messages (history/branch creation, parent links, blank rejection) and the no-history early return of title generation. Not covered yet: Ollama-backed paths in `services/*` (`send`, `resend`, `edit`, `sync_models`, title generation) which need an injectable base URL, `ollama.rs`, `commands/*`, and the frontend.

## Structure

### Backend — `src-tauri/src/`
- `lib.rs`: composition root. Initializes the DB pool, manages `Mutex<ChatSession>`, registers all 19 commands, and syncs the model catalog from Ollama on startup when the `models` table is empty (emitting `models_changed` when it did).
- `commands/chat_command.rs`: Tauri boundary (the only layer with `AppHandle`). Commands: `send_message`, `resend_message`, `edit_message`, `load_chat`, `switch_branch`, `get_variants`, `new_chat`, `delete_history`, `cancel_generation`, `get_session`, `get_history`, `search_history`. Also maps `ChatEvent` to Tauri events.
- `commands/model_command.rs`: model catalog commands: `sync_models`, `download_model`, `cancel_model_download`, `delete_model`, `get_models`, `get_active_model`, `set_active_model`. `sync_models`, `delete_model` and `set_active_model` emit `models_changed`; `download_model` emits `model_download_progress` and resolves when the pull ends, whether it completed or was cancelled.
- `services/model_service.rs`: all model catalog logic — fetches `/api/tags` plus `/api/show` for each model (vision check), caches the result in the `models` table, preserves or falls back the active selection,   and resolves the active model for generation. `sync_if_empty` reports whether it had to sync. `download_model` takes the download token from `ChatSession::begin_download`, streams a pull from Ollama as `ModelDownloadProgress` events, then re-syncs the catalog and emits `ModelsChanged`; a cancelled pull returns `false` before re-syncing. `cancel_download` cancels the active token. There is no downloadable-model catalog: Ollama only lists installed models, so `download_model` takes any model name from the caller.
- `services/chat_service.rs`: all chat logic — persisting the user message, creating history/branches, streaming Ollama output, finalizing/cancelling generation, edit-as-fork, title generation.
- `state.rs`: `ChatSession` — in-memory source of truth (`history_id`, `messages`, `active` stream + cancellation token, `next_generation_id`, `active_branch_id`, `download` cancellation token) with `begin_generation`, `end_generation`, `begin_download`, `end_download`, `push_user`, `push_assistant_placeholder`, `append_content`, `append_thinking`, `pop_last_assistant`, `truncate`, `replace`, `reset`, `snapshot`.
- `models/ai_model.rs`: `AiModel` (stored model details + `supports_vision` + `active`) and Ollama `/api/tags` / `/api/show` payload types.
- `models/chat_model.rs`: `ChatMessage`, `Role` (`User` | `Assistant`), `SessionSnapshot`, `MessageVariant`, `GetHistory`, `ActiveStream`, Ollama request/response types.
- `models/chat_event.rs`: `ChatEvent` enum sent to the UI (tagged `type`): snapshot, deltas, generation finished, error, `HistoryChanged`, `ModelsChanged`, `ModelDownloadProgress`.
- `models/error_model.rs`: `ChatError`; serialized to the frontend as its message string.
- `database/init.rs`: opens `kai.db`, sets WAL + foreign keys, applies migrations.
- `database/repository.rs`: all SQL. Branch loading uses a recursive CTE from `branches.head_message_id` up through `parent_message_id`; `delete_last_message` moves the head to the new last message. Model catalog reads/writes: `get_models`, `replace_models`, `set_active_model`.
- `database/migrations/001_initial_schema.sql`: `historys`, `branches`, `messages`.
- `database/migrations/002_add_fts5.sql`: `history_fts5` virtual table plus triggers for history titles and `role = 'user'` messages.
- `database/migrations/003_add_models.sql`: `models` table plus a partial unique index allowing only one `active` row.
- `database/migrations/004_rename_historys.sql`: renames `historys` to `histories` and drops the unused `branches.name` column.
- `ollama.rs`: `post_chat` (streaming `/api/chat`), `post_generate` (`/api/generate`, used for titles), `get_tags` (`/api/tags`), `show_model` (`/api/show`), `pull_model` (`/api/pull`, takes a `CancellationToken` and returns `Ok(false)` when cancelled, streaming NDJSON progress to a callback, including a final line without a trailing newline; an `error` field, a non-JSON error body, or a stream that ends without the final `success` status becomes `ChatError::ModelDownloadFailed`), `delete_model` (`DELETE /api/delete`).

### Frontend — `src/`
- `App.tsx`: shell. Installs keymap + chat event listener, recovers session with `get_session`, renders `Home` when `messages.length === 0` and `Chat` otherwise, plus `InputBar`, `HistoryPanel`, `SideLabels`.
- `store/messages.ts`: chat state and model catalog state (`models`, `modelsVersion`) from the backend; the active model is the `active` row in `models`. `loadModels` refetches when `modelsVersion` changes; `selectModel` switches the active model and closes the panel, `syncModels` re-syncs — both let `models_changed` drive the refetch. `chatReducer` handles `ChatEvent`; helpers `run` (invoke + error capture), `fetchVariants`, `refreshVariants`, `switchBranch`, `parentKey`.
- `store/ui.ts`: UI-only state — `mode` (`idle` | `compose` | `edit`), `overlay` (`history` | `models` | `downloads` | null), drafts, `editIndex`, `thinkingIndex`, history list/index/armed-delete/search, `modelIndex`, `downloadQuery`, `scrollPinSignal`. `getActiveLayer` resolves `overlay ?? mode`.
- `events/chatListener.ts`: maps Tauri events (`chat_event`, `update_snapshot`, `update_message_content`, `update_thinking_content`, `emit_done`, `chat_error`, `history_changed`, `models_changed`) into `useMessages.dispatch`.
- `keymap.ts`: every keyboard binding, scoped by active UI layer; `handleKeydown` dispatches.
- `components/`: `InputBar` (compose/edit textarea, send/edit submit, error line), `MessageView` (user/assistant rendering, variant switching), `MarkdownMessage`, `ThinkingBlock`, `BranchNavigator`, `HistoryPanel` (overlay, FTS search on query change, delete confirm), `ModelPanel` (keyboard-only overlay, loads on `modelsVersion`, lists cached models with the active one marked), `DownloadPanel` (keyboard-only download overlay: model-name input plus progress bar), `SideLabels`, `Copy`, `Logo`.
- `pages/`: `Home.tsx` (centered logo), `pages/chat/Chat.tsx` (scroll container, bottom-pin scroll behavior, edit scrolling).
- `types.ts`: frontend mirrors of `Message`, `MessageVariant`, `SessionSnapshot`, `ChatEvent`, `Conversation`.
- `App.css`: theme tokens, layout widths, overlay z-layers.

## Data flow
1. User input: `keymap.ts` (ctrl+m / ctrl+e) or `InputBar` → `invoke("<command>")` through `run()` in `store/messages.ts`.
2. `commands/chat_command.rs` forwards to `services/chat_service.rs` with the managed `ChatSession` and DB connection.
3. The service mutates `ChatSession`, persists through `database/repository.rs`, and calls Ollama through `ollama.rs`.
4. Progress is reported as `ChatEvent` → emitted as Tauri events → `events/chatListener.ts` → `useMessages.dispatch` → `chatReducer`.
5. Components read only from the Zustand stores; UI-only state lives in `useUI`, backend-derived state in `useMessages`.

## Data model
- `histories`: one row per conversation (`title`, `active_branch_id`, `created_at`, `updated_at`).
- `branches`: a conversation can have several; `parent_branch_id`, `fork_message_id`, `head_message_id` (head = last message in that branch).
- `messages`: `history_id`, `branch_id`, `parent_message_id`, `role` (`user` | `assistant`), `content`, `thinking`, `created_at`.
- `history_fts5`: FTS5 index over history titles and user-message content; used only by `search_history`.
- `models`: cached Ollama models — `name` (primary key), `family`, `parameter_size`, `quantization_level`, `size_bytes`, `modified_at`, `supports_vision`, `active` (exactly one row can be active). Rows are replaced on `sync_models`, so no Ollama call is needed to read the catalog.
- A conversation's visible messages are the chain from `branches.head_message_id` back through `parent_message_id`.

## Current features
- Streaming chat with Ollama including separate `thinking` deltas.
- Composer (`ctrl+m`) and in-place edit (`ctrl+e`, `ctrl+ArrowUp/Down` to jump between user messages).
- Edit forks a new branch from the previous message and regenerates; `BranchNavigator` switches between sibling variants.
- Resend last response (`ctrl+r`), cancel generation (`ctrl+s`, cancellation token).
- New chat (`ctrl+n`), history overlay (`ctrl+h`) with arrow/Enter selection, two-step delete (`ctrl+Delete` / `ctrl+Backspace`).
- Model picker (`ctrl+k`, keyboard only): overlay lists cached models with family, parameter size, quantization, disk size, modified date and a vision icon for image-capable models, with the active model marked; Enter or click switches, `ctrl+r` re-syncs from Ollama. The catalog refetches only when the backend emits `models_changed`.
- History search: server-side FTS5 over titles and user messages, requested on each query change; results from a superseded query are discarded. The search field stays visible while a query is active, so an empty result set ("No results") can still be edited or cleared.
- Model catalog: `sync_models` loads models from Ollama `/api/tags`, checks the `vision` capability per model via `/api/show`, and stores the details in SQLite; `get_models` / `get_active_model` read the cache, `set_active_model` switches models. `download_model` pulls any Ollama model by name with live progress and re-syncs the catalog; `delete_model` removes a model through `/api/delete` and re-syncs (backend only, no UI caller).
- Model download overlay (`ctrl+d`, keyboard only): type any Ollama model name, Enter starts the pull, and a footer bar shows status, percent and transferred size from `model_download_progress`; errors from Ollama appear in the panel. `ctrl+s` cancels the running pull (Ollama resumes the partial pull on the next attempt) and the bar clears when the command resolves. Starting a download clears the previous error, a download that ends, completes or is cancelled, clears the bar when the command resolves, and a failed or interrupted download reports the error and clears the bar, leaving the partial pull to resume on the next attempt. Nothing is fetched when the overlay opens. Chat and title generation use the active model, and `ModelPanel` drives all four commands. If the cache is empty, the first chat request syncs the catalog on demand and emits `models_changed`, so no manual refresh is needed.
- Title generation for a new conversation runs in a detached task after `send_message` resolves; the title arrives through `history_changed`.
- Session recovery on mount; branch selection persisted via `historys.active_branch_id`.
- Assistant messages support markdown, syntax highlighting, copy, and collapsible thinking.
- Frameless window (`decorations: false`) with drag region; ctrl+shift+m minimize, F11 fullscreen, ctrl+w close.

## Known issues / open points
- Ollama base URL is hardcoded in `ollama.rs`; no settings.
- Only `database/repository.rs` has tests; no frontend tests and no CI.
