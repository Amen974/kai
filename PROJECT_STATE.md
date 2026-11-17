# PROJECT_STATE

## Overview
`kai` is a local-first desktop AI chat app: Tauri 2 shell, React 19 + TypeScript + Tailwind 4 frontend, Rust backend, SQLite storage, and a local Ollama server (`http://localhost:11434`) for generation. Model is hardcoded to `qwen3:4b`.

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
There are no automated tests in the project.

## Structure

### Backend — `src-tauri/src/`
- `lib.rs`: composition root. Initializes the DB pool, manages `Mutex<ChatSession>`, registers all 16 commands, and syncs the model catalog from Ollama on startup when the `models` table is empty.
- `commands/chat_command.rs`: Tauri boundary (the only layer with `AppHandle`). Commands: `send_message`, `resend_message`, `edit_message`, `load_chat`, `switch_branch`, `get_variants`, `new_chat`, `delete_history`, `cancel_generation`, `get_session`, `get_history`, `search_history`. Also maps `ChatEvent` to Tauri events.
- `commands/model_command.rs`: model catalog commands: `sync_models`, `get_models`, `get_active_model`, `set_active_model`.
- `services/model_service.rs`: all model catalog logic — fetches `/api/tags` plus `/api/show` for each model (vision check), caches the result in the `models` table, preserves or falls back the active selection, and resolves the active model for generation.
- `services/chat_service.rs`: all chat logic — persisting the user message, creating history/branches, streaming Ollama output, finalizing/cancelling generation, edit-as-fork, title generation.
- `state.rs`: `ChatSession` — in-memory source of truth (`history_id`, `messages`, `active` stream + cancellation token, `next_generation_id`, `active_branch_id`) with `begin_generation`, `end_generation`, `push_user`, `push_assistant_placeholder`, `append_content`, `append_thinking`, `pop_last_assistant`, `truncate`, `replace`, `reset`, `snapshot`.
- `models/ai_model.rs`: `AiModel` (stored model details + `supports_vision` + `active`) and Ollama `/api/tags` / `/api/show` payload types.
- `models/chat_model.rs`: `ChatMessage`, `Role` (`User` | `Assistant`), `SessionSnapshot`, `MessageVariant`, `GetHistory`, `ActiveStream`, Ollama request/response types.
- `models/chat_event.rs`: `ChatEvent` enum sent to the UI (tagged `type`).
- `models/error_model.rs`: `ChatError`; serialized to the frontend as its message string.
- `database/init.rs`: opens `kai.db`, sets WAL + foreign keys, applies migrations.
- `database/repository.rs`: all SQL. Branch loading uses a recursive CTE from `branches.head_message_id` up through `parent_message_id`. Model catalog reads/writes: `get_models`, `get_active_model_name`, `replace_models`, `set_active_model`.
- `database/migrations/001_initial_schema.sql`: `historys`, `branches`, `messages`.
- `database/migrations/002_add_fts5.sql`: `history_fts5` virtual table plus triggers for history titles and `role = 'user'` messages.
- `database/migrations/003_add_models.sql`: `models` table plus a partial unique index allowing only one `active` row.
- `ollama.rs`: `post_chat` (streaming `/api/chat`), `post_generate` (`/api/generate`, used for titles), `get_tags` (`/api/tags`), `show_model` (`/api/show`).

### Frontend — `src/`
- `App.tsx`: shell. Installs keymap + chat event listener, recovers session with `get_session`, renders `Home` when `messages.length === 0` and `Chat` otherwise, plus `InputBar`, `HistoryPanel`, `SideLabels`.
- `store/messages.ts`: chat state from the backend. `chatReducer` handles `ChatEvent`; helpers `run` (invoke + error capture), `fetchVariants`, `refreshVariants`, `switchBranch`, `parentKey`.
- `store/ui.ts`: UI-only state — `mode` (`idle` | `compose` | `edit`), `overlay` (`history` | null), drafts, `editIndex`, `thinkingIndex`, history list/index/armed-delete/search, `scrollPinSignal`. `getActiveLayer` resolves `overlay ?? mode`.
- `events/chatListener.ts`: maps Tauri events (`chat_event`, `update_snapshot`, `update_message_content`, `update_thinking_content`, `emit_done`, `chat_error`, `history_changed`) into `useMessages.dispatch`.
- `keymap.ts`: every keyboard binding, scoped by active UI layer; `handleKeydown` dispatches.
- `components/`: `InputBar` (compose/edit textarea, send/edit submit, error line), `MessageView` (user/assistant rendering, variant switching), `MarkdownMessage`, `ThinkingBlock`, `BranchNavigator`, `HistoryPanel` (overlay, debounced FTS search, delete confirm), `SideLabels`, `Copy`, `Logo`.
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
- `historys`: one row per conversation (`title`, `active_branch_id`, `created_at`, `updated_at`).
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
- History search: debounced (150 ms) server-side FTS5 over titles and user messages.
- Model catalog (backend only): `sync_models` loads models from Ollama `/api/tags`, checks the `vision` capability per model via `/api/show`, and stores the details in SQLite; `get_models` / `get_active_model` read the cache, `set_active_model` switches models. Chat and title generation use the active model.
- Session recovery on mount; branch selection persisted via `historys.active_branch_id`.
- Assistant messages support markdown, syntax highlighting, copy, and collapsible thinking.
- Frameless window (`decorations: false`) with drag region; ctrl+shift+m minimize, F11 fullscreen, ctrl+w close.

## Known issues / open points
- Ollama base URL is hardcoded in `ollama.rs`; no settings.
- The model catalog has no frontend UI yet (commands only). If Ollama is unreachable on a fresh database, the table stays empty and chat fails with "no active model selected" until a sync succeeds.
- `send_message` awaits title generation after the response finishes, so the command resolves only after the extra `/api/generate` request.
- `repository::truncate_messages` is unused (`#[allow(dead_code)]`).
- Errors are dropped in several places (`let _ = state.pop_last_assistant()`, `let _ = cancel_generation(...)` in `chat_service.rs`, `refreshVariants().catch(() => {})` in `store/messages.ts`), which conflicts with the "never swallow errors silently" rule.
- `handleKeydown` special-cases a `/` key that no binding uses.
- Tablename `historys` is a legacy spelling; `branches.name` is defined but unused.
- No tests and no CI.
