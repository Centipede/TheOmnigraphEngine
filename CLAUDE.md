# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

The OmniGraph Engine is an automatic system for ingestion, transcription, and indexing of scanned books and documents. It preserves physical layout, structural hierarchy (chapters/subchapters/footnotes), and enables flexible search (vector, PostgreSQL FTS, selective chapter inclusion). An interactive UI allows manual correction of OCR errors.

The author has a working but non-public Django/PostgreSQL/BeautifulSoup prototype (hOCR processing) that this project rewrites for public release.

## Status

In development. Stack chosen: Rust/Axum backend + Vue 3/Vite frontend.

## Architecture

Two apps under `Apps/`:

**`Apps/OmniGraph/`** — Rust/Axum web server (port 8080)
- `src/main.rs` — entry point, binds listener
- `src/state.rs` — `AppState` (holds `projects_dir: PathBuf`, read from `PROJECTS_DIR` env var, defaults to `./projects`)
- `src/routes/mod.rs` — combines API router with static file fallback; `build_router()` is the entry point
- `src/routes/projects/` — project handlers, models, storage, forms, images
- `static/` — Vue production build output goes here (gitignored); embedded into binary via `rust-embed` at compile time

**`Apps/OmniGraph-UI/`** — Vue 3 + TypeScript + Vite (dev port 5173)
- `vite.config.ts` — `build.outDir` points to `../OmniGraph/static`; `/api` proxied to `http://127.0.0.1:8080` in dev

The Ingestor handles scanned image/PDF ingestion, page naming, and section structure creation. See [Documentation/Planning/Ingestor.md](Documentation/Planning/Ingestor.md) for full spec.

**Transcription/Indexing** — planned follow-on components (not yet specified).

## Development Commands

**Rust server** (from `Apps/OmniGraph/`):
```bash
cargo build           # compile
cargo run             # run on http://127.0.0.1:8080
cargo watch -x run    # auto-restart on file changes (install: cargo install cargo-watch)
```

**Vue frontend** (from `Apps/OmniGraph-UI/`):
```bash
npm run dev     # dev server with HMR on http://localhost:5173
npm run build   # compile into Apps/OmniGraph/static/ (do before cargo build for production)
```

**Production build order:** `npm run build` → `cargo build --release`

## PR procedure

- Each week (Monday-Friday) the planners and testers branch off from 'main' into a planning branch. This is the "planning branch". Some coding will also take place, but mostly research and adjustments. Unit testing as well. Comments are added to a planning github issue that goes with the branch.
- Each weekend (Saturday-Sunday) is where the actual development happens. First order of business is to wrap up the planning branch.
- When the weekend finishes, the result is squash merged into the main branch.
- Copy PR description into the squash commit.
- The PR message must look like:

```
[Briefly, what is accomplished with this PR? Example:]
Adding a deeper exploration level below each entity in the categorised entity lists.

Summary

[List of items that has taken place during the branch. Example:]
* Adds a collapsible box below the entity (collapsed on reload, non-persistent)
* Requests from the backend a json body for the expanded entit
* Displays the key information from the server. If the user further expands sub elements, displays them too. 

Test plan

[Lists of instructions for a human tester. Must contain the markdown-friendly [ ] checkbox container. Example:]
* [ ] Open a section in a book containing entities. The list should be brief and condensed
* [ ] Click on a name, a box should open up below the name which shows mentions
* [ ] Click on a mention, it should expand with a quote and a link to the paragraph containing it
```

Unlike other projects, we run development and testing staggered. The developers simply merge in the PR and copies the description into the squash commit. 
The testers will later come back to the PR (now closed) and perform all the instructions in the test plan. If any fails, they will give feedback in the planning github issue.