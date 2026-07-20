//! Display gates — **what the artist SEES is what the tool painted** (phase D, 2026-07-15).
//!
//! Enio's 2026-07-14 smoke reported two display bugs with impasto on screen: (a) the **Anchored**
//! method's relief vanishes at pen-up, (b) a **jittered** stroke's relief comes out stretched. The
//! tool was proven innocent the same day (its own gate holds the pen-up frame byte-equal to a full
//! recompose, *inside* the tool). So the defect — if it is reachable without a window — lives in
//! the seam these tests own: the shell's preview drain → upload plan → slot texture, the bytes the
//! sprite shader actually samples.
//!
//! Everything here is the REAL pipeline: the real `PainterTool` under the smoke's arming, the real
//! drain (`take_preview_arc` / `take_preview_upload_bbox`), the real decision ([`plan_upload`]),
//! the real gather ([`extract_region`]) and the real `premultiply_rgba8`. The only substitution is
//! the wgpu copy at the very end, modelled as what a texture upload is — a byte copy into an array
//! ([`Screen::frame`]). That is not a mirror of the product; it is the product minus the driver.
//!
//! The oracle models the APPEARANCE, never the mechanism: after any frame, the slot's bytes must
//! equal the premultiplied composite the tool just claimed; after a whole gesture, they must equal
//! a from-scratch recompose of the document. Anything the protocol drops, misplaces or staleness
//! leaks — vanished relief, displaced patch rects, missed settle frames — is a byte diff here.

use super::painter_bridge::{UploadPlan, extract_region, plan_upload};
use crate::app_state::{PainterPreview, PainterPreviewGpu};
use ph2d_editor::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_render::premultiply_rgba8;
use ph2d_tool_painter::PainterTool;
use std::sync::Arc;

/// The entity bits the harness previews under — any constant; the protocol only compares them.
pub(super) const ENTITY: u64 = 42;

pub(super) fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A white canvas + the impasto smoke's exact arming (`impasto_smoke::arm_brush_once`: size 40,
/// impasto ON, everything else the shipped default) — the brush in Enio's hand when he saw both
/// symptoms.
pub(super) fn impasto_tool(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(40.0);
    t.toggle_brush_impasto();
    t
}

/// The screen's half of the CPU preview lane, run exactly as `painter_bridge::dispatch` runs it
/// each frame: drain if dirty (a clean frame keeps the previous cache), plan, apply the plan to
/// the slot bytes, update the bookkeeping.
struct Screen {
    cache: Option<PainterPreview>,
    gpu: Option<PainterPreviewGpu>,
    /// `(texture_id, width, premultiplied bytes)` — the slot texture the sprite samples.
    slot: Option<(u32, u32, Vec<u8>)>,
    next_texture_id: u32,
}

impl Screen {
    fn new() -> Self {
        Self {
            cache: None,
            gpu: None,
            slot: None,
            next_texture_id: 0,
        }
    }

    /// One frame of the preview lane. Returns the plan it executed (for the gates' own asserts).
    fn frame(&mut self, painter: &mut PainterTool) -> UploadPlan {
        let mut dirty_bbox = None;
        if let Some((rgba, w, h)) = painter.take_preview_arc() {
            dirty_bbox = painter.take_preview_upload_bbox();
            self.cache = Some(PainterPreview {
                entity_bits: ENTITY,
                rgba,
                width: w,
                height: h,
            });
        }
        let Some(preview) = self.cache.as_ref() else {
            return UploadPlan::Skip; // nothing drained yet — dispatch's gated block never ran
        };
        let plan = plan_upload(preview, self.gpu, dirty_bbox, false);
        match plan {
            UploadPlan::Skip => {}
            UploadPlan::Partial {
                texture_id,
                rect: (bx, by, bw, bh),
            } => {
                let mut region = extract_region(&preview.rgba, preview.width, bx, by, bw, bh);
                premultiply_rgba8(&mut region);
                let (id, w, bytes) = self
                    .slot
                    .as_mut()
                    .expect("a Partial plan names a seeded slot");
                assert_eq!(
                    *id, texture_id,
                    "the plan patches the texture it was planned against"
                );
                let row_bytes = (bw * 4) as usize;
                for row in 0..bh {
                    let dst = (((by + row) * *w + bx) * 4) as usize;
                    let src = (row * bw * 4) as usize;
                    bytes[dst..dst + row_bytes].copy_from_slice(&region[src..src + row_bytes]);
                }
                self.gpu = Some(PainterPreviewGpu {
                    texture_id,
                    width: preview.width,
                    height: preview.height,
                    arc_token: Arc::as_ptr(&preview.rgba) as usize,
                    entity_bits: ENTITY,
                });
            }
            UploadPlan::Full { reuse } => {
                let mut bytes = (*preview.rgba).clone();
                premultiply_rgba8(&mut bytes);
                let texture_id = reuse.unwrap_or_else(|| {
                    self.next_texture_id += 1;
                    self.next_texture_id
                });
                self.slot = Some((texture_id, preview.width, bytes));
                self.gpu = Some(PainterPreviewGpu {
                    texture_id,
                    width: preview.width,
                    height: preview.height,
                    arc_token: Arc::as_ptr(&preview.rgba) as usize,
                    entity_bits: ENTITY,
                });
            }
        }
        plan
    }

