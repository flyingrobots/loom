# JITOS is now Loom

Just to set the record straight: "JITOS" was never an acronym. In fact, it didn't mean anything. It was always supposed to just be a placeholder for something else, especially because everybody who sees "JITOS" thinks "JIT? OK, must mean: *Just-In-Time* OS". I was waiting for the right name to come to me, and last night, it did.

## Why "JITOS"?

This project has been really interesting to work on. It wasn't carefully planned, but rather, bits and pieces of it have just sort of... emerged, when the time was ripe.

It all started one evening as I realized that *every project* I had been working on over the past year was actually an aspect of something larger: an operating system (hence, the "OS").

OK, but what about the "JIT" part?

Well, at the time, I was on this "make Git do things Git isn't supposed to do" kick. Let's turn Git into a distributed key value store. Let's turn git into a semantic graph database. Let's turn Git into a deployment ledger.

Then I had an absurd idea: *Linus Torvalds created both Git and Linux. What if we made Git into a Linux distro?*

The problem with that idea was Git doesn't really scale. It doesn't handle having lots of refs very well. It's just not designed with that use case in mind (trade-offs). So, what to do?

I had another absurd idea: what if we could still use Git's CLI, but replace its implementation with something that would scale the way I needed it to? That way, I could still say "See? It's Git! (-like)" Thus, "G.I.T." was born: "Git Inversion Tech".

To make sure "G.I.T." wouldn't be confused with "Git", I made sure to canonicalize an important fact: "G.I.T." was to be pronounced "JIT", kind of like how "gif" is actually pronounced "jif" (so I'm told).

Of course, spelling JIT as "G.I.T." only lasted for about half a day. Before long, it morphed into "JIT".

And then the "Holy shit, I'm building an Operating System, aren't I?" realization dawned on me. That's where "JITOS" came from:

```math
"JIT" + "OS" = "JITOS"
```

And, of course. Pretty much IMMEDIATELY someone said "JITOS, Eh? Obviously, that means: '*Just-In-Time OS*'".  Every time I heard "Just-In-Time OS", I cringed, but I didn't want to rush giving this project its final name. I knew the right name would be revealed "*just-in-time.*"

## Loom?

First, let's get this out of the way: Yes, I know there's already an app called "Loom". I am aware of the screen-casting tool. But guess what? That Loom was absorbed into Atlassian, which means that long-term, that "Loom" likely gets rebranded. I don't know why they chose the name "Loom" for a screen-casting tool, but I do know why "Loom" is the perfect name for this project, so let me explain:

Over the last few weeks, the system's vocabulary converged on something fundamental. As the architecture crystalized, the old name just stopped working and I realized the name that describes what the system actually *is*.

## Learning to see what was always there

In the AIΩN Foundations Series, WARP graphs are used to describe a new computational model, based on WARP graph rewrites. The way this works is simple: when a graph rewrite rule is proposed or otherwise activates, the *scheduler* says "Alright, let's pop these off". This is what we call "processing a *tick*". Rules that don't "overlap" commute, but when there's a conflict, the scheduler deterministically selects which rule pops off, and which rules are banished to the shadow realm.

To enrich discourse regarding this process, I've decided to introduce new terminology. So, what's going on here, exactly? Well, every moment, the computer does what it do: some computation, which produces a record to append to the immutable provenance ledger. This process in slow-motion transforms the Bundle of potential graph rewrites into the worldline. How? The system orders the rules and applies them, flipping the tick. As a result, the system records 'what' happened, as well as 'why', and 'how'. But it *also* records 'what else could have happened, but wasn't selected by the scheduler'. Those "leftover" possibilities that didn't execute are stored in the *Umbra Index*. So, the computer is a machine that orders threads to weave the fabric of reality. That machine is a "loom".

Here's something to think about: our minds are also looms. Physics is also a loom. Computation, physics, and consciousness are all looms. And because they are looms, they experience time's arrow.

Classical computing does all of this too, but it only exposes the finalized execution trace. Everything else—the alternatives, the constraints, the ordering decisions, the near-misses—has always existed, but since we didn't have a name for these concepts, this machinery remained implicit, discarded, or inaccessible.

Loom makes that structure explicit:

At its core, computation is not *just* transformation of values. It is more accurate to say that computation is the **construction of history under constraint**.

Computers are looms. Don't believe me yet? Consider that a loom is the oldest human tool that is designed specifically to process:

- many independent threads
- many valid interleavings
- constraints that rule most of them out
- one irreversible fabric that remains

Loom is the honest name for the tool invented to operate on this exact structure.

---

## **What Changed (and What Didn’t)**

- The underlying ideas are **unchanged**
- The architecture is **unchanged**
- The goals are **unchanged**

What changed is the **language**.

The new vocabulary reflects the system’s true ontology instead of hiding it behind implementation terms.

> **Loom** is a history-native computational system in which execution is woven into an immutable fabric.

Events are committed by an irreversible write operation, guided by scheduling constraints. Unrealized alternatives are preserved as structured counterfactuals, enabling replay, comparison, and analysis without collapsing them into reality.

Classical systems only show us the projection of this process, keeping the machine hidden. Loom exposes the machine that decides what gets woven.

---

## **Glossary**

### **Loom** (n)

The realized fabric of execution.   The ordered, irreversible history that exists because it was committed. What is in the Loom is *real* in the strongest operational sense: it happened.

---

### **Stylus** (n)

The irreversible act that commits an event. The Stylus **writes** to the Loom. Once invoked, a choice becomes history and cannot be undone without destroying structure.

---

### **Scheduler** (n)

The constraint resolver that governs when and how the Stylus may act. Schedulers do not write history; they determine which histories are admissible.

---

### **Worldline** (n)

A single execution trajectory through possibility space. A worldline becomes real only when extended by the Stylus and projected into the Loom.

---

### **Umbra Index**

The structured archive of unrealized possibilities. Counterfactual worldlines that were explored but never committed are recorded here: queryable, comparable, informative—but not real in the way the Loom is real.

---

### **Observer**

A system embedded in the Loom that orders events locally. Observers do not experience time as a flow. They experience **the accumulation of ordering**.

> Minds are not clocks, they are looms.

---

### **Rulial Space**

The space of possible rule-consistent evolutions. Understanding requires finite reachability within this space. Learning is the process by which new basis vectors are acquired by observers, collapsing previously infinite Rulial distances.

---

### **Projection (Stylus → Loom)**

The only thing classical computation exposes. Traditional systems show us *what happened*, not *why this instead of something else*. Loom makes the hidden structure explicit.

---

## **What Loom Is Not**

- Not a metaphor layered on top of existing systems
- Not a rebranding exercise
- Not a claim that “everything is subjective”
- Not mysticism, simulationism, or multiverse fluff

Loom is a **structural description** of how ordering, commitment, and history actually work in embedded systems.

---

## Looking at the Loom

Now that we've established this new vocabulary, we can note how:

- Stylus writes to the Loom.
- The Loom holds what is real.
- The Umbra remembers what might have been.

Everything else is implementation.

## Just-In-Time

JITOS served its purpose. **Loom** is the accurate name that fits.
