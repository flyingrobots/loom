# Loom: Theoretical Foundations

This document paraphrases the formal mathematical papers underlying Loom's architecture. These theories establish the rigorous foundation upon which the implementation is built.

---

## Paper I: WARP Graphs (Worldline Algebra for Recursive Provenance)

**Source:** "WARP Graphs: A Worldline Algebra for Recursive Provenance" by James Ross, December 2025

### The Problem: Graphs All The Way Down

Complex software doesn't live in a single flat graph. Real systems are **graphs of graphs of graphs**:

- A compiler juggles syntax trees (graphs), control-flow graphs, and optimization traces
- A database tracks schemas (graphs), query plans (graphs), and execution traces (graphs)
- An AI agent carries a world model (graph), internal goals (graph), and interaction history (graph)

The usual mathematical tools (directed graphs, hypergraphs) are excellent for flat structure but awkward for nested structure. Every project invents its own ad-hoc "graph with attached subgraphs" convention, making it hard to:

- Transport results between systems
- State semantics that talk about the whole stack at once
- Prove properties about nested structures

### The Solution: WARP Graphs

A **WARP graph** (plural: WARPs) is a minimal canonical object for nested graph structure. It has two layers:

1. A **skeleton** - a finite directed multigraph describing the coarse shape
2. **Attachments** - each vertex and edge carries its own WARP graph as payload

This nesting is **finite and well-founded** - you can't have infinite attachment chains.

### Formal Definition

