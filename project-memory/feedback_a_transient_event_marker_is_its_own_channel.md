---
name: feedback_a_transient_event_marker_is_its_own_channel
description: A UI mark for a transient EVENT must be event-sourced with its own lifetime, not derived from a standing-state field, or it dies when the state does
metadata:
  type: feedback
---

A visual that marks an **event** (a beginning, a hit, a transition) must be **sourced from the event and carry its own lifetime**. Deriving it from a standing-state property makes it live *exactly as long as that state*, so a transient — an event shorter than the mark's intended lifetime, or one that never enters the standing set at all — is under-shown or invisible.

**Concrete (physics contact flash, W-TickContacts 2026-07-22):** the begin-flash (`×`) rode `BodyContact.age_ticks` — a property of the STANDING contact. So a short bounce flashed only for the few ticks the pair actually touched (under-flash), and a FAST touch, which is resolved and separated inside one tick and so never enters the standing list at all, **never flashed**. The fix was to make the flash its OWN channel: a bridge-owned `ContactFlash` list, seeded from `Began` events and decayed in sim ticks, dropped past a fixed lifetime. Now a beginning flashes its full life whether or not the pair is still touching. `age_ticks`/`began` were then dead and removed.

**Why:** a standing state answers *"what is true now"*; an event answers *"what just changed"*. They have different lifetimes, and a marker that hangs off the state inherits the state's lifetime instead of the event's. The trap hides because for the COMMON case (a lasting touch) the two lifetimes roughly coincide — it only fails for the brief/absent-from-state case, which no fixture built around resting bodies exercises.

**How to apply:** when a UI marks a transition/event, give it a dedicated channel with its own decay; do NOT derive it from the entity's/pair's current-state record. **Test where the event is SHORTER than the state** (a fast or momentary transition) — that is the only place the derived version silently fails, and it is exactly the case a resting-body fixture omits.

Sibling: [[feedback_a_sequential_accumulation_is_sampling_dependent]] — same root, other axis. There a per-interval SAMPLE misses events shorter than the interval; the answer is to sample finer (here, diff per tick over the sub-step union). Here a per-STATE marker misses events the state does not outlive; the answer is to source the marker from the event. Both are "a channel tuned to one timescale cannot report a faster one."
