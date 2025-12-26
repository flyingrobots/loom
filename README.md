# JITOS Monorepo

The unified Rust workspace for the **Just-In-Time Operating System**.

## Structure

- **crates/jitos-core**: Shared types and Intent ABI (SLAPS).
- **crates/jitos-graph**: WARP Graph structure and content addressing.
- **crates/jitos-scheduler**: Echo Radix Scheduler for deterministic concurrency.
- **crates/jitos-inversion**: SWS Collapse and conflict resolution logic.
- **crates/jitos-policy**: Rhai host for sandboxed logic.
- **crates/jitos-planner**: HTN planning logic.
- **crates/jitos-provenance**: Shiplog (WAL) and BTR generation.
- **crates/jitos-resilience**: Deterministic I/O patterns (Ported from Ninelives).
- **crates/jitos-wasm**: Browser bridge.

## Alignment

This repository implements the architecture defined in the **AION Foundations Series (Papers I--VI)**.