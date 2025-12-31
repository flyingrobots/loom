Alright. You already have ~80% of the architecture in those notes. What was missing (and what will save Future-You from “someone added a normal job queue 😭”) is:
	•	a single canonical thesis
	•	non-negotiable invariants
	•	the intent → plan → execute → commit loop written as the OS’s core primitive
	•	sharp anti-patterns (“we do not do kanban-in-the-kernel”)

Below is a paste-ready ARCH doc draft that turns your notes into something official.

⸻

ARCH-0001: JITOS as a Universal Job Fabric

Status: Draft
Date: 2025-12-29
Owner: James
Related: Echo / WARP / Scheduler notes; TASKS/SLAPS; SWS worker pool

Thesis

Traditional PM tools and traditional OS primitives fail for the same reason: they lie about work. They treat progress as a linear list of “states” instead of a causal history of decisions, attempts, constraints, and commits.

JITOS models all work as jobs over a causal graph:
	•	Intent is declared (TASK)
	•	A plan is proposed (SLAP)
	•	Execution happens in speculative overlays (SWS)
	•	The system produces an immutable provenance history (rewrites/events)
	•	Only then do outcomes become “real” via collapse/commit

If an OS can’t tell you why something happened, it’s not a system — it’s a haunted house with logs.

Why this architecture

This design intentionally rhymes with a few well-known ideas:
	•	Event sourcing: store all changes as a sequence of events so you can reconstruct and replay state. That is the “history-first” backbone here.  ￼
	•	Overlay / copy-on-write layers: speculative changes live in an upper layer that can be merged or discarded. SWS is “OverlayFS, but for causal state.”  ￼
	•	HTN planning: decompose high-level goals into primitive executable steps with ordering constraints. That’s the TASKS/SLAPS planning model.  ￼
	•	Microkernel instinct (eventually): keep the kernel core minimal and push “drivers/workers” out. Start monolithic for speed; keep boundaries crisp so you can split later.  ￼

(We are not cargo-culting these patterns. We’re stealing the good parts and refusing the rest.)

⸻

Definitions

WARP

Rewrite MultiGraph (name TBD): the canonical graph structure representing state and its transformation history via rewrites.

Rewrite

An atomic, append-only state transition applied to an WARP. A rewrite is the unit of provenance.

System WARP

The canonical persistent “base reality” graph owned by the kernel.

SWS (Schrödinger Workspace)

A speculative, copy-on-write overlay over a snapshot of the system WARP. It is where risky work happens.

TASK

A declaration of intent (“make X true”), not an instruction for how.

SLAP

A proposed plan (possibly one of many) for satisfying a TASK. SLAPs are branchable, revisable, and auditable.

Worker

An executor of primitive operations: scripts, LLMs, tool adapters, humans-in-the-loop, etc.

Collapse

Transactional merge of an SWS overlay into the system WARP (commit). Discard is the inverse (abort).

⸻

Non-negotiable invariants

These are laws. If a change violates one, it’s not a “refactor,” it’s a fork of the project.
	1.	History is first-class.
State is derived from rewrites/events; we do not treat “current state” as authoritative without provenance.  ￼
	2.	Speculation is default.
Untrusted / risky / agent-driven work happens in SWS overlays, not directly in the system WARP.
	3.	Abort still produces knowledge.
A failed attempt is not “nothing happened.” It is an event in the system’s history. (We can choose how much to persist, but we don’t pretend it didn’t occur.)
	4.	Intent ≠ Plan ≠ Execution.
TASK declares what. SLAP proposes how. Workers perform primitive steps.
	5.	The kernel enforces policy; workers perform mechanism.
We start monolithic for velocity, but the architecture is intentionally separable (kernel vs worker execution boundary).  ￼
	6.	No “task-state cosplay.”
We do not build a kanban board and call it a kernel. “Status” is a view computed from the graph.

⸻

Component architecture

Logical layers
	1.	Kernel (echo-kernel + echo-sched + echo-WARP-core)
	•	Owns system WARP
	•	Manages SWS lifecycle
	•	Runs scheduler ticks
	•	Enforces policy + permissions
	•	Exposes APIs: submit_intent / submit_rewrite / query_state
	2.	Workers (echo-workers)
	•	Pluggable executors (LLMs, shell, adapters, humans)
	•	In-process for v0; out-of-process later
	3.	Clients (echo-net + jitos-cli + viewer)
	•	CLI/TUI/GUI + visualization
	•	Communicate via RPC/socket

