# Wave 3.2 — final shell decomp (close HR-18 cap)

**Status post Wave 3.1:** HR-18 cap ≤ 600 LOC still violated by 2
files in `shells/desktop/src/`:

| File | LOC | Why marker stays |
|------|-----|------------------|
| `main.rs` | 928 | struct App + AppGfx + ImageEditSnapshot + HeroLive defs (~280 LOC) + 3 large `impl App` methods (`handle_dropped_files`, `handle_editor_key`, `dispatch_panel_pointer`, ~400 LOC combined) + `ApplicationHandler` trait impl (~80 LOC) + tests + `main()`. |
| `render_loop.rs` | 1603 | `App::run_render_frame` body lifted verbatim from Wave 3.1 stage C. Body holds a massive `let AppGfx { ... } = gfx;` destructure (~35 fields) and uses all of them across 1500+ LOC of orchestration. Per-phase split (snapshots / dispatch / paint / present) needs careful borrow restructure. |

Wave 3.1 reduced main.rs from 2607 → 928 LOC (−1679) and lifted
the render_frame body to a sibling. The remaining work to close
HR-18 fully is **mechanical but high-risk** because both files
hold large `AppGfx` borrow scopes that need to be re-threaded.

This brief covers the work + the trade-offs. It is **optional**
— Wave 2.5 + Wave 3.1 already delivered the multi-agent
collision protection (each `pending_X` migrated to bus +
hero_intents split per domain). Closing HR-18 fully is hygiene,
not unblock.

---

## Goal

Bring both files ≤ 600 LOC. Remove both `// ph2d-loc-cap:`
markers. `cargo test -p ph2d-host-desktop --test file_loc_caps`
emits `HR-18 loc-cap exceptions inventory: NONE (cap fully
active)`.

---

## Plan — 2 stages

### Stage A — `render_loop.rs` directory-module split (highest risk)

Convert `shells/desktop/src/render_loop.rs` → `shells/desktop/src/render_loop/`:

```
shells/desktop/src/render_loop/
├── mod.rs          (~150 LOC) — orchestrator: destructure + phase calls
├── snapshots.rs    (~250 LOC) — live_hierarchy / grid_view / stats /
│                                 gizmo_view / inspector_* snapshot publish
├── dispatch.rs     (~500 LOC) — consolidated bus drain + view_focus +
│                                 hierarchy intents + inspector commits
├── image_edit.rs   (~300 LOC) — trim / make_square / bgremoval / undo /
│                                 import drain dispatches
└── present.rs      (~250 LOC) — paint_hero_screen + tool panel +
                                  acquire_frame + render + tonemap +
                                  compositor + vello + submit + title
```

**Borrow-restructure plan**: orchestrator (`mod.rs::run_render_frame`)
does the `let AppGfx { ... } = gfx;` destructure once, then calls
phase fns passing the individual `&mut` fields they need. Each
phase fn has a long arg list (~10-15 `&mut` refs) but no borrow
conflicts since each takes its own subset.

**Risk**: borrow-checker is the main hazard. The current body
freely uses any field at any point; the consolidated bus drain
match captures locals that are consumed by later dispatches. The
split needs to carry those locals across phase boundaries —
either as orchestrator-level vars passed to phases, or as a
shared `FrameLocals` struct.

**Estimated**: 4-6h work + careful borrow iteration.

### Stage B — `main.rs` decomp

Extract from `impl App`:

```
shells/desktop/src/app_state.rs       (~280 LOC) — struct App,
                                                    AppGfx,
                                                    ImageEditSnapshot,
                                                    HeroLive defs +
                                                    impl App::new
shells/desktop/src/input_handlers.rs  (~400 LOC) — fn handle_dropped_files,
                                                    fn handle_editor_key,
                                                    fn dispatch_panel_pointer
                                                    (all as `impl App` via
                                                    split impl block, like
                                                    render_loop)
```

After extraction, `main.rs` should be ≤ 400 LOC:
- mod declarations + imports (~80 LOC)
- consts SPRITE_COUNT/WORLD_HALF/EPS/MIN_SPRITE_SIZE (~10 LOC)
- `Velocity` component (~5 LOC)
- helper free fns (~100 LOC) — drop_undo_pre_source_if_individual etc.
- `ApplicationHandler` trait impl (~80 LOC)
- `fn main()` (~10 LOC)
- module-level tests (~100 LOC)

**Risk**: low. The 3 input-handler methods are self-contained
`&mut self` methods — same split-impl pattern that worked for
render_loop.rs in Wave 3.1.

**Estimated**: 2-3h.

---

## Trade-offs (be honest with Enio)

**Cost**: ~6-9h focused work + multiple smoke windows.

**Value**:
- HR-18 cap fully active → architecture test no longer carries
  exceptions → future contributors can't accidentally grow the
  shell.
- No new feature unlocks. The multi-agent collision protection
  is already delivered (Wave 2.5 + Wave 3.1).
- Mostly architectural hygiene.

**Recommendation**: defer unless:
- A future agent collision pattern surfaces inside main.rs or
  render_loop.rs (e.g., 2 LLMs editing render_loop.rs phases
  simultaneously — same problem Wave 1/2/2.5 solved for hero.rs).
- OR Enio specifically wants the HR-18 inventory clean for
  documentation closure.

The Wave 3.1 closure is an honest stopping point. Both markers
have updated text explaining the post-Wave-3.1 reality and
pointing here.

---

## Quick-start checklist

When picking this up:

- [ ] Confirm HEAD is at `e309e80` or newer.
- [ ] Confirm CI verde on that sha.
- [ ] Read [`render_loop.rs`](../../shells/desktop/src/render_loop.rs)
      end-to-end — map phase boundaries before splitting.
- [ ] Read [`PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md`](PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md)
      §7 (metrics) — update "Files em shells > 600 LOC" row
      from 2 → 0 at end.
- [ ] Read [`STATE.md`](../IntegracaoMultiAgente/STATE.md)
      header — current `sha bom`.

Then attack Stage A (the harder one first — if borrow-checker
blows up the whole plan is in question, better to know early).
