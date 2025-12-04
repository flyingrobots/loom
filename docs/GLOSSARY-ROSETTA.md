# JITOS Rosetta Stone

> **A Translation Guide for Git/OS Users**

This document maps familiar computing concepts to JITOS terminology. If you're coming from Git, Unix, or traditional operating systems, this is your bridge to understanding JITOS's causal operating system model.

---

## Core Abstractions

| **Familiar Term (What Git/OS users know)** | **JITOS Term (What we call it)** | **Why the name?** |
|---|---|---|
| **Process / Thread** | **Shadow Working Set (SWS)** | A process is an isolated branch of the causal graph, existing in superposition until committed—like Schrödinger's cat, but for computation. |
| **Commit** | **Collapse Operator** | Transforms speculative edits into objective truth; inspired by quantum measurement where observation collapses possibility into reality. |
| **Working Directory** | **Materialized Head (MH)** | A projected filesystem view generated from the causal DAG; the shadow on Plato's cave wall—not the authoritative state, but a helpful illusion. |
| **Merge / Rebase** | **Inversion Engine** | Resolves divergent histories without mutating the past; creates inversion-rewrite nodes that map old structure to new without destroying causality. |
| **File / Directory Tree** | **Recursive Meta-Graph (RMG)** | A graph-of-graphs where every node can contain sub-graphs; enables multi-scale event representation from keystrokes to semantic meaning. |
| **Git Repository** | **Causal DAG (Directed Acyclic Graph)** | The immutable, append-only source of truth; every node cryptographically encodes its entire ancestry—Git's object database, but philosophically grounded. |
| **Operating System Kernel** | **JITD / JITOS Daemon** | The long-running privileged process that manages the DAG, enforces invariants, coordinates SWS lifecycle, and mediates all state transitions. |
| **System Calls (syscalls)** | **RPC + ABI (Remote Procedure Call + Application Binary Interface)** | The stable interface through which external tools interact with JITD; like Unix syscalls but designed for causal operations instead of file I/O. |

---

## State & History

| **Familiar Term** | **JITOS Term** | **Why the name?** |
|---|---|---|
| **Branch** | **Ref (Reference)** | A named pointer to a snapshot node in the DAG; identical to Git refs but exists in a causal universe where time is append-only. |
| **HEAD** | **HEAD (same)** | The current position in the causal timeline; points to the snapshot you're observing—familiarity preserved where it makes sense. |
| **Commit Hash** | **Node ID** | A BLAKE3 hash of the node's canonical CBOR encoding; deterministic, cryptographically secure, and architecture-independent. |
| **Commit Message** | **Provenance Node (metadata)** | Human intent, reasoning traces, and semantic context attached to changes; goes beyond text to capture the *why* as structured data. |
| **Git Object (blob/tree/commit)** | **Node (with type: file-chunk, tree, snapshot, etc.)** | The fundamental unit of the RMG; every node is holographic—it cryptographically encodes its entire causal history. |
| **Git Log** | **Write-Ahead Log (WAL)** | The ordered sequence of all collapse events; the temporal backbone that enables deterministic replay and crash recovery. |
| **Staging Area / Index** | **SWS Overlay Graph** | Temporary edits held in the active Shadow Working Set; exists in epistemic isolation until collapse integrates it into objective reality. |

---

## Operations & Workflow

| **Familiar Term** | **JITOS Term** | **Why the name?** |
|---|---|---|
| **`git add`** | **Apply Overlay** | Adds a change to the active SWS overlay graph; does not modify the DAG—only the local subjective worldline. |
| **`git commit`** | **Collapse SWS** | Deterministically converts the SWS overlay into a new snapshot node; the irreversible transition from speculation to truth. |
| **`git checkout`** | **Anchor SWS to Ref** | Creates a new Shadow Working Set based on a specific snapshot; forks a new speculative universe from a known point in history. |
| **`git merge`** | **Inversion Integration** | The Inversion Engine reconciles two divergent timelines by creating inversion-rewrite nodes; preserves both histories without mutation. |
| **`git rebase`** | **Inversion Rewrite** | Replays a sequence of changes atop a new base, mediated by the Inversion Engine; produces new nodes that reference the old structure. |
| **`git status`** | **Query MH Diff** | Compares the Materialized Head's virtual tree index against the current SWS overlays; shows what's changed in human-readable form. |
| **`git log`** | **Traverse DAG** | Walks the parent pointers backward through the causal graph; every node knows its ancestry through cryptographic hashes. |
| **`git push/pull`** | **Sync Protocol (MUFP)** | Multi-Universe Federation Protocol; synchronizes causal DAGs across machines while preserving determinism and auditability. |

