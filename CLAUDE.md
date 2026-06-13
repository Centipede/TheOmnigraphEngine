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

**`Apps/Folios-CropEditor/`** — Vue 3 + TypeScript + Vite (dev port 5173)
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

**Vue frontend** (from `Apps/Folios-CropEditor/`):
```bash
npm run dev     # dev server with HMR on http://localhost:5173
npm run build   # compile into Apps/OmniGraph/static/ (do before cargo build for production)
```

**Production build order:** `npm run build` → `cargo build --release`