    /// The bytes on screen (the slot the sprite samples).
    fn shown(&self) -> &[u8] {
        &self.slot.as_ref().expect("a seeded slot").2
    }

    /// The pair oracle: the slot must hold exactly the premultiplied form of the composite the
    /// tool most recently handed the bridge. On a Full frame that is trivially the upload; on a
    /// Partial frame it additionally asserts the *untouched* slot pixels still match — i.e. the
    /// tool's dirty bbox really covered everything that changed, lighting included.
    fn assert_shows_the_cache(&self, ctx: &str) {
        let preview = self.cache.as_ref().expect("a drained composite");
        let mut expected = (*preview.rgba).clone();
        premultiply_rgba8(&mut expected);
        assert_screen_equals(self.shown(), &expected, preview.width, ctx);
    }
}

/// What the artist SHOULD be seeing: the document composited from scratch (the public structural
/// door forces the full lane — re-setting the active layer's opacity to its CURRENT value is a
/// byte-neutral `invalidate_composite`; reading it first matters, or the oracle would silently
/// EDIT a half-opacity layer to 1.0 and compare the screen against a different document — it did,
/// for one red run of the handoff gate), premultiplied the way the upload premultiplies.
pub(super) fn screen_truth(painter: &mut PainterTool) -> Vec<u8> {
    let active = painter.layers().active().expect("a layer");
    let current = painter
        .layers()
        .get(active)
        .expect("the active layer exists")
        .opacity;
    painter.set_layer_opacity(active, current);
    let (rgba, _, _) = painter
        .take_preview_arc()
        .expect("invalidate marks the preview dirty");
    let mut bytes = (*rgba).clone();
    premultiply_rgba8(&mut bytes);
    bytes
}

pub(super) fn assert_screen_equals(shown: &[u8], truth: &[u8], width: u32, ctx: &str) {
    assert_eq!(shown.len(), truth.len(), "{ctx}: buffer sizes");
    let (mut differing, mut worst, mut first) = (0usize, 0i32, None);
    for i in 0..shown.len() {
        let d = (i32::from(shown[i]) - i32::from(truth[i])).abs();
        if d != 0 {
            differing += 1;
            worst = worst.max(d);
            first.get_or_insert(i);
        }
    }
    if differing > 0 {
        let i = first.expect("differing > 0");
        let px = (i / 4) as u32;
        panic!(
            "{ctx}: the screen diverged from the tool at {differing} bytes (first at \
             ({}, {}) channel {}, worst {worst} levels) — what the artist sees is not what the \
             tool painted",
            px % width,
            px / width,
            i % 4
        );
    }
}

/// **D(a) — the Anchored stroke's relief must still be on screen after pen-up.**
///
/// The symptom (Enio, live app): drag with the Anchored method — the drag sizes one disc from the
/// press point — and the lit relief is there; lift the pen and it vanishes. The tool keeps it (its
/// own pen-up gate, `impasto_smoothing_settles_every_stroke_...`, holds the post-up frame equal to
/// a full recompose). Here the SAME gesture runs through the shell's drain → plan → slot protocol,
/// frame by frame at the app's cadence, and the slot is held to the from-scratch truth after the
/// up — plus one idle frame, because a one-frame lag is invisible but a slot nobody updates again
/// IS the bug.
#[test]
fn the_screen_shows_the_anchored_strokes_relief_after_pen_up() {
    let size = 240u32;
    let mut t = impasto_tool(size);
    t.set_brush_stroke_method(2); // Anchored
    let mut screen = Screen::new();
    screen.frame(&mut t); // the bind's first drain seeds the slot (source-push marks dirty)
    let seed = screen.shown().to_vec();

    t.on_canvas_pointer(cp([120.0, 120.0], PointerPhase::Down));
    screen.frame(&mut t);
    for i in 1u8..=5 {
        t.on_canvas_pointer(cp([120.0 + 8.0 * f32::from(i), 120.0], PointerPhase::Move));
        screen.frame(&mut t);
        screen.assert_shows_the_cache("Anchored mid-drag");
    }
    let mid_drag = screen.shown().to_vec();
    assert_ne!(
        mid_drag, seed,
        "fixture: the drag painted something on screen"
    );

    t.on_canvas_pointer(cp([160.0, 120.0], PointerPhase::Up));
    screen.frame(&mut t);
    screen.frame(&mut t); // idle frame — the settle may only reach the screen here

    let truth = screen_truth(&mut t);
    assert_ne!(
        truth, seed,
        "fixture: the committed stroke exists in the document (else 'still on screen' is vacuous)"
    );
    assert_screen_equals(screen.shown(), &truth, size, "Anchored after pen-up");
}

/// **The relief the screen shows is LIT relief — the presence sibling.**
///
/// Every equality above is vacuous if the composite never carried shading in the first place (an
/// unlit pipeline agrees with an unlit truth). So: the same committed Anchored stroke must render
/// DIFFERENTLY with the light pass on vs off. This is the anti-vacuity anchor for the whole file
/// ([[feedback_absence_gate_needs_a_presence_sibling]]).
#[test]
fn the_anchored_strokes_screen_pixels_carry_the_light() {
    let size = 240u32;
    let mut t = impasto_tool(size);
    t.set_brush_stroke_method(2);
    let mut screen = Screen::new();
    screen.frame(&mut t);
    t.on_canvas_pointer(cp([120.0, 120.0], PointerPhase::Down));
    screen.frame(&mut t);
    t.on_canvas_pointer(cp([150.0, 120.0], PointerPhase::Move));
    screen.frame(&mut t);
    t.on_canvas_pointer(cp([150.0, 120.0], PointerPhase::Up));
    screen.frame(&mut t);
    screen.frame(&mut t);

    let lit = screen_truth(&mut t);
    t.toggle_impasto_show();
    let unlit = screen_truth(&mut t);
    assert_ne!(
        lit, unlit,
        "the committed stroke must be SHADED on screen — light off changes nothing, so either the \
         stroke laid no relief or the composite never carried the light, and every byte-equality \
         in this file is comparing two unlit canvases"
    );
}

/// **D(b) — a jittered stroke's screen must track the tool frame by frame.**
///
/// The symptom (Enio, live app): with per-dab Jitter armed the relief comes out stretched. Jitter
/// Scale + Rotate churn small, scattered rects all along the stroke — the heaviest client of the
/// B.1 partial-upload lane. A patch rect that is displaced, undersized, or that misses pixels the
/// LIGHT changed (shading reaches past the stamped bbox: the normal reads neighbours) leaves stale
/// bytes standing next to fresh ones — which on a relief reads exactly as smearing/stretching.
/// The pair oracle after every frame holds every slot byte, patched and untouched alike, to the
/// composite the tool just handed over; the end oracle holds the final screen to a from-scratch
/// recompose.
#[test]
fn the_screen_tracks_the_tool_through_a_jittered_stroke_frame_by_frame() {
    let size = 240u32;
    let mut t = impasto_tool(size);
    t.set_brush_jitter_scale(1.0);
    t.set_brush_jitter_rotate(1.0);
    let mut screen = Screen::new();
    screen.frame(&mut t);

    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    screen.frame(&mut t);
    screen.assert_shows_the_cache("jitter frame 0 (down)");
    for i in 1u8..=10 {
        t.on_canvas_pointer(cp(
            [40.0 + 16.0 * f32::from(i), 40.0 + 14.0 * f32::from(i)],
            PointerPhase::Move,
        ));
        screen.frame(&mut t);
        screen.assert_shows_the_cache("jitter mid-drag");
    }
    t.on_canvas_pointer(cp([200.0, 180.0], PointerPhase::Up));
    screen.frame(&mut t);
    screen.frame(&mut t);

    let truth = screen_truth(&mut t);
    assert_screen_equals(screen.shown(), &truth, size, "jittered stroke after pen-up");
}