---

## Architecture & Internals

| **Familiar Term** | **JITOS Term** | **Why the name?** |
|---|---|---|
| **Git Object Database** | **DAG Store (backed by RocksDB/LMDB)** | The persistent storage layer for immutable nodes; append-only, content-addressed, and cryptographically tamper-evident. |
| **Process Memory** | **SWS Overlay Memory** | Mutable working memory local to a Shadow Working Set; evaporates upon collapse, leaving only the committed snapshot node. |
| **Transaction Log** | **Write-Ahead Log (WAL)** | Ordered record of all collapse events; enables deterministic replay, crash recovery, and forms the arrow of time itself. |
| **Filesystem Watch / inotify** | **Filesystem Watcher (in MH)** | Detects edits to the working directory and routes them into SWS overlays; the bridge from human tools to the causal substrate. |
| **Conflict Markers (`<<<<<<<`)** | **Conflict Markers (same)** | Projected into Materialized Head when the Inversion Engine detects unresolvable divergence; familiarity aids human resolution. |
| **Garbage Collection** | **Tiered Storage Eviction (Hot/Warm/Cold)** | Old nodes migrate from fast local storage to compressed archives; the DAG never forgets, but thermal management optimizes access patterns. |
| **Repository Clone** | **Universe Sync** | Replicates the entire causal DAG to a new machine; every replica is cryptographically identical, down to the bit. |

---

## Philosophical Concepts (New to JITOS)

