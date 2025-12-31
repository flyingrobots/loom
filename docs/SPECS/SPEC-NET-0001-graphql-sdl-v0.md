# SPEC-NET-0001: JITOS GraphQL SDL v0 + Roadmap

**Status:** Draft
**Date:** 2025-12-29
**Scope:** Public API surface for querying JITOS views + issuing command-style mutations + streaming events.

## Design stance
*   GraphQL is the read model + command surface.
*   WARP rewrites/events are the write truth.
*   Mutations are domain commands (no “setField(nodeId, key, value)” nonsense).
*   Subscriptions are the live feed for viewer + tooling.

---

## Milestone 1 Subset (Kernel Genesis / Alpha Contract)

Milestone 1 is a strict, deterministic subset of this SDL. This section is normative for Milestone 1 implementation: it exists to prevent “interpretation drift” between spec, daemon, and clients.

### Frozen contract choices (M1)

- **Hash encoding:** `Hash` strings are lowercase hex, length 64, representing 32-byte BLAKE3 digests. No `0x` prefix.
- **Routing:** `applyRewrite` routes **only** via the `view: ViewRefInput!` argument. `RewriteInput` contains ops only; there is no secondary routing source of truth.
- **Digest surface:** `graph(view)` returns `GraphSnapshot.digest: Hash!` as the canonical digest for external determinism validation.
- **Timestamps:** all `Timestamp` fields return `null` in Milestone 1.
- **Pagination:** `PageInput.first` is supported; `PageInput.after` returns `NOT_IMPLEMENTED` in Milestone 1.
- **Rewrite ops (M1 supports exactly one op):** `RewriteInput.ops` contains JSON objects conforming to the “AddNode” schema below.
- **Receipts (M1):** `applyRewrite` returns a deterministic `ReceiptV0` with:
  - `rewriteIdx: U64` (global monotone sequence since boot)
  - `view: ViewRef`
  - `viewDigest: Hash` (digest after applying the rewrite)
- **Errors:** GraphQL errors include `extensions.code` with one of:
  - `INVALID_INPUT` (bad ID format, schema mismatch, bad base64, missing required fields)
  - `NOT_FOUND` (SWS id doesn’t exist)
  - `NOT_IMPLEMENTED` (unsupported op variant, `after` cursor, `collapseSws`, `submitIntent`, etc.)
  - `INTERNAL` (kernel loop down, invariant violated, unexpected errors)

### AddNode op schema (M1)

