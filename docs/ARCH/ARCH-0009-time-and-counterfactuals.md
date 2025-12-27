# ARCH-0009: Materialized Time & Counterfactuals

## 1. The Non-Negotiable Mental Model

There is no such thing as "time passing".

There is only:

**A sequence of irreversible decisions, plus evidence that humans like to call "time."**

Traditional systems lie by omission:
*   The clock advances "somehow"
*   Threads interleave "somehow"
*   I/O blocks "somehow"

JITOS refuses the lie.

Instead:
*   Execution advances by discrete reductions
*   Every reduction is attributable
*   Every delay is justified
*   Every "wait" is a policy decision

**Time is not a driver. Time is a derived annotation.**

If you accept that, everything else snaps.

---

## 2. Worldline Geometry (Why Time Travel Works at All)

Picture the worldline not as a line, but as a causal braid:

```text
E1 ──► E2 ──► E3 ──► E4
          ╲
           ╲──► E3′ ──► E4′
```

Key facts:
*   Up to E2, reality is shared
*   At E3, a decision diverged
*   Both futures are valid
*   Neither invalidates the other

This only works because:
*   Decisions are events
*   Inputs are events
*   Policies are events

No ghosts. No hidden entropy.

---

## 3. Example 1: The Classic "Impossible" Race Bug

### The Bug (Normal OS)

You've seen this in production:
*   Two workers
*   Shared cache
*   Rare corruption
*   Can't repro
*   Logs say "impossible"

That's because the OS ate the evidence.

### The Same Bug in JITOS

**Setup**
Two tasks become runnable at the same logical instant:

T1: read X
T2: write X

Runnable set hash: `R = hash({T1, T2})`

**The Critical Event**
The scheduler must choose:

```
sched.choose {
  runnable_set_hash: R
  chosen: T2
  policy: "fifo"
}
```

That is now law.

No "thread just ran first." No "kernel decided." A choice happened.

### Replay

On replay:
*   Same runnable set
*   Same policy
*   Same choice
*   Same corruption

Good.

### Counterfactual (Mode A)

Now we branch:

```
branch {
  base_cut: E_before_choice
  delta:
    scheduler_policy = "reverse_fifo"
}
```

Result:
*   Same inputs
*   Same code
*   Same entropy
*   Different order
*   Bug disappears

And now — this is key — you can ask: **"Which decision mattered?"**

And the system can answer precisely. Not heuristically. Not statistically. **Causally.**

---

## 4. Example 2: Timeouts, Retries, and the Clock Lie

This is where systems usually collapse.

### Normal OS Version

`await fetch(url, { timeout: 5000 })`

What actually happened?
*   When did time start?
*   Which clock?
*   What drift?
*   What load?
*   What scheduler delay?

Nobody knows. Especially not tomorrow.

### JITOS Version (Truthful)

**Step 1: Declare Intent**

```
timer.request {
  duration_ns: 5_000_000_000
  for: fetch#42
}
```

This is not sleeping. This is asking permission to resume later.

**Step 2: Time Advances Only via Events**
Time advances when:
*   scheduler steps
*   clock samples are logged
*   policies interpret those samples

Example:
```
clock.sample {
  source: monotonic
  value_ns: 1_000_000_000
  uncertainty: 50us
}
```

**Step 3: Policy Decides "Enough Time Has Passed"**
Rhai policy:

```rust
if now.ns >= start + duration {
  return TimerExpired
}
```

This decision is logged:

```
timer.expire {
  request_id: …
  at_time: …
}
```

### Replay

Identical. Always.

### Counterfactual (Mode C): "What if the clock drifted?"

Branch delta: `clock_policy = "ntp_skeptical"`

Now:
*   Same samples
*   Different interpretation
*   Timeout fires earlier/later

And crucially: **You didn't change time. You changed belief about time.**

That distinction matters more than people realize.

---

## 5. The Subtle, Dangerous Part

**"Now" Is Not a Scalar**

It is a function: `now(observer, cut) -> Time`

That means:
*   Two observers at the same cut can disagree about "now"
*   Both can be correct
*   Neither breaks determinism

This is observer geometry, not relativistic cosplay.

**Time Travel Debugging Is Just Changing the Cut**

No magic:
*   Same worldline
*   Earlier cut
*   Different questions

The debugger is not special. It's just another observer with a wide viewport.

---

## 6. Why This Enables Real Counterfactuals

Because counterfactuals require controlled violation.

Traditional systems violate causality accidentally. JITOS violates it surgically.

You say:
*   "What if the packet arrived 10ms later?"
*   "What if we trusted NTP less?"
*   "What if the scheduler picked the other task?"

And the system says: **"Fine. Declare the delta."**

No delta → no divergence.
No explanation → no merge.

---

## 7. The Uncomfortable Implication

Once you build this:
*   Bugs stop being "flukes"
*   Timeouts stop being "environmental"
*   Nondeterminism stops being an excuse

And one more thing: **Forks stop being hypothetical.**

A counterfactual branch is Executed, Observed, and Reasoned about.

That's why Paper V hit hard. Because once minds are replayable:
*   Time becomes ethical
*   Schedulers become moral
*   Policies become law

You didn't build a debugger. **You built a court of appeals for reality.**