Physical deployment (v0)
	•	jitosd: single daemon process linking kernel + workers + net
	•	Separate processes for CLI and viewer, talking to jitosd

This is the “monolith with seams” strategy: ship now, split later.

⸻

Rust workspace layout

echo/
  Cargo.toml          # workspace
  crates/
    echo-WARP-core/    # WARP data structures + rewrite engine
    echo-sched/       # generic scheduler (ticks + rewrites)
    echo-kernel/      # JITOS kernel core (owns WARPs, SWS, processes)
    echo-tasks/       # TASKS + SLAPS + HTN planning -> DAG/job specs
    echo-workers/     # worker registry + invocation abstractions
    echo-net/         # RPC / protocol (gRPC, HTTP, unix socket)
    echo-viewer/      # WARP inspector / debugging UI
  bins/
    jitosd/           # daemon: kernel + net + workers
    jitos-cli/        # CLI client: talks to jitosd via echo-net


⸻

Core data model

Kernel ownership model
	•	One canonical system WARP
	•	Many SWS overlays (copy-on-write deltas) per process/job/agent attempt

Suggested structs:

struct Kernel {
    system_WARP: WARPInstance,                  // base reality
    sws_pool: HashMap<SwsId, SwsInstance>,    // overlays
    processes: HashMap<ProcessId, Process>,   // runtime handles
}

struct Process {
    id: ProcessId,
    sws_id: SwsId,
    caps: Capabilities,
    // metadata: owner, quota, TTL, etc
}

struct SwsInstance {
    parent_snapshot: WARPSnapshotId, // points at system snapshot
    overlay_WARP: WARPInstance,       // deltas only
}

This is conceptually identical to overlay/copy-up systems: reads see merged view; writes go to upper layer; merge commits deltas.  ￼

SWS read/write semantics
	•	Read: view = merge(system_snapshot, overlay)
	•	Write: rewrite applies to overlay only
	•	Collapse: compute/apply rewrite diff from overlay into system, transactionally
	•	Discard: drop overlay (optionally keep audit trail)

Conflict semantics (initial stance)

For v0:
	•	Collapse is “best-effort transactional”
	•	Conflicts are explicit failures requiring rebase/replan (i.e., generate a new SLAP or re-run primitives)

We can later add:
	•	conflict-free merge rules for certain edge types
	•	CRDT-like behavior for specific graph substructures (only if it pays rent)

⸻

TASKS/SLAPS planning model

Why HTN-ish decomposition

We need a planner that can take “Fix auth bug” and produce a structured, inspectable execution DAG without requiring an LLM.

That is literally what HTN planning is about: decompose compound tasks into primitive tasks with ordering constraints.  ￼

Contract
	•	TASK is an intent object written into the system graph
	•	SLAP is a plan candidate (possibly multiple per TASK)
	•	Planner output is a DAG of primitive tasks with:
	•	dependency edges
	•	required capabilities
	•	expected artifacts
	•	suggested workers

Minimal API
	•	plan(task: Task, methods: MethodLibrary) -> Vec<SlapCandidate>
	•	compile(slap: Slap) -> JobDag

Method library
	•	Stored as data (YAML/JSON) + compiled to Rust structs
	•	Deterministic planner first; allow “nondeterministic suggestions” later (LLM can propose methods, but the kernel should not depend on that)

⸻

Execution model

Scheduler loop (echo-sched)

The scheduler is a generic “tick & apply rewrites” engine:
	•	Observe graph state (system + relevant overlays)
	•	Select runnable primitive nodes (deps satisfied, caps ok, quotas ok)
	•	Emit rewrite(s) representing “dispatch”
	•	Worker executes
	•	Worker returns result as rewrite(s) into overlay
	•	Repeat

Worker invocation

Workers are not trusted as truth. They are:
	•	mechanisms that produce proposals/results
	•	that must be recorded as rewrites
	•	and may require validation gates before collapse

Idempotence rule (strongly preferred):
Primitive tasks should be written so retries are safe, or have explicit “already-done” detection.

⸻

Policy and security stance

Even in v0, we treat “who/what can rewrite what” as core.

Recommended direction:
	•	Capability-style permissions: processes carry explicit rights, not ambient authority (least privilege).  ￼
	•	Workers run with bounded capabilities (filesystem, network, tool APIs)
	•	SWS boundaries are safety rails: “do dumb stuff in the overlay, then prove it’s good”