| **Concept** | **JITOS Term** | **Why the name?** |
|---|---|---|
| **Time** | **Echo** | Every moment carries an echo of its predecessor; time is not a clock, but the pattern of what the system remembers. |
| **Deterministic Replay** | **WAL Replay** | Given the same WAL, JITOS reconstructs identical state on any machine; the universe becomes a pure function of its history. |
| **Version Control** | **Causal Substrate** | Not just tracking changes, but preserving the geometric structure of causality; every event is a point in causal spacetime. |
| **Concurrency** | **Parallel Shadows** | Multiple SWS can exist simultaneously atop the same base node; isolation is epistemic, not just mechanical. |
| **Observer-Relative State** | **Subjective vs Objective** | SWS overlays are subjective (local to an agent); snapshot nodes are objective (committed to the universal DAG). |
| **No Mutation** | **Append-Only Universe** | The past is immutable—never deleted, never modified; all transformations create new nodes that reference (but don't replace) the old. |
| **Multi-Scale Events** | **RMG Regions** | A single "commit" contains micro-events (keystrokes), macro-events (conceptual changes), and semantic graphs (provenance)—all in one holographic structure. |

---

## Security & Identity

| **Familiar Term** | **JITOS Term** | **Why the name?** |
|---|---|---|
| **User / Author** | **Agent Identity (AIS)** | Cryptographic identity tied to public/private keypairs; agents (human or AI) are first-class entities with auditable actions. |
| **GPG Signature** | **Node Signature (optional metadata)** | Cryptographic proof of authorship; stored separately from the NodeID to preserve deterministic hashing. |
| **Permissions (Unix rwx)** | **Capability-Based Security (future)** | JITOS will use capability tokens for fine-grained access control; replaces ambient authority with explicit grants. |
| **Authentication** | **AIS Verification** | Agents prove identity via cryptographic challenge-response; no passwords, only keys. |

---

## Data Structures

| **Familiar Term** | **JITOS Term** | **Why the name?** |
|---|---|---|
| **Linked List** | **Parent Chain** | Every node references its parent(s); walking backward is traversing causality itself. |
| **Merkle Tree** | **Tree Node (with subtree hashes)** | Directories are represented as trees where each node's hash covers its entire subtree; enables O(1) change detection. |
| **Hash Map** | **Virtual Tree Index (VTI)** | BTreeMap<Path, NodeID> maintained by Materialized Head; deterministic ordering ensures cross-machine consistency. |
| **Event Sourcing** | **WAL + DAG** | WAL is the event log; DAG is the materialized state; together they form a CQRS-like architecture where writes and reads are separated. |

---

## Performance & Storage

| **Familiar Term** | **JITOS Term** | **Why the name?** |
|---|---|---|
| **Cache** | **Hot Tier** | Frequently accessed nodes kept in-memory or on fast SSD; LRU eviction to warm/cold tiers based on access patterns. |
| **Archive** | **Cold Tier** | Compressed, remote storage for historical nodes; rehydrated on-demand when traversing deep history. |
| **Lazy Loading** | **Rehydration** | Fetching cold-tier nodes back into hot tier when accessed; transparent to the application layer. |
| **Compression** | **Zstd/LZ4 (in cold tier)** | Nodes compressed during eviction to save space; decompressed during rehydration. |

---

## Key Differences from Git

| **What Git Does** | **What JITOS Does Differently** |
|---|---|
| Files are the atomic unit | Nodes are the atomic unit; files are projections |
| Working directory is mutable state | Working directory is a shadow projection; SWS is the mutable layer |
| Commits are top-level only | Commits are RMG regions containing micro/macro/semantic events |
| Merge conflicts are textual | Inversion Engine resolves structurally, creating rewrite nodes |
| History can be rewritten (`git rebase -i`, `git commit --amend`) | History is immutable; rewrites create new nodes that reference the old |
| Git operates at human scale only | JITOS operates at multiple scales (keystrokes → concepts → semantic graphs) |
| Processes are external to Git | Shadow Working Sets ARE the process abstraction |
| No provenance beyond commit messages | Provenance nodes capture reasoning, tool traces, agent decisions |

---

## Usage Example: Git → JITOS

### Git Workflow
```bash
git checkout -b feature
# (edit files)
git add src/main.rs
git commit -m "Add feature X"
git checkout main
git merge feature
```

### JITOS Equivalent (conceptual)
```bash
jit shadow create --base main  # Returns SWS ID: sws-abc123
# (edit files via your editor; MH filesystem watcher detects changes)
jit shadow collapse sws-abc123 --message "Add feature X"  # Deterministic collapse
jit ref update main sws-abc123  # Move main ref to new snapshot
```

**Key Difference:** In JITOS, the Shadow Working Set is explicit—it's not hidden behind "checkout." You can inspect SWS state, diff it, even run multiple shadows in parallel without interference.

---

## Further Reading

- **For the impatient:** [QUICKSTART.md](./QUICKSTART.md) (when available)
- **For implementers:** [Architecture Document](./ARCH/ARCH-0000-ToC.md)
- **For the philosophically curious:** [ARCH-0000-intro.md](./ARCH/ARCH-0000-intro.md) (The Origin Story)
- **For spec details:** [RFCs](./RFC/) and [ADRs](./ADR/)

---

## Glossary of Abbreviations

- **SWS** — Shadow Working Set
- **MH** — Materialized Head
- **RMG** — Recursive Meta-Graph
- **DAG** — Directed Acyclic Graph
- **WAL** — Write-Ahead Log
- **JITD** — JITOS Daemon (the kernel process)
- **ABI** — Application Binary Interface
- **RPC** — Remote Procedure Call
- **AIS** — Agent Identity System
- **MUFP** — Multi-Universe Federation Protocol
- **VTI** — Virtual Tree Index
- **NICe** — Node Identity & Canonical Encoding (RFC-0001)
- **CDI** — Causal DAG Invariants (RFC-0002)

---

# **CΩMPUTER • JITOS**
© 2025 James Ross • [Flying • Robots](https://flyingrobots.dev)
All Rights Reserved
