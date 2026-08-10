# HANDOFF (CLOSURE) — `line/audio` · W7: AI Denoise (DeepFilterNet via `tract`)

> Closes the build-out briefed in [`HANDOFF_line_audio_w7_ml_denoise.md`](HANDOFF_line_audio_w7_ml_denoise.md).
> **The line is fenced and waiting.** It does NOT integrate or push — that is Enio's explicit
> order, via a dedicated integrator (CLAUDE.md §0.7). Author: build-out agent, 2026-07-15.

---

## 0 — TL;DR

W7 is **built and green**. A native DeepFilterNet3 denoiser reaches the user as **"AI Denoise"**
in the audio editor, behind the **`audio-ml` feature (OFF by default)**. The wrapper reproduces the
official CLI's gain — **19.4 dB SI-SDR** on the author's 0 dB-SNR fixture (CLI: 20.6; the W5 spectral
denoise bought only +1.9 over the input, this buys +12.8). `libDF` 0.5.6 is **vendored**, so no git
dependency and CI stays hermetic. All W7 gates pass; the feature-OFF build is byte-untouched.

**Pending:** the human smoke (`PH2D_AUDIO_ML_SMOKE=1`, §6) — nobody has clicked the button yet.

---

## 1 — What landed (rounds 1→4 of the briefing §4)

- **R1 — engine.** New crate `crates/ph2d-audio-ml` (`#![forbid(unsafe_code)]`):
  `denoise_ml(&SampleData, amount) -> SampleData`. Mirrors the W5 denoise: `amount 0` returns the
  input byte-for-byte, `amount 1` is the pure model output. At the boundary it resamples to the
  model's fixed 48 kHz (via `ph2d_audio_edit::conform`, and back), de-interleaves to channel-major,
  pads by the model delay so the tail flushes, feeds hop-sized blocks, slices the delay off (the
  CLI's `-D`, keeping the tail the CLI drops), then dry/wet-blends by `amount`. `RuntimeParams`
  replicate the CLI exactly (no post-filter, `atten_lim 100`, thresholds `-15/35/35`,
  `reduce_mask MAX`).
- **R2/R3 — integration.** `AudioEditCmd::DenoiseMl` + `AEDIT_SPEC_DENOISE_ML` + the **"AI Denoise"**
  button in the panel's Spectral section. The panel is **feature-agnostic**: the shell publishes
  availability via `spectral_state::set_ml_available(cfg!(feature = "audio-ml"))` and the button is
  painted **only when true** — feature OFF ⇒ no button, no dead UI, panel paint byte-identical. Shell
  bridge `editor_denoise_ml()` (`#[cfg(feature = "audio-ml")]`) → `commit_rendered` (1 undo step) →
  hot-swap. `amount 0` guarded so a no-op never costs an undo step.
- **R4 — vendoring + gates + smoke** (this closure).

**Commits (`git log --oneline main..HEAD`, newest first):**
```
289b0c4a feat(audio): W7 smoke -- PH2D_AUDIO_ML_SMOKE stages a noisy clip for AI Denoise
ea214590 chore(audio): split engine.rs tests to a sibling file -- inherited 712>700 LOC
0a04b208 chore(audio): green two pre-existing (non-W7) gate failures inherited on the line
1554cc18 chore(audio): W7 -- vendor DeepFilterNet libDF (drop the git dependency)
08e8d6db feat(audio): W7 R2/R3 -- AI Denoise wired into the editor, gated OFF by default
b9d875a3 feat(audio): W7 R1 -- ph2d-audio-ml denoise, proven at parity with the DFN CLI
```
(Everything at/below `c075e837` was already on the line before this agent.)

---

## 2 — The numbers (measured, not claimed)

| signal | SI-SDR vs clean | gain over noisy |
|---|---|---|
| noisy input (fixture) | 6.56 dB | — |
| **our `denoise_ml`** (amount 1.0) | **19.39 dB** | **+12.83 dB** |
| reference CLI `deep-filter -D` | 20.59 dB | +14.04 dB |

Fixture: the DeepFilterNet author's `clean_freesound_33711.wav` / `noisy_snr0.wav` (real speech, 0 dB
SNR, sample-aligned), trimmed to 6 s, committed at `crates/ph2d-audio-ml/tests/fixtures/`. The 1.2 dB
gap to the CLI is alignment/tail handling — it clears the ≥18 dB gate with room and is far above the
W5. Metric: SI-SDR with cross-correlation delay alignment (ported to the Rust gate; validated to
reproduce the ADR-0123 §3.5 numbers exactly).

