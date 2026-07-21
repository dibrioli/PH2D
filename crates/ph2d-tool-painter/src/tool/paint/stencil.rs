//! Stencil texture **handle editing** — drag the on-canvas overlay to move / resize the image-space
//! stencil rect (Angle comes from the card's number box, so no `atan2` / transcendental). The rect's
//! frame lives in the brush's DEDICATED [`ph2d_painter_brush::TextureSettings`] `stencil_*` fields
//! (`stencil_offset` = centre, `stencil_size` = extent, `stencil_angle_deg`) — independent of the
//! texture tiling — so this editor has no session state of its own; only the in-progress grab is held
//! in [`super::PaintState`]. Grabbing a handle consumes the pointer; a drag that starts away from
//! every handle falls through to normal painting (the handles disambiguate — no modifier needed).
//! The Stencil card's number boxes write the same `stencil_*` fields via [`PainterTool::route_brush_stencil_event`].

use super::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PanelEvent, PointerPhase};
use ph2d_painter_brush::{
    BrushSpec, ImageMask, TEX_ANGLE_MAX_DEG, render_stencil_preview, stencil_frame,
};
use std::sync::Arc;

/// Snapshot of everything that changes how the pending shape stroke LOOKS — the brush spec plus the
/// Colour-Ramp / texture / shape state that the `BrushSpec` doesn't carry. Compared before/after a panel
/// event to drive the real-time re-fill ([`PainterTool::appearance_sig`]).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct AppearanceSig {
    brush: BrushSpec,
    tex_ramp_enabled: bool,
    tex_ramp_bw: bool,
    shape_ramp_enabled: bool,
    shape_ramp_bw: bool,
    tex_image_version: u64,
    shape_image_version: u64,
    shape_ramp_version: u64,
    layers_version: u64,
    tex_ramp_len: usize,
    shape_ramp_len: usize,
    ramp_dirty: bool,
    /// The Offset slider track — changing it must re-fill the open shape (it shifts the stamped path).
    shape_offset: f32,
    /// The Offset **Trim** checkbox — toggling it must re-fill (it cuts the spine's self-intersections).
    offset_trim: bool,
    /// The two ramps' **Alpha Mode**. Their setters deliberately skip the LUT dirty flag ("the mode is
    /// applied at STAMP time, no re-bake needed") — correct for the LUT, and the reason this signature was
    /// written without them. But an OPEN shape editor's preview *is* a pending stamp: changing Alpha Mode
    /// with a Curve on screen changed nothing until some other knob moved (sweep 2026-07-12).
    tex_ramp_alpha: ph2d_painter_brush::RampAlphaMode,
    shape_ramp_alpha: ph2d_painter_brush::RampAlphaMode,
    /// **Tiling** — same story: "it only affects future stamps" is true of the canvas, but the open shape's
    /// preview is a stamp that has not landed yet, and Tiling wraps it across the sprite edges.
    tiling: [bool; 2],
}

/// The rotate ring reaches this multiple of the scale-grab radius PAST each corner — a click inside the
/// corner scales, a click in the ring just outside it rotates (the sprite gizmo's feel; Enio 2026-06-26).
const STENCIL_ROTATE_BAND: f32 = 2.6;
/// Seconds the in-gizmo texture preview lingers after a panel param change before it fades out (the
/// handle DRAG shows it via the live grab instead, so the drag path clears this on release).
pub(super) const STENCIL_PREVIEW_HOLD_S: f32 = 0.35;
/// Longest side (px) of the rendered in-gizmo preview buffer; the other axis tracks the rect aspect.
const STENCIL_PREVIEW_MAX: u32 = 192;

