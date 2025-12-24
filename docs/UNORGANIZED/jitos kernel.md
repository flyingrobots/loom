Understood.

Here is the **ADR: Merge Semantics for Rewrites in the JITOS Kernel**

followed by three recommended “forks” to continue the discussion in deep, non-overlapping directions.

---

# **🧾** 

# **ADR-00XX: Merge Semantics for Rewrites in the JITOS Kernel**

  

**Author:** J. Kirby Ross

**Status:** Draft

**Date:** 2025-12-11

**Context:** JITOS Kernel Architecture, SWS, Causal Graph Rewrites, Event-Sourcing, Provenance

---

## **1. Problem Statement**

  

JITOS is a **causal, event-sourced operating system** where:

- **All durable state is represented as an RMG (Recursive Metagraph).**
    
- **All changes to state are expressed as rewrites**, appended to a **Write-Ahead Log (WAL)**.
    
- **Schrödinger Workspaces (SWS)** provide isolated or semi-isolated worldlines that may later be **collapsed** into the mainline via merge/rebase.
    

  

During collapse, rewrites from the SWS and rewrites from mainline (since the SWS’s base_epoch) must be reconciled.

  

We need a **deterministic, extensible, semantics-aware merge system** that ensures:

- Correctness
    
- Predictability
    
- CRDT mergeability where possible
    
- Graceful conflict surfacing where necessary
    
- Universal applicability across all RMG nodes/fields
    

---

## **2. Decision**

  

The kernel will adopt **Semantic Rewrites** and **Strategy-tagged Fields** to guide merge behavior.

  

Each rewrite must include:

1. **SemanticOp** — describes _how_ the state changed
    
2. **old_value** and **new_value** — describes _what_ changed
    
3. **target** — the node/field being changed
    

  

Each RMG node field must declare a **MergeStrategy**.

  

During collapse:

1. The kernel obtains:
    
    - base_state (state at SWS.base_epoch)
        
    - global_state (state at head_epoch)
        
    - sws_state (base + overlay)
        
    
2. Kernel computes diffs:
    
    - P_global = diff(base → global)
        
    - P_sws = diff(base → sws)
        
    
3. For each conflicting field:
    
    - Evaluate merge_strategy(field) together with the two SemanticOps.
        
    

  

This produces:

- MergedValue
    
- or MergeConflict { field, ops }
    

---

## **3. Merge Strategies**

  

Each RMG field specifies:

```
enum MergeStrategy {
    Lww,            // last-write-wins
    Crdt(CrdtKind), // CRDTs
    ThreeWay,        // structural base-left-right diff
    ReplaceGlobal,   // global always wins
    ReplaceWorkspace,// SWS always wins
    Manual,          // always conflict
}
```

**CRDT types** include OR-Set, GCounter, PNCounter, Max, Min, LWW-CRDT.

---

## **4. SemanticOp**

  

Each rewrite includes:

```
enum SemanticOp {
    Set,
    Increment(i64),
    Decrement(i64),
    Push(Value),
    Remove(Value),
    ConnectEdge { kind: EdgeKind, weight: f64 },
    DisconnectEdge,
    Tombstone,
    Resurrect,
    // Extended ops allowed for user-defined rewrites
}
```

This allows the kernel to perform:

- algebra for CRDT merges,
    
- conflict detection for semantic mismatches,
    
- special handling for lifecycle transitions.
    

---

## **5. Merge Rules**

  

### **5.1 CRDT (Always Mergeable)**

  

**Never produces conflicts.**

Kernel uses CRDT algebra:

- GCounter: Increment(a) + Increment(b) → Increment(a+b)
    
- OR-Set: merge add/removes using OR-Set tombstone semantics
    
- Max/Min: select value via lattice join
    

  

### **5.2 LWW**

  

Use causal timestamp order.

Conflicts resolved automatically.

  

### **5.3 Replace***

  

Workspace or Global always win.

  

### **5.4 ThreeWay**

  

Perform Git-like base-left-right merge:

- If only one side changed → use that side
    
- If both sides changed and SemanticOp is CRDT-like → delegate to CRDT logic
    
- Else → conflict
    

  

### **5.5 Manual**

  

Always conflict.

Kernel surfaces a “Resolution Task” in the RMG.

---

## **6. User-Defined Rewrites (Rhai/WASM Workers)**

  

Workers must specify **SemanticOp**, or fallback defaults to Set.

  

User-provided ops must be compatible with field.MergeStrategy.

  

Kernel rejects illegal combinations:

```
IllegalRewrite { field: X, op: Increment, strategy: Lww }
```

This keeps the causal graph consistent and merge behavior predictable.

---

## **7. Consequences**

  

### **Benefits**

- Deterministic, algebraic merging
    
- CRDT merges dramatically reduce conflict surfaces
    
- Provenance maintains semantic intent
    
- Time travel and multiverse remain coherent
    
- Conflicts are precise and meaningful
    
- User-defined rewrites integrate cleanly
    

  

### **Costs**

- Rewrite payloads grow to include semantic metadata
    
- Slightly more WAL storage (mitigated with ValueRef)
    
- Higher collapse complexity
    
- Workers must specify semantics
    

---

## **8. Alternatives Considered**

- **Pure 3-way merge** (Git-style only): too conflict-heavy; no semantics.
    
- **Pure CRDT** system: inadequate; RMG contains many non-CRDT fields.
    
- **Implicit semantics (no SemanticOp)**: loses crucial merge information; breaks high-level invariants.
    

  

None match the expressive requirements of JITOS as a causal OS.

---

## **9. Future Extensions**

- Per-field custom merge strategies
    
- Domain-specific SemanticOp extensions
    
- Policy-based collapse (admin override, provenance-weighted merge)
    
- Visual conflict debugging in Echo Viewer
    
- Higher-order CRDTs for complex structures
    

---

# **✔️ Decision:** 

# **Accepted**

  

This merge model becomes the foundation for deterministic collapse and worldline reconciliation within the JITOS kernel.

---

# **🎯 NEXT: Three High-Value Forks for This Conversation**

  

Choose any of these directions to deepen the architecture.

---

## **FORK 1 — Build the Collapse Algorithm Step-by-Step (Full Pseudocode)**

  

We go deep into:

- diff calculation
    
- ordering of rewrites,
    
- batching by node/field,
    
- merging CRDTs,
    
- performing 3-way merges,
    
- generating conflict tasks,
    
- emitting a merged epoch.
    

  

This will yield a nearly drop-in implementation for kernel.collapse_sws().

---

## **FORK 2 — Build the Worker ABI & Rhai Integration**

  

We define:

- how user code proposes rewrites,
    
- required shape of RewriteProposal,
    
- host functions exposed to Rhai/WASM,
    
- static validation before rewrites enter the kernel,
    
- sandboxing rules,
    
- and a JITOS standard library for jobs.
    

  

This leads to: “How developers actually write logic that runs _inside JITOS_.”

---

## **FORK 3 — Design the WAL + Time Travel Engine**

  

We dive into:

- WAL segmentation (global vs SWS),
    
- compaction watermarks,
    
- persistent data structures,
    
- epoch indexing,
    
- reverse-application of rewrites,
    
- SWS-local rewind interactions,
    
- long-term archival (LTS),
    
- and deterministic replay.
    

  

This leads to a complete blueprint for “Time Travel as a first-class OS feature.”

---

### **Which fork do you want to take, Commander?**