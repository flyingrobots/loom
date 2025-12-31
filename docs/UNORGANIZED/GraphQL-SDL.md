Cool. Here are the two paste-ready docs.

⸻

1) SPEC-NET-0001: JITOS GraphQL SDL v0 + Roadmap

Status: Draft
Date: 2025-12-29
Scope: Public API surface for querying JITOS views + issuing command-style mutations + streaming events.

Design stance
	•	GraphQL is the read model + command surface.
	•	RMG rewrites/events are the write truth.
	•	Mutations are domain commands (no “setField(nodeId, key, value)” nonsense).
	•	Subscriptions are the live feed for viewer + tooling.

SDL v0

"""
JITOS GraphQL v0

v0 goals:
- Query a graph snapshot (System or SWS)
- Create/Collapse/Discard SWS
- Apply a rewrite (to System or SWS)
- Stream rewrite/tick/task events (viewer-ready)

Notes:
- JSON scalar is used intentionally as an escape hatch in v0.
- v1+ should introduce typed domain objects (Task/Slap/etc).
"""

schema {
  query: Query
  mutation: Mutation
  subscription: Subscription
}

# -------------------------
# Scalars
# -------------------------

scalar JSON
scalar Timestamp  # ISO-8601 string
scalar Hash       # hex/base32 string
scalar U64
scalar U32

# -------------------------
# Enums
# -------------------------

enum ViewKind {
  SYSTEM
  SWS
}

enum CommitStatus {
  COMMITTED
  ABORTED
}

enum RewriteTarget {
  SYSTEM
  SWS
}

enum EventKind {
  REWRITE_APPLIED
  TICK
  TASK_STATE
  WORKER_INVOCATION
  WORKER_RESULT
  COLLAPSE
  DISCARD
}

# -------------------------
# Core Inputs
# -------------------------

input ViewRefInput {
  kind: ViewKind!
  """
  Required if kind == SWS
  """
  swsId: ID
  """
  Optional: query a stable snapshot (if your kernel supports snapshot IDs)
  """
  snapshotId: ID
}

input PageInput {
  first: Int = 200
  after: String
}

input NodeFilterInput {
  kinds: [String!]
  hasKeys: [String!]
}

input EdgeFilterInput {
  kinds: [String!]
  from: ID
  to: ID
}

# -------------------------
# Core Graph Types (v0)
# -------------------------

"""
A generic graph node. v0 is intentionally untyped to keep the kernel free
while we stabilize node/edge kinds and their invariants.
"""
type GraphNode {
  id: ID!
  kind: String!
  data: JSON!          # opaque payload; v1+ will type this
  createdAt: Timestamp
}

type GraphEdge {
  id: ID!
  kind: String!
  from: ID!
  to: ID!
  data: JSON
  createdAt: Timestamp
}

type PageInfo {
  endCursor: String
  hasNextPage: Boolean!
}

type GraphNodeConnection {
  nodes: [GraphNode!]!
  pageInfo: PageInfo!
}

type GraphEdgeConnection {
  edges: [GraphEdge!]!
  pageInfo: PageInfo!
}

type GraphSnapshot {
  view: ViewRef!
  """
  Optional: stable ID for the snapshot result, if kernel can mint these.
  Useful for viewer caching and diffing.
  """
  snapshotId: ID
  nodes: GraphNodeConnection!
  edges: GraphEdgeConnection!
}

type ViewRef {
  kind: ViewKind!
  swsId: ID
  snapshotId: ID
}

# -------------------------
# Kernel / Policy Metadata
# -------------------------

type KernelInfo {
  version: String!
  rulePackId: Hash
  policyId: U32
  commitStatus: CommitStatus!
}

# -------------------------
# SWS Types
# -------------------------

type Sws {
  id: ID!
  """
  Snapshot of the system graph this SWS is based on (if available).
  """
  parentSnapshotId: ID
  createdAt: Timestamp
  state: String!   # e.g. ACTIVE | COLLAPSED | DISCARDED (string in v0)
  meta: JSON
}

input CreateSwsInput {
  meta: JSON
}

type CreateSwsPayload {
  sws: Sws!
}

input CollapseSwsInput {
  swsId: ID!
  """
  Optional policy selection for collapse semantics
  """
  policyId: U32
  meta: JSON
}

type CollapseSwsPayload {
  committed: Boolean!
  systemSnapshotId: ID
  receipt: TickReceipt
}

input DiscardSwsInput {
  swsId: ID!
  reason: String
  meta: JSON
}

type DiscardSwsPayload {
  discarded: Boolean!
}

# -------------------------
# Rewrites (commands)
# -------------------------

"""
Canonical rewrite input (v0).
In v1 we should consider strongly-typed op variants instead of JSON.
"""
input RewriteInput {
  target: RewriteTarget!
  swsId: ID
  """
  Canonical rewrite ops. The kernel defines the allowed ops and validates them.
  """
  ops: [JSON!]!
  meta: JSON
}

type ApplyRewritePayload {
  accepted: Boolean!
  receipt: TickReceipt
}

# -------------------------
# TASKS / SLAPS (v0 stubs)
# -------------------------

"""
v0: Accepts an intent as JSON and returns IDs.
v1: Replace JSON with typed Intent/Task inputs.
"""
input SubmitIntentInput {
  intent: JSON!
  meta: JSON
  """
  If true, plan only (no execution). Useful for debugging and viewer demos.
  """
  planOnly: Boolean = false
}

