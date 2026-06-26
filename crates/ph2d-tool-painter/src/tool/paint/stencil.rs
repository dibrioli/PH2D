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
use ph2d_painter_brush::stencil_frame;

/// The stencil overlay snapshot for the shell: the rect's 4 corners + centre (image-space px), plus
/// which handle (if any) is being dragged. Handle indices: `0..=3` corners (scale), `4` centre
/// (move).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StencilOverlay {
    /// The rect's 4 corners in image-space px (`[--, +-, ++, -+]` along the rotated axes).
    pub corners: [[f32; 2]; 4],
    /// The rect centre in image-space px (the move handle).
    pub center: [f32; 2],
    /// The grabbed handle (`0..=3` corner, `4` centre) for highlighting, or `None`.
    pub grabbed: Option<u8>,
}

/// The in-progress stencil drag. Held in [`super::PaintState`] only between pointer-down and up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum StencilGrab {
    /// Dragging the centre handle moves the rect; the cursor keeps its grab offset from the centre.
    Move { offset: [f32; 2] },
    /// Dragging corner `0..=3` scales the rect about its centre.
    Scale { corner: u8 },
}

impl StencilGrab {
    /// The handle index for overlay highlighting (`4` = centre, else the corner).
    fn handle(self) -> u8 {
        match self {
            Self::Move { .. } => 4,
            Self::Scale { corner } => corner,
        }
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

    /// Pointer-down: grab the handle under the cursor (corners first, then the centre). Returns
    /// `false` when nothing is grabbed, so the caller paints normally.
    fn stencil_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        let Some(o) = self.stencil_overlay() else {
            return false;
        };
        for (i, &c) in o.corners.iter().enumerate() {
            if dist(c, pos) <= tol {
                self.paint.stencil_grab = Some(StencilGrab::Scale { corner: i as u8 });
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
        }
        true
    }

    /// Pointer-up: release the grab. Returns `true` when one was active (so the event is consumed).
    fn stencil_up(&mut self) -> bool {
        self.paint.stencil_grab.take().is_some()
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

    /// Route the **flatten/rotate gizmo** values (Shape panel; Enio 2026-06-26): the panel decodes the
    /// handle drag into a flatten (`0..1`) / angle (degrees) and forwards them as `SetValue`. Returns
    /// `true` iff the event was a gizmo value (so the caller stops routing).
    pub(crate) fn route_brush_dab_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
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
        Some(StencilOverlay {
            corners: [
                corner(-1.0, -1.0),
                corner(1.0, -1.0),
                corner(1.0, 1.0),
                corner(-1.0, 1.0),
            ],
            center,
            grabbed: self.paint.stencil_grab.map(StencilGrab::handle),
        })
    }
}

/// Euclidean distance between two image-space points.
fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}
