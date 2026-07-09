//! Sprite-style transform gizmo for vector shapes (ADR-0108 + sprite gizmo reuse).
//!
//! Reuses the generic `ph2d_editor::gizmo` module *unchanged* — the same bbox +
//! 8 scale handles + 4 rotate zones + pivot dot the sprite editor draws. The only
//! shell glue: (1) a `VecPath`-union → [`GizmoView`] builder ([`paint`]), and
//! (2) a [`GizmoDragState`] → `translate/scale/rotate_path` applier ([`App::advance_vec_gizmo`]).
//!
//! A vector path has no stored transform (geometry is baked into the vertices), so
//! the drag applies INCREMENTALLY from a pristine snapshot captured at Down: each
//! Move restores the selected paths and re-applies the single active transform about
//! the gizmo pivot. The pivot is a plain world point ([`App::vec_gizmo_pivot`]),
//! repositioned only via the panel's "Set Center" mode (no sprite `anchor`
//! compensation needed); the dot itself is not grabbable.
//!
//! Canvas-press priority (see `input_dispatch`): pivot-edit mode → gradient handles
//! → gizmo handles → pen. The gizmo box is outset by [`GIZMO_PAD_PX`] so its handles
//! never cover a path anchor, and its interior only moves the shape when the press
//! actually lands INSIDE it (`path_contains_point`), leaving empty bbox space to the pen.

use crate::app_state::App;
use ph2d_editor::{
    GizmoCamera, GizmoDragKind, GizmoDragState, GizmoModifiers, GizmoSnap, GizmoTarget, GizmoView,
    HitIndex, TransformSnapshot, compute_gizmo_transform, gizmo_kind_for_id, paint_sprite_gizmo,
};
use ph2d_tokens::Theme;
use ph2d_vec_scene::{VecPathId, VecScene};
use ph2d_vector::VectorScene;

/// Screen-px outset of the gizmo box off the artwork. Without it the 12 px corner /
/// edge scale handles land EXACTLY on a path's anchors whenever those coincide with
/// the bbox (a rectangle's or polygon's corners always do), making those vertices
/// impossible to grab with the pen. The outset keeps a clickable core around each
/// anchor and moves the edge handles off the outline (so the pen's segment-insert
/// still works). Standard behaviour in vector editors.
const GIZMO_PAD_PX: f64 = 14.0;

/// World units for [`GIZMO_PAD_PX`] at this camera. World-per-pixel is
/// `camera_height_world / window_h` (the same value `pixel_world` derives).
fn gizmo_pad_world(cam_height: f32, win_h: f32) -> f64 {
    if win_h <= 0.0 {
        return 0.0;
    }
    GIZMO_PAD_PX * f64::from(cam_height) / f64::from(win_h)
}

/// The ORIENTED union bbox of `selected` paths at angle `r` (radians), inflated by
/// `pad` world-units, returned as the UNROTATED rect `(min, max)` centered at the
/// world center `C` (so `min = C − half`, `max = C + half`) — the [`GizmoView`]
/// rotates it about `C` by `r`. `None` if empty. At `r = 0`, `pad = 0` this is the
/// plain axis-aligned curve bbox.
fn oriented_bbox(
    scene: &VecScene,
    selected: &[VecPathId],
    r: f64,
    pad: f64,
) -> Option<([f64; 2], [f64; 2])> {
    let (c, s) = (r.cos(), r.sin());
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    let mut any = false;
    for &id in selected {
        if let Some((l, h)) = scene.path_curve_bbox_in_frame(id, c, s) {
            lo[0] = lo[0].min(l[0]);
            lo[1] = lo[1].min(l[1]);
            hi[0] = hi[0].max(h[0]);
            hi[1] = hi[1].max(h[1]);
            any = true;
        }
    }
    if !any {
        return None;
    }
    // Center in the rotated frame → world (Rot(+r)·cr); half-extents are frame-local
    // and inflated by the outset so the handles clear the artwork.
    let cr = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let half = [(hi[0] - lo[0]) * 0.5 + pad, (hi[1] - lo[1]) * 0.5 + pad];
    let cw = [cr[0] * c - cr[1] * s, cr[0] * s + cr[1] * c];
    Some((
        [cw[0] - half[0], cw[1] - half[1]],
        [cw[0] + half[0], cw[1] + half[1]],
    ))
}

fn center_of(lo: [f64; 2], hi: [f64; 2]) -> [f64; 2] {
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
}