Milestone 1 requires a strict JSON shape (no unknown fields):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jitos://schemas/rewrite-op/AddNode.v0.json",
  "type": "object",
  "required": ["op", "data"],
  "additionalProperties": false,
  "properties": {
    "op": { "const": "AddNode" },
    "data": {
      "type": "object",
      "required": ["kind", "payload_b64"],
      "additionalProperties": false,
      "properties": {
        "kind": {
          "type": "string",
          "minLength": 1,
          "maxLength": 128,
          "pattern": "^[A-Za-z0-9_.:-]+$"
        },
        "payload_b64": {
          "type": "string",
          "minLength": 0,
          "contentEncoding": "base64"
        }
      }
    }
  }
}
```

Notes:
- `payload_b64` decodes to bytes; bytes are hashed as-is (M1 kernel does not canonicalize structured data).
- unsupported op variants → `NOT_IMPLEMENTED` (not `INVALID_INPUT`).
- malformed JSON / schema mismatch / bad base64 → `INVALID_INPUT`.

### Milestone 1 implementation notes (Codex-proof)

This section defines deterministic error mapping so implementations can’t “interpret” their way into divergent clients/tests.

General rules:
- Prefer **rejecting** ambiguous inputs over silently ignoring them.
- When a capability is deliberately deferred (cursoring, snapshots, multi-op batching), use `NOT_IMPLEMENTED`.
- When input violates a frozen schema/format, use `INVALID_INPUT`.
- Always include `extensions.code` on GraphQL errors.

#### View routing (`ViewRefInput`) error mapping

- `view.kind == SYSTEM` and `view.swsId` is provided → `INVALID_INPUT`
- `view.kind == SWS` and `view.swsId` is missing → `INVALID_INPUT`
- `view.snapshotId` is provided (Milestone 1 has no snapshots) → `NOT_IMPLEMENTED`
- `view.kind == SWS` and referenced SWS does not exist → `NOT_FOUND`

#### Pagination (`PageInput`) error mapping (Milestone 1)

- `page.after` is provided → `NOT_IMPLEMENTED`
- `page.first` missing → use default
- `page.first <= 0` → `INVALID_INPUT`

#### `applyRewrite(view, rewrite)` error mapping (Milestone 1)

Routing:
- view errors follow the rules above.

`rewrite.ops` shape:
- `rewrite.ops` is empty → `INVALID_INPUT`
- `rewrite.ops` has more than 1 element → `NOT_IMPLEMENTED` (batching deferred)

For the single op in `rewrite.ops[0]`:
- JSON is not an object, or missing `op`/`data` → `INVALID_INPUT`
- unknown fields at any level (violates `additionalProperties: false`) → `INVALID_INPUT`
- `"op"` is not `"AddNode"` → `NOT_IMPLEMENTED`
- `"data.kind"` violates pattern/length → `INVALID_INPUT`
- `"data.payload_b64"` is not valid base64 → `INVALID_INPUT`

Side effects:
- Any wall-clock timestamps in responses must be `null` (Milestone 1).

#### Other stubbed operations (Milestone 1)

- `collapseSws(...)` → `NOT_IMPLEMENTED`
- `submitIntent(...)` → `NOT_IMPLEMENTED`
- `ticks` / `taskEvents` subscriptions → `NOT_IMPLEMENTED`

---

## SDL v0

```graphql
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
scalar Hash       # lowercase hex string, length 64, 32-byte BLAKE3 digest
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
  Deterministic digest of (nodes, edges) for this snapshot.
  This is the canonical digest surface for Milestone 1 determinism validation.
  """
  digest: Hash!
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
  """
  Canonical rewrite ops. The kernel defines the allowed ops and validates them.
  """
  ops: [JSON!]!
  meta: JSON
}

type ApplyRewritePayload {
  accepted: Boolean!
  receipt: ReceiptV0!
  """
  v0+ future: tick receipts once scheduler/ticks exist.
  Milestone 1 returns null.
  """
  tickReceipt: TickReceipt
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

type ReceiptV0 {
  rewriteIdx: U64!
  view: ViewRef!
  viewDigest: Hash!
}

type EventRef {
  id: ID!
  kind: EventKind!
  at: Timestamp
  meta: JSON
}

type RewriteEvent {
  id: ID!
  idx: U64!
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

  """
  Append-only rewrite log for a view since boot (Milestone 1: in-memory).
  Deterministic ordering: ascending by idx.
  """
  rewrites(view: ViewRefInput!, page: PageInput): [RewriteEvent!]!
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
```

---

## Roadmap

### v0 (now): “viewer + daemon” API

**Goal:** Make JITOS observable and controllable without inventing the whole typed universe.
*   ✅ `graph(view)` snapshot query
*   ✅ SWS lifecycle (`create`/`collapse`/`discard`)
*   ✅ `applyRewrite` command
*   ✅ subscriptions for rewrite/tick/task events

**What v0 is not:** a stable domain schema. It’s a stable control plane.

### v1: Typed domain objects (Task/Slap/Primitive) + stable IDs

**Goal:** stop hiding everything behind JSON, but only after node/edge kinds stabilize.
*   Task, Slap, Primitive, Artifact, Capability, Policy types
*   typed filters (`taskById`, `tasksByState`, `dagForTask`)
*   formalize `NodeKind`/`EdgeKind` enums (instead of `kind: String`)
*   `GraphDelta` subscription (diffs, not just events)

### v2: “Wesley mode” — schema-first generation

**Goal:** One schema drives everything.
*   GraphQL SDL becomes the canonical schema for:
    *   Rust types + validation
    *   viewer introspection metadata
    *   kernel registries (node/edge kinds)
*   introduce a generator crate (e.g. `echo-wesley-bridge` or directly inside Wesley)

### v3: Federation / subgraphs as plugin boundary

**Goal:** let subsystems publish their own schema and compose cleanly.
*   `kernel` subgraph: SWS + rewrites + provenance
*   `tasks` subgraph: intent/planning DAG
*   `workers` subgraph: capabilities + invocation + receipts
*   optional adapters: git, fs, net, etc.

### v4: Production hardening defaults

**Goal:** make it safe in hostile environments.
*   persisted query manifest mode (non-dev)
*   query depth limits + cost budgets
*   auth directives mapped to kernel capabilities
*   audit-grade request logging as events
*   per-view rate limits (SWS streams can get spicy)

---

## External blockers (things v0 depends on elsewhere in JITOS)

These are the “you can’t GraphQL your way out of physics” constraints:

1.  **Stable node/edge identity rules:** GraphQL needs stable IDs; the kernel must define canonical NodeId/EdgeId semantics.
2.  **Canonical rewrite op schema:** `RewriteInput.ops: [JSON]` is a placeholder until `WarpOps`/`WarpTickPatchV1` are nailed.
3.  **SWS correctness + collapse semantics:** Collapse needs deterministic diff/merge rules and conflict policy.
4.  **Event stream model:** Subscriptions require a coherent event bus: tick events, rewrite applied events, task events.
5.  **Auth/capabilities model:** Even if localhost-only initially, the architecture must map GraphQL calls → kernel capabilities.
6.  **Pagination/index strategy:** Graph snapshots can be huge. Need sane limits and cursoring, or the viewer will DDoS your own daemon.
7.  **echo-tasks integration (for submitIntent to be real):** Until planning exists, intent can be stored as nodes but not expanded into a DAG.
