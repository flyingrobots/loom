# Chapter 4 — Double-Pushout Physics (DPO): The Rule of Rules

There’s a moment in every engineer’s career when you stop asking:

“What is the system?”
and start asking:

“How does the system change?”

In debugging, in compilers, in distributed systems, in game engines, in databases, in AI systems —
you don’t care about what is.

You care about what happened,
what is happening,
and what will happen if you touch this thing.

And sooner or later you run into a deeper question:

“What does it even mean to change a system?”

This is not a philosophical question.
This is a practical one.

When you mutate state, when you apply a function, when you compile code, when you propagate an event, when you update a scene graph —
you’re rewriting structure.

But rewriting complex, recursive structure turns out to be… hard.

Hard enough that most people never formalize it.

Hard enough that systems break because of it.

Hard enough that entire industries fall over because nobody asked what a “change” really is.

So this chapter introduces a tool with a reputation:

Double-Pushout Rewriting (DPO)

or in the language of this book:

the physics of RMGs.

The rule of rules.

---

## 4.1 RMGs Come to Life Only When You Apply Rules

RMGs give us:

- nested structure
- wormholes
- recursive universes
- compositional worlds

But structure is inert.

A graph without rules is a map of a universe that does nothing.

To compute, you need:

- transitions
- transformations
- laws
- behavior
- semantics

That’s where DPO comes in.

If RMG is the space,
DPO is the physics.

---

## 4.2 The Wormhole Needs a Contract

From [Chapter 3](005-chapter-003.md) you learned:

Edges are wormholes — structured tunnels with internal geometry.

But wormholes can’t just rewrite anything.

They need interfaces.

They need constraints.

They need a contract defining:

- what they accept (input structure)
- what they preserve (invariant structure)
- what they output (new structure)

And THIS is the precise conceptual role of the DPO rule’s famous triplet:

```math
L  —  K  —  R
(Left-hand side, Interface, Right-hand side)
```

Let’s break it down in human terms.

---

$L$ — The Entrance to the Wormhole

The pattern that must be present.
The shape the wormhole expects to “match.”

If the graph doesn’t contain $L$,
the wormhole won’t open.

---

$K$ — The Interface (The Mouth of the Wormhole)

The structure that must remain identical on both sides.
The part preserved across the rewrite.

Think of $K$ as:

- the shared boundary
- the stable part
- the invariant
- the “shape” of the wormhole’s throat
- the identity that survives the transformation

If $K$ doesn’t match, the rewrite is illegal.

---

$R$ — The Exit of the Wormhole

The new structure that emerges.

This replaces $L\K$ while preserving $K$.

This is the “after” picture.

---

## 4.3 “Typed Wormholes” — the Intuition That Makes DPO Obvious

This is the cleanest way to think about DPO:

**A DPO rule is a typed wormhole.

L defines what the wormhole accepts.
K defines what must survive.
R defines what emerges.**

If the RMG at runtime matches $L$,
and the boundary matches $K$,
the wormhole fires,
and $R$ is installed.

If not?
The rule is illegal.

This matches our engineering reality:

- a compiler expects valid AST
- an API expects a valid payload
- a serialization step expects valid structure
- a database transaction expects valid schemas
- an optimizer expects legal IR

In every case, invalid input = no transition.

Wormholes have types.

---

## 4.4 DPO’s “Dangling Condition,” Explained Without Pain

DPO requires:

- no dangling edges
- no illegal merges
- no broken boundaries

In engineer language:

The wormhole cannot rip a hole in the universe.

Everything it deletes must be entirely inside $L$.
Everything it preserves must match $K$.
Everything it outputs must respect $R$.

Replace “universe” with “RMG,”
and you get the idea.

DPO rules are safe not because they’re clever,
but because they follow the simplest possible invariant:

Rewrite only what you matched.
Preserve what you promised.

Everything else is implementation detail.

---

## 4.5 Example: A Compiler Pass as a DPO Rule

Let’s revisit our wormhole from [Chapter 3](005-chapter-003.md):

```text
 [Source Code]
       |
       |  (Compiler Wormhole)
       v
 [Machine Code]
```

Inside that wormhole:

- $L$ is the AST pattern to match
- $K$ is the parts of the program that remain intact
- $R$ is the optimized IR or generated code

This explains why:

- invalid syntax kills the compile
- partial ASTs don’t rewrite
- optimizations must preserve meaning
- symbol table entries survive
- IR nodes mutate

The compiler is a DPO rewrite engine in a fancy hat.

---

> 📦 FOR THE SKEPTICAL ENGINEER™
> 
> “Bro Just Discovered Function Calls.”
> 
> Let’s get this objection out of the way.
> 
> You might be thinking:
> 
> “Isn’t this just a function? L → R?”
> 
> Sort of.
> But also absolutely not.
> 
> Function calls:
> - single input
> - single output
> - no internal rewrite
> - no structured edges
> - no nested universes
> - no multi-graphs
> - no rule legality
> - no K-interface
> - no pattern matching
> - no transformation of the function itself
> 
> RMG + DPO edges:
> - accept complex subgraphs
> - contain entire universes of computation
> - may include closures
> - can have environments
> - can have concurrency inside
> - can be rewritten themselves
> - use L/K/R typing
> - enforce safety (dangling condition)
> - support multi-scale recursion
> - are part of a geometric space of possible rewrites
> 
> A function call is a wormhole.
> An RMG edge is a civilization in a tunnel.
> 
> We will revisit this fully in the CΩDEX.
> 
> (End sidebar.)

---

## 4.6 DPO Enables Computation to Be Composable

Here’s the real power:

DPO allows you to:

- build small rewrite rules
- combine them
- compose them
- apply them across recursive structure
- reuse them
- nest them
- evolve systems in modular steps

If RMG gives us “space,”
DPO gives us “law.”

Together they give us:

- semantics
- behavior
- evolution
- flow
- causality

This is how we start to build worldlines.

---

## 4.7 DPO Is the Bridge to Geometry

This is the bridge to Part II.

With RMG + DPO, we can finally define:

- alternative computational universes
- transitions between them
- minimal rewrite sequences
- adjacency in possibility space
- and eventually…
- Rulial Distance.

This is the conceptual door into geometry.

DPO tells us:

A universe can change legally
if and only if the wormholes match.

But which changes are “close”?
Which universes are “neighbors”?
Which paths are “straight”?
Which ones “curve”?

That takes us to:

---

👉 Chapter 5 — Rulial Space and Rulial Distance

Where we give computation its geometry —
not literal geometry,
but a metaphorical surface for reasoning about universes of computation.

---

# **CΩMPUTER • JITOS** 
© 2025 James Ross • [Flying • Robots](https://flyingrobots.dev)
All Rights Reserved