/// Rotate local offset `off` by `r` (radians) → world delta (for oriented handles).
fn rot(off: [f64; 2], r: f64) -> [f64; 2] {
    let (c, s) = (r.cos(), r.sin());
    [off[0] * c - off[1] * s, off[0] * s + off[1] * c]
}

/// Paint the transform gizmo over the object selection + register its hit rects.
/// A free fn (mirror of `draw_overlays`) so the caller can pass disjoint borrows of
/// `App` while `vec_scene` / `target` are already borrowed in the render loop.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    vec_scene: &VecScene,
    selected_paths: &[VecPathId],
    pivot: Option<[f64; 2]>,
    rotation: f32,
    pivot_edit: bool,
    last_pointer: (f32, f32),
    cam_center: [f32; 2],
    cam_height: f32,
    win_w: f32,
    win_h: f32,
    theme: Theme,
    hits: &mut HitIndex,
    target: &mut VectorScene,
) {
    hits.clear_for_frame();
    let pad = gizmo_pad_world(cam_height, win_h);
    let Some((lo, hi)) = oriented_bbox(vec_scene, selected_paths, f64::from(rotation), pad) else {
        return;
    };
    let piv = pivot.unwrap_or_else(|| center_of(lo, hi));
    let view = GizmoView {
        bbox_min_world: [lo[0] as f32, lo[1] as f32],
        bbox_max_world: [hi[0] as f32, hi[1] as f32],
        rotation, // gizmo box tracks the accumulated shape rotation
        camera_center: cam_center,
        camera_height_world: cam_height,
        window_w: win_w,
        window_h: win_h,
        canvas: ph2d_editor::zones::Rect::new(0.0, 0.0, win_w, win_h),
        cursor_screen: Some(last_pointer),
        pivot_world: [piv[0] as f32, piv[1] as f32],
        // Emphasize the pivot dot only while the "Set Center" mode is active.
        pivot_tool_active: pivot_edit,
    };
    paint_sprite_gizmo(target, &view, theme, hits);
}

impl App {
    /// Oriented union bbox of the object selection (unrotated rect centered at the
    /// world center) at the current gizmo rotation, inflated by `pad` world-units.
    fn vec_gizmo_bbox(&self, pad: f64) -> Option<([f64; 2], [f64; 2])> {
        let gfx = self.gfx.as_ref()?;
        oriented_bbox(
            &gfx.vec_scene,
            self.vec_pen.selected_paths(),
            f64::from(self.vec_gizmo_rotation),
            pad,
        )
    }

    /// The current rotation/scale pivot (user-set, else the selection bbox center).
    /// The outset never moves the center, so `pad = 0` here.
    pub(crate) fn vec_gizmo_pivot_world(&self) -> Option<[f64; 2]> {
        if let Some(p) = self.vec_gizmo_pivot {
            return Some(p);
        }
        let (lo, hi) = self.vec_gizmo_bbox(0.0)?;
        Some(center_of(lo, hi))
    }

    fn vec_gizmo_camera(&self, cc: [f32; 2], ch: f32, ww: f32, wh: f32) -> GizmoCamera {
        let _ = self;
        GizmoCamera {
            center: cc,
            height_world: ch,
            window_w: ww,
            window_h: wh,
        }
    }

    /// Begin a vector gizmo drag: snapshot the scene (pristine + undo baseline).
    fn start_vec_gizmo(
        &mut self,
        kind: GizmoDragKind,
        cursor_world: [f32; 2],
        pivot: [f32; 2],
        half: [f32; 2],
        ctrl: bool,
    ) {
        // start_transform carries the current gizmo rotation so the scale math
        // projects the cursor into the shape's ORIENTED axes (and rotate accumulates
        // off it). Translation/scale start at identity.
        let start_transform = TransformSnapshot {
            translation: [0.0, 0.0],
            rotation: self.vec_gizmo_rotation,
            scale: [1.0, 1.0],
        };
        self.vec_gizmo_drag = Some(GizmoDragState {
            kind,
            entity_bits: 0,
            start_screen: self.last_pointer,
            cursor_screen: self.last_pointer,
            start_transform,
            pivot_world: pivot,
            start_cursor_world: cursor_world,
            sprite_half_intrinsic: half,
            anchor_is_center: ctrl,
            target: GizmoTarget::PrimaryIndividual,
            parent_world: TransformSnapshot::IDENTITY,
        });
        let snap = self.gfx.as_ref().map(|g| g.vec_scene.clone());
        if let Some(snap) = snap {
            self.vec_history.begin(&snap);
            self.vec_gizmo_start = Some(snap);
        }
    }

    /// While the panel "Set Center" mode is active, a canvas press positions the
    /// rotation/scale pivot (drag to fine-tune). Returns `true` when it consumed the
    /// press. The mode is left on Up (see [`Self::end_vec_gizmo`]).
    pub(crate) fn vec_pivot_edit_down(&mut self, cc: [f32; 2], ch: f32, ww: f32, wh: f32) -> bool {
        if !self.vec_pivot_edit {
            return false;
        }
        let cam = self.vec_gizmo_camera(cc, ch, ww, wh);
        let cw = cam.screen_to_world(self.last_pointer);
        self.vec_gizmo_pivot = Some([f64::from(cw[0]), f64::from(cw[1])]);
        self.start_vec_gizmo(GizmoDragKind::MovePivot, cw, cw, [0.0, 0.0], false);
        true
    }

    /// A Down on a vector gizmo handle → start the matching drag. Returns `true`
    /// when it consumed the press (caller must not also draw/select). No-op when the
    /// press isn't on a handle, or an interior press where the pen would grab a
    /// vertex/outline (so vertex editing still works inside the shape).
    pub(crate) fn vec_gizmo_down(&mut self, cc: [f32; 2], ch: f32, ww: f32, wh: f32) -> bool {
        let Some(id) = self
            .vec_gizmo_hits
            .hit(self.last_pointer.0, self.last_pointer.1)
        else {
            return false;
        };
        let cam = self.vec_gizmo_camera(cc, ch, ww, wh);
        let cw = cam.screen_to_world(self.last_pointer);
        // Same outset the painter used, so the handle rects and the scale anchors agree.
        let Some((lo, hi)) = self.vec_gizmo_bbox(gizmo_pad_world(ch, wh)) else {
            return false;
        };
        let c = center_of(lo, hi);
        let center = [c[0] as f32, c[1] as f32];
        let half = [
            ((hi[0] - lo[0]) * 0.5) as f32,
            ((hi[1] - lo[1]) * 0.5) as f32,
        ];
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();

        // The pivot dot is NOT grabbable directly — the rotation center is edited
        // only via the panel "Set Center" button (see `vec_pivot_edit_down`).
        let Some(kind) = gizmo_kind_for_id(id) else {
            return false;
        };
        // Interior translate is the LOOSEST hit (the whole bbox), so it defers twice:
        //  - near a vertex / outline → the pen edits it;
        //  - not actually INSIDE a selected shape → the pen draws (empty bbox space
        //    must stay usable, e.g. starting a new path inside a selection's box).
        if kind == GizmoDragKind::Translate
            && let Some(gfx) = self.gfx.as_ref()
        {
            let px = pixel_world(&cam);
            let wp = [f64::from(cw[0]), f64::from(cw[1])];
            if self
                .vec_pen
                .path_at(&gfx.vec_scene, wp, 10.0 * px)
                .is_some()
            {
                return false;
            }
            let on_shape = self
                .vec_pen
                .selected_paths()
                .iter()
                .any(|&pid| gfx.vec_scene.path_contains_point(pid, wp));
            if !on_shape {
                return false;
            }
        }
        // Scale pivot = the opposite corner / edge midpoint IN THE ORIENTED FRAME
        // (or center under Ctrl); rotate pivot = the (possibly user-set) pivot point.
        let r = f64::from(self.vec_gizmo_rotation);
        let off = match kind {
            GizmoDragKind::ScaleCorner { dx_sign, dy_sign } if !ctrl => {
                Some([f64::from(-dx_sign * half[0]), f64::from(-dy_sign * half[1])])
            }
            GizmoDragKind::ScaleEdge { axis, sign } if !ctrl => Some(if axis == 0 {
                [f64::from(-sign * half[0]), 0.0]
            } else {
                [0.0, f64::from(-sign * half[1])]
            }),
            _ => None,
        };
        let pivot = match kind {
            GizmoDragKind::Rotate => {
                let p = self.vec_gizmo_pivot_world().unwrap_or(c);
                [p[0] as f32, p[1] as f32]
            }
            _ => match off {
                Some(off) => {
                    let d = rot(off, r);
                    [(c[0] + d[0]) as f32, (c[1] + d[1]) as f32]
                }
                None => center,
            },
        };
        self.start_vec_gizmo(kind, cw, pivot, half, ctrl);
        true
    }

