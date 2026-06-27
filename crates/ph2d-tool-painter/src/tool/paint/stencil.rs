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
    /// Dragging corner `0..=3` scales the rect about its centre.
    Scale { corner: u8 },
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
            Self::Scale { corner } | Self::Rotate { corner, .. } => corner,
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
                self.paint.stencil_grab = Some(StencilGrab::Scale { corner: i as u8 });
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
            StencilGrab::Scale { .. } => {
                // Half-extent = |projection of (cursor − centre) onto each rotated axis|; symmetric
                // about the centre. stencil Size = 2·half / canvas.
                let rel = [pos[0] - center[0], pos[1] - center[1]];
                let du = (rel[0] * u[0] + rel[1] * u[1]).abs();
                let dv = (rel[0] * v[0] + rel[1] * v[1]).abs();
                self.set_brush_stencil_size(0, 2.0 * du / canvas[0]);
                self.set_brush_stencil_size(1, 2.0 * dv / canvas[1]);
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

    /// The live brush spec (cheap `Copy`) — snapshot it before handling a panel event so
    /// [`Self::refill_if_brush_changed`] can detect a change and re-fill an open shape editor.
    pub(crate) fn current_brush_spec(&self) -> BrushSpec {
        self.paint.brush
    }

    /// Re-fill the open shape editor's painted preview (Curve / Free Hand / Circle / Polygon) with the
    /// current brush — each refill no-ops unless its editor is open + editing, so at most one runs.
    pub(crate) fn refill_open_shape(&mut self) {
        self.curve_refill();
        self.circle_refill();
        self.polygon_refill();
    }

    /// Re-fill the open shape editor IFF the brush changed since `before` — so a size / spacing / flow /
    /// falloff tweak from the panel updates the pending Curve/Circle/Polygon stroke in REAL TIME, not only
    /// when a gizmo handle is nudged. Apply/Delete clicks + layer ops don't touch the brush → no re-stamp.
    pub(crate) fn refill_if_brush_changed(&mut self, before: BrushSpec) {
        if self.paint.brush != before {
            self.refill_open_shape();
        }
    }

    /// Route the brush **dab/stroke** chrome events: the flatten/rotate gizmo `SetValue`s (Shape panel),
    /// plus the **Apply** / **Apply & Keep** stroke buttons (`Click` → bake the pending shape-editor stroke,
    /// discarding or keeping the editable curve). Returns `true` iff the event was consumed.
    pub(crate) fn route_brush_dab_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        if let PanelEvent::Click(id) = event {
            if *id == core_ids::PAINTER_BRUSH_STROKE_APPLY {
                self.commit_open_shape();
                return true;
            }
            if *id == core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP {
                self.commit_open_shape_keep();
                return true;
            }
            if *id == core_ids::PAINTER_BRUSH_STROKE_DELETE {
                self.cancel_open_shape(); // drop the open shape without baking
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