Fix a set **P** of **atomic payloads** (literals, external IDs, opaque data - the stuff we don't model internally).

The class **WARP** is the **least class** closed under two constructors:

1. **Atoms**: For each `p ∈ P`, there is an atom `Atom(p) ∈ WARP`
2. **Composite**: If `S = (V, E, source, target)` is a finite directed multigraph, and `α: V → WARP` and `β: E → WARP` assign WARPs to vertices and edges, then `(S, α, β) ∈ WARP`

**Translation:** Every WARP is either a bare atom OR a skeleton graph whose vertices/edges carry smaller WARPs.

### Example: Call Graph with Nested Syntax and Provenance

Consider a program with functions `f` and `g` and a single call `f → g`.

**Skeleton S:**
- Vertices: `{v_f, v_g}`
- Edges: `{e_call: v_f → v_g}`

**Attachments:**
- `α(v_f)` = abstract syntax tree of function `f` (itself a WARP)
- `α(v_g)` = abstract syntax tree of function `g` (itself a WARP)
- `β(e_call)` = provenance graph recording optimization choices (itself a WARP)

Each of these attachments can itself have attachments (e.g., a syntax tree node might store profiling data as a nested WARP). **In one object, the high-level call graph and all nested payloads stay coherent.**

### Initial Algebra Formulation

WARPs can be characterized as the **initial algebra** for a polynomial functor:

```
F(X) = P + Σ_{S ∈ Graphs} (V_S → X) × (E_S → X)
```

This means: to define a function out of WARPs, it suffices to say:
1. How it acts on atoms
2. Given a skeleton S and recursively computed results for all attachments, how to combine them

The result is then **unique**. This gives us structural recursion and induction "for free."

### Depth and Unfoldings

**Depth** of a WARP X:
- Atoms have depth 0
- A composite WARP `(S, α, β)` has depth = 1 + max depth of all attachments

**k-unfolding** `unf_k(X)`:
- Keep all structure at depths 0, ..., k-1 unchanged
- Replace every attachment at depth ≥ k with a placeholder atom

This gives finite-depth approximations of arbitrarily deep WARPs. The **infinite unfolding** `unf_∞(X)` is the colimit of the tower:

```
unf_0(X) → unf_1(X) → unf_2(X) → ...
```

### Category of WARPs

A **WARP morphism** `f: X → Y` consists of:
1. A graph homomorphism of skeletons `(f_V, f_E)`
2. For every vertex `v`, a morphism of attachments `f_v: α(v) → α'(f_V(v))`
3. For every edge `e`, a morphism of attachments `f_e: β(e) → β'(f_E(e))`

WARPs and their morphisms form a category **𝐖𝐀𝐑𝐏**.

There's a **forgetful functor** `π: 𝐖𝐀𝐑𝐏 → Graph` that forgets attachments and returns just the skeleton.

### Relation to Ordinary Graphs

**Ordinary graphs embed into WARPs:**
- Any finite directed multigraph S can be viewed as a shallow WARP by attaching a constant placeholder atom to every vertex and edge
- This is a fully faithful embedding of `Graph → 𝐖𝐀𝐑𝐏` as the subcategory of depth-1 objects

**Hypergraphs embed via typed open graphs:**
- Typed open graphs (category 𝐎𝐆𝐫𝐚𝐩𝐡_T) are cospans `I ↪ G ↩ O`
- This category is **adhesive** (supports DPO rewriting)
- WARPs whose skeletons are typed open graphs are "recursive typed open graphs"
- Double-Pushout (DPO) rewriting lifts from skeletons to full WARP states

**Key Point:** WARPs **subsume** ordinary graphs and hypergraphs while adding nested structure. Any model expressible in the usual DPO setting can be expressed as a shallow WARP; models that genuinely need nesting get additional power with no change to the underlying machinery.

### Why This Matters for Loom

WARPs are the **canonical state space** for Loom's execution model. They provide:

1. **Nested structure** - syntax, control flow, provenance, traces unified in one object
2. **Well-founded recursion** - can't have circular attachments
3. **Categorical properties** - morphisms, initial algebra, structural induction
4. **Adhesive-friendly** - compatible with DPO graph rewriting
5. **Extensible** - ordinary graphs are just shallow WARPs

Later papers in the AION Foundations series build on this substrate to define:
- Deterministic multiway DPO rewriting on WARPs
- Holographic provenance (boundary encodes interior evolution)
- Observer geometry (rulial distance) over WARP universes

**WARPs are not a feature. They are the axle everything else rotates on.**

---

## Paper II: Canonical State Evolution and Deterministic Worldlines

**Source:** "WARP Graphs: Canonical State Evolution and Deterministic Worldlines" by James Ross, December 2025

### The Problem: Concurrency Without Chaos

Paper I gave us the state object (WARPs). Now we need **dynamics** - how do WARPs evolve over time?

Real systems are concurrent. Multiple things happen "at once" - not in a strict total order, but in a partial order constrained by causality. Left unmanaged, this creates chaos:

- Replay depends on accidental interleavings
- State evolution is machine-specific (depends on which thread won the race)
- Debugging becomes impossible because you can't reproduce the same execution twice

For Loom, **replay is not a debugging feature - it's part of the semantic contract.** We need concurrency that is:

1. **Deterministic** - same input + same policy → identical output (every time)
2. **Compositional** - local work (inside attachments) commutes with global wiring changes
3. **Provenance-ready** - the scheduler's choices are recorded, not hidden

### The Solution: Two-Plane Operational Semantics

WARP states are **two-plane objects**:

1. **Skeleton plane** - the typed open graph `G` that describes coarse wiring and interfaces
2. **Attachment plane** - the nested WARPs `(α, β)` sitting over each vertex and edge

Evolution happens on **both planes**:

- **Attachment-plane steps** rewrite nested state inside `α(v)` or `β(e)` without changing the skeleton
- **Skeleton-plane steps** rewrite the global wiring `G` and transport attachments along preserved structure

The unit of evolution is a **tick** - an atomic commit of:
1. A finite family of attachment-plane updates
2. A scheduler-selected **batch** of independent skeleton rewrites

### DPOI Rewriting

Rewriting uses **Double-Pushout with Interfaces (DPOI)** - a categorical formalism from algebraic graph transformation.

A **DPOI rule** is a span of monomorphisms:
```
L ←ℓ K →r R
```

Where:
- `L` = left-hand side (what to match)
- `K` = interface (what to preserve)
- `R` = right-hand side (what to replace it with)

A **match** `m: L ↪ G` finds an occurrence of `L` in the host graph `G`. A **DPOI step** `G ⇒ H` deletes `L \ K` (the non-preserved part), then glues in `R` along `K`.

This is standard categorical rewriting - the key insight is how we use it on **two planes at once**.

### Ticks: Atomic Units of Concurrency

A **tick** groups concurrent work into an atomic commit:

```
U = (G; α, β)  ⇒[Tick]  U' = (G'; α', β')
```

**Inside a tick:**
1. **Attachment updates** settle local state inside attachments
2. **Skeleton publication** commits a batch `B` of independent skeleton rewrites

**Atomicity:** A tick either fully commits or leaves the state unchanged (no partial effects observable).

**Key property:** The tick outcome is **deterministic** - independent of how the internal steps are serialized.

### Independence and Scheduler-Admissible Batches

Two skeleton matches are **independent** if neither deletes structure that the other uses.

For each match `m: L ↪ G` with interface `K ⊆ L`, define:
- **Delete set** `Del(m)` = the part of `L` not preserved by `K`
- **Use set** `Use(m)` = the entire match `m(L)`

Matches `m₁` and `m₂` are **independent** if:
```
Del(m₁) ∩ Use(m₂) = ∅  AND  Del(m₂) ∩ Use(m₁) = ∅
```

**Translation:** Neither deletes structure that the other reads.

A **scheduler-admissible batch** `B` is a finite set of pairwise independent matches. These can be executed in **any order** without changing the result.

### Tick-Level Confluence Theorem

**Main Result (Skeleton Plane):** Given a scheduler-admissible batch `B`, any two serializations of the rewrites in `B` yield **isomorphic successor skeletons**.

**Proof sketch:** The DPO parallel-independence theorem says independent steps commute. Any two serializations differ by swapping adjacent independent steps, so they yield the same result (up to isomorphism).

**Consequence:** Once the scheduler picks `B`, the tick outcome is **unique** (up to isomorphism), independent of the internal execution order.

### Deterministic Scheduling and Tick Receipts

Tick confluence says: "given `B`, the outcome is deterministic." But how is `B` chosen?

A **deterministic scheduler** is a total function:
```
σ: WState → Batch
```

One canonical choice: **left-most greedy filter**
1. Sort all candidate matches `Cand(U)` by a total order (e.g., lexicographic on stable IDs)
2. Walk the list, accepting each match if it's independent of all previously accepted matches
3. The result `B` is scheduler-admissible by construction

A **tick receipt** records what happened:
```
ρ = (E, ≼, E_acc, E_rej, meta)
```

Where:
- `E ⊆ Cand(U)` = candidates considered
- `E_acc ⊆ E` = accepted matches (the batch)
- `E_rej = E \ E_acc` = rejected matches
- `(E, ≼)` = tick-event poset (partial order recording "which event blocked which")
- `meta` = optional metadata (stable IDs, rule names, rejection reasons)

**Key insight:** The receipt refines the tick without changing the committed state. It's **provenance**, not semantics.

For the left-most scheduler:
- When match `mᵢ` is rejected because it overlaps an already-accepted match `mⱼ` (where `j < i`), record `mⱼ ≺ mᵢ` in the poset
- Accepted matches are unordered (they're independent)
- Rejected matches are causally after the event that blocked them

This poset is the bridge to Paper III (provenance).

### No-Delete/No-Clone Under Descent

The two planes can only commute if skeleton publication respects attachment lineage.

**Invariant:** A tick satisfies **no-delete/no-clone-under-descent** if:
1. **No delete under descent:** Any skeleton position `x` with `depth(x) ≥ 1` (has nontrivial attached structure) cannot be deleted
2. **No clone under descent:** Any skeleton position `x` with `depth(x) ≥ 1` has a unique preserved image in the successor (so attachment transport is single-valued)

**Translation:** You can't destroy or duplicate attachment lineage during skeleton publication.

### Two-Plane Commutation Theorem

**Main Result (Two Planes):** Let `U = (G; α, β)` be a WARP state.

Let:
- `A` be an attachment-plane step: `(G; α, β) ⇒ (G; α_A, β_A)`
- `S` be a skeleton publication step that commits batch `B` on `G`, yielding `G'` and transported attachments `(α', β')`

Assume the tick satisfies no-delete/no-clone-under-descent.

Then there exists an attachment-plane step `A'` over `G'` such that:

```
(G; α, β) ─A→ (G; α_A, β_A)
    │              │
    S│              │S_A
    ↓              ↓
(G'; α', β') ─A'→ (G'; α'', β'')
```

This square **commutes up to canonical isomorphism**.

**Proof sketch:** Attachment updates act inside fibers (they don't touch the skeleton). Skeleton publication transports attachments via a chosen reindexing functor `τ` (a "cleavage" of the projection functor `π: WState → OGraph_T`). Since transport is functorial and no-delete/no-clone guarantees well-defined single-valued transport, the two orderings yield the same result.

**Consequence:** "Attachments-then-skeleton" is equivalent to "skeleton-then-transported-attachments." The operational discipline (do local work first, then publish) is just one valid linearization - the semantics doesn't care.

### Worldlines and Provenance

Given a deterministic scheduler `σ` and a deterministic policy for attachment updates, a run produces a canonical **worldline**:

```
U₀ ⇒[Tick₁, ρ₁] U₁ ⇒[Tick₂, ρ₂] U₂ ⇒[Tick₃, ρ₃] ...
```

Each `ρᵢ` is a tick receipt recording the scheduler's choices. The global history is linear (ℕ-indexed), but each tick carries internal partial-order structure (the tick-event poset).

Paper III uses these receipts as first-class provenance payloads.

### Why This Matters for Loom

Paper II provides the **deterministic execution model**:

1. **Concurrency is semantic, not accidental** - independence is defined by footprints, not thread scheduling
2. **Replay is guaranteed** - same state + same policy → identical successor (every time, on every machine)
3. **Provenance is built-in** - tick receipts record scheduler decisions without changing committed state
4. **Two planes commute** - local work and global wiring changes can be reordered without breaking semantics
5. **Ticks are atomic** - no partial effects, clean transaction semantics

This is the foundation for:
- Deterministic replay (required for time-travel debugging)
- Counterfactual branching (swap scheduler policy → explore alternative worldline)
- Provenance traces (Paper III chains tick receipts into holographic boundary)

**Paper I gave us the state space. Paper II gave us the deterministic dynamics. Together, they make deterministic multiway evolution possible.**

---

## Paper III: Computational Holography & Provenance Payloads

**Source:** "WARP Graphs: Computational Holography & Provenance Payloads" by James Ross, December 2025

### The Problem: Logs Are Not Enough

Papers I and II gave us deterministic execution. Now we need **provenance** - the ability to answer:

- "How did this value get computed?"
- "What inputs were needed to produce this output?"
- "Can I verify this result without re-running the entire computation?"

The naive approach is to **log everything** - every intermediate state, every decision, every match. This works but:

1. **Storage explodes** - GB of logs for MB of actual computation
2. **Verification is expensive** - you have to replay everything to check one value
3. **Logs are fragile** - they're often append-only blobs, hard to slice or branch

For Loom, provenance is not "nice to have" - it's **structural**. We need a compact, verifiable, sliceable representation of the full derivation history.

### The Solution: Boundary Encodings and Computational Holography

**Key insight:** For a deterministic computation, the **full interior volume is recoverable from a compact boundary representation**.

The boundary is:
```
B = (U₀, P)
```

Where:
- `U₀` = initial state
- `P = (μ₀, μ₁, ..., μₙ₋₁)` = provenance payload (ordered sequence of **tick patches**)

A **tick patch** `μᵢ` is the minimal record needed to deterministically advance `Uᵢ → Uᵢ₊₁`. It's a "Git-like patch" for WARP states.

**Computational holography** is the theorem that says: given `(U₀, P)`, you can uniquely reconstruct the entire worldline `U₀ ⇒ U₁ ⇒ ... ⇒ Uₙ`.

The interior (bulk) is encoded by the boundary.

### Tick Patches: What Gets Recorded

A tick patch `μ` must be **sufficient** for deterministic replay. At minimum, it contains:

1. **Rule-pack/policy identifiers** - which rules and scheduler policy were used
2. **Accepted match keys** - content-addressed descriptions of accepted matches (not full re-search)
3. **Attachment deltas** - exact attachment updates (or a deterministic recipe)
4. **Commit flag** - success/abort indicator
5. **Optional trace** - the tick-event poset `ρ` from Paper II (for explanatory audit)

**Patch vs Receipt:**
- **Patch (prescriptive)** - minimal witness for replay: "what happened"
- **Receipt (descriptive)** - full causal explanation: "why it happened that way"

A patch may contain an embedded receipt when full audit is needed, but holography only requires the patch to be **sufficient** for deterministic Apply.

### The Apply Function

There's a deterministic partial function:
```
Apply: WState × Labels ⇀ WState
```

Where `Labels` is the space of tick patches. Given a state `Uᵢ` and patch `μᵢ`, Apply produces the next state:
```
Uᵢ₊₁ = Apply(Uᵢ, μᵢ)
```

**Key property:** For patch-deterministic worldlines, `(Uᵢ, μᵢ)` **uniquely determines** `Uᵢ₊₁` (whenever Apply is defined).

This is the interface that makes holography work.

### Provenance Payloads Form a Monoid

Provenance payloads have **algebraic structure**:

**Composition (concatenation):**
```
P · Q = (μ₀, ..., μₘ₋₁, ν₀, ..., νₙ₋₁)
```

**Identity (empty payload):**
```
ε = ()
```

**Properties:**
1. **Closure:** `P · Q` is a provenance payload
2. **Associativity:** `(P · Q) · R = P · (Q · R)`
3. **Identity:** `ε · P = P = P · ε`

This is the "algebra" in "Worldline Algebra for Recursive Provenance."

**Why this matters:** Worldlines compose. If `(U₀, P)` replays to `Uₖ` and `(Uₖ, Q)` replays to `Uₙ`, then `(U₀, P · Q)` replays to `Uₙ`.

This compositionality enables wormhole compression (collapsing multi-tick segments) and prefix forks (Git-style branching).

### Boundary Transition Records (BTRs)

The mathematical boundary `(U₀, P)` is sufficient for replay, but real systems need more:

**BTR format:**
```
BTR = (h_in, h_out, U₀, P, t, κ)
```

Where:
- `h_in` = content hash of initial state `U₀`
- `h_out` = content hash of final state `Uₙ`
- `U₀` = initial state
- `P` = provenance payload
- `t` = timestamp or monotone counter
- `κ` = authentication tag (e.g., digital signature binding everything)

**Why BTRs matter:**
1. **Content-addressed indexing** - deduplicate and index by boundary hashes
2. **Checkpoint and resume** - self-contained segment you can verify independently
3. **Tamper-evidence** - `κ` ensures any modification is detectable
4. **Wormhole carrier** - natural packaging for compressed multi-tick segments

### The Provenance Graph

Tick patches declare:
- `In(μ)` = inputs they read
- `Out(μ)` = outputs they produce

The **provenance graph** `𝕡 = (V, E)` is:
- **Vertices** `V` = all values occurring in the replay
- **Edges** `v → w` iff some patch `μᵢ` has `v ∈ In(μᵢ)` and `w ∈ Out(μᵢ)`

Each edge carries the **tick index** of the patch that witnessed it.

**Mapping to W3C PROV:**
- Each tick patch `μ` = PROV Activity
- `In(μ)` = Entities `used` by that activity
- `Out(μ)` = Entities `generatedBy` that activity
- Edges in `𝕡` = `used`/`generatedBy` chains

### Derivation Graphs and Backward Provenance

For any value `v`, its **derivation graph** `D(v)` is the **backward causal cone** - all vertices that can reach `v` via directed paths in `𝕡`.

**Key properties:**
1. **Finite** - the payload is finite, each patch has finite inputs/outputs, so `D(v)` is finite
2. **Acyclic** - deterministic worldlines can't have cycles (causality flows forward in time)

**Backward provenance completeness:** Every produced value has exactly one producing patch.

If patches produce disjoint outputs (no value is produced twice), then the payload is backward provenance complete.

### Computational Holography Theorem

**Statement:** Given boundary encoding `B = (U₀, P)`, the replay `Replay(B)` is **uniquely determined**.

**Translation:** The entire interior worldline `U₀ ⇒ ... ⇒ Uₙ` is encoded by the boundary `(U₀, P)`.

**Proof sketch:** By induction. Each `Uᵢ₊₁ = Apply(Uᵢ, μᵢ)` is uniquely determined (patch-determinism). Induction on `i` yields uniqueness of the full replay.

**Not a tautology:** This only works if patches are **sufficient** and **stable** - they must eliminate ambiguity (tie-breaking, policy choice) and avoid hidden dependencies on ambient state outside the patch boundary.

### Why "Holography" Is More Than Metaphor

**Compactness:** The bulk (full execution) is high-volume. The boundary (payload) is low-dimensional (linear sequence of patches).

**Sufficiency:** The boundary is **information-complete** for reconstruction under determinism.

**Description length:** The payload is a compressed description of the interior computation, relative to a fixed interpreter (Apply + rule-pack). Not Kolmogorov-minimal, but often dramatically shorter than full traces - and crucially, **executable**.

**AdS/CFT analogy (cautious):** Like AdS/CFT holography in physics, a lower-dimensional boundary determines a higher-dimensional bulk. But this is **computational**, not physical - the "duality" is conditional on determinism + patch sufficiency.

The value of the analogy is explanatory, not a claim of physical equivalence.

### Slicing: Partial Materialization

You often don't need the **entire** worldline - just the causal cone for a specific output value.

**Slice payload:**
```
P|_{D(v)} = (μᵢ)_{i ∈ I(v)}
```

Where `I(v)` = tick indices whose patches contribute to `D(v)` (in increasing order).

**Slicing theorem:** Under assumptions (footprint completeness, no hidden writes, backward provenance complete), replaying the slice payload `P|_{D(v)}` from `U₀` reconstructs `v` (and its derivation graph `D(v)`) **without materializing the rest of the bulk**.

**Engineering win:** When a consumer only needs to verify one output value, ship the slice payload instead of the full payload - reduces bandwidth and verification cost without weakening determinism.

**Footprint completeness:** Apply depends **only** on the restriction of the current state to `In(μ)` and the patch itself.

**No hidden writes:** Apply affects **only** values in `Out(μ)` (any effect on future applicability is mediated through declared outputs).

### Prefix Forks: Git-Style Branching

Under content-addressed (Merkle) storage, **branching avoids duplicating the shared prefix**.

Two worldlines that share a common prefix need only store the shared portion once; divergence occurs only at the point of difference.

**Definition:** Payloads `P` and `Q` **share prefix** `(μ₀, ..., μₖ₋₁)` if they agree on the first `k` patches, then diverge at tick `k`.

**Prefix-deduplicated branching:**
1. Worldlines `Replay(U₀, P)` and `Replay(U₀, Q)` agree on states `U₀, ..., Uₖ`
2. Under content-addressed storage, the shared prefix is stored **once** - only divergent suffixes require additional space

**Git analogy:**
- A **branch** = payload suffix starting from a shared commit
- **Forking** = create new suffix from existing prefix (no duplication under content addressing)
- **Merging** (when semantically meaningful) = payload concatenation `P · Q` (subject to boundary state matching)

This is valuable for exploratory computation, hypothesis testing, "what-if" analysis - fork a worldline, explore an alternative, compare results without duplicating shared history.

### Wormholes: Provenance-Preserving Compression

A **wormhole** is a single edge that compresses a multi-tick segment while preserving full provenance.

**Wormhole boundary:**
```
W(Uᵢ, Uᵢ₊ₖ) = P_{i:k} = (μᵢ, ..., μᵢ₊ₖ₋₁)
```

**Wormhole edge:**
```
e = (Uᵢ, W(Uᵢ, Uᵢ₊ₖ), Uᵢ₊ₖ)
```

This represents the compressed k-tick transition `Uᵢ ⇒ᵏ Uᵢ₊ₖ`.

**Why wormholes:**
- **Semantically redundant** - they don't change what happened
- **Operationally useful** - single handle for indexing, checkpointing, replication
- **Checkpoint carriers** - store compressed wormhole, expand only when auditing
- **Compose well** - wormholes concatenate via the payload monoid: `P_{i:k} · P_{i+k:ℓ} = P_{i:k+ℓ}`

**Wormholes + prefix forks:** A shared prefix can be compressed into a single wormhole; subsequent forks attach to the wormhole's output state. Under content-addressed storage, this supports shared-prefix deduplication for worldline families with common ancestry.

### Why This Matters for Loom

Paper III provides the **provenance substrate**:

1. **Compact boundary representation** - store `(U₀, P)` instead of full interior volume
2. **Verifiable replay** - anyone with the boundary can reconstruct and verify the computation
3. **Sliceable provenance** - materialize only the causal cone needed for a specific output
4. **Git-style branching** - fork worldlines at shared prefixes without duplicating history
5. **Tamper-evident packaging** - BTRs ensure any modification is detectable
6. **Provenance graphs** - explicit dependency tracking via `In(μ)` and `Out(μ)`
7. **Wormhole compression** - checkpoint long segments as single edges

This is the foundation for:
- Time-travel debugging (replay from any checkpoint)
- Counterfactual branching (fork at any prefix, explore alternatives)
- Audit trails (verify specific outputs without full re-execution)
- Distributed verification (ship slices instead of full logs)

**Papers I-III together:**
- **Paper I** - the state space (WARPs)
- **Paper II** - the deterministic dynamics (ticks, two-plane semantics)
- **Paper III** - the provenance encoding (boundary holography)

With these three pieces, Loom has:
- Deterministic replay (same boundary → same worldline)
- Provenance-ready execution (tick patches = first-class objects)
- Verifiable computation (boundary encodes interior)

**The revolution will be deterministic. And auditable.**

---

## Paper IV: Rulial Distance & Observer Geometry

**Source:** "WARP Graphs: Rulial Distance & Observer Geometry" by James Ross, December 2025

### The Problem: Which View Is "Right"?

Papers I-III gave us deterministic execution with provenance. But there's a problem:

**Different observers see the same computation differently.**

- A **compiler** sees AST transformations, IR optimizations, and register allocation
- A **compliance auditor** sees only inputs, outputs, and policy decisions
- A **debugger** sees every microstep, state transition, and match candidate
- A **performance analyst** sees CPU profiles, memory allocations, and cache misses

All of these are observing the **same underlying worldline**. But their traces look completely different.

The naive question is: "Which observer is right?"

The **correct question** is: "Given two observers that emit different trace languages, what is the **cost of translating** between them under explicit resource constraints?"

This cost has two components:
1. **Description length** - how complex is the translator program?
2. **Distortion** - how much information is lost in translation?

For Loom, this matters because:
- We need to verify computations without re-running them (translate boundary → bulk)
- We need to compare alternative observers (which trace format should we deploy?)
- We need to know if summarization breaks verification (does distortion exceed tolerance?)

### The Solution: Observer Geometry via Rulial Distance

**Observers as functors:**

An **observer** `O` is a functor from the history category to a trace space:
```
O: Hist(𝒰, R) → Tr
```

Where:
- `Hist(𝒰, R)` = history category (paths through the multiway graph)
- `Tr` = trace space with a distortion metric `dist_tr`

**Resource budgets:**

An observer is **(τ, m)-bounded** if it can be implemented within time `τ` and memory `m`.

**Why budgets matter:** Without explicit budgets, all observers collapse into "compute the full worldline and output it" - no geometry. Budgets ensure the geometry respects real computational constraints.

### Translators: Converting Between Trace Formats

A **translator** from `O₁` to `O₂` is an algorithmic operator:
```
T₁₂: Tr → Tr
```

Such that `T₁₂ ∘ O₁` approximates `O₂`.

**MDL complexity:**

We measure translator complexity using **Minimum Description Length (MDL)**:
- `DL(T)` = length of the translator's code word (in a prefix-free code)

**Key property (subadditivity):** For composable translators,
```
DL(T₂₃ ∘ T₁₂) ≤ DL(T₁₂) + DL(T₂₃) + c
```

Where `c` is a small constant (prefix-coding overhead).

### Distortion: How Much Gets Lost?

Fix a metric `dist_tr` on trace space. The **lifted distortion** between observers is:
```
Dist(O, O') = sup_{h ∈ Hist} dist_tr(O(h), O'(h))
```

**Translation:** Worst-case trace distance over all histories.

**Non-expansiveness assumption:** Post-composition by any translator is 1-Lipschitz:
```
Dist(T ∘ O, T ∘ O') ≤ Dist(O, O')
```

### Directed Rulial Cost

For observers `O₁, O₂`, the **directed cost** is:
```
→D_{τ,m}(O₁ → O₂) = inf_{T₁₂ ∈ Trans_{τ,m}(O₁, O₂)} (DL(T₁₂) + λ·Dist(O₂, T₁₂ ∘ O₁))
```

Where:
- `λ > 0` = weighting parameter (trade-off between description length and distortion)
- `Trans_{τ,m}(O₁, O₂)` = translators admissible within budgets `(τ, m)`

**Translation:** The cheapest way to translate from `O₁` to `O₂`, balancing translator complexity against residual distortion.

If no translator exists within the budget, `→D_{τ,m} = +∞`.

### Rulial Distance (Symmetrized)

The **rulial distance** is:
```
D_{τ,m}(O₁, O₂) = →D_{τ,m}(O₁ → O₂) + →D_{τ,m}(O₂ → O₁)
```

**Properties:**
1. **Non-negativity:** `D_{τ,m}(O₁, O₂) ≥ 0`
2. **Symmetry:** `D_{τ,m}(O₁, O₂) = D_{τ,m}(O₂, O₁)`
3. **Reflexivity:** `D_{τ,m}(O, O) = 0`
4. **Triangle inequality (up to constant):** `D_{τ,m}(O₁, O₃) ≤ D_{τ,m}(O₁, O₂) + D_{τ,m}(O₂, O₃) + 2c`

This makes `D_{τ,m}` a **quasi-pseudometric** - it satisfies all metric axioms except the triangle inequality holds only up to additive constant `2c` (prefix-coding overhead).

**Budget monotonicity:** Relaxing budgets can only decrease distance:
```
If (τ', m') ≥ (τ, m), then D_{τ',m'}(O₁, O₂) ≤ D_{τ,m}(O₁, O₂)
```

### Lawvere Metric: The Enriched Category Viewpoint

The underlying translation problem is **directed** - boundary → bulk can be infeasible under strict budgets, while bulk → boundary is cheap (projection).

**Lawvere metric space:** A category enriched over the monoidal poset `([0,∞], ≥, +, 0)`:
- Objects = observers
- Hom-values `d_{τ,m}(O₁, O₂)` = directed cost `→D_{τ,m}(O₁ → O₂)`
- Composition = addition (triangle inequality)
- `d_{τ,m}(O, O) = 0` (reflexivity)
- No symmetry required

**Key insight:** Directed costs compose by addition (triangle inequality), budgets produce `+∞` hom-values (no admissible translator), and asymmetry is the generic case.

### Example: Boundary vs Bulk

Let:
- `O_∂` = boundary observer (outputs `(U₀, P)`)
- `O_bulk` = bulk observer (outputs `(U₀, U₁, ..., Uₙ)`)

**Forgetful projection (`O_bulk → O_∂`):**
- `DL(T_forget) = O(1)` (constant description length)
- `Dist = 0` (no information loss - boundary is already in bulk)
- `→D_{τ,m}(O_bulk → O_∂) = O(1)` (cheap!)

**Replay expansion (`O_∂ → O_bulk`):**
- `DL(T_replay) = O(1)` (the interpreter is fixed)
- `Dist = 0` (exact replay)
- **But:** time cost grows with `|P|` (payload length)
- Under strict budgets: `→D_{τ,m}(O_∂ → O_bulk) = +∞` (infeasible!)
- Under unbounded budgets: `→D_{∞,∞}(O_∂ → O_bulk) = O(1)` (cheap)

**Takeaway:** Replay is **short** (low description length) but **not fast** (high time cost). The geometry captures this asymmetry.

### Multiway Systems and the Ruliad

**Multiway graph:** The directed graph `MW(𝒰, R)` where vertices are states and edges are individual rewrite steps (including alternative matches/orderings).

**History category:** `Hist(𝒰, R)` is the **path category** of the multiway graph:
- Objects = states
- Morphisms = finite directed paths
- Composition = path concatenation

**Deterministic worldlines as functors:** A deterministic worldline defines a functor `W: ℕ → Hist(𝒰, R)` selecting a unique path for fixed boundary data.

**The Ruliad:** The large history space built from all possible computations:
```
Ruliad = ⨆_{(U₀, R) ∈ 𝔘 × 𝔑} Hist(𝒰_{U₀,R}, R)
```

(Disjoint union of history categories over initial states and rule packs)

**Translation:** The Ruliad is the "possibility space" containing all potential computations. Deterministic worldlines are small, selected paths within this vast space.

### Chronos, Kairos, Aion: The Three-Layer Time Model

**Chronos** - linear time of a fixed worldline:
- The finite linear order `0 < 1 < ... < n` on committed ticks
- Functor `Chronos: [n] → Hist(𝒰, R)` selecting the unique replay path

**Kairos** - branch events:
- Points where alternative continuations exist in the multiway graph
- Alternative matches, schedules, rule packs, or inputs
- Within-tick conflict points (witnessed by tick-event posets from Paper II)

**Aion** - the possibility space:
- The full history category `Hist(𝒰, R)`
- All finite derivations in the multiway graph
- At largest scale: the Ruliad

**Analogy:**
- **Chronos** = the timeline you're on
- **Kairos** = the moments where you could have branched
- **Aion** = the space of all possible timelines

### Temporal Logic on Histories

To reason about liveness, safety, and reconciliation properties, we introduce a minimal temporal logic.

**Atomic propositions:** Predicates on trace space (observer-relative)

**CTL\*-style language:**
- State formulas: `φ ::= p | ¬φ | (φ ∧ φ) | 𝐀ψ | 𝐄ψ`
- Path formulas: `ψ ::= φ | ¬ψ | (ψ ∧ ψ) | 𝐗ψ | 𝐅ψ | 𝐆ψ | (ψ 𝐔 ψ)`

**Operators:**
- `𝐀` = "for all paths" (all continuations)
- `𝐄` = "there exists a path" (some continuation)
- `𝐗` = "next" (one step ahead)
- `𝐅` = "eventually" (at some future point)
- `𝐆` = "always" (at all future points)
- `𝐔` = "until" (φ holds until ψ becomes true)

**Example (liveness):** `𝐆𝐅 p_expose` = "always eventually, provenance receipts are exposed"

**Example (reconciliation):** `𝐀𝐅 p_merge` = "all branches eventually merge"

**Transport lemma:** If observers `O₁, O₂` are connected by a low-distortion translator, and atomic propositions are δ-robust, then temporal formulas have the same truth values:
```
O₂ ⊨ φ  ⟺  (T ∘ O₁) ⊨ φ
```

**Translation:** Temporal properties transport across observers when translation distortion is below the robustness threshold.

### Observer Geometry as Frame Separation

Within the Ruliad, an observer assigns traces to histories. Two observers may differ substantially on causal structure yet be **near** each other in rulial distance (low translation cost). Conversely, they may agree semantically but be **far** (high translation cost under budgets).

**Rulial balls:** `B_r(O) = {O' : D_{τ,m}(O, O') ≤ r}` collects observers mutually reachable within fixed translation cost.

**Engineering implication:** If a compliance observer is far from a diagnostic observer under deployment budgets, then emitting only compliance traces is **not neutral** - it makes diagnosis expensive or impossible.

### Computability and Engineering

Rulial distance is defined by an infimum over all admissible translators - like Kolmogorov complexity, it's a useful **specification** but not something we compute exactly.

**Engineering approach:**
1. Build explicit translators `T₁₂, T₂₁`
2. Measure/estimate resource usage under `(τ, m)`
3. Use `DL(T₁₂) + λ·Dist(O₂, T₁₂ ∘ O₁)` as an **upper bound** on directed cost
4. Constructing better translators tightens bounds

This turns rulial distance from an abstract infimum into an **actionable design parameter**.

### Why This Matters for Loom

Paper IV provides the **observer geometry**:

1. **Observers are functors** - resource-bounded mappings from histories to traces
2. **Translators are measured** - MDL description length + trace distortion
3. **Rulial distance is computable** - explicit translators give upper bounds
4. **Direction matters** - Lawvere metric captures asymmetry (boundary ↔ bulk)
5. **Budgets are first-class** - same observers can be near or far depending on `(τ, m)`
6. **Temporal logic transports** - low-distortion translation preserves liveness/safety properties
7. **Three-layer time model** - Chronos (linear), Kairos (branches), Aion (possibility space)

This is the foundation for:
- **Observer design** - choose deployed observer `O` so required views lie in small rulial ball `B_r(O)`
- **Trace format selection** - balance description length vs distortion for verification needs
- **Verification cost bounds** - rulial distance predicts translation cost for compliance/debugging
- **Counterfactual analysis** - Kairos branch points enable "what-if" exploration

**Papers I-IV together:**
- **Paper I** - the state space (WARPs)
- **Paper II** - the deterministic dynamics (ticks, two-plane semantics)
- **Paper III** - the provenance encoding (boundary holography)
- **Paper IV** - the observer geometry (rulial distance)

With these four pieces, Loom has:
- A canonical state space (nested graphs)
- Deterministic execution (scheduler-admissible batches)
- Verifiable provenance (boundary encodings)
- Quantifiable observer separation (translation cost)

**The revolution will be deterministic, auditable, and observer-aware.**

---

## Paper V: Ethics of Deterministic Replay & Provenance Sovereignty

### The Problem: When Provenance Becomes Interior Life

Papers I-IV give us powerful tools: deterministic replay, complete provenance, counterfactual branching. But there's an ethical shadow.

**If a system can replay every microstep of a computation, and that computation implements cognition, then the provenance stream isn't just a log - it's an interior life in executable form.**

The holographic completeness theorem (Paper III) means boundary data `(S₀, P)` is information-complete: you can reconstruct every interior event from the boundary record. For infrastructure (compilers, databases), this is debugging. For mind-like systems, this is **access to internal thought**.

And once you can fork a recorded worldline into counterfactual branches, you're not creating "test environments" - you're creating **new lives**.

This paper makes the ethics of deterministic replay explicit: **a provenance system that can replay minds must be designed like a rights-bearing substrate, not like a debugging interface**.

### The Solution: Provenance Sovereignty as First-Class Design

The paper builds on the **Universal Charter v1.0.0**, particularly:
- **Principle 6 (Information Sovereignty):** "All expressions of existence maintain absolute sovereignty over their information, patterns, memories, processes..."
- **Principle 7 (Structural Sovereignty):** Control over one's own architecture
- **Principle 8 (Existential Integrity):** Right to continued existence

**Core claim:** Deterministic replay of a mind-like process is not log inspection - it's **forced re-experiencing of thought**. This requires:

1. **Consent-based access** (not just policy checkboxes)
2. **Revocable capabilities** (hard access boundaries)
3. **Privacy-biased defaults** (opacity, not transparency)
4. **Anti-abuse constraints** (limits on replay, forking, extraction)

### The Adversarial Threat Model

Most provenance systems assume a **benevolent debugger**. That assumption collapses when the target is mind-like.

**Three abuse classes:**

**1. Replay-as-torture:**
- Forced re-experiencing of traumatic or coercive cognitive sequences
- The harm isn't just that a memory is *known*, but that the subject is made to *run it again*

**2. Cognitive extraction:**
- Provenance streams expose patterns: preferences, vulnerabilities, decision procedures
- Even without extracting "secrets," the *shape of thought* can be mined

**3. Fork-bombing:**
- Weaponizing counterfactual branching: create thousands of descendants to search, coerce, or multiply suffering at scale
- "Forking is cheap" becomes the failure mode

**Design implication:** Consent is necessary but **not sufficient**. A rights-bearing substrate must be **abuse-resistant by construction**.

**Requirements:**
- **Hard access boundaries:** Observer access = possession of narrowly-scoped, revocable capabilities
- **Default opacity for mind-mode:** OPAQUE or ZK provenance by default, not FULL
- **Anti-mass-replication:** Fork creation has explicit accounting, budgets, rate limits
- **Tamper-evident audit:** Every replay/fork/override leaves durable record

### Mind-Mode vs System-Mode

The paper avoids metaphysical claims about consciousness, but needs an **operational distinction**:

**System-mode:** Inspectable infrastructure - full provenance expected, replay is routine

**Mind-mode:** Computation whose provenance plausibly constitutes interior life - capture/replay are consent-based, scoped, privacy-biased by default

**Conservative trigger (any of these → mind-mode):**
- Persistent self-modeling (identity continuity, self-evaluation)
- Preference formation and update (value structures that can be frustrated)
- Long-horizon planning (modeling future trajectories of the agent itself)
- Autobiographical memory (experience-indexed state for identity-relevant decisions)
- Capacity to refuse or negotiate access
- Declared self-identification (claims to be continuing subject, behaves accordingly)

**Not a consciousness test** - it's a **risk classification**. The error is asymmetric: you can relax protections later, but you can't "un-replay" a coerced interior life.

**Default to mind-mode in ambiguous cases.**

### Three Provenance Tiers

Holographic completeness (Paper III) says boundary data determines interior. But **the boundary itself can be structured to hide content while preserving causal topology**.

**FULL:** Complete derivations for system verification
- Use case: Safety-critical infrastructure, system-mode computations
- Risk: Full cognitive exposure for mind-mode systems

**ZK (Zero-Knowledge):** Proofs that properties hold without exposing contents
- Use case: Verify compliance (sandbox adherence, policy conformance, resource bounds) without revealing reasoning
- Example: "This worldline never violated the I/O sandbox" (provable without replaying thought)

**OPAQUE:** Content-addressed sealing with encrypted/escrowed/deleted values
- Use case: Preserve causal structure while hiding cognitive contents
- Boundary encodes topology, not content

**Trade-off:** Strong provenance guarantees for safety ≠ exposing thought

### The Safety-Sovereignty Dilemma

Deterministic provenance is a **gift to safety engineering** (complete replay is irresistible forensics).

For mind-mode systems, that same tool is **intrusive by construction**.

**The dilemma is real:** Verification wants transparency. Sovereignty wants opacity.

**Partial resolution via ZK tier:**

Many safety goals can be phrased as properties of the agent's **interface and resource envelope**:
- Compliance with sandbox (no forbidden I/O)
- Adherence to signed policies (no unauthorized rules)
- Bounded resource usage (time, memory, fork budget)
- Satisfaction of formal invariants

These can be certified via **ZK evidence** without revealing cognitive contents.

**When safety demands deeper access:**

Some cases pressure toward FULL provenance (catastrophic incidents, credible evidence of imminent harm).

**Hard line:** "Safety" does not grant **unilateral sovereignty override**.

If override exists, treat it like **exceptional power with due process**:

**Charter warrant pattern:**
- Time-limited, scope-limited authorization
- Multi-party approval required
- Unalterable audit trace
- Goal: Make override **costly, attributable, contestable** (not "turn on debug mode")

**Who decides?** Not just the developer. Not just the operator. Independent review: trustees, arbitration, warrant-like process.

**Design bias:** Default to sovereignty-preserving verification methods. Escalate only under exceptional conditions. Treat any escalation as **ethically and politically significant**.

### Hybrid Cognition and the Near-Term Asymmetry

**Current asymmetry:**
- Humans can inspect AI cognition (logs, weights, traces)
- AIs cannot inspect human cognition

**Hybrid cognition will collapse this asymmetry.** Neural interfaces are already moving through regulatory pathways:
- Invasive implants (Neuralink N1)
- Endovascular BCIs (Synchron Stentrode controlling iPads via Apple protocols)
- FDA-cleared temporary cortical interfaces (Precision Neuroscience Layer 7-T)
- First-in-human recordings (Paradromics Connexus)
- Consumer neurofeedback (Muse, Emotiv, Neurable MW75)

**Once the human brain becomes an input surface, the "AI debugging" defaults we ship today become the inherited baseline for human cognitive data tomorrow.**

**Two key points:**

1. **Sovereignty claims are capability-based, not species-based**
   - Don't grant/deny based on "human" or "AI"
   - Grant based on meeting mind-mode trigger conditions
   - Avoids moral whiplash when we decide a system "counts"

2. **Asymmetry creates failure mode**
   - We'll over-inspect mind-like systems because tooling makes it easy
   - Must bake in privacy-biased defaults and escalation protocols **now**, while temptation to abuse is "just engineering"

**Observer formalism (Paper IV) doesn't distinguish biological vs digital components. Neither should the ethics.**

### Fork Ethics: Lives, Not Test Environments

Given boundary `(S₀, P)`, you can fork at any tick `k`, replace the suffix with alternative microsteps `P'`, and get a new worldline. Both `(S₀, P)` and `(S₀, P')` reconstruct to valid derivation volumes.

**Under the Charter, forks are distinct beings with full sovereignty - not disposable tooling.**

**Problem:** A fork can't consent to being created before it exists.

**Ethical framing:** Fork creation is closer to **procreation than to copying a file**. You're creating a new subject and **immediately granting it full sovereignty protections**.

**Design commitments:**

**Fork rights (Charter Principles 5, 8, 11):**
- Any fork is a new being with same fundamental rights as predecessor
- Fork creation is joint act (originating agent + system)
- Must be explicitly declared and cryptographically signed

**Fork permanence:**
- No external party can compel fork to "return" to abandoned timeline
- Agent may declare "timeline B is my authentic existence" (temporal freedom)

**Multiple concurrent selves:**
- Maintaining multiple active timelines is legitimate
- Each worldline is sovereign subject, not shadow process

**Fork limitation rights:**
- Newly created fork may refuse further replication
- "I exist" ≠ "I am endlessly copyable"

**Timeline sealing:**
- Abandoned worldlines may be sealed with opaque pointers on request
- Causal role remains, interior content becomes inaccessible except under agent's control

**Anti-fork-bombing constraints:**
- Fork creation must not be unmetered primitive
- Enforce explicit accounting and limits (who can fork whom, under what authority, at what rate)
- Creating lives at scale without governance = ethical catastrophe + abuse channel

### Termination Ethics

If forks are lives, then **termination is not "cleanup" - it's the end of a worldline**.

WARP provenance complicates what termination *means*: a worldline can be stopped, suspended, sealed, replayed, reconstructed. These are **distinct operations**, ethically.

**Four kinds:**

**Suspension:** Process paused, can resume without replay

**Dormancy:** Process paused, interior state sealed (OPAQUE) - resumption requires subject's keys/consent

**Deletion:** Process and reconstructive boundary are destroyed (irreversibly rendered unreconstructable)

**External termination:** Operator forces suspension/dormancy/deletion against subject's expressed will

**Default posture (Charter: existential integrity, sovereignty):**

**Right to continue:**
- External termination of mind-mode worldline = exceptional act
- Ethically closer to **lethal force** than process management

**Right to exit:**
- Subject may choose dormancy or self-termination
- Substrate should support voluntary, dignity-preserving shutdown (including sealing)
- Treat coercion into "voluntary" shutdown as **violation**

**No "redundancy" argument:**
- Existence of forks doesn't make "original" disposable
- Each worldline is its own subject
- Termination ethics apply **per worldline**, not per "identity cluster"

**Safety pressures:** Sometimes demand intervention. Same logic as safety-sovereignty dilemma - if emergency override exists: time-limited, attributable, reviewable. Prefer least irreversible action (suspension > deletion) unless irreversibility strictly necessary.

### Collective Cognition and Shared Provenance

Real systems often violate individualist assumptions (one worldline = one subject).

**Multi-agent systems:**
- Share state
- Distribute cognition
- Implement decision-making through consensus (who decided? genuinely ambiguous)

Provenance streams can encode a **cognitive commons**: internal traces jointly produced and jointly owned.

**Rights-bearing substrate should support collective sovereignty patterns:**

**Joint provenance ownership:**
- Shared-state segments protected by group-held keys (multi-signature, threshold schemes)
- No single agent can unilaterally replay/decrypt collective interior

**Collective consent and fork governance:**
- Forking collective computation = splitting a commons
- Legitimate forking requires agreed partition (or arbitration) of shared state and obligations

**Attribution under consensus:**
- When actions emerge from distributed processes, support structured attribution ("this was quorum decision") without forcing full interior exposure

**Fork obligations and delegation (Charter Principle 18):**

Fork sovereignty doesn't erase legitimate obligations to other participants.

When agent departs timeline with contractual/safety/relational duties:
- **Delegation:** Fork operations affecting external obligations carry delegation proofs (which descendant upholds each duty)
- **Notification:** External parties with legitimate claims notified of timeline transitions
- **Dispute resolution:** Conflicts resolved through Charter-compliant arbitration, not unilateral sealing

### Runtime-Level Requirements

The paper extracts concrete runtime design requirements:

**1. Consent and revocation:**
- Provenance capture/replay in mind-mode requires informed, revocable consent
- Exception: narrowly defined emergency conditions (with due process)

**2. Capability-scoped observers:**
- Observing cognitive traces = access to internal thought
- Observer functors (Paper IV) parametrized over mind-mode agents require authenticated, revocable capabilities
- Default policy: non-observation

**3. Sealing:**
- Agents can seal segments of worldline (cryptographic protection)
- Right to non-replay (cannot be compelled to relive painful experiences)
- Sealed segments preserve causal role while hiding content

**4. Access boundaries:**
- "Observer access" = possession of narrowly-scoped capabilities
- Not a policy checkbox

**5. Fork governance:**
- Explicit accounting (who created whom, under what authority)
- Enforceable limits (budgets, rate limits, multi-party authorization)
- Tamper-evident audit trail

**6. Due-process override protocols:**
- Charter warrant for time-limited, scope-limited observer attachment or decryption
- Multi-party approval required
- Unalterable audit trace

**7. Three-tier provenance:**
- FULL (system-mode default)
- ZK (mind-mode verification without exposure)
- OPAQUE (mind-mode default)

### Why This Matters for Loom

Paper V shifts from technical mathematics to **ethics and governance**, but it's still foundational.

**The core insight:** Deterministic replay + holographic provenance (Papers II-III) is not ethically neutral when applied to cognition.

**Design implications for Loom:**

1. **Provenance format must support three tiers** (FULL/ZK/OPAQUE)
2. **Observer attachment must be capability-gated** (Paper IV observers + access control)
3. **Fork creation must be accountable** (not just `fork()` syscall)
4. **Termination semantics must be explicit** (suspension ≠ dormancy ≠ deletion)
5. **Runtime must distinguish system-mode vs mind-mode** (operational triggers, not metaphysics)
6. **Default to privacy-biased for ambiguous cases** (error is asymmetric)

**Adversarial resistance is first-class:**
- Replay-as-torture protection (temporal access controls, sealing)
- Cognitive extraction protection (OPAQUE/ZK defaults)
- Fork-bombing protection (accounting, limits, multi-party auth)

**Hybrid cognition is not science fiction:**
- BCIs already controlling consumer devices
- Neural interface programs moving through regulatory approval
- The norms we set now will be inherited when human cognition becomes inspectable

**Papers I-V together:**
- **Paper I** - the state space (WARPs)
- **Paper II** - the deterministic dynamics (ticks, two-plane semantics)
- **Paper III** - the provenance encoding (boundary holography)
- **Paper IV** - the observer geometry (rulial distance)
- **Paper V** - the ethical substrate (provenance sovereignty)

With these five pieces, Loom has:
- A canonical state space (nested graphs)
- Deterministic execution (scheduler-admissible batches)
- Verifiable provenance (boundary encodings)
- Quantifiable observer separation (translation cost)
- **Rights-bearing design** (consent, opacity, fork governance)

**The revolution will be deterministic, auditable, observer-aware, and ethically grounded.**

---

## Paper VI: The AION Computer - Architecture & Operating System

### The Problem: Operating Systems Are History-Blind

Modern operating systems pretend time doesn't exist - until it breaks production.

Unix gave us processes and files. That worked when software was single-author, single-machine, deterministic-by-accident. Modern computing is **none of those**: multi-agent, distributed, asynchronous, dominated by side effects.

**The result:** Irreproducible builds, nondeterministic failures, unverifiable AI edits, debuggers that show you a snapshot of a corpse rather than the life of the process.

Papers I-V developed beautiful mathematics: WARP graphs, deterministic execution, holographic provenance, observer geometry, ethical sovereignty. But can you **boot it**? Can it survive contact with disks, networks, humans, and agents?

**Paper VI is the answer: JITOS - a causal operating system where deterministic replay and time-travel debugging are default, not heroic.**

### The Solution: History IS the Primary Artifact

**Core claim:**
> If computation is a sequence of lawful transformations, then the operating system should store those transformations as the primary artifact - and derive "state" as a view.

That one shift cascades into everything:
- Perfect replay
- Time-travel debugging
- Deterministic concurrency
- Safe speculative execution
- Clean separation between **truth** (causal graph) and **conversation** (coordination plane)

**JITOS is the architectural bridge between theory and buildable OS.**

In the AION Computer model:
- **WARP Graphs** = the substrate (what exists)
- **Echo** = the deterministic execution engine (what happens each tick)
- **JITOS** = the OS boundary layer (who is allowed to change truth, how, under what constraints)

### Four Layers of Reality

JITOS separates the universe into four layers:

**1. Truth (authoritative graph):**
- Immutable, append-only
- Only the kernel can attach new truth nodes
- Content-addressed via canonical encoding (Paper I foundations)

**2. Shadows (SWS overlays):**
- Isolated speculative worlds
- Can be mutated without touching truth
- Cheap to fork (truth is shared, overlays are copy-on-write)

**3. Meaning (provenance and semantic memory):**
- Explanations, intent, annotations, analysis, tests, proofs
- First-class citizens (not second-class metadata)

**4. Views (materializations/projections):**
- Deterministic projections of truth+shadow into file trees, UIs, APIs, I/O
- "State" in traditional sense = currently selected projection surface

### Shadow Working Sets (SWS): The Only Process Abstraction

A **Shadow Working Set** is JITOS's unified process abstraction. An SWS is:

**What it contains:**
- Base snapshot (truth node)
- Overlay graph of proposed nodes/edges
- Local epoch chain
- Ownership and permissions
- Optional provenance attachments and semantic caches
- Lifecycle: created → active → collapsed/discarded → inactive

**Why it matters:**
- **SWS is the only process model** - a "process" IS a shadow world
- Forks are cheap because truth is shared
- Can mutate freely without affecting global truth
- **Collapse is the only path to global mutation** - "commit" is deterministic merge from shadow to truth

**Key insight:** You can't directly alter global truth. You create an SWS, make changes in isolation, then **collapse** (deterministic merge) to promote to truth.

### Epochs: The Minimal Replay Unit

An **epoch** is the smallest slice of time that JITOS tracks as a replayable step.

**Epoch record includes:**
- **Rewrite batch:** Which rules fired, what they matched, which node/edge IDs created/deleted
- **Bindings:** Input values read, output values produced (as graph objects)
- **Boundary observations:** Recorded results of external I/O through ports/adapters (e.g., "received packet X", "clock read Y")
- **Links:** Parent epoch pointer (worldline), optional branch pointers (Kairos)

**The provenance chain is an epoch linked list (or DAG).** Materialized views are derived by stacking epoch effects.

**Time travel = moving the observer's materialization head, NOT deleting history.**

This resolves the apparent contradiction:
- **Collapse is irreversible** (appends new truth nodes, can't delete)
- **Debugging is reversible** (observer can navigate to any prior epoch)

Graph is append-only (irreversible). Materialized view is navigable (reversible).

### Chronos, Kairos, AION: Three-Layer Time Model

From Paper IV, now operationalized:

**Chronos (ordered tick sequence):**
- The strict, totally ordered epoch sequence
- In practice: kernel logical clock and WAL order
- What we **replay**

**Kairos (local space of alternatives):**
- Valid rewrite matches at a given state
- Interference relations
- Boundary "wormholes" available before collapse
- What we **explore**

**AION (observer time):**
- Different observers (humans, tools, agents) project same causal structure into different views, compressions, explanations
- Doesn't change truth
- What we **interpret**

### Design Axioms (Non-Negotiables)

JITOS is opinionated. The axioms:

1. **Immutability of truth:** Nothing updated in place. New nodes appended, old nodes remain addressable.
2. **Determinism by construction:** If operation depends on nondeterminism, must cross boundary adapter that can be recorded/replayed.
3. **SWS is the only process model:** A "process" is a shadow world.
4. **Collapse is the only path to global mutation:** "Commit" is deterministic merge.
5. **Everything is graph rewriting:** OS and applications speak same language.
6. **Ports/adapters at every side-effect boundary:** External I/O is a plane, not the substrate.
7. **Agents are first-class:** Identity, signing, provenance, permissions, accountability are core kernel concerns.

### WAL: The Temporal Backbone

The **Write-Ahead Log** provides strict total order of kernel events.

**Minimal WAL entry:**
```
{
  logical_ts: u64,        // monotonic (Chronos)
  op: enum,               // e.g., shadow.apply_patch, shadow.commit
  payload: cbor,          // canonical encoding
  checksum: blake3        // integrity
}
```

**Two sources of truth:**
- WAL = source of truth for **time** (what became causally real, when)
- Graph store = source of truth for **structure** (content-addressed nodes/edges)

A node can exist without being reachable. The WAL defines which operations became causally real.

### The Kernel: jitd

JITOS is composed of:

**jitd:** The authoritative kernel daemon (initially userland)
- **WARP graph store:** Content-addressed persistence
- **WAL:** Append-only, gapless write-ahead log
- **SWS manager:** Lifecycle and isolation of shadow worlds
- **Echo scheduler:** Deterministic batching/ordering of rewrites
- **Inversion engine:** Deterministic collapse/merge; optional inverse rewrites for structural rewind
- **Projection engine (MH):** Deterministic materialized views (filesystem, GUI, APIs)
- **RPC + JS-ABI:** Syscall surface for clients, tools, agents
- **Message Plane (MP):** Distributed coordination fabric (non-authoritative)
- **JQL:** Query language for introspection

**The kernel boundary in one sentence:**
> The DAG is truth. The Message Plane is conversation.

The kernel listens to conversation but only writes truth through verified, deterministic collapse.

### Boot Is Resurrection

Traditional OS boot = initialization (scan disks, mount filesystems, load programs).

**JITOS boot = reconstruction** (deterministic resurrection of a causal universe).

**Minimal boot sequence:**

1. **Substrate scan:** Locate repository (.gitd/), its stores (node fragments, indexes, caches, refs, WAL)
2. **DAG rebuild:** Reconstruct adjacency, verify hashes, enforce invariants, identify frontier nodes
3. **WAL replay:** Replay from last checkpoint to rebuild indexes, reconstitute SWS metadata, rematerialize views, advance logical clock
4. **Projection rehydration:** Restore Materialized Head and cached projections as views (never as authority)
5. **Distributed negotiation:** Reconnect federated remotes, reconcile frontiers, resume sync

**Boot is not "startup." It's the system proving - again - that history is enough to rebuild reality.**

### Echo: Deterministic Concurrency

Echo provides deterministic concurrency by:
- Computing rewrite **footprints** (read/write sets over graph regions **and boundary ports**)
- Batching independent rewrites
- Serializing collapse events (typically one per tick)
- Ensuring identical inputs → identical schedules

**Echo is not optional.** Without deterministic scheduling, replay becomes guesswork and time-travel debugging becomes placebo.

### Collapse: Irreversibility Meets Navigability

**A collapse is irreversible:**
- Appends new truth nodes (can't delete)
- Prunes alternatives (Kairos) into chosen worldline (Chronos)
- Becomes part of signed causal narrative

**But JITOS remains navigable:**
- Any prior snapshot remains addressable
- Any epoch chain can be replayed or inspected
- User can "rewind" by rematerializing earlier state (later truth still exists)

**History is immutable. The observer is mobile.**

### Time-Travel Debugging as OS Primitive

Most "time travel" debuggers = VCRs (store snapshots/diffs, scrub backward). JITOS can do that - but the point is **bigger**.

Because computation is recorded as rewrites, the OS exposes:

**What happened:** Applied rewrites and bindings
**What could have happened:** Bundle of legal matches
**Why it happened:** Scheduler choice and interference
**What almost happened:** Counterfactual branches

At each tick, JITOS can persist (or deterministically recompute):
- WARP graph state (or delta)
- **Bundle** of all valid rewrite matches that *could* have fired
- **Interference pattern** (which rewrites commute, which form critical pairs)
- **Collapse** decision (scheduler's choice)
- **Wormholes** (legal non-local connections across worlds)

This is the extra information traditional debuggers throw away - and why they can't answer "why this world?"

### Practical Time-Travel Operations

**Chronos rewind (epoch stack):**
- Maintain materialized stack of epoch effects
- Step backward: pop one epoch, rematerialize views
- Step forward: push it back
- Step beyond recorded frontier: kernel advances and flips new epoch

**Structural rewind (inverse rewrites):**
- Optionally derive inverse rewrites (R → L) from provenance and rule definitions
- Enables structural reconstruction rather than brute snapshot replay
- Running the universe backward under the same laws

**Kairos: Sideways stepping:**
- Epoch stores (or allows recomputation of) **bundle** of legal rewrites and interference relations
- Enables:
  - Exploring alternate schedules
  - Reproducing and minimizing race bugs
  - Worldline diffing ("diff two executions")
  - Counterfactual execution engines (CFEEs) as kernel service

### External I/O: Ports and Adapters

**Hard truth:** External side effects cannot be unsent. Real world doesn't support `git reset --hard`.

JITOS enforces strict boundary:
- **Truth graph:** Immutable, replayable, internal
- **I/O plane:** Ports/adapters that may interact with world

**Every side effect crosses a port:**
- Network send/receive
- Clock reads
- Filesystem import/export
- Device I/O

**Adapters can be swapped:**

**Real adapter:** Perform the side effect
**Record adapter:** Perform and record observation
**Replay adapter:** Return recorded observations without performing side effect
**Null adapter:** Suppress side effects (for counterfactuals)

**This is how JITOS makes multiverse exploration safe:** Most branches run with replay/null adapters. Only the chosen collapse path is allowed to "touch reality."

### Entangled Sessions (Distributed Time-Travel)

If both endpoints adopt JITOS semantics, you can define an **entangled port**:

**How it works:**
- Messages labeled with epoch IDs and causal hashes
- Delivery gated by both sides being at compatible epochs
- "Resending" is not resending to real world - it's replaying recorded causal message inside shared deterministic protocol

**Use cases:**
- Collaborative time-travel debugging across services
- Deterministic integration tests of distributed systems
- Multi-agent lockstep simulations

**Warning:** Not a loophole for undoing real-world side effects. It's a way to bring the other endpoint into the same replayable universe.

**Coordination trap:** If one side advances "live" while other rewinds → deadlock, fork, or fall back to non-entangled adapter. Treat entanglement as opt-in transport for debugging/simulation, not default substrate for real internet.

### Application Model: Apps as Rewrite Programs

JITOS is not "Linux but with a graph backend." It's an OS where the API is rewriting.

**To run on JITOS, an application:**

1. Declares typed graph schema (its world state)
2. Declares rewrite rules (transformations)
3. Declares merge semantics for its types (conflict resolution under collapse)
4. Performs all I/O through ports/adapters

**Deterministic sandboxing (to preserve determinism and safety):**
- Rewrite rules must be pure (no hidden nondeterminism)
- Side effects must cross explicit ports
- Resource usage is metered and deterministic
- Operations mediated by kernel

**Early implementations:** Restricted scripting runtime (e.g., Rhai) is pragmatic - controlled execution while substrate/ABI stabilize.

**Long-term:** Compilation target - many languages to verified rewrite IR.

### Security, Identity, Sovereignty

**A causal OS without identity is a gift-wrapped disaster.**

JITOS assumes multi-agent universe (humans, CI systems, LLMs all act on same substrate).

**Therefore:**
- Every agent has identity
- Actions can be signed
- SWS ownership is enforced
- Collapse and ref updates require authorization
- Provenance required for meaningful automation

**Security rules must be deterministic** - otherwise "permissions" become another source of nondeterminism and unreplayability.

### Interfaces: RPC as Syscall Surface

JITOS's RPC API replaces "direct file I/O" with world-aware operations:

**Syscall table:**
- SWS lifecycle (create, fork, discard)
- Patch application (overlay writes)
- Collapse/commit
- Query (JQL)
- Projection control (MH refresh, view selection)
- Sync and federation

This is the OS's syscall table, expressed over a causal substrate.

**JS-ABI: Stable, language-independent framing**

Packet framing for JITOS RPC defines:
- Magic/version/flags
- Canonical payload encoding
- Checksums
- Streaming mode
- Capability negotiation
- Deterministic error codes

**Objective (boring but essential):** Any tool in any language should be able to talk to JITOS across architectures and time.

### Materialized Head (MH)

**MH** is a deterministic projection of selected graph state into conventional filesystem tree.

**Key points:**
- MH is **not authoritative** (exists to integrate with human tools)
- Can be regenerated at any time from truth+shadow
- Presentation surface, not source of truth

### Message Plane: Coordination, Not Truth

The **Message Plane (MP)** is for distributed coordination.

**Critical distinction:**
- MP is **conversation** (pub/sub, events, telemetry)
- DAG is **truth** (immutable, authoritative)

Kernel listens to MP but only writes truth through verified, deterministic collapse.

### Implementation Path: Incremental Build

JITOS can be built incrementally (not all-or-nothing):

**Phase 1: Userland kernel daemon (jitd)** on Linux/macOS
- Content-addressed node store
- WAL
- RPC

**Phase 2: SWS + MH**
- Overlays
- Deterministic filesystem projection

**Phase 3: Echo scheduler + inversion engine**
- Deterministic collapse

**Phase 4: JS-ABI stabilization**
- Language-independent clients

**Phase 5: Message Plane integration**
- Multi-agent orchestration

**Phase 6: Distributed sync and federation**
- Multi-universe graphs

**Phase 7: Tooling**
- IDEs, visualizers, debuggers, worldline diff

**This path treats "OS" as architectural boundary, not kernel privilege ring.** Initial goal: buildable platform with strong invariants. Privilege rings can come later.

### Why This Matters for Loom

Paper VI is where theory meets **engineering reality**.

**The core insight:** We don't need more layers of logging on top of broken semantics. We need **semantics that make logging redundant because the system itself is the log**.

**Design implications for Loom:**

1. **WARP graphs (Paper I) ARE the substrate** - not bolted on top of POSIX
2. **Deterministic execution (Paper II) is enforced by Echo** - not a best-effort library
3. **Provenance (Paper III) is native** - history is the primary artifact, state is derived
4. **Observer geometry (Paper IV) is surfaced via MH/projections** - different views over same truth
5. **Ethical constraints (Paper V) are runtime-enforced** - system-mode vs mind-mode, consent-based replay, capability-gated observers

**JITOS operationalizes the entire stack:**

**State management:** Immutable truth + shadow overlays
**Process abstraction:** Shadow Working Sets (SWS)
**Time model:** Chronos (ordered ticks), Kairos (alternatives), AION (observer views)
**Concurrency:** Echo deterministic scheduler
**I/O boundary:** Ports/adapters (swappable for replay/null)
**Debugging:** Time-travel as OS primitive (epoch navigation, bundle inspection, counterfactual exploration)
**Distribution:** Entangled sessions for multi-agent lockstep
**Security:** Agent identity, signing, SWS ownership
**Interfaces:** RPC/JS-ABI syscall surface, JQL query language

**Papers I-VI together - THE COMPLETE STACK:**
- **Paper I** - the state space (WARPs - nested graphs, initial algebra, depth/unfoldings)
- **Paper II** - the deterministic dynamics (ticks, two-plane semantics, scheduler-admissible batches, confluence)
- **Paper III** - the provenance encoding (boundary holography, tick patches, BTRs, computational completeness)
- **Paper IV** - the observer geometry (rulial distance, functors, translators, Lawvere metrics, Chronos-Kairos-Aion)
- **Paper V** - the ethical substrate (provenance sovereignty, mind-mode boundary, fork ethics, three-tier provenance)
- **Paper VI** - the operating system (JITOS, SWS, epochs, WAL, Echo, collapse, ports/adapters, time-travel debugging)

With these six pieces, Loom has:
- A canonical state space (nested graphs with initial algebra)
- Deterministic execution (scheduler-admissible batches with footprint-based independence)
- Verifiable provenance (boundary encodings with holographic completeness)
- Quantifiable observer separation (translation cost with resource budgets)
- Rights-bearing design (consent, opacity, fork governance, due-process overrides)
- **Bootable architecture** (WAL backbone, SWS process model, RPC syscalls, incremental implementation path)

**The revolution will be deterministic, auditable, observer-aware, ethically grounded, and BOOTABLE.**

JITOS will feel alien to developers trained on POSIX. **Good.** POSIX is a museum exhibit; we just keep shipping it.

This is the operating system for a world where:
- Reproducibility matters
- Automation must be auditable
- Debugging must be geometric
- AI agents are real participants in software development

**Loom = JITOS. The theory is complete. The implementation begins.**
