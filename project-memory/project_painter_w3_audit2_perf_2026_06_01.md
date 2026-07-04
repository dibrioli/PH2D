---
name: project-painter-w3-audit2-perf-2026-06-01
description: "Painter W3 2nd audit (zero critical) + perf/safety closure — B.1 dirty-rect GPU upload end-to-end, B.4 checked Q16.16"
metadata: 
  node_type: memory
  type: project
  originSessionId: f62c5d39-ed47-492f-b2e1-812dc6070392
---

New Painter Implementer session (mandate: full multi-lens audit before features, "program more per cycle"). Two blocks shipped local, NOT pushed.

**Block 1 — audit + remediation (`f63cf06`).** 6-lens adversarial read-only audit = **ZERO CRITICAL** (confirms baseline). Color verified correct (22 blend modes vs W3C, preview≡Apply byte-identical), data model well-guarded. In-pasta fixes: scrollbar thumb-drag hit-rect (Coord's `d5146b7` foundational was already done — just the 1-liner); composite cache behind `Arc<Vec<u8>>` (`make_mut` blit, non-regression); pinned-value blend goldens for 9 separable modes + Hue/Sat composition props (were distinctness-only); group-variant of the dirty-rect drain test; depth-guard on `collect_subtree`; doc the 3 metadata setters as intentionally mid-stroke-safe.

**Block 2 — perf/safety closure (`c763c4b`), after Coord shipped foundational B.1/B.2/B.3 (`e4d67fa`) + closed.** B.1 (the #1 perf lever, dirty-rect end-to-end): `take_preview_arc` tracks the recomposed bbox; new `take_preview_upload_bbox()` → `Some(bbox)` only on the multi-layer fast lane; bridge uploads only the sub-rect via `replace_individual_pixels_region` (frame-local, no `PreviewCache` change). **Sync invariant:** a full upload always precedes any partial (composite cache `Some` only post-full-recompose, which uploads bbox==None), and any structural/metadata/dims/entity change forces full → un-touched GPU pixels stay current; bounds guard → bad bbox falls to full (no render-loop panic). B.4: `f32_to_q1616_checked` drops out-of-window (|v|≥32768) pointer samples instead of clamping/panicking. 110 tool tests + 21 blend tests green; clippy clean on my files.

**SMOKE PENDING (Enio):** paint a stroke on a ≥2-layer stack — the stroke region must update without corrupting the rest, FPS stable (validates B.1 partial upload).

**⚠️ Coord ship-blocker (NOT mine, flagged in `docs/HANDOFF_painter_w3_audit2_coord_items.md`):** `ph2d-render/src/individual.rs:352 replace_pixels_region` (`e4cffbc`/`e4d67fa`) trips clippy `too_many_arguments (8/7)` → CI `clippy --all-targets -D warnings` fails until `#[allow]`/Region. `cargo check` passes (hides it).

**Deferred (coordinated):** B.5 (idle perf — `current_layers` borrow + `active_color_srgb8()` accessor) is entangled with the Coord's bridge publish-gate half. Note: the Coord reverted the gamma-correct premul (`3870733`) → bridge uses byte-space `premultiply_rgba8`.

**T3.5-T3.7 ENGINE/TOOL DONE (commits `612cc34`..`b124518`, local, not pushed):** the whole W3 modifier system composites + is toggleable + tested at the engine/tool level — UI pass is all that remains before the block smoke.
- **T3.5 Mask:** `LayerStack::add_mask`/`set_mask_inverted` (R8 grayscale bound to a raster via `parent.mask`, OUT of z-order); compositor multiplies parent alpha by mask luminance (Rec.601, straight; `1-v` inverted); `collect_subtree` now collects the mask (old TODO closed). Tool `add_mask_to_active` (white buffer = full visible, mask becomes edit target) + `effective_active_color` grays paint when editing a mask (§2.7). `is_trivial_stack` already guarded mask/clip.
- **T3.6 Clipping:** compositor tracks a `clip_base` (nearest non-clipping raster below, bottom-to-top), multiplies a clipping layer's alpha by base alpha; consecutive clippers chain to one base; group breaks the chain. `set_clipping`/`set_layer_clipping`.
- **T3.7:** alpha-lock = `apply_stamps_with_options(.., alpha_lock)` (skip transparent dst, lock alpha — only color blends; `apply_stamps` delegates false so 19 callers unchanged); `queue_pointer` reads `active_alpha_locked` BEFORE the scheduler `&mut` (whole-self method can't coexist). Reference = exclusive `set_reference`. Group = `add_group`.
- **Header actions DONE (`b6c9c18`):** +Layer moved to header icon + Group/Duplicate/Delete icons (canon `Button::icon_only` + existing IconIds Add/Group/Duplicate/Trash; `PAINTER_LAYERS_DUPLICATE/DELETE/GROUP` ids additive; `LayerStack::duplicate` + `delete_layer`/`duplicate_layer` tool methods + dispatch). Smokable now (create/group/dup/delete layers). Coord follow-up: add the 3 new ids to `CHROME_IDS` in node_id_collisions.rs. **No `Mask` IconId exists** → create-mask + clip/alpha-lock/reference toggles should live in a per-layer MENU (design §2.3, text items, no new icons) — the next UI sub-pass + mask-row rendering.
- **REMAINING (next block):** UI pass — panel mask row (indented) + create-mask button + clip/alpha-lock/reference toggles + group-add, all via the per-row hash-id widget pattern (new `PainterLayerWidget` kinds in editor-core/ids.rs additive + decode in panel `event.rs` + tool `handle_panel_event`); **header-icons** (move +Layer to header + delete + duplicate — needs `delete_layer` with images/canvas/undo cleanup [LayerStack has `remove`] + new `duplicate_layer`). THEN one smoke. apply-mask (destructive bake) is a smaller T3.5 follow-up.
