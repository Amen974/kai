---
name: testing
description: How to write and run tests in kai (Tauri v2, Rust backend with rusqlite, React/TypeScript/Vite frontend). Use whenever asked to add, fix, or run tests.
---

# Testing skill

Scope: this skill covers tests only. Follow the project rules file (no duplicate code, no commits unless told, update the project state file) in addition to everything below.

## Hard rules

1. **Tests only.** Do not edit production code to make a test pass. If a test fails, the code is either wrong or the test is wrong. Decide which, then report it. Never change an assertion to match behavior you suspect is a bug.
2. **Production changes need approval.** If code cannot be tested without a change (e.g. a function that only opens the on-disk `kai.db`, a hardcoded Ollama URL), stop. State the smallest change required and wait for approval. Do not refactor on your own.
3. **Never claim tests pass without running them.** Run the command, read the output, report the real result. If you cannot run it, say so.
4. **One module per task.** Do not write tests for the whole codebase in one pass.
5. **No network, no real Ollama, no real `kai.db`, no real filesystem outside a temp dir.** Tests must be deterministic and runnable offline.
6. **Do not skip or `#[ignore]` a failing test** to get a green run. Report it.
7. If something is unclear (schema, expected behavior, function contract), read the source and migrations first. If still unclear, ask. Do not guess.

## Workflow

1. Read the target file and everything it calls.
2. List the behaviors to test as short sentences, including failure cases that can actually occur. Show the list before writing code.
3. Write the tests.
4. Run them. Fix test mistakes only.
5. Report: what is covered, what is deliberately not covered, any bug found, and any production change you need approved.

## Backend (Rust, `src-tauri/`)

Commands: `cargo test` from `src-tauri/`. Single test: `cargo test <name>`.

### Where tests go

- **Unit tests:** in the same file as the code, at the bottom:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
  }
  ```
- **Integration tests:** `src-tauri/tests/<name>.rs`, a sibling of `src/`. Only use these when testing across modules through the public API. They require the module to be `pub` in `lib.rs`. Prefer unit tests so no visibility changes are needed.
- Never create a `tests/` folder inside `src/`.

### What to test, in priority order

1. `database/repository.rs` and the migrations, against an in-memory SQLite database (`rusqlite::Connection::open_in_memory()`). Apply the real migrations from `database/migrations/` so the tests run against the real schema. Cover the branching history logic: parent/child message links, branch head pointers, loading a branch, the FTS5 search and its triggers.
2. `services/*`: business logic and error paths.
3. `ollama.rs`: only if the HTTP base URL or client can be injected. If not, apply rule 2 and ask.

Do **not** write tests for `commands/*` if they only unwrap state and delegate to services, and do not test third-party crates.

### Rules for Rust tests

- Define the in-memory DB setup helper **once** (a `#[cfg(test)]` function in a single shared place) and reuse it. Do not copy it per file.
- Read the actual migrations and the actual function signatures. Do not invent table names, columns, or function signatures.
- One behavior per test. Name tests after the behavior: `load_branch_returns_messages_in_parent_order`, not `test1`.
- Assert on specific values, not just `is_ok()`. Test error cases with the real error type from `models/error_model.rs`.
- Use `?` and `Result<(), ...>` return types in tests instead of `unwrap()` chains where it reads cleaner.
- If FTS5 is unavailable in the build, the FTS tests fail. Report it, do not remove them.
- No `sleep`, no randomness without a fixed seed, no test depending on another test's data or order.

## Frontend (React + TypeScript, Vitest)

Commands: `npx vitest run` (single run), `npx vitest` (watch).

### Setup (only if not already present)

`npm i -D vitest jsdom @testing-library/react @testing-library/jest-dom @testing-library/user-event`, and in `vite.config.ts` add `test: { environment: 'jsdom' }`. Check what is already in `package.json` before installing anything.

### Where tests go

Next to the source, same name plus `.test.ts` / `.test.tsx`: `src/store/messages.test.ts`, `src/components/BranchNavigator.test.tsx`.

### What to test, in priority order

1. Pure logic: `store/messages.ts` (branch state), `format.ts`, `keymap.ts`. No mocking needed.
2. `events/chatListener.ts`.
3. Components with real logic, e.g. `BranchNavigator.tsx`.

Do not write tests for purely presentational components or CSS. No snapshot tests.

### Rules for frontend tests

- Mock Tauri `invoke` with `mockIPC` from `@tauri-apps/api/mocks`, and call `clearMocks()` in `afterEach`. Mock only the commands the code under test calls.
- For `chatListener.ts` and any `listen` / event API: check the installed `@tauri-apps/api` version's docs or source for how events are mocked. Do not guess the API.
- Test behavior a user can observe. Query by role or label (`getByRole`), not by class name or component internals.
- Reset store state between tests so tests do not depend on order.
- Use `userEvent` for interactions and `await` async assertions (`findBy*`, `waitFor`).

## Finishing

- Do not commit unless told to.
- Update the project state file with: which modules now have tests, the commands used to run them, and known gaps.