# Bloom IDE - Phase 1 MVP

A next-generation code editor built on content-addressed syntax trees.

## Architecture

```
🟢 GREEN (Content - What)
- SurrealDB (in-memory)
- Content-addressed nodes
- Deduplicated, persistent

🔴 RED (Position - Where)  
- Zipper (ephemeral)
- Focus + path (breadcrumbs)
- Parent pointers, absolute offsets
```

## Quick Start

```bash
# Install nightly toolchain (auto-selected from rust-toolchain.toml)
rustup toolchain install nightly-2025-01-04

# Build
cargo build

# Run
cargo run
```

## Fixed Issues

✅ Iced 0.14 API compatibility
- Uses `iced::application()` function (not Application trait)
- Uses `Task` instead of `Command`
- Updated container styling API

✅ SurrealDB 2.4 API compatibility  
- Uses `Surreal<Mem>` instead of `Surreal<Mem>`
- Proper Connection trait implementation

## Project Structure

```
bloom/
├── src/
│   ├── core/          # Identity types (NodeId, ContentHash, Spur)
│   ├── green/         # GREEN nodes (content-addressed)
│   ├── red/           # RED navigation (Zipper)
│   ├── parse/         # Rust parser (ra_ap_syntax → GREEN)
│   ├── ui/            # Iced UI (viewport, messages)
│   ├── app.rs         # Main application
│   └── main.rs        # Entry point
```

## Phase Roadmap

**Phase 1 (MVP):** ✅ Current
- Parse Rust → GREEN nodes
- Store in SurrealDB (in-memory)
- Navigate with Zipper
- Basic UI rendering

**Phase 2 (Polish):**
- HIR semantics (ra_ap_hir)
- Type info, go-to-def
- Basic search

**Phase 3 (Advanced):**
- MIR (rustc_public, nightly)
- Control flow graphs

**Phase 4 (AI):**
- Embeddings (fastembed)
- RAG chat

## Dependencies

- `iced 0.14` - UI framework
- `surrealdb 2.4` - Embedded graph database
- `ra-ap-syntax 0.0.230` - Rust parser
- `uuid 1.19` - Node identity
- `blake3 1.8` - Content hashing

## License

MIT