/// **Every stroke method's screen survives its own commit path — the table.**
///
/// The tool's pen-up bug lived in the paths nobody connected (the five shape methods never
/// committed), and its gate is a table over all ten methods for exactly that reason. The screen's
/// gate takes the same shape: for every method, the same drain-per-event cadence, the pair oracle
/// on every frame and the from-scratch truth at the end — through pen-up for the freehand five,
/// and through the OPEN shape for the editor five (an open shape is a live preview with an Apply
/// button; the screen must show it too). A method whose commit forgets to tell the preview — or
/// tells it with a bbox smaller than what the settle re-lit — goes red here by name.
#[test]
fn the_screen_tracks_every_stroke_method_through_its_commit() {
    let size = 240u32;
    for method in 0u8..=9 {
        let mut t = impasto_tool(size);
        t.set_brush_stroke_method(method);
        let mut screen = Screen::new();
        screen.frame(&mut t);
        let seed = screen.shown().to_vec();

        if method == 5 {
            // The Line is a POLYLINE — authored click-by-click; a drag paints nothing (the tool
            // table gate paid this lesson).
            for p in [[60.0f32, 120.0], [180.0, 120.0]] {
                t.on_canvas_pointer(cp(p, PointerPhase::Down));
                screen.frame(&mut t);
                t.on_canvas_pointer(cp(p, PointerPhase::Up));
                screen.frame(&mut t);
                screen.assert_shows_the_cache("Line click");
            }
        } else {
            t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Down));
            screen.frame(&mut t);
            for i in 1u8..=6 {
                t.on_canvas_pointer(cp([60.0 + 18.0 * f32::from(i), 120.0], PointerPhase::Move));
                screen.frame(&mut t);
                screen.assert_shows_the_cache("mid-drag");
            }
            t.on_canvas_pointer(cp([168.0, 120.0], PointerPhase::Up));
            screen.frame(&mut t);
        }
        screen.frame(&mut t); // idle frame

        let truth = screen_truth(&mut t);
        assert_ne!(
            truth, seed,
            "method {method}: fixture painted nothing — the equality below would be vacuous"
        );
        assert_screen_equals(
            screen.shown(),
            &truth,
            size,
            &format!("stroke method {method} after its gesture"),
        );
    }
}

/// Deposit two thick ridges (relief for the spatula to work), then enter **Sculpt** — the actual
/// context of the 2026-07-14 smoke: both display symptoms were reported while SCULPTING, and the
/// sculpt's commit path (`sculpt_session`, per-stroke law, settle-at-up) is entirely different
/// plumbing from the deposit's. Every event is followed by a frame, at the app's cadence.
fn sculpted_setup(size: u32, screen: &mut Screen) -> PainterTool {
    let mut t = impasto_tool(size);
    screen.frame(&mut t);
    for y in [100.0f32, 140.0] {
        t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
        screen.frame(&mut t);
        for i in 1u8..=8 {
            t.on_canvas_pointer(cp([40.0 + 20.0 * f32::from(i), y], PointerPhase::Move));
            screen.frame(&mut t);
        }
        t.on_canvas_pointer(cp([200.0, y], PointerPhase::Up));
        screen.frame(&mut t);
    }
    t.set_paint_tool_mode("sculpt");
    t.set_sculpt_mode(7); // Inflate — the smoke's protagonist that whole journey
    t.set_brush_strength(1.0);
    t
}

/// **D(a), in the smoke's own context: a SCULPT stroke through the Anchored method must still be
/// on screen after pen-up.** The sculpt re-renders from a frozen `pre` and commits at the up — a
/// commit that swaps buffers without touching a pixel of pigment is exactly the shape of bug that
/// leaves the tool's buffers right and the screen stale (the tool's own 19601 lesson, one path
/// over). Frame-per-event cadence; pair oracle mid-gesture; from-scratch truth after the up.
#[test]
fn the_screen_shows_the_sculpted_relief_after_an_anchored_pen_up() {
    let size = 240u32;
    let mut screen = Screen::new();
    let mut t = sculpted_setup(size, &mut screen);
    let before_sculpt = screen_truth(&mut t);
    screen.frame(&mut t); // drain the truth oracle's own invalidation before the gesture

    t.set_brush_stroke_method(2); // Anchored, armed on the Sculpt mode's brush slot
    t.on_canvas_pointer(cp([120.0, 100.0], PointerPhase::Down));
    screen.frame(&mut t);
    for i in 1u8..=5 {
        t.on_canvas_pointer(cp([120.0 + 8.0 * f32::from(i), 100.0], PointerPhase::Move));
        screen.frame(&mut t);
        screen.assert_shows_the_cache("Anchored sculpt mid-drag");
    }
    t.on_canvas_pointer(cp([160.0, 100.0], PointerPhase::Up));
    screen.frame(&mut t);
    screen.frame(&mut t);

    let truth = screen_truth(&mut t);
    assert_ne!(
        truth, before_sculpt,
        "fixture: the Anchored sculpt changed nothing on the lit canvas — the equality below is \
         vacuous (arming dead? light off?)"
    );
    assert_screen_equals(screen.shown(), &truth, size, "Anchored sculpt after pen-up");
}

