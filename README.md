# Bloom IDE

A code editor where **rowan is the source of truth**.

## Architecture

```
rowan SyntaxNode (RED)     ←── Source of truth, editor state
       ↓
     Zipper               ←── Navigation cursor  
       ↓
   UI renders             ←── Direct projection of rowan tree
       
       ↓ (async, background)
       
   SurrealDB (GREEN)      ←── Side-channel for analytics/RAG
```

**The rowan tree IS the editor.** SurrealDB is for queries rowan can't do:
- Cross-file symbol graphs
- Temporal history  
- RAG embeddings
- Complex graph queries

## Quick Start

```bash
cargo run
```

## Navigation

- **⬆ Up** - Move to parent node
- **⬇ (n)** - Descend into nth child  
- **← Prev / Next →** - Sibling navigation

## Project Structure

```
bloom/
├── src/
│   ├── red/           # Zipper - navigates rowan directly
│   ├── green/         # SurrealDB projection (side-channel)
│   ├── parse/         # Thin wrapper around ra_ap_syntax
│   ├── ui/            # Iced UI
│   ├── app.rs         # Main application
│   └── main.rs        # Entry point
```

## Dependencies

- `ra_ap_syntax` - Rowan-based Rust parser (THE source of truth)
- `iced` - UI framework
- `surrealdb` - Side-channel for analytics
- `blake3` - Content hashing for deduplication

## License

MIT