    /// Advance an in-progress vector gizmo drag (called on every cursor move).
    /// Returns `true` while a drag is live (caller early-returns like the other
    /// canvas drags). Restores the pristine snapshot then re-applies the transform.
    pub(crate) fn advance_vec_gizmo(&mut self, cc: [f32; 2], ch: f32, ww: f32, wh: f32) -> bool {
        let Some(mut drag) = self.vec_gizmo_drag else {
            return false;
        };
        drag.cursor_screen = self.last_pointer;
        self.vec_gizmo_drag = Some(drag);
        let cam = self.vec_gizmo_camera(cc, ch, ww, wh);

        if drag.kind == GizmoDragKind::MovePivot {
            let w = cam.screen_to_world(self.last_pointer);
            self.vec_gizmo_pivot = Some([w[0] as f64, w[1] as f64]);
            return true;
        }

        let mods = GizmoModifiers {
            shift: self.modifiers.shift_key(),
            ctrl: self.modifiers.control_key() || self.modifiers.super_key(),
            alt: self.modifiers.alt_key(),
        };
        let out = compute_gizmo_transform(&drag, &cam, mods, GizmoSnap::default(), None);
        let pivot = [drag.pivot_world[0] as f64, drag.pivot_world[1] as f64];
        let r = f64::from(drag.start_transform.rotation);
        let ids = self.vec_pen.selected_paths().to_vec();
        let Some(start) = self.vec_gizmo_start.clone() else {
            return true;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return true;
        };
        for id in ids {
            // Reset the path to its pristine geometry before re-applying.
            if let Some(src) = start.paths().iter().find(|p| p.id == id).cloned()
                && let Some(dst) = gfx.vec_scene.path_mut(id)
            {
                dst.verts = src.verts;
                dst.fill = src.fill;
            }
            match drag.kind {
                GizmoDragKind::Translate => {
                    let dx = f64::from(out.translation[0] - drag.start_transform.translation[0]);
                    let dy = f64::from(out.translation[1] - drag.start_transform.translation[1]);
                    gfx.vec_scene.translate_path(id, dx, dy);
                }
                GizmoDragKind::ScaleCorner { .. } | GizmoDragKind::ScaleEdge { .. } => {
                    // Oriented scale: un-rotate to the shape's axes, scale, rotate back
                    // — so the handles scale along the (rotated) box edges, not world.
                    gfx.vec_scene.rotate_path_by(id, -r, pivot);
                    gfx.vec_scene.scale_path(
                        id,
                        f64::from(out.scale[0]),
                        f64::from(out.scale[1]),
                        pivot,
                    );
                    gfx.vec_scene.rotate_path_by(id, r, pivot);
                }
                GizmoDragKind::Rotate => {
                    let ang = f64::from(out.rotation - drag.start_transform.rotation);
                    gfx.vec_scene.rotate_path_by(id, ang, pivot);
                }
                GizmoDragKind::MovePivot => {}
            }
        }
        // Rotation accumulates onto the gizmo so its box tracks the shape.
        if drag.kind == GizmoDragKind::Rotate {
            self.vec_gizmo_rotation = out.rotation;
        }
        true
    }

    /// End a vector gizmo drag (pointer Up). Commits ONE undo step iff the scene
    /// changed. Returns `true` if a drag was active.
    pub(crate) fn end_vec_gizmo(&mut self) -> bool {
        let Some(drag) = self.vec_gizmo_drag.take() else {
            return false;
        };
        // "Set Center" is a one-shot mode: positioning the pivot exits it on Up.
        if drag.kind == GizmoDragKind::MovePivot {
            self.vec_pivot_edit = false;
        }
        self.vec_gizmo_start = None;
        let scene = self.gfx.as_ref().map(|g| g.vec_scene.clone());
        if let Some(scene) = scene {
            self.vec_history.commit_if_changed(&scene);
        }
        true
    }
}

/// World units per screen pixel (for the pen deferral hit radius).
fn pixel_world(cam: &GizmoCamera) -> f64 {
    let a = cam.screen_to_world((0.0, 0.0));
    let b = cam.screen_to_world((1.0, 0.0));
    (f64::from(b[0] - a[0]).powi(2) + f64::from(b[1] - a[1]).powi(2)).sqrt()
}