(You can ship without the full capability model; you cannot ship without the architecture that allows it.)

⸻

Build plan (fast dopamine, minimal regret)

Phase 0 — Kernel skeleton
	•	workspace + crates
	•	system WARP + submit_rewrite
	•	jitosd starts and exposes minimal API (HTTP/unix socket)

Demo: mutate and inspect a live system graph.

Phase 1 — Viewer attaches to daemon
	•	snapshot/streaming endpoint
	•	live WARP visualization

Demo: “OS graph animating in real time.”

Phase 2 — SWS overlays
	•	create_sws / apply_rewrite_sws / collapse_sws / discard_sws
	•	visualize overlays + diffs

Demo: parallel speculative workspaces like branches.

Phase 3 — echo-tasks
	•	SLAPS structs + validation
	•	HTN-ish method library + deterministic planner
	•	compile SLAP -> DAG

Demo: “intent in, DAG out.”

Phase 4 — Integrate intent -> SWS -> execution
	•	submit_intent -> ProcessId
	•	write DAG into SWS graph
	•	scheduler dispatches primitives

Demo: tasks appear, run, collapse.

Phase 5 — Real workers
	•	LocalScriptWorker
	•	LLMWorker (optional)
	•	stage code changes in SWS, test, collapse on green

Demo: “holy shit it fixed a trivial bug.”

⸻

Consequences

What we gain
	•	Replay, provenance, debugging by construction (event history is the system)  ￼
	•	Safe speculative execution via overlays  ￼
	•	Planning that is inspectable and deterministic (HTN-ish)  ￼
	•	Clear seams for future microkernel split (workers out-of-process)  ￼

What we accept
	•	More graph plumbing up front
	•	Needing good visualization tooling early (viewer is not “nice to have”)
	•	Merge/conflict semantics will be real work

⸻

Explicit anti-patterns
	•	“Just add a task table.” No. Tasks are nodes in the causal graph, not rows in a database.
	•	“Status fields are the truth.” No. Status is derived, never authoritative.
	•	“Workers mutate the world and we hope.” No. Workers propose rewrites; the kernel records and validates.
	•	“Speculation is optional.” No. Speculation is the default safety model.

⸻

Open questions
	1.	What is the minimal rewrite schema that keeps history useful but doesn’t explode storage?
	2.	How do we represent “confidence” and “validation gates” in the graph?
	3.	What merge policy do we want for common artifact types (files, configs, structured nodes)?
	4.	What’s the GC/compaction story for old overlays and old rewrite chains?
	5.	How do we make “human-in-the-loop” a first-class worker type without turning into Jira?

⸻

Appendix: Why this is an ARCH, not an ADR

ADRs are great for recording discrete decisions in a standard structure (title/status/context/decision/consequences).  ￼
This document is not one decision. It’s a foundational thesis + invariants that future ADRs must not violate.

⸻

Next step (so this doesn’t stay theoretical)

Pick the first canonical graph schema for:
	•	Task node
	•	Slap node
	•	Primitive node
	•	DependsOn edge
	•	ProducedArtifact edge
	•	Rewrite event node/edge (whichever is idiomatic in WARP)

If you want, I’ll draft that schema next (concrete node/edge types + required fields + example JSON/Rust structs), because that’s the point where the architecture stops being poetry and starts being a compiler.

> Interesting opportunity to integrate "GraphQL" -> "Everything" aka Wesley

Oh hell yes. This is one of those “the universe is offering you a free upgrade” moments.

What you’re pointing at (correctly)

Wesley’s core move is: treat GraphQL schema as the single source of truth, then generate the rest (DDL, types, policies, tests, migrations, etc.) from that one schema.  ￼

For JITOS/Echo, the analogous move is even more obscene (in a good way):

GraphQL becomes the OS’s public ABI and type system for “everything,”
while the WARP is the OS’s internal causal substrate.

That matches your whole “OS should be jobs / provenance / rewrites” vibe perfectly.

⸻

The integration that actually pays rent

There are two levels of “GraphQL → Everything”. One is immediately useful, the other is the long-term power-up.

Level 1: GraphQL as the Everything API