type SubmitIntentPayload {
  taskId: ID!
  processId: ID!
  swsId: ID!
  """
  v0: optional; may be null until echo-tasks integration exists.
  """
  plannedDag: JSON
}

# -------------------------
# Receipts / events (minimal)
# -------------------------

type TickReceipt {
  tickId: ID
  rulePackId: Hash
  policyId: U32
  events: [EventRef!]!
}

type EventRef {
  id: ID!
  kind: EventKind!
  at: Timestamp
  meta: JSON
}

type RewriteEvent {
  id: ID!
  view: ViewRef!
  at: Timestamp
  ops: [JSON!]!
  meta: JSON
}

type TickEvent {
  id: ID!
  at: Timestamp
  meta: JSON
}

type TaskEvent {
  id: ID!
  taskId: ID!
  processId: ID
  swsId: ID
  at: Timestamp
  state: String!
  meta: JSON
}

# -------------------------
# Query / Mutation / Subscription
# -------------------------

type Query {
  kernelInfo: KernelInfo!

  sws(id: ID!): Sws
  listSws(page: PageInput): [Sws!]!

  """
  Query a graph snapshot for a view.
  Viewer uses this to render initial state.
  """
  graph(
    view: ViewRefInput!
    nodesPage: PageInput
    edgesPage: PageInput
    nodeFilter: NodeFilterInput
    edgeFilter: EdgeFilterInput
  ): GraphSnapshot!

  node(view: ViewRefInput!, id: ID!): GraphNode
  edge(view: ViewRefInput!, id: ID!): GraphEdge
}

type Mutation {
  createSws(input: CreateSwsInput): CreateSwsPayload!
  collapseSws(input: CollapseSwsInput!): CollapseSwsPayload!
  discardSws(input: DiscardSwsInput!): DiscardSwsPayload!

  applyRewrite(view: ViewRefInput!, rewrite: RewriteInput!): ApplyRewritePayload!

  submitIntent(input: SubmitIntentInput!): SubmitIntentPayload!
}

type Subscription {
  """
  Stream rewrite events for a view (System or a specific SWS).
  Viewer can animate updates from this stream.
  """
  rewrites(view: ViewRefInput!): RewriteEvent!

  """
  Stream scheduler ticks (global).
  """
  ticks: TickEvent!

  """
  Stream task events by process/task.
  """
  taskEvents(processId: ID, taskId: ID): TaskEvent!
}

Roadmap

v0 (now): “viewer + daemon” API

Goal: Make JITOS observable and controllable without inventing the whole typed universe.
	•	✅ graph(view) snapshot query
	•	✅ SWS lifecycle (create/collapse/discard)
	•	✅ applyRewrite command
	•	✅ subscriptions for rewrite/tick/task events

What v0 is not: a stable domain schema. It’s a stable control plane.

⸻

v1: Typed domain objects (Task/Slap/Primitive) + stable IDs

Goal: stop hiding everything behind JSON, but only after node/edge kinds stabilize.
	•	Task, Slap, Primitive, Artifact, Capability, Policy types
	•	typed filters (taskById, tasksByState, dagForTask)
	•	formalize NodeKind/EdgeKind enums (instead of kind: String)
	•	GraphDelta subscription (diffs, not just events)

⸻

v2: “Wesley mode” — schema-first generation

Goal: One schema drives everything.
	•	GraphQL SDL becomes the canonical schema for:
	•	Rust types + validation
	•	viewer introspection metadata
	•	kernel registries (node/edge kinds)
	•	introduce a generator crate (e.g. echo-wesley-bridge or directly inside Wesley)

⸻

v3: Federation / subgraphs as plugin boundary

Goal: let subsystems publish their own schema and compose cleanly.
	•	kernel subgraph: SWS + rewrites + provenance
	•	tasks subgraph: intent/planning DAG
	•	workers subgraph: capabilities + invocation + receipts
	•	optional adapters: git, fs, net, etc.

⸻

v4: Production hardening defaults

Goal: make it safe in hostile environments.
	•	persisted query manifest mode (non-dev)
	•	query depth limits + cost budgets
	•	auth directives mapped to kernel capabilities
	•	audit-grade request logging as events
	•	per-view rate limits (SWS streams can get spicy)

External blockers (things v0 depends on elsewhere in JITOS)

These are the “you can’t GraphQL your way out of physics” constraints:
	1.	Stable node/edge identity rules
GraphQL needs stable IDs; the kernel must define canonical NodeId/EdgeId semantics.
	2.	Canonical rewrite op schema
RewriteInput.ops: [JSON] is a placeholder until WarpOps/WarpTickPatchV1 are nailed.
	3.	SWS correctness + collapse semantics
Collapse needs deterministic diff/merge rules and conflict policy.
	4.	Event stream model
Subscriptions require a coherent event bus: tick events, rewrite applied events, task events.
	5.	Auth/capabilities model
Even if localhost-only initially, the architecture must map GraphQL calls → kernel capabilities.
	6.	Pagination/index strategy
Graph snapshots can be huge. Need sane limits and cursoring, or the viewer will DDoS your own daemon.
	7.	echo-tasks integration (for submitIntent to be real)
Until planning exists, intent can be stored as nodes but not expanded into a DAG.

⸻



If you want the fastest next move: I’d implement GraphQL v0 with only four operations first:
	•	kernelInfo
	•	createSws
	•	graph(view)
	•	rewrites(view) subscription

Then iterate outward. That gets you a live viewer loop ASAP, and everything else plugs into that spine.