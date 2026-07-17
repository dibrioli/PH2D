# ADR-0124 — A range edit must be **told** its range, not made to rediscover it

- **Status:** accepted (implemented, `line/audio`, 2026-07-16)
- **Amends:** [ADR-0117](0117-audio-editor-memory-is-measured-not-declared.md) (same sentence, other axis) ·
  [ADR-0120](0120-audio-preview-is-a-buffer-you-own-not-a-buffer-you-rebuild.md) (uses its `get_mut`)
- **Scope:** `ph2d-audio-edit` (the editor's document layer) · `ph2d-audio::SampleData::version` ·
  the shell's audio caches

## Context — the report, and the measurement

> *"With this big audio, common operations (like raising the gain) on small selected ranges became
> slow. Here everything must be real-time."* — Enio

Measured, on the reported fixture (a 3-minute **mono** clip, 34.5 MB), `EditClip::apply_gain` on the
real path:

```text
selection FIXED (100 ms), clip growing:
   4 s -> 0.76 ms |  30 s -> 5.77 ms |  60 s -> 12.02 ms | 180 s -> 22.37 ms   (linear in the CLIP)
clip FIXED (180 s), selection growing 1000x:
  10 ms -> 22.4 ms | 100 ms -> 22.4 ms | 1 s -> 22.4 ms | 10 s -> 22.4 ms      (FLAT)
```

**The cost was O(clip) and the size of the selection did not matter at all.** 22 ms is past a 60 fps
frame, on an operation the user repeats. The three passes, each O(clip):

| pass | 180 s mono |
|---|---|
| `ops::in_range` → `splice` — rebuilds the whole buffer to replace one selection | 7.78 ms |
| `history::diff` — scans **both** buffers to rediscover the range | 2.12 ms |
| `PeakCache::build` — rebuilds the whole waveform envelope | 10.81 ms |
| **total** | **22.37 ms** |

~138 MB of memory traffic to touch 0.4 MB of audio.

**The thing to see:** `apply_gain`'s *first line* is `let t = self.target()`. It has the range. It
then throws it away, and three separate pieces of machinery downstream spend O(clip) each
re-deriving — or ignoring — a fact the caller was holding the whole time.

ADR-0117 established that **an edit is an interval** and rebuilt the undo timeline around it. That
was the *memory* axis. This is the same sentence on the axis of **time**: whatever is downstream of
an edit has to be *told* the interval, never made to rediscover it.

## Decision

**The range travels with the edit, to all three consumers.**

`EditClip::edit_range(r, op)` is the one funnel. It hands `op` the extracted region — so the DSP sees
byte-for-byte what it saw before and computes byte-for-byte what it computed before — and then:

1. **Writes the region where it lies.** `SampleData::get_mut` (ADR-0120) hands out the slice iff the
   document is the sole owner, which is the ordinary editing case. Nothing is rebuilt.
2. **Tells the history the range** (`History::push_rewrite`), which skips the scan.
3. **Patches only the bins the range touches** (`PeakCache::patch`).

Result, same measurement: **22.37 ms → 0.011 ms**, flat in the clip, and now **linear in the
selection** (10 ms → 0.001 ms · 100 ms → 0.008 ms · 1 s → 0.103 ms · 10 s → 1.023 ms). The cost of an
edit tracks the audio it edits, and nothing else.

### What is deliberately NOT O(selection)

- **A whole-clip edit** (no selection). Irreducible: you cannot change every sample for less than
  every sample. It takes the old path, untouched.
- **Edits that move audio** — trim, delete, paste, force-mono. Every frame after the cut really does
  change index, so the diff and the full waveform rebuild are honest work, not waste. They keep the
  `diff` path, which is why `diff` stays.
- **An edit while the mixer is playing the clip.** The buffer is shared, `get_mut` refuses, and the
  splice runs. That is not a wart: a buffer the RT thread is reading must not be scribbled on (HR-3).
  Correct, merely not fast — and the fallback *is* the old code, so there is no second implementation
  to drift.

## The consequence that nearly shipped as a silent bug

Six caches in the shell identified a clip buffer by its **address**, each repeating the same
reasoning in its own comment:

> *"`SampleData` is an immutable `Arc<[f32]>`, so a new buffer is a new pointer — and any edit hands
> us a different one."*

That sentence was true, and load-bearing, and **this ADR falsifies it**: an in-place edit keeps the
address and changes the contents. The spectrogram would have drawn the pre-edit waveform, the
delivery panel priced the pre-edit bytes, the platform set priced the pre-edit conform, the mono view
kept playing the pre-edit downmix, and the AI-Denoise staleness check would have called a stale result
fresh. None of it would look broken.

The address was a *proxy* for "the content changed", valid only while every edit reallocated. So the
question gets an explicit answer in the one place that can keep it honest:

- **`SampleData::version() -> BufferVersion`** — cache on this, never on `samples().as_ptr()`.
- **`get_mut` bumps it**, because `get_mut` is precisely and solely the operation that can change a
  buffer's contents without moving it. It probes before it bumps: a version that moved on a *refused*
  write would invalidate every cache for nothing.
- **`samples_mut` was a byte-identical duplicate of `get_mut`** under another name. It now delegates:
  two doors to one question is how a caller rewrites samples through the door that forgot to tell
  anyone.

## Gates

- `measure_range_edit.rs` — **the bug**: the same 1 s selection in clips **8× apart** must cost the
  same. The bar is a **ratio**, deliberately: `ci-test` builds at `opt-level = 1`, so a wall-clock bar
  measures the profile, not the code (the landmine `measure_preview.rs` documents). Measured **0.99×**
  for 8× the clip; before, ~8×.
- `measure_range_edit_alloc.rs` — the same claim in a number that **cannot flake**: bytes allocated
  by one gain nudge, identical (0.073 MB) in a 22 s and a 180 s clip. Allocation is not a proxy for
  the old cost — it *was* the old cost.
- `a_range_edit_is_the_same_edit.rs` — byte-identity of the two paths across 10 ops × 6 selections ×
  mono/stereo; undo/redo against a whole-snapshot oracle (the ADR-0117 A7 oracle, aimed at the
  informed step); the no-op guarantee; `patch` == `build`; and the version behaviour.
- **`a_sole_owner_writes_the_range_where_it_lies`** — that the fast path *fires*. The observable needs
  no test-only API: writing in place is *defined* by the samples not moving.

**This gate earned its place immediately.** The suite was written with `EditClip::new(data.clone())`
in its fixture — and a `clone()` bumps the `Arc`, so the *test* was the second owner, `get_mut`
refused, and every "fast vs slow" comparison was the slow path against itself. **Green, over an
optimisation that never ran.** It is the exact trap ADR-0120 documented, sprung on the ADR that cites
it. A second instance hid in the undo oracle, which held `data().clone()` across each edit.

Mutation-tested — each mutation lands on the gate that names the claim:

| mutation | RED |
|---|---|
| history told the wrong range start (`lo` → `lo+ch`) | both undo gates |
| history told a range one frame too short | both undo gates |
| peak cache told an empty range | fast-vs-slow (waveform) |
| the fast path never fires | perf ratio + alloc + 3 correctness |
| peaks rebuilt instead of patched | perf ratio + alloc |
| `get_mut` stops bumping the version | in-place-moves-the-version |
| the version bumps on a *refused* write | *(survived → gate added)* `a_refused_write_does_not_move_the_version` |

## Consequences

- Gain, normalize (peak/LUFS), reverse, invert, remove-DC, fade, silence and the rack's **Apply** are
  all O(selection) on a clip the editor owns. `apply_effect` routes through `render_effect_region`
  rather than `edit_range`, because an effect over a mid-clip selection is **pre-rolled** with the
  audio before it and a region trimmed without that warm-up would click at its leading edge.
- `History::push_rewrite` takes the caller's word for the range. That is the one dangerous thing here,
  so it is the one thing with the most gates: the promise is *"nothing outside this range differs"*,
  its only caller has just written the range itself, and both undo gates go red the moment the claim
  is off by a single frame.
- Steps are still built in **one** place (`step_for`). `diff` hands it the whole buffer because it has
  not been told the range; `push_rewrite` hands it the range because it has. Trimming still happens
  *inside* the given bounds, so an informed step is bit-for-bit the step the diff would have found.

## Open

- **The knob-drag preview still rebuilds the whole waveform every frame.** `PreviewScratch::step`
  ends in `EditClip::new(buf.clone())`, and `EditClip::new` is `PeakCache::build`: **21.9 ms per
  frame** on a 3-minute stereo clip. So ADR-0120's 62× win (0.27 ms) never reached the product — its
  own measurement drives the region write directly and never calls `step`. The fix wants an `EditClip`
  that can be handed a pre-patched waveform (a preview clip has no undo timeline), which is a design
  decision about that ADR's surface, not a mechanical change; it is left named rather than rushed.
- `ph2d-audio-edit`'s contract is not frozen (no gate caps its surface). `version()` is new public
  API on a foundational type and should be considered when that surface is capped.
