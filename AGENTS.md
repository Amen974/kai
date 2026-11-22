# AI Rules

Read this file and `PROJECT_STATE.md` before every task. Follow these rules exactly. If a rule conflicts with the task, stop and ask.

## 1. Scope
1. Change only what the task asks for. Nothing else.
2. Touch the fewest files possible. List the files you will edit before editing.
3. Do not refactor, rename, reformat, reorder imports, or "clean up" code that is not part of the task.
4. Do not delete code or files unless the task explicitly says to.
5. Do not add dependencies. If one is required, stop and ask.
6. Do not change config files (`vite.config`, `tsconfig`, `package.json`, `.env`, etc.) unless the task names them.
7. If the task needs a change larger than 200 lines or more than 6 files, stop and propose stages first.

## 2. Before writing code
1. State a short plan: what changes, in which files, why.
2. If the task is ambiguous or information is missing, ask. Do not guess.
3. Search the codebase for an existing function, component, hook, or type that already does what you need. Reuse it.
4. Never import a library or call an API you have not confirmed exists in `package.json` or in the current code. If you are unsure whether an API exists, say "I can't verify this" instead of guessing.

## 3. Writing code
1. Write the minimum code that fully solves the task.
2. No duplicate code. If similar logic exists, reuse or extend it. Do not copy it.
3. No dead code, no unused variables/props/imports, no commented-out code.
4. No code for inputs or states that cannot occur. No parameters, flags, or abstractions without a current caller.
5. Follow the existing code style, naming, and file structure. Match what is already there.
6. Comments only for non-obvious reasons, never to restate what the code does.
7. Do not use `any` or `@ts-ignore` to silence type errors. Fix the type.

## 4. State file
1. `PROJECT_STATE.md` is the source of truth for the app: structure, key modules, data flow, current features, known issues.
2. Read it before every task. Base decisions on it, not on assumptions.
3. After every task, update only the sections your change affected. Keep it short and factual. No history log, no opinions.
4. If the code and the state file disagree, report it. Do not silently pick one.

## 5. Verification
1. After changes, verify with the commands in `PROJECT_STATE.md`: `npx tsc --noEmit` for frontend changes, `cargo check` in `src-tauri/` for backend changes, both if you touched both. Never run `npm run dev` or `npm run tauri dev`.
2. Report the real result. Never claim something was tested or runs if you did not run it.
3. If a check fails and the fix is outside the task's scope, stop and report. Do not fix unrelated things.

## 6. Git
1. Work only on the `ai` branch. Never touch `main`.
2. do not commit uless i tell u.
3. Never force-push, rebase, or rewrite history.

## 7. Final report
End every task with:
- Files changed (list)
- What changed and why (2-4 sentences)
- Verification results
- Anything uncertain or anything you chose not to do

## 8. Project-specific rules

### Backend (`src-tauri/src/`)
1. `ChatSession` (`state.rs`) is the source of truth for app state. Any state change goes through `ChatSession` first, then is persisted via `database/repository.rs`.
2. Every error is a `ChatError` variant in `models/error_model.rs`. Propagate with `?`. Never swallow errors silently: no `let _ =` or `.ok()` that discards a `Result`, no `unwrap()`/`expect()` outside startup. Add a variant instead of using a string.
3. Layers:
   - `commands/`: the only layer that uses `AppHandle`. Talks to the frontend. No logic.
   - `services/`: all logic.
   - `models/`: type definitions only.
   - `database/repository.rs`: the only place with SQL.
   - `ollama.rs`: the only place that calls Ollama.
4. Existing files in `database/migrations/` are immutable. A schema change is a new numbered migration file.
5. A new Tauri command must be registered in `lib.rs`.

### Frontend (`src/`)
1. Zustand stores hold app state. `useMessages` holds state derived from the backend. `useUI` holds UI-only state. Components do not keep backend-derived state in local `useState`.
2. Call the backend only through `run()` in `store/messages.ts`.
3. All Tauri event listening lives in `events/chatListener.ts` and dispatches to `useMessages`. Components never listen to Tauri events.
4. `keymap.ts` holds every key handler. Add new bindings there, scoped to a layer.
5. A new or changed `ChatEvent` must be updated in `models/chat_event.rs`, `types.ts`, and `chatReducer`.

### Known issues
Listed in `PROJECT_STATE.md`. They are not tasks. Do not fix them unless the task says so.