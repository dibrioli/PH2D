# ADR-0119 — Loop regions live in the mixer (and intro→loop falls out)

- **Status:** ACCEPTED (Enio, 2026-07-12)
- **Supersedes:** nothing. **Amends:** ADR-0118 (streaming voices) — the stream's published
  "length" becomes a published *region*.
- **Context:** the Audio Editor's Loop section (W6) authors loop points that **nothing can play**.

---

## 1. The hole

Three facts, each verifiable in one grep:

1. **The mixer has no loop region.** `PlayParams.looping` is a `bool` — *"loop the sample instead
   of stopping at its end"*. The whole buffer, or nothing. There is no way to say *"play `0..N`
   once, then repeat `A..B` forever"*, which is the structure of essentially every piece of game
   music ever written.

2. **The editor's loop audition is a preview-only fabrication.** `EditClip::loop_audition_buffer`
   builds a **separate buffer** containing only the region, with a crossfade, and the shell plays
   *that* on whole-buffer loop. So what the user hears is not what a game would play — because a
   game **cannot** play it.

3. **The metadata is written and never read back.** `ph2d-audio-encode` writes `smpl` (loop points)
   and `cue`+`LIST/adtl` (markers) on export, and it *has readers for both* — `read_loop_regions`
   and `read_markers`, each covered by a round-trip unit test. **Nothing in the application calls
   them.** Export a WAV with a loop, Load it back in the editor: the loop is gone. The readers are
   proven in isolation and connected to nothing.

Put together: Set Loop, the zero-crossing snap, the Crossfade slider and the `smpl` chunk are
authoring for an **external tool**. That is the "unit-green, integration-dead" shape the project has
a memory about (`feedback_tool_unit_green_integration_dead`), reached honestly — every piece works,
and the line between them was never drawn.

## 2. The decision

**The loop region is a property of a playing voice**, expressed in source frames, and the runtime
honours it. Intro→loop is not a second feature; it is what a region *is*:

```text
  play  [0 .. end)  once          <- the intro is whatever lies before `start`
  then  [start .. end)  forever
```

`looping: true` with **no** region keeps meaning exactly what it means today (the whole buffer), and
must render **byte-identical** to it — this ADR adds a capability, it does not reinterpret one.

### The streamed side (amends ADR-0118)

A streamed voice must sound **bit-identical** to a resident one; that was ADR-0118's A2 and it is not
negotiable here. The stream therefore publishes its **effective region** (start, end) rather than
just its length — the producer is the only side that knows what the file really contains, and it
learns it before the voice's first wrap needs it (the same `Release`/`Acquire` argument as the
length did). A whole-buffer loop publishes `(0, L)` and collapses to today's behaviour exactly.

The producer emits the sequence `[0..end)` then `[start..end)` repeatedly. It reaches the loop start
by **rewinding and discarding**, not by seeking: seeking is per-format and coarse, and a loop that
is a few frames off is a loop that clicks. Discarding is exact for every format, costs one re-decode
of the intro per lap, and happens on a worker thread that is already running far ahead of playback.

### The crossfade

**A runtime loop jumps. It does not crossfade** — a crossfade needs a second read head, and on a
stream it needs audio the producer has already thrown away. Every loop-point format (`smpl`
included) and every game audio engine works this way: the *asset* is authored to loop cleanly.

So the Crossfade control stops being a preview trick and becomes a **destructive bake** (one undo
step): it writes the seam into the audio, using the **intro as pre-roll** — the audio approaching
`start` is faded into the audio approaching `end`, so that jumping `end → start` is continuous. What
gets exported already loops cleanly, and the preview and the game hear the same thing because there
is only one thing to hear.

A loop with `start == 0` has no pre-roll to fade from, so the bake is refused there (and the control
is dim). That is a real limit, stated rather than hidden: with no intro, the tool is the
zero-crossing snap.

## 3. Frozen acceptance (written BEFORE the code)

- **A1 — Nothing changes for anyone who did not ask.** `looping: true, loop_region: None` renders
  **byte-identical** to today, on both the resident and the streamed path.
- **A2 — Intro→loop, resident.** With `Some(start..end)`: `[0..end)` plays once, then `[start..end)`
  repeats. The intro is heard **exactly once**. The frame after `end - 1` is `start`, and the
  interpolation across the seam reads `start` — not a held last frame, not silence.
- **A3 — Streamed == resident, bit for bit**, including across the wrap. (ADR-0118's standard; the
  1-ulp lesson is why this is a gate and not a listen.)
- **A4 — The metadata round-trips through the *application*.** Author a loop + markers → Export →
  **Load** → they come back. The existing readers get called.
- **A5 — The editor auditions the real thing.** The loop preview plays the clip *with a region*, not
  a fabricated buffer. The `playing_loop_region` special case and its playhead-offset bookkeeping
  are deleted.
- **A6 — Crossfade is a bake**, one undo step, using the intro as pre-roll; refused (and dim) when
  `start == 0`.
- **A7 — HR-3 holds** on the region path: no allocation, free, decode or lock on the audio thread.
- **A8 — A degenerate region is refused, not obeyed.** `end <= start`, `end` past the source, or a
  region shorter than one output frame → treated as **no region** (whole-buffer loop). Never a hang,
  never a stutter, never a silent voice.

## 4. Consequences

- `PlayParams` gains an **appended** optional field. Existing constructions (`..Default::default()`)
  are unaffected.
- The editor's loop transport gets **simpler**, not more complex: the region is the thing, so the
  offset arithmetic that mapped a fabricated buffer's playhead back onto the real clip goes away.
- `ph2d-audio-edit::crossfaded_loop` stops being a preview path and becomes the bake's engine.
- Streaming a region costs one intro re-decode per lap on the producer thread. If that ever shows up
  (a very long intro under a very short loop), the fix is a real seek in the readers — not a change
  here.

## 5. Not in this cut

- **Multiple loop regions.** `smpl` can hold several; games use one. The reader keeps taking the
  first.
- **Runtime crossfade** (a second read head). Deliberately refused above.
- **Sample-accurate loop switching** (swap the region on a playing voice without a click). The
  region is set at `play`; changing it live re-triggers the existing whole-buffer path.