/// **D(b), in the smoke's own context: a JITTERED sculpt tracks the tool frame by frame.** Jitter
/// Scale/Rotate on the spatula churns the same scattered small rects as on the brush, but through
/// the sculpt's own render (memo tiles + light). Any patch displaced or under-covering what the
/// light re-shaded reads as smeared/stretched relief on screen.
#[test]
fn the_screen_tracks_a_jittered_sculpt_frame_by_frame() {
    let size = 240u32;
    let mut screen = Screen::new();
    let mut t = sculpted_setup(size, &mut screen);
    let before_sculpt = screen_truth(&mut t);
    screen.frame(&mut t);

    t.set_brush_jitter_scale(1.0);
    t.set_brush_jitter_rotate(1.0);
    t.on_canvas_pointer(cp([50.0, 120.0], PointerPhase::Down));
    screen.frame(&mut t);
    screen.assert_shows_the_cache("jittered sculpt frame 0");
    for i in 1u8..=10 {
        t.on_canvas_pointer(cp([50.0 + 15.0 * f32::from(i), 120.0], PointerPhase::Move));
        screen.frame(&mut t);
        screen.assert_shows_the_cache("jittered sculpt mid-drag");
    }
    t.on_canvas_pointer(cp([200.0, 120.0], PointerPhase::Up));
    screen.frame(&mut t);
    screen.frame(&mut t);

    let truth = screen_truth(&mut t);
    assert_ne!(
        truth, before_sculpt,
        "fixture: the jittered sculpt changed nothing on the lit canvas — vacuous"
    );
    assert_screen_equals(screen.shown(), &truth, size, "jittered sculpt after pen-up");
}

/// **The plan itself refuses the stale-skip and the mismatched patch** — the two protocol
/// mutations that would silently freeze or misplace the screen. Pure-decision checks (no tool):
/// a changed buffer identity must never plan `Skip`; a dims change must never plan `Partial`.
#[test]
fn the_upload_plan_never_skips_a_new_composite_nor_patches_a_mismatched_slot() {
    let rgba = Arc::new(vec![0u8; 16 * 16 * 4]);
    let preview = PainterPreview {
        entity_bits: ENTITY,
        rgba: Arc::clone(&rgba),
        width: 16,
        height: 16,
    };
    let seeded = PainterPreviewGpu {
        texture_id: 7,
        width: 16,
        height: 16,
        arc_token: Arc::as_ptr(&preview.rgba) as usize,
        entity_bits: ENTITY,
    };
    // Same buffer, same identity → Skip (the idle-frame no-op).
    assert_eq!(
        plan_upload(&preview, Some(seeded), None, false),
        UploadPlan::Skip
    );
    // A NEW composite buffer (new Arc) with a stale token must upload — Skip here is the frozen
    // screen.
    let fresh = PainterPreview {
        entity_bits: ENTITY,
        rgba: Arc::new(vec![1u8; 16 * 16 * 4]),
        width: 16,
        height: 16,
    };
    assert_eq!(
        plan_upload(&fresh, Some(seeded), None, false),
        UploadPlan::Full { reuse: Some(7) }
    );
    // A dirty bbox against a slot of DIFFERENT dims must not patch (the slot pixels outside the
    // rect belong to another geometry) — full re-seed instead.
    let grown = PainterPreview {
        entity_bits: ENTITY,
        rgba: Arc::new(vec![1u8; 32 * 32 * 4]),
        width: 32,
        height: 32,
    };
    assert_eq!(
        plan_upload(&grown, Some(seeded), Some((0, 0, 8, 8)), false),
        UploadPlan::Full { reuse: Some(7) }
    );
    // The happy partial: seeded slot, matching dims, in-bounds rect.
    let patched = PainterPreview {
        entity_bits: ENTITY,
        rgba: Arc::new(vec![2u8; 16 * 16 * 4]),
        width: 16,
        height: 16,
    };
    assert_eq!(
        plan_upload(&patched, Some(seeded), Some((4, 4, 8, 8)), false),
        UploadPlan::Partial {
            texture_id: 7,
            rect: (4, 4, 8, 8)
        }
    );
}