/// The stencil overlay snapshot for the shell: the rect's 4 corners + centre (image-space px), the
/// grabbed handle, and the hit radii so the shell can show the rotate cue. Handle indices: `0..=3`
/// corners (scale OR rotate), `4` centre (move).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StencilOverlay {
    /// The rect's 4 corners in image-space px (`[--, +-, ++, -+]` along the rotated axes).
    pub corners: [[f32; 2]; 4],
    /// The rect centre in image-space px (the move handle).
    pub center: [f32; 2],
    /// The grabbed handle (`0..=3` corner, `4` centre) for highlighting, or `None`.
    pub grabbed: Option<u8>,
    /// The active grab is a rotation — the shell draws the corners as rotate circles, not scale squares.
    pub rotating: bool,
    /// Image-px hit radius: `≤ scale_tol_px` from a corner = scale; the ring `scale_tol_px..=rotate_tol_px`
    /// just outside it = rotate. The shell scales these by the sprite footprint for the hover cue.
    pub scale_tol_px: f32,
    /// Outer edge of the rotate ring (image px) — see [`Self::scale_tol_px`].
    pub rotate_tol_px: f32,
}

/// A transient texture preview to blit INSIDE the gizmo while the user transforms it. `rgba` is RGBA8
/// `w`×`h` in the rect's LOCAL (axis-aligned) frame; the shell maps buffer-px → image-px via `center`
/// ± `half` along `u` (and its left perpendicular), then to screen through the sprite affine.
#[derive(Clone, Debug)]
pub struct StencilPreview {
    /// The rendered grain pattern (RGBA8, row-major), shareable so the shell holds it without a copy.
    pub rgba: Arc<Vec<u8>>,
    /// Preview buffer width in px.
    pub w: u32,
    /// Preview buffer height in px.
    pub h: u32,
    /// The rect centre in image-space px.
    pub center: [f32; 2],
    /// The rect half-extent in image-space px (per rotated axis).
    pub half: [f32; 2],
    /// The rect's rotation unit vector `[cos, sin]` (its local +x axis).
    pub u: [f32; 2],
}

/// The in-progress stencil drag. Held in [`super::PaintState`] only between pointer-down and up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum StencilGrab {
    /// Dragging the centre handle moves the rect; the cursor keeps its grab offset from the centre.
    Move { offset: [f32; 2] },
    /// Dragging corner `0..=3` scales the rect about its centre. `start_size` is the rect's Size at grab,
    /// so a **uniform** (Shift) scale can preserve the aspect ratio by the dominant-axis factor.
    Scale { corner: u8, start_size: [f32; 2] },
    /// Dragging the ring just outside corner `0..=3` rotates the rect about its centre. `grab_angle` is
    /// the pointer's angle (rad) at grab; `start_deg` the stencil angle then — the drag adds the delta.
    Rotate {
        corner: u8,
        grab_angle: f32,
        start_deg: f32,
    },
}

impl StencilGrab {
    /// The handle index for overlay highlighting (`4` = centre, else the corner).
    fn handle(self) -> u8 {
        match self {
            Self::Move { .. } => 4,
            Self::Scale { corner, .. } | Self::Rotate { corner, .. } => corner,
        }
    }

    /// Whether this grab rotates the rect (drives the corner square→circle cue).
    fn is_rotate(self) -> bool {
        matches!(self, Self::Rotate { .. })
    }
}

impl PainterTool {
    /// `true` when the Stencil handle editor is live (a texture is assigned and mapped Stencil).
    pub(super) fn stencil_edit_active(&self) -> bool {
        let t = &self.paint.brush.texture;
        t.is_active() && t.mapping.is_stencil()
    }

    /// Set whether a Stencil corner scale is UNIFORM (Shift held) this event — the shell forwards the live
    /// Shift state before each pointer event, like [`PainterTool::set_line_constrain`].
    pub fn set_uniform_scale(&mut self, on: bool) {
        self.paint.scale_uniform = on;
    }