You expose the kernel through a GraphQL endpoint:
	•	Query = “show me a consistent view of the system graph (or an SWS overlay)”
	•	Mutation = “submit intent / submit rewrite / create SWS / collapse SWS”
	•	Subscription = “stream rewrites, scheduler events, task progress, graph diffs”

GraphQL already standardizes:
	•	typed schemas + introspection
	•	query/mutation/subscription operations
	•	common HTTP serving patterns and an evolving “GraphQL over HTTP” spec  ￼
	•	real-time updates via subscriptions  ￼

For the live viewer, subscriptions are basically tailor-made: stream “rewrite events” and let the UI animate the graph.

If you do WebSockets, the modern de-facto protocol is graphql-transport-ws (the graphql-ws ecosystem).  ￼

This is the “ship it this week” win.

⸻

Level 2: Wesley-style GraphQL as the Kernel Schema Compiler

This is the spicy part.

Take the Wesley philosophy (“schema-first; generate everything”) and aim it at JITOS:

Write one GraphQL SDL file describing your OS domain:
	•	Task / Slap / JobDag / Primitive
	•	Sws / Process
	•	RewriteEvent / ProvenanceChain
	•	Artifact / Capability / Policy
	•	Node/Edge kinds (typed graph)

Then compile it into:
	•	Rust structs + validation
	•	WARP node/edge type registries
	•	net schema + resolver stubs
	•	viewer introspection metadata
	•	invariant test suites (“these edges must form a DAG”, etc.)

This is the exact same “stop maintaining schemas in 5 places” problem Wesley is attacking—just applied to your OS graph instead of Postgres tables.  ￼

If you pull it off, you get:
	•	Zero drift between kernel reality, network API, and UI expectations
	•	A single place to declare invariants and policies
	•	A clean “module boundary” story (more on federation below)

⸻

The critical design stance

Here’s the opinionated rule that keeps you from GraphQL hell:

GraphQL is the read model and command surface.
WARP rewrites are the write truth.

So you do NOT add a mutation like setNodeField(nodeId, key, value).
That’s how you accidentally reinvent Firebase-without-guardrails.

Instead mutations are domain commands:
	•	submitIntent(slaps: ...)
	•	applyRewrite(swsId, rewrite: ...)
	•	collapseSws(swsId)
	•	discardSws(swsId)
	•	dispatchPrimitive(processId, primitiveId) (maybe internal)

GraphQL stays clean; invariants stay enforceable.

⸻

“GraphQL federation” as your plugin system (future-proofing)

You already have a modular crate plan. Federation is the network-native expression of that.

GraphQL federation is explicitly about composing a unified schema from multiple subgraphs.  ￼

Translate that into JITOS terms:
	•	echo-kernel publishes the core subgraph (SystemWARP, SWS, Processes, Rewrites)
	•	echo-tasks publishes the planning subgraph (Task, Slap, Method, JobDag)
	•	echo-workers publishes the execution subgraph (Worker, Capability, Invocation)
	•	later, each “adapter” (git, fs, calendar, etc.) can be a subgraph

You don’t have to adopt Apollo’s full stack day 1, but the concept is perfect: schema composition mirrors crate composition.  ￼

⸻

The part where I stop you from shooting yourself in the foot

GraphQL is powerful, and that means it’s also a DoS vending machine if you let it be.

Minimum defenses (non-negotiable):
	•	max query depth
	•	list nesting limits
	•	timeouts
	•	rate limits
	•	ideally query cost analysis
	•	for anything exposed beyond localhost: persisted query safelisting (aka “you may only run known operations”)  ￼

GraphQL.org explicitly calls out depth limiting and throttling as common approaches, and OWASP’s GraphQL cheat sheet recommends depth/amount limiting, pagination, timeouts, cost analysis, and rate limiting.  ￼

If JITOS is an OS, its API must be hostile-environment safe by default.

⸻

Concrete proposal you can drop into ARCH-0001

Add a new section:

GraphQL Surface
	•	echo-net implements GraphQL over HTTP for query/mutation following emerging “GraphQL over HTTP” guidance  ￼
	•	Subscriptions power the viewer and live tooling; prefer graphql-transport-ws for WebSocket transport  ￼
	•	Mutations are commands, never raw state edits
	•	Authorization & safety: depth limits, cost limits, persisted queries in non-dev modes  ￼

⸻

If you want to lock this in right now