---

## 3 — Gates (all green)

| gate | where | proves |
|---|---|---|
| `denoise_ml_reproduces_the_reference_cli_gain` | `ph2d-audio-ml/tests/parity_with_reference_cli.rs` | the wrapper = the model (presence). Pairs pinned-noisy input (~6 dB) with ≥18 dB out |
| `amount_zero_is_byte_identical…` (×2) | ml crate | the rack's neutral point, on the real fixture and a tone |
| `no_ml_runtime_reaches_the_mixer` | `ph2d-audio/tests/` | `tract`/`deep_filter`/`ph2d-audio-ml`/`ndarray` never a dep of the RT mixer (absence; the parity gate is its presence sibling) |
| `audio_ml_is_off_by_default` | `shells/desktop/tests/` | structural proof from the manifest that a default build resolves no `tract`. **Mutation-checked** (adding `audio-ml` to `default` turns it RED) |

Also verified 1× over the accumulated diff: `cargo deny check` (advisories/bans/licenses/**sources**
all ok — the vendoring cleared `unknown-git`), `cargo machete` clean, `cargo fmt --all --check`,
clippy `-D warnings` on `ph2d-audio-ml`/`ph2d-audio`/`ph2d-panel-audio-editor` and on
`ph2d-host-desktop --features audio-ml`, LOC caps (shell + workspace), `typos`,
`cargo build -p ph2d-host-desktop` (OFF) and `--features audio-ml` (ON), full `-p ph2d-host-desktop
--tests` (9 binaries), `ph2d-editor-core --tests`, `ph2d-panel-audio-editor` (74), `ph2d-audio` (52+).
The `cargo tree` form of the feature boundary was confirmed by hand (OFF: no `tract`; ON: pulls
`deep_filter`/`tract`).

---

## 4 — The vendoring (`crates/ph2d-audio-ml/vendor/deep_filter`)

`libDF` 0.5.6 (MIT OR Apache-2.0) copied in, not git-dep'd, because the repo is hermetic on purpose
(`deny.toml`: `unknown-git = "deny"`, empty `allow-git`) — the same reason `ort` was rejected. Same
pattern as `ph2d-audio-opus`. See `vendor/deep_filter/VENDOR.md`. Key points for a future re-vendor:

- **Trimmed** to `tract` + `default-model` (+ `transforms`/`logging`). Dropped the dataset/bin/
  vorbis/flac/capi/wav-utils features **and libDF's own git dep `hdf5`** (which alone would break the
  policy). The 7 source files those features compile were deleted; only `lib.rs`/`tract.rs`/
  `transforms.rs` remain (the latter's dead `#[cfg(test)]` block trimmed), formatted to repo style.
  Only logic change: the model `include_bytes!` path `../../models` → `../models`.
- **Isolated from our tooling:** root `[workspace] exclude` + an empty `[workspace]` in the vendored
  manifest (so `cargo metadata`/`fmt`/clippy/machete/deny don't police it as a member);
  `[package.metadata.cargo-machete]` ignores for the crate-lib renames machete misses
  (`deep_filter`→`df`, `rust-ini`→`ini`) + the transitive `rustfft`; `[lints]` allowing
  `unexpected_cfgs`/`mismatched_lifetime_syntaxes`; `.gitattributes` marks `*.tar.gz`/`*.onnx` binary.
- The **7.6 MB model** (`models/DeepFilterNet3_onnx.tar.gz`) rides in the repo — Enio's one veto point
  (ADR-0123 §7): he accepted the model size + the redistribution reading (§3.4).

---

## 5 — Inherited debt greened to close the line (NOT W7 — for the original owners)

The line was **already red** at three gates before W7 began (base `c075e837`), from earlier fast-mode
commits. Greened so the line closes clean, in clearly-labeled `chore` commits, but flagged here for
their owners:

1. **`shells/desktop/src/audio/fx_presets.rs` — 631 LOC** (> 600), since `a5ec9d7a` ("Gate /
   Expander" added the GATE_TIGHTEN preset). Used the gate's sanctioned `// ph2d-loc-cap:` marker
   with the true reason. **Proper fix (deferred to the preset owner):** split the `static` preset
   DATA into a sibling `data` module.
2. **`crates/ph2d-audio/src/engine.rs` — 712 LOC** (> 700 workspace cap), since `a7e7ab13` grew it
   with reproduction presence-gate tests. Split the `#[cfg(test)] mod tests` into a sibling
   `engine_tests.rs` via `#[path]` (still `crate::engine::tests`; **no RT/render code moved**).
   712→621. A real decomposition, not a `FILE_OVERAGE_OK` bump.
3. **`crates/ph2d-editor-core/tests/architecture_adr_numbers_are_unique.rs`** — committed
   unformatted; ran `rustfmt`. Cosmetic.

---

## 6 — Smoke (Enio, at the end)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && \
PH2D_AUDIO_ML_SMOKE=1 cargo run -p ph2d-host-desktop --features audio-ml
```
Stages 3 s of a voiced tone under broadband hiss (~0 dB SNR) into the audio editor. **Do:** open the
Audio Editor (top bar) → expand the **Spectral** section → **Play** (hiss under a tone) → click **AI
Denoise** → **Play** again. The hiss should fall away and the tone stay. (Compare against the W5
Denoise: Learn a quiet gap, then Denoise — it removes far less.) The one check no gate can make: it
sounds better.

**Without `--features audio-ml`** the clip still stages but the AI Denoise button does not exist (by
design) — the default build is untouched by W7.

---

## 7 — Open / follow-ups

- **Progress bar — BUILT (2026-07-16), and it is APP infrastructure.** See §8 below and
  `docs/Audio/03_o_que_falta.md` §2.1. The AI Denoise is the first consumer, not the owner.
- **Stereo** clips are processed by running the model per the (resampled) channel count; the
  boundary keeps the layout. Not exercised by the mono fixture — worth a stereo smoke if it matters.
- The dev-profile parity test runs DFN at opt-0 (~6 s). Acceptable for a gate; if it ever grates,
  add a `[profile.dev.package]` opt bump for the tract crates (mirrors what `-spectral` did).
- Vendored `deep_filter` is trimmed to the compiled subset; a future re-vendor must re-apply §4.
- The three inherited-debt items in §5 want their owners' proper fixes (esp. the `fx_presets` split).

**Do NOT integrate or push. Fence held; hand this to Enio.**


---

## 8 — Long-operation progress (2026-07-16) — the shell's first async pattern

Enio's call: **it serves the whole app**, not the audio. Batch LUFS, export, upscale and the
painter have the identical shape, and each inventing its own thread + bar would give the app four
progress idioms and four sets of bugs. The AI Denoise is the first consumer.

### 8.1 What landed

**`crates/ph2d-editor-core/src/progress.rs`** (+ `progress/tests.rs`), sibling of `toast.rs`:

| | |
|---|---|
| `Progress` | Cloneable, thread-safe (`Arc` + `AtomicU32` fixed-point ppm + `AtomicBool` + label). Worker stores, painter loads, **one `Arc`**. |
| `Job<T>` | `spawn(label, FnOnce(&Progress) -> T)` · `try_take()` **never blocks** · `is_finished()`. |
| `JobQueue` | Holds `Progress` (not `Job<T>`) · `tick()` per frame · cap 8, silent drop. |

Shell: `AppGfx.jobs` (beside `toasts`), ticked at the toast tick, painted at the toast paint.
`AudioSystem` keeps the typed `ml_job` + a one-shot `started_job` outbox (the `take_*` idiom the
panel bridge already uses — the spawn happens deep inside `editor_apply`, which has no business
reaching for the shell's chrome).

### 8.2 The decisions that matter (and the two the brief got wrong)

1. **The design system ALREADY had a progress widget.** The brief said it did not. `widget::
   ProgressBar` (determinate/indeterminate, `show_percent`, `Role::ProgressIndicator`, the
   small-value radius clamp) existed with the **gallery as its only consumer**. The job bar is
   that widget — a hand-rolled track would be a second answer to "what does progress look like
   here", and the day someone retunes the widget the job bars would keep the old look. The card
   around it (elevated surface, border, label) is the *column's* chrome and is `progress.rs`'s.
2. **`impl Paint for JobQueue` could NOT go in `paint.rs`.** The brief asked for it there;
   `paint.rs` is **frozen at 884 LOC** in `architecture_workspace_file_loc_cap`'s allowlist —
   "may shrink, never grow", and raising it needs Coordenador sign-off + an ADR. It is colocated
   with the model instead, which is this crate's own documented pattern anyway ("data + state enum
   + tokens + a11y::Node + colocated `paint_X` helper"). paint.rs 884 → **879**.
3. **Column order: toasts on top, bars below.** A toast self-destructs in 3 s — one chance to be
   read — so its slot must not depend on whether an unrelated job happens to be running. A bar is
   persistent and self-announcing (it is the thing *moving*), so it absorbs the offset. One ruler
   (`column_row`), which the toast painter now measures against too, so they cannot drift.
4. **The `-ml` crate takes `&dyn Fn(f32)`, not `Progress`.** The containment that put `tract` in
   there cuts both ways. `denoise_ml` delegates with an empty callback — **one code path**, so the
   parity gate still measures the function the app runs.
5. **`done` comes from a drop guard.** `denoise_ml` has two `.expect()`s; a panicking worker that
   left the flag false would leave a bar that can never advance and never leave.
6. **Stale results are dropped.** The UI stays live (the point), and only Spectral is dimmed —
   Cut/Paste/Normalize are still there. The worker returns `MlDenoise { source, out }` and the
   installer compares buffer identity. Sound because `source` comes back **alive**: an address
   cannot have been recycled by a buffer we are still holding.
7. **The view toggle stays live during a job** — it draws, it does not edit. Pinned by a gate so
   it is not "fixed" later.

### 8.3 Gates

| Gate | Where |
|---|---|
| Work leaves the UI thread; `try_take` does not block; result arrives | `progress/tests.rs` (deterministic: the worker is held on a channel the test controls — no sleep) |
| A panicking worker still takes its bar down | same |
| `tick` drops the finished, keeps the running (out-of-order) | same |
| The bar paints, an empty queue does not, the fill tracks the fraction | same — **counts `Scene::encoding().n_paths`**, not "did not panic" |
| a11y bounds are the track that gets painted | same |
| **Progress climbs across the run** (not `{0,1}`, spread over all four quarters) | `ph2d-audio-ml/tests/progress_is_reported_as_the_model_runs.rs` |
| Watching the run changes no sample; `amount 0` reports nothing | same |
| Every Spectral control is inert while a job runs — **and works again after** | `ph2d-panel-audio-editor/tests/seam.rs` |
| Parity with the reference CLI still green | `parity_with_reference_cli` (untouched) |

**Mutation-tested (4 mutants, all killed):**
- hop loop reports nothing → survivor `[1.0]`, killed by the *spread* assertion. **A naive "did
  progress arrive?" test would have passed** — that is why the assertion is about the middle.
- wrong denominator (samples, not hops) → progress crawls to 0.6 %, monotone, ends at 1.0 → killed.
- `ml_busy` refusal removed → seam red.
- `paint_bar` no-op → both paint gates red.

**The seam gate found a real bug while being written:** the busy check was per-arm and **Repair was
forgotten**. Fixed by asking once for all four ([[feedback_a_condition_that_enumerates_its_readers_rots]]).

### 8.4 Smoke — and how to actually SEE the bar

The default 4 s clip denoises in ~0.16 s: **the bar flashes past**, which is the product being fast.
`PH2D_AUDIO_ML_SMOKE_SECS` stages the case the bar exists for:

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && \
  PH2D_AUDIO_ML_SMOKE=1 PH2D_AUDIO_ML_SMOKE_SECS=180 \
  cargo run --release -p ph2d-host-desktop --features audio-ml
```

3-minute take → ~5.4 s of inference (the console prints the estimate for whatever you staged).
Expect: the window keeps redrawing, the bar climbs top-centre, the percentage moves, the Spectral
section is dimmed with the reason on its status line, the view toggle still works — and the hiss is
gone at the end. An indicator nobody can observe has not been verified, which is why this knob is
part of the feature.

### 8.5 Open

- **One consumer.** Batch LUFS / export / upscale / painter are next; copying is their work.
- **No cancel.** Nothing aborts a job in flight. Defensible at 5 s; not at the first minutes-long
  job — and then it is an `AtomicBool` the callback reads (it is already called per hop).
- **`build_a11y` is not grafted into the tree** — neither is `ToastQueue`'s. Designed, unconsumed:
  same status as the rest of the design system. When the shell grafts toasts, it grafts bars.
- **No error toast if a worker panics** (the panic goes to stderr; the bar comes down).
- The bar sits at 0 % during the boundary resample for a clip that is not already 48 kHz.

**Do NOT integrate or push. Fence held; hand this to Enio.**