    /// Route a canvas pointer through the Stencil handle editor. Returns `true` only when it
    /// consumes the event (a handle is grabbed / a drag is in progress); otherwise the caller paints.
    pub(super) fn stencil_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.stencil_down(ev.pos),
            PointerPhase::Move => self.stencil_drag(ev.pos),
            PointerPhase::Up => self.stencil_up(),
            PointerPhase::Hover => false,
        }
    }

    /// Pointer-down: grab the handle under the cursor — a corner (scale), the ring just outside a corner
    /// (rotate), or the centre (move), in that priority. Returns `false` when nothing is grabbed, so the
    /// caller paints normally.
    fn stencil_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        let rot_tol = tol * STENCIL_ROTATE_BAND;
        let Some(o) = self.stencil_overlay() else {
            return false;
        };
        // Inner ring on a corner = scale.
        for (i, &c) in o.corners.iter().enumerate() {
            if dist(c, pos) <= tol {
                self.paint.stencil_grab = Some(StencilGrab::Scale {
                    corner: i as u8,
                    start_size: self.paint.brush.texture.stencil_size,
                });
                return true;
            }
        }
        // Ring just OUTSIDE a corner = rotate about the centre. "Outside" = within the band AND farther
        // from the centre than the corner itself, so points inside the rect near a corner still paint.
        for (i, &c) in o.corners.iter().enumerate() {
            let d = dist(c, pos);
            if d > tol && d <= rot_tol && dist(o.center, pos) > dist(o.center, c) {
                let grab_angle = (pos[1] - o.center[1]).atan2(pos[0] - o.center[0]);
                let start_deg = f32::from(self.paint.brush.texture.stencil_angle_deg);
                self.paint.stencil_grab = Some(StencilGrab::Rotate {
                    corner: i as u8,
                    grab_angle,
                    start_deg,
                });
                return true;
            }
        }
        if dist(o.center, pos) <= tol {
            self.paint.stencil_grab = Some(StencilGrab::Move {
                offset: [pos[0] - o.center[0], pos[1] - o.center[1]],
            });
            return true;
        }
        false
    }

    /// Pointer-move: apply the grabbed handle's edit to the brush's stencil frame.
    fn stencil_drag(&mut self, pos: [f32; 2]) -> bool {
        let Some(grab) = self.paint.stencil_grab else {
            return false;
        };
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return false;
        }
        let canvas = [w as f32, h as f32];
        let (center, _half, u) = stencil_frame(&self.paint.brush.texture, canvas);
        let v = [-u[1], u[0]];
        match grab {
            StencilGrab::Move { offset } => {
                let nc = [pos[0] - offset[0], pos[1] - offset[1]];
                // centre px → stencil Offset in [-1, 1].
                self.set_brush_stencil_offset(0, nc[0] / canvas[0] * 2.0 - 1.0);
                self.set_brush_stencil_offset(1, nc[1] / canvas[1] * 2.0 - 1.0);
            }
            StencilGrab::Scale { start_size, .. } => {
                // Half-extent = |projection of (cursor − centre) onto each rotated axis|; symmetric
                // about the centre. stencil Size = 2·half / canvas.
                let rel = [pos[0] - center[0], pos[1] - center[1]];
                let du = (rel[0] * u[0] + rel[1] * u[1]).abs();
                let dv = (rel[0] * v[0] + rel[1] * v[1]).abs();
                let (mut nsx, mut nsy) = (2.0 * du / canvas[0], 2.0 * dv / canvas[1]);
                // Shift = UNIFORM scale (the Sprite gizmo's aspect-lock): keep the grab-time aspect ratio
                // by applying the dominant-axis factor (larger relative change) to both axes.
                if self.paint.scale_uniform {
                    let rx = ratio(nsx, start_size[0]);
                    let ry = ratio(nsy, start_size[1]);
                    let f = if (rx - 1.0).abs() > (ry - 1.0).abs() {
                        rx
                    } else {
                        ry
                    };
                    nsx = start_size[0] * f;
                    nsy = start_size[1] * f;
                }
                self.set_brush_stencil_size(0, nsx);
                self.set_brush_stencil_size(1, nsy);
            }
            StencilGrab::Rotate {
                grab_angle,
                start_deg,
                ..
            } => {
                // Add the pointer's angle delta (about the centre) to the angle at grab; wrap to `0..360`.
                let cur = (pos[1] - center[1]).atan2(pos[0] - center[0]);
                let delta_deg = (cur - grab_angle).to_degrees();
                let target = (start_deg + delta_deg).rem_euclid(f32::from(TEX_ANGLE_MAX_DEG));
                self.set_brush_stencil_angle(target);
            }
        }
        true
    }

    /// Pointer-up: release the grab. Returns `true` when one was active (so the event is consumed). Also
    /// clears the in-gizmo preview at once — the drag showed it live; releasing should end it crisply
    /// (the panel-param path keeps its own short linger instead).
    fn stencil_up(&mut self) -> bool {
        let had = self.paint.stencil_grab.take().is_some();
        if had {
            self.paint.stencil_preview_s = 0.0;
        }
        had
    }

    /// Route the **Stencil card** number boxes (Size X/Y, Offset X/Y, Rotation) to the brush's
    /// dedicated `stencil_*` setters. Brush-only (texture LAYERS never expose the Stencil mapping), so
    /// it runs unconditionally before the brush-texture chain. Returns `true` iff the event was a
    /// stencil field (so the caller stops routing).
    pub(crate) fn route_brush_stencil_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        let PanelEvent::SetValue(id, v) = event else {
            return false;
        };
        let v = *v as f32;
        match *id {
            x if x == core_ids::PAINTER_BRUSH_STENCIL_SIZE_X => self.set_brush_stencil_size(0, v),
            x if x == core_ids::PAINTER_BRUSH_STENCIL_SIZE_Y => self.set_brush_stencil_size(1, v),
            x if x == core_ids::PAINTER_BRUSH_STENCIL_OFFSET_X => {
                self.set_brush_stencil_offset(0, v)
            }
            x if x == core_ids::PAINTER_BRUSH_STENCIL_OFFSET_Y => {
                self.set_brush_stencil_offset(1, v)
            }
            x if x == core_ids::PAINTER_BRUSH_STENCIL_ANGLE => self.set_brush_stencil_angle(v),
            _ => return false,
        }
        true
    }

    /// The current **appearance signature** — everything that changes how the pending stroke LOOKS but
    /// lives outside the `BrushSpec`: the Colour-Ramp enable/B&W flags + stop edits (`*_ramp_dirty`), the
    /// imported Grain/Shape image versions, the Shape ramp version, and the per-layer-colour version.
    /// Snapshot it before a panel event so [`Self::refill_if_appearance_changed`] re-fills an open shape
    /// when ANY of these change (a Color-Ramp toggle / stop edit / image assign updates the preview live,
    /// not only on a gizmo nudge — Enio 2026-06-27).
    pub(crate) fn appearance_sig(&self) -> AppearanceSig {
        AppearanceSig {
            brush: self.paint.brush,
            tex_ramp_enabled: self.paint.texture_ramp_enabled,
            tex_ramp_bw: self.paint.texture_ramp_bw,
            shape_ramp_enabled: self.paint.shape_color_ramp_enabled,
            shape_ramp_bw: self.paint.shape_color_ramp_bw,
            tex_image_version: self.paint.texture_image_version,
            shape_image_version: self.paint.shape_image_version,
            shape_ramp_version: self.paint.shape_ramp_version,
            layers_version: self.paint.shape_layers.version(),
            tex_ramp_len: self.paint.texture_ramp.len(),
            shape_ramp_len: self.paint.shape_color_ramp.len(),
            ramp_dirty: self.paint.texture_ramp_dirty || self.paint.shape_ramp_dirty,
            shape_offset: self.paint.shape_offset_norm,
            offset_trim: self.paint.offset_trim,
            tex_ramp_alpha: self.paint.texture_ramp_alpha_mode,
            shape_ramp_alpha: self.paint.shape_color_ramp_alpha_mode,
            tiling: self.paint.tiling,
        }
    }

    /// Re-fill the open shape editor's painted preview (Curve / Free Hand / Ellipse / Polygon / Line) with
    /// the current brush — each refill no-ops unless its editor is open + editing, so at most one runs.
    pub(crate) fn refill_open_shape(&mut self) {
        self.curve_refill();
        self.ellipse_refill();
        self.polygon_refill();
        self.line_refill();
    }

    /// Re-fill the open shape editor IFF its appearance changed since `before` — so a size / spacing /
    /// flow / falloff tweak OR a Colour-Ramp / texture / shape edit updates the pending stroke in REAL
    /// TIME. Apply/Delete clicks + layer ops touch none of it → no re-stamp.
    pub(crate) fn refill_if_appearance_changed(&mut self, before: AppearanceSig) {
        if self.appearance_sig() != before {
            self.refill_open_shape();
        }
    }

    /// Set the active paint operation from the left-rail tool selection: `"smear"` → the Smear drag,
    /// `"blur"` → the Blur/soften, `"eraser"` → normal paint with the Erase-Alpha override, anything
    /// else → normal Brush paint. Keeps `paint_mode` + the eraser override in sync so switching rail
    /// tools never leaves a stuck state (e.g. Brush after Smear returns to normal painting). Beside
    /// `route_brush_dab_event`, which drives it (moved here off `brush_settings.rs` for the LOC cap).
    pub fn set_paint_tool_mode(&mut self, mode: &str) {
        // Leaving Fill with a live ColorDrop → commit it (push its undo) so switching tools never loses
        // the fill or its undo step. No-op unless a fill is in progress.
        self.fill_commit();
        // The transient Mask scratch is NOT discarded on a tool switch: it persists while its target
        // layer stays active (`mask_scratch_active()` self-gates on target == active), so you can leave
        // Mask, retouch the concealed area with the Brush, and return. Apply promotes it to a layer mask;
        // switching layers lets it go dormant. (This setter is on `arch_mode_has_reconcile`'s BENIGN list:
        // it writes only simple mode fields, with no derived cache to settle.)
        // Any tool switch disarms a pending Eyedropper pick; only "eyedropper" re-arms it below.
        self.paint.eyedropper_armed = false;
        use super::PaintMode;
        let new_mode = match mode {
            "smear" => PaintMode::Smear,
            "blur" => PaintMode::Blur,
            "clone" => PaintMode::Clone,
            "mask" => PaintMode::Mask,
            "inpaint" => PaintMode::Inpaint,
            "fill" => PaintMode::Fill,
            "selection" => PaintMode::Selection,
            "deform" => PaintMode::Deform,
            "sculpt" => PaintMode::Sculpt,
            "knife" => PaintMode::Knife,
            "wetpaint" => PaintMode::WetPaint,
            // "eraser" INSIDE Wet Paint STAYS in Wet Paint (W2.6): the fluid
            // engine has its own eraser (`Tool::Erase`) and leaving the mode
            // would BAKE the very painting the artist is trying to correct —
            // the water must survive its eraser (Rebelle's semantics). The
            // same-mode transition skips every teardown below (`old != new`
            // guards), so the session lives on.
            "eraser" if self.paint.paint_mode == PaintMode::WetPaint => PaintMode::WetPaint,
            // With the Wet Paint checkbox ARMED, "brush" IS the fluid — this
            // is what makes the arm survive tool round-trips (eraser /
            // selection / smear and back), the Watercolor/Impasto pattern
            // (Enio 2026-07-21). Only the explicit "brush" wire: eyedropper
            // stays a momentary Paint, and unknown wires keep their
            // conservative fallback.
            "brush" if self.paint.wetpaint.armed => PaintMode::WetPaint,
            // "brush" / "eraser" / "eyedropper" / anything else → normal Paint.
            _ => PaintMode::Paint,
        };
        // Independent-tools model: load the target mode's OWN brush settings (Smear/Blur/Clone slots seed
        // Spacing 5% for dense dabs — see `state_default`). No-op when settings are linked. Must run while
        // `paint_mode` still holds the OLD mode, so the current tool's edits save to the right slot.
        self.switch_brush_slot(new_mode);
        // Leaving Deform bakes any live Transform float (committing it as one undo entry) and ends the
        // Reshape session (drops the `pre`/displacement) so a later mode's edits aren't re-warped from a
        // stale baseline; the Reshape pixels are already committed per-stroke.
        // Leaving the Smear ends its warp session for the same reason: a `disp` and a frozen `pre`
        // that belong to one tool must never be read by another, and `pre` describes a canvas that
        // the next tool's edits will move out from under it.
        if self.paint.paint_mode.smears() && !new_mode.smears() {
            self.end_warp_session();
        }
        if self.paint.paint_mode == PaintMode::Deform && new_mode != PaintMode::Deform {
            self.end_transform(true);
            self.end_deform_session();
        }
        // ENTERING Deform: the temperament opens UNSELECTED — the artist must pick Reshape or Transform
        // each time the panel is entered, so re-picking Transform always re-lifts a fresh gizmo (Enio
        // 2026-07-04). Any prior transform was already baked on the last leave.
        if self.paint.paint_mode != PaintMode::Deform && new_mode == PaintMode::Deform {
            self.paint.deform.temperament = super::DEFORM_TEMPERAMENT_NONE;
        }
        // Leaving Sculpt parks nothing: the carving is already committed to the layer (the sculpt writes
        // `heights` live, it does not stage it), so all that is left to do is let go of the frozen source.
        // Keeping it would mean a Radius drag made in some OTHER tool re-rendered a stroke the artist has
        // moved on from — a knob reaching back through time.
        if self.paint.paint_mode == PaintMode::Sculpt && new_mode != PaintMode::Sculpt {
            self.end_sculpt_session();
        }
        // Leaving Wet Paint ends the fluid session: the last composite already IS the canvas (ending
        // is the bake), and the sim must stop moving paint the moment another tool owns the pixels.
        if self.paint.paint_mode == PaintMode::WetPaint && new_mode != PaintMode::WetPaint {
            self.wetpaint_end_session();
        }
        self.paint.paint_mode = new_mode;
        // Entering Wet Paint by ANY door arms the checkbox — painting wet
        // with the Enable reading OFF would be a lying checkbox. Leaving
        // does NOT disarm (the arm is the persistent authored state; only
        // the checkbox / its reset disarm), exactly like Watercolor's flag.
        if new_mode == PaintMode::WetPaint {
            self.paint.wetpaint.armed = true;
        }
        // The brush slot has just been swapped underneath us, so the "does anything read `Dab::dir`?" answer
        // has to be re-asked: leaving Sculpt must clear the Chisel's heading need, entering it must restore
        // it. (The slot round-trips the flag, but the LIVE brush is what `Stroke::new` reads.)
        self.sync_stroke_heading_need();
        self.arm_tool_falloff_defaults();
        // Leaving the Selection tool auto-hides its gizmos (the "Show Selection Gizmos" checkbox unchecks) —
        // the gizmos belong to Select and would otherwise linger over another tool (Enio 2026-07-03).
        if new_mode != PaintMode::Selection && self.paint.selection_edit_mode {
            self.exit_selection_edit();
        }
        // Per-mode flags. Smear/Blur/Clone leave the eraser override as-is (unchanged behaviour); the
        // colour-painting modes clear it; Eyedropper additionally arms the pick.
        match mode {
            // Sculpt joins Smear/Blur/Clone here for the same reason: it processes what is already on the
            // canvas rather than laying colour, so the Eraser override is none of its business.
            "smear" | "blur" | "clone" | "sculpt" => {}
            "eyedropper" => {
                self.paint.eraser = false;
                self.paint.eyedropper_armed = true;
            }
            "eraser" => self.paint.eraser = true,
            // mask / inpaint / fill / brush / default.
            _ => self.paint.eraser = false,
        }
    }

    /// The active paint mode as the `set_paint_tool_mode` string id — its inverse, used by the shell to
    /// CAPTURE the current tool before a momentary Fill drag and RESTORE it after (Enio 2026-07-03).
    #[must_use]
    pub fn active_paint_mode_id(&self) -> &'static str {
        use super::PaintMode;
        match self.paint.paint_mode {
            PaintMode::Smear => "smear",
            PaintMode::Blur => "blur",
            PaintMode::Clone => "clone",
            PaintMode::Mask => "mask",
            PaintMode::Inpaint => "inpaint",
            PaintMode::Fill => "fill",
            PaintMode::Selection => "selection",
            PaintMode::Deform => "deform",
            PaintMode::Sculpt => "sculpt",
            PaintMode::Knife => "knife",
            // The wet eraser is still the ERASER in the artist's hand — the
            // rail radio must light that chip, not the brush (W2.6).
            PaintMode::WetPaint if self.paint.eraser => "eraser",
            // Wet Paint is the BRUSH's flavour (the checkbox, like
            // Watercolor) — the rail lights Brush, and a momentary
            // capture/restore round-trips through "brush" back into the
            // fluid because the arm persists.
            PaintMode::WetPaint => "brush",
            PaintMode::Paint if self.paint.eraser => "eraser",
            PaintMode::Paint => "brush",
        }
    }

    /// Whether the active paint operation is **Smear** — the panel snapshot mirrors this so the
    /// incompatible brush controls (colour / blend / ramps / eraser) hide.
    #[must_use]
    pub fn is_smear_mode(&self) -> bool {
        self.paint.paint_mode.smears()
    }

    /// Whether the active paint operation is **Blur** — the panel hides the same colour-family controls
    /// as Smear (Blur processes pixels, it paints no colour).
    #[must_use]
    pub fn is_blur_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Blur)
    }

    /// Whether the active paint operation is **Mask** — paints a grayscale mask value. The panel keeps
    /// the full brush but hides Colour / Randomize / Composite and locks the ramps to B&W.
    #[must_use]
    pub fn is_mask_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Mask)
    }

    /// Whether the active paint operation is **Inpaint** — the content-aware heal brush. Like Smear/Blur
    /// it paints no colour (it marks a defect region), so the panel hides the same colour-family controls.
    #[must_use]
    pub fn is_inpaint_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Inpaint)
    }

    /// Whether the active operation is **Selection** — the panel shows ONLY the selection controls
    /// (mode-exclusive, like Inpaint), nothing shared with the other tools.
    #[must_use]
    pub fn is_selection_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Selection)
    }

    /// Whether the active operation is **Deform** — the panel shows ONLY the deform controls (mode-exclusive,
    /// like Selection): mode segmented · Size/Pressure/Distortion/Momentum/Strength · Freeze · actions.
    #[must_use]
    pub fn is_deform_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Deform)
    }

    /// Route the brush **dab/stroke** chrome events: the flatten/rotate gizmo `SetValue`s (Shape panel),
    /// plus the **Apply** / **Apply & Keep** stroke buttons (`Click` → bake the pending shape-editor stroke,
    /// discarding or keeping the editable curve). Returns `true` iff the event was consumed.
    pub(crate) fn route_brush_dab_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        // Clone-card events (Set Source / Aligned) — delegated here so `handle_panel_event`'s route
        // chain stays at its line cap (this router is already in that chain).
        if self.route_clone_event(event) {
            return true;
        }
        // Mask-section events (sub-brush / canvas ops / overlay colour) — delegated here for the same
        // route-chain-LOC reason as the Clone card above.
        if self.route_mask_event(event) {
            return true;
        }
        // Left-rail paint-tool selection → the brush operation (Brush / Eraser / Smear). Routed here
        // (a brush-operation router) so `trait_impls::handle_panel_event` stays at its LOC cap.
        if let PanelEvent::SelectOption(id, value) = event
            && *id == core_ids::PAINTER_PAINT_MODE
        {
            self.set_paint_tool_mode(value);
            return true;
        }
        if let PanelEvent::Click(id) = event {
            // Shape buttons (Apply / Apply & Keep / Delete / Edit / Operation / Simplify) → their verbs.
            if self.route_stroke_shape_button(id) {
                return true;
            }
            if *id == core_ids::PAINTER_BRUSH_OFFSET_TRIM {
                self.toggle_offset_trim();
                return true;
            }
            if *id == core_ids::PAINTER_BRUSH_SYNC {
                self.toggle_link_shared_settings(); // "Sync with other tools" (see `tool_link`)
                return true;
            }
            if *id == core_ids::PAINTER_BRUSH_LINE_DIMENSIONS {
                self.toggle_line_show_dimensions(); // Line CAD dimensions overlay (see `line_dim`)
                return true;
            }
        }
        let PanelEvent::SetValue(id, v) = event else {
            return false;
        };
        let v = *v as f32;
        match *id {
            x if x == core_ids::PAINTER_BRUSH_DAB_FLATTEN => self.set_brush_dab_flatten(v),
            x if x == core_ids::PAINTER_BRUSH_DAB_ANGLE => self.set_brush_dab_angle(v),
            _ => return false,
        }
        true
    }

    /// The stencil overlay for the shell, or `None` unless a texture is assigned and mapped Stencil.
    /// The corners are derived from [`ph2d_painter_brush::stencil_frame`] — the same frame the dab
    /// masks against — so the outline and the painted mask agree exactly.
    #[must_use]
    pub fn stencil_overlay(&self) -> Option<StencilOverlay> {
        let t = &self.paint.brush.texture;
        if !t.is_active() || !t.mapping.is_stencil() {
            return None;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        let (center, half, u) = stencil_frame(t, [w as f32, h as f32]);
        let v = [-u[1], u[0]];
        let corner = |sx: f32, sy: f32| {
            [
                center[0] + sx * half[0] * u[0] + sy * half[1] * v[0],
                center[1] + sx * half[0] * u[1] + sy * half[1] * v[1],
            ]
        };
        let tol = self.paint.shape_grab_tol_px;
        Some(StencilOverlay {
            corners: [
                corner(-1.0, -1.0),
                corner(1.0, -1.0),
                corner(1.0, 1.0),
                corner(-1.0, 1.0),
            ],
            center,
            grabbed: self.paint.stencil_grab.map(StencilGrab::handle),
            rotating: self.paint.stencil_grab.is_some_and(StencilGrab::is_rotate),
            scale_tol_px: tol,
            rotate_tol_px: tol * STENCIL_ROTATE_BAND,
        })
    }

    /// The in-gizmo Grain preview, or `None` unless the Stencil editor is live AND a transform is in
    /// progress (a handle drag, or a panel param change within the last [`STENCIL_PREVIEW_HOLD_S`]). The
    /// pattern is rendered in the rect's LOCAL frame; the shell rotates + places it (see [`StencilPreview`]).
    #[must_use]
    pub fn stencil_preview(&self) -> Option<StencilPreview> {
        if !self.stencil_edit_active() {
            return None;
        }
        if self.paint.stencil_grab.is_none() && self.paint.stencil_preview_s <= 0.0 {
            return None;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        let t = &self.paint.brush.texture;
        let (center, half, u) = stencil_frame(t, [w as f32, h as f32]);
        let (pw, ph) = preview_dims(half);
        let mut buf = vec![0u8; (pw as usize) * (ph as usize) * 4];
        let image = self.brush_texture_image().map(|(lum, iw, ih)| ImageMask {
            lum,
            width: iw,
            height: ih,
        });
        render_stencil_preview(t, image.as_ref(), &mut buf, pw, ph);
        Some(StencilPreview {
            rgba: Arc::new(buf),
            w: pw,
            h: ph,
            center,
            half,
            u,
        })
    }

    /// Arm the transient in-gizmo preview (panel-param path): show the Grain inside the rect for
    /// ~[`STENCIL_PREVIEW_HOLD_S`] after a Stencil/texture param change, then fade. The handle DRAG shows
    /// it via the live grab instead. Gated to a draw only while the Stencil editor is active.
    pub(super) fn arm_stencil_preview(&mut self) {
        self.paint.stencil_preview_s = STENCIL_PREVIEW_HOLD_S;
    }
}

/// Euclidean distance between two image-space points.
fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

/// `num / den`, or `1.0` for a (near) zero denominator — a degenerate axis can't define a scale factor.
fn ratio(num: f32, den: f32) -> f32 {
    if den.abs() > 1e-6 { num / den } else { 1.0 }
}

/// Preview buffer dims: the longer rect axis gets [`STENCIL_PREVIEW_MAX`] px; the other tracks the rect
/// aspect (≥ 1) so the grain isn't pre-squashed before the shell stretches it onto the rect.
fn preview_dims(half: [f32; 2]) -> (u32, u32) {
    let max = STENCIL_PREVIEW_MAX as f32;
    if half[0] >= half[1] {
        let h = (max * half[1] / half[0]).round().clamp(1.0, max) as u32;
        (STENCIL_PREVIEW_MAX, h)
    } else {
        let w = (max * half[0] / half[1]).round().clamp(1.0, max) as u32;
        (w, STENCIL_PREVIEW_MAX)
    }
}