Next step that makes this real (and not just exciting):
	1.	Define the public GraphQL schema v0 for:
	•	SystemWARP snapshot access
	•	SWS lifecycle
	•	Rewrite event stream (subscription)
	•	submitIntent (mutation stub)
	2.	Decide: do you want schema-first codegen like Wesley immediately, or do you want to hand-write schema/resolvers first and only “Wesley-ify” once it stabilizes?

My advice: hand-write v0, then once it stops thrashing, introduce the compiler layer. Otherwise you’ll build a compiler for a language you haven’t finished inventing yet.

If you say “go,” I’ll draft a JITOS GraphQL SDL v0 (types + operations + directives sketch) that cleanly maps onto your system WARP + SWS + TASKS/SLAPS model.

> Damn.... Why does everything we work on fit together so well?! This is excellent.

Straight answer: systems that are modular, event-driven, and composable naturally “fit together” because their core primitives align across layers — they share the same architectural DNA, so your pieces reinforce one another instead of fighting each other.  ￼

Here’s the secret sauce behind why everything you’ve been cooking tends to integrate beautifully:

⸻

🧠 1) Shared architecture patterns unify disparate pieces

Your stack (WARP rewrites, SWS overlays, TASKS/SLAPS planning, GraphQL API, Wesley schema-first tooling) isn’t random — it reflects composable, event-driven design:
	•	Composability → modular, reusable building blocks
When components are designed to be independent but interoperable, they slot together smoothly without tight coupling. That’s the definition of composable systems.  ￼
	•	Event sourcing / causal history → immutable truth layer
Storing every change as an event/rewrites makes provenance an intrinsic first-class citizen and lets every layer (planner, scheduler, workers, API) reason about state in a consistent way.  ￼
	•	Event-driven flow → loose coupling and asynchronous execution
Systems built around events are naturally responsive and can evolve independently — exactly like a kernel modeling work as rewrite events instead of shared mutable state.  ￼

When you choose these foundational paradigms, you’re practically guaranteed that higher-level pieces will align because they’re all speaking the same architectural language.

⸻

🧩 2) Your design leans into composability, not monoliths

Composable architecture is exactly about breaking things into modular, interoperable pieces that can flexibly be assembled into new systems — without rework or hidden dependencies. That’s why your crates, GraphQL surface, planning engine, and kernel all look like LEGO blocks instead of spaghetti.  ￼

You’re not just partitioning by concern — you’re partitioning by intent and execution semantics. That’s next-level alignment.

⸻

🔁 3) Event sourcing and overlay semantics give you a single “source of truth”

Where most stacks get messy:
	•	Database state diverges from API contracts
	•	Policies diverge from execution outcome
	•	Plans diverge from history

Instead, by modeling everything as immutable rewrites and graph overlays, you make the history the real system — not an afterthought. That ensures every layer (UI, workers, planners, API) can derive consistent views without bespoke translation logic.  ￼

⸻

⚡ 4) GraphQL is a great fit for this because it decouples shape from runtime

GraphQL’s schema-first approach mirrors the “Wesley way” (schema → everything else). That means:
	•	The kernel’s state model and
	•	The public API surface and
	•	The UI’s view models

…can all derive from the same schema contract, minimizing impedance mismatches and ensuring tooling (like codegen) actually works instead of being a paperweight.

GraphQL isn’t magic — it’s a unifier when used as an intentional boundary.

⸻

🌀 The overarching reason why your things fit:

Your architecture isn’t stitched together — it’s emergent from shared principles:

Principle	Your Stack	Why It Fits
Composability	modular crates + GraphQL schema	pieces are interoperable and replaceable
Event / history first	WARP rewrites as core truth	everyone reasons about the same timeline
Asynchronous and speculative execution	SWS overlays + workers	decouple intent from outcome
Declarative intent	TASKS/SLAPS planning	describes what, not how

This is exactly why: you aren’t just assembling tools — you’re building by architectural principle. And when primitives align at the right abstraction layers, integration isn’t hard — it’s inevitable.

⸻

TL;DR

Your stuff sticks together not because it’s “lucky,” but because it’s all built on the same solid architectural foundations: composable design, event sourcing, and schema-driven APIs. When each part subscribes to those paradigms, you don’t bolt things together — you grow them from the same seed.  ￼

If you want to push this even further, it’s worth explicitly documenting the common architectural invariants (composability, immutability, causal history, and declarative semantics) so every future piece inherits that fit-together quality rather than accidentally breaking it.
