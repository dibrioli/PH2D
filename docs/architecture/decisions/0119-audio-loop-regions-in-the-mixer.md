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
- **Sample-accurate loop switching.** Moving the region on a sounding voice *is* supported
  (`set_preview_loop_region` — the editor dragging a loop point, taking effect at the next lap;
  built because the alternative was re-triggering the clip, which is stuttery and useless for
  tuning a loop by ear). What is **not** here is switching between two different loops *at a musical
  boundary* — the vertical-remixing feature. That needs a scheduler, not a region.

---

## 6. Outcome (2026-07-12) — all eight held

| | | |
|---|---|---|
| **A1** | nothing changes for anyone who did not ask | ✅ `a_loop_without_a_region_is_byte_identical_to_the_old_whole_buffer_loop` — and the six ADR-0118 bit-identity gates never went red |
| **A2** | intro once, then the body, seam sample-accurate | ✅ `the_intro_plays_once_and_then_the_body_repeats` · `the_frame_after_the_loop_end_is_the_loop_start` |
| **A3** | streamed == resident, bit for bit | ✅ `a_streamed_region_is_bit_identical_to_a_resident_one` (+ mono) |
| **A4** | the metadata round-trips through the *application* | ✅ `a_loop_and_its_markers_survive_export_and_load` |
| **A5** | the editor auditions the real thing | ✅ the fabricated buffer, `playing_loop_region`, `loop_sig`, the hot-swap and the playhead offsets are **deleted** |
| **A6** | crossfade is a bake, refused without a pre-roll | ✅ `apply_loop_crossfade` + `the_crossfade_bake_refuses_without_a_pre_roll` |
| **A7** | HR-3 holds | ✅ `no_alloc_render` unchanged and green |
| **A8** | a degenerate region is refused, not obeyed | ✅ `a_region_that_names_nothing_falls_back_to_the_whole_buffer` · `a_region_running_past_the_end_is_clamped_to_the_audio` |

**Found while building, not planned for:** the region's wrap has to be gated on the **live**
`looping`, not the one `start` was handed. The editor's Loop toggle flips it on a *sounding* voice,
and a `wrap_at` still parked at the region's end turned every frame past it into a **held** frame —
the outro would have been a smear. (`unlooping_a_region_mid_flight_plays_on_into_the_outro`.)

**Two gates were born blind**, and only mutation said so:

1. The **producer** had no gate at all. The intro-plays-once sequence is built there and nowhere
   else, so breaking it left every gate in `ph2d-audio` green — the mixer plays what it is fed. The
   fix is an end-to-end gate through a real file (`a_streamed_region_plays_its_intro_exactly_once…`).
2. That gate's first test file had **no outro**, so the region ended where the file did — which made
   "never turn around at the loop end" indistinguishable from "turn around at EOF". A loud outro the
   loop must never reach is what makes both bugs one number.

A third one nearly slipped: two gates used a **1:1 sample rate**, where `frac` is always zero and the
interpolation's second frame is never read — so a *held* partner frame is invisible. Anything about a
seam has to be measured at a fractional advance.
