//! `ShapeTool` — drag-to-size primitive drawing for the new vector pipeline
//! (ADR-0108, Fase 1). Sibling of [`PenTool`](crate::PenTool): the shell routes
//! canvas Down/Move/Up here when the Vector tool's draw-mode is a shape
//! (Rectangle / Ellipse / Polygon) instead of the pen.
//!
//! The interaction is the standard vector-editor gesture: **press** anchors one
//! corner and pushes a live (degenerate) path into the scene; **drag** resizes
//! it against the bounding box; **release** keeps it (if larger than a few px)
//! or removes it (a stray click → cancel). Because the path lives in the scene
//! the whole time, the existing render draws the preview for free — no overlay.
//!
//! Geometry comes from the pure constructors in `ph2d-vec-scene`
//! (`rectangle` / `ellipse` / `regular_polygon`); this module only owns the
//! state machine + styling (a closed shape gets a fill + stroke like a closed
//! pen path).

use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::PenStyle;

/// The primitive a [`ShapeTool`] draws. `Default` = `Rectangle` so the struct
/// derives `Default`; the shell picks the kind per press from the tool's mode.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ShapeKind {
    #[default]
    Rectangle,
    Ellipse,
    Polygon,
    Star,
    RoundRect,
    Spiral,
    /// Segmento reto (aberto, sem fill).
    Line,
    /// Arco de elipse de `arc_degrees` graus (aberto, sem fill).
    Arc,
}

/// Per-shape parameters (the shell mirrors these from the Vector tool). Only the
/// field(s) for the active [`ShapeKind`] matter; the rest are ignored.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShapeParams {
    /// Polygon side count.
    pub sides: u32,
    /// Star point count.
    pub star_points: u32,
    /// Star inner/outer radius ratio.
    pub star_inner_ratio: f64,
    /// Rounded-rect corner radius in **screen pixels** (× `px_to_world` at draw).
    pub corner_radius_px: f64,
    /// Spiral turn count.
    pub spiral_turns: u32,
    /// Arc span in degrees (`ShapeKind::Arc`).
    pub arc_degrees: f64,
}

impl Default for ShapeParams {
    fn default() -> Self {
        Self {
            sides: 5,
            star_points: 5,
            star_inner_ratio: 0.5,
            corner_radius_px: 8.0,
            spiral_turns: 3,
            arc_degrees: 180.0,
        }
    }
}

/// A drag below this many **screen pixels** on both axes is treated as a stray
/// click and the shape is discarded on release (so a mis-click on the canvas
/// doesn't litter zero-size paths).
const MIN_DRAG_PX: f64 = 3.0;

/// Drag-to-size shape drawing. State machine only — the document (`VecScene`)
/// and undo (`History`) live in the shell, exactly like [`PenTool`](crate::PenTool).
#[derive(Default)]
pub struct ShapeTool {
    /// Style for the shape being drawn (synced from the tool by the shell each
    /// frame, mirror of `PenTool`).
    style: PenStyle,
    /// The live path being sized (between press and release); `None` when idle.
    active: Option<VecPathId>,
    kind: ShapeKind,
    params: ShapeParams,
    /// The pressed corner in world-space.
    start: [f64; 2],
    /// World units per screen pixel (captured at press) — sizes the stroke +
    /// the min-drag cancel threshold.
    px_to_world: f64,
    /// The last committed shape (the shell selects it so it's immediately
    /// editable / recolourable).
    committed: Option<VecPathId>,
}

impl ShapeTool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the style applied to shapes drawn next (the shell syncs this from the
    /// Vector tool each frame, mirror of `PenTool::set_style`).
    pub fn set_style(&mut self, style: PenStyle) {
        self.style = style;
    }

    /// Is a shape drag in progress?
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// The last committed shape (for the shell to select on release).
    #[must_use]
    pub fn selected(&self) -> Option<VecPathId> {
        self.committed
    }

    /// O path que o arrasto está construindo. Ele é reescrito em coordenadas de
    /// MUNDO a cada Move (`on_drag` regenera `verts`), então a shell não pode
    /// assentar a origem dele no meio do gesto — a geometria e o `Transform` ficariam
    /// somando (ADR-0112).
    #[must_use]
    pub fn active_path(&self) -> Option<VecPathId> {
        self.active
    }

    /// Begin a shape at world-space `p`. Pushes a degenerate path (renders live)
    /// and records the anchor + kind/params + `px_to_world`. Returns the new id.
    /// A prior in-progress shape (shouldn't happen) is dropped first.
    pub fn on_press(
        &mut self,
        scene: &mut VecScene,
        kind: ShapeKind,
        params: ShapeParams,
        p: [f64; 2],
        px_to_world: f64,
    ) -> VecPathId {
        if let Some(id) = self.active.take() {
            scene.remove_path(id);
        }
        self.kind = kind;
        self.params = params;
        self.start = p;
        self.px_to_world = px_to_world;
        let id = scene.push_path(self.build(p));
        self.active = Some(id);
        id
    }

    /// Resize the active shape to the bbox `start..p`. `true` if it consumed the
    /// move (a shape drag is live).
    pub fn on_drag(&mut self, scene: &mut VecScene, p: [f64; 2]) -> bool {
        let Some(id) = self.active else {
            return false;
        };
        let rebuilt = self.build(p);
        let Some(path) = scene.path_mut(id) else {
            self.active = None;
            return false;
        };
        path.verts = rebuilt.verts;
        path.closed = rebuilt.closed;
        path.fill = rebuilt.fill;
        path.stroke = rebuilt.stroke;
        true
    }

    /// Finish the drag: keep the shape if it spans ≥ [`MIN_DRAG_PX`] on either
    /// axis (records it as the committed selection), else remove it (cancel).
    /// Returns `true` iff a shape was committed.
    pub fn on_release(&mut self, scene: &mut VecScene) -> bool {
        let Some(id) = self.active.take() else {
            return false;
        };
        let min = MIN_DRAG_PX * self.px_to_world;
        let keep = scene
            .paths()
            .iter()
            .find(|pp| pp.id == id)
            .is_some_and(|pp| bbox_span(pp) >= min);
        if keep {
            self.committed = Some(id);
            true
        } else {
            scene.remove_path(id);
            false
        }
    }

    /// Drop any in-progress shape (tool switch / secondary click / Esc).
    pub fn cancel(&mut self, scene: &mut VecScene) {
        if let Some(id) = self.active.take() {
            scene.remove_path(id);
        }
    }

    /// Build the styled shape path for the current kind between `start` and
    /// `cur`. A closed region gets the fill (unless None) + stroke, like a
    /// closed pen path.
    fn build(&self, cur: [f64; 2]) -> VecPath {
        let mut path = match self.kind {
            ShapeKind::Rectangle => ph2d_vec_scene::rectangle(self.start, cur),
            ShapeKind::Ellipse => {
                let (c, rx, ry) = bbox_center_radii(self.start, cur);
                ph2d_vec_scene::ellipse(c, rx, ry)
            }
            ShapeKind::Polygon => {
                let (c, rx, ry) = bbox_center_radii(self.start, cur);
                ph2d_vec_scene::regular_polygon(c, rx, ry, self.params.sides)
            }
            ShapeKind::Star => {
                let (c, rx, ry) = bbox_center_radii(self.start, cur);
                ph2d_vec_scene::star(
                    c,
                    rx,
                    ry,
                    self.params.star_points,
                    self.params.star_inner_ratio,
                )
            }
            ShapeKind::RoundRect => {
                let radius = self.params.corner_radius_px * self.px_to_world;
                ph2d_vec_scene::rounded_rect(self.start, cur, radius)
            }
            ShapeKind::Spiral => {
                let (c, rx, ry) = bbox_center_radii(self.start, cur);
                ph2d_vec_scene::spiral(c, rx, ry, self.params.spiral_turns)
            }
            ShapeKind::Line => ph2d_vec_scene::line(self.start, cur),
            ShapeKind::Arc => {
                let (c, rx, ry) = bbox_center_radii(self.start, cur);
                ph2d_vec_scene::arc(c, rx, ry, self.params.arc_degrees)
            }
        };
        let stroke_w = self.style.stroke_w_px * self.px_to_world;
        // Formas ABERTAS (linha, arco) nunca têm fill — não há interior.
        let open = matches!(self.kind, ShapeKind::Line | ShapeKind::Arc);
        path.fill = if open || self.style.fill.a == 0 {
            None
        } else {
            Some(ph2d_vec_scene::Paint::solid(self.style.fill))
        };
        path.stroke = Some(self.style.stroke_spec(stroke_w));
        path
    }
}

/// Center + semi-axes of the bbox spanned by two opposite corners.
fn bbox_center_radii(a: [f64; 2], b: [f64; 2]) -> ([f64; 2], f64, f64) {
    let c = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let rx = (b[0] - a[0]).abs() * 0.5;
    let ry = (b[1] - a[1]).abs() * 0.5;
    (c, rx, ry)
}

/// Larger of the path's anchor-bbox extents (used for the min-drag threshold).
fn bbox_span(p: &VecPath) -> f64 {
    let (mut minx, mut miny) = (f64::MAX, f64::MAX);
    let (mut maxx, mut maxy) = (f64::MIN, f64::MIN);
    for v in &p.verts {
        minx = minx.min(v.anchor[0]);
        maxx = maxx.max(v.anchor[0]);
        miny = miny.min(v.anchor[1]);
        maxy = maxy.max(v.anchor[1]);
    }
    (maxx - minx).max(maxy - miny)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::VertexKind;

    const PTW: f64 = 0.01; // world-units per pixel (fake camera)

    #[test]
    fn rectangle_press_drag_release_commits_a_selected_closed_path() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Rectangle,
            ShapeParams::default(),
            [0.0, 0.0],
            PTW,
        );
        assert!(shape.is_active());
        shape.on_drag(&mut scene, [4.0, 3.0]);
        assert!(shape.on_release(&mut scene), "a real drag commits");
        assert!(!shape.is_active());
        assert_eq!(scene.paths().len(), 1);
        let p = &scene.paths()[0];
        assert!(p.closed && p.verts.len() == 4);
        assert!(p.fill.is_some() && p.stroke.is_some());
        assert_eq!(shape.selected(), Some(p.id));
    }

    #[test]
    fn tiny_drag_is_discarded_on_release() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Rectangle,
            ShapeParams::default(),
            [0.0, 0.0],
            PTW,
        );
        // Move less than MIN_DRAG_PX * PTW on both axes.
        shape.on_drag(&mut scene, [0.01, 0.01]);
        assert!(!shape.on_release(&mut scene), "stray click cancels");
        assert!(scene.is_empty(), "cancelled shape leaves no path");
    }

    #[test]
    fn ellipse_uses_smooth_verts_polygon_uses_corners() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Ellipse,
            ShapeParams::default(),
            [0.0, 0.0],
            PTW,
        );
        shape.on_drag(&mut scene, [4.0, 2.0]);
        shape.on_release(&mut scene);
        assert!(
            scene.paths()[0]
                .verts
                .iter()
                .all(|v| v.kind == VertexKind::Smooth)
        );

        let mut scene2 = VecScene::new();
        let mut shape2 = ShapeTool::new();
        shape2.on_press(
            &mut scene2,
            ShapeKind::Polygon,
            ShapeParams {
                sides: 7,
                ..Default::default()
            },
            [0.0, 0.0],
            PTW,
        );
        shape2.on_drag(&mut scene2, [4.0, 4.0]);
        shape2.on_release(&mut scene2);
        let p = &scene2.paths()[0];
        assert_eq!(p.verts.len(), 7);
        assert!(p.verts.iter().all(|v| v.kind == VertexKind::Corner));
    }

    #[test]
    fn star_and_roundrect_build_the_right_vertex_counts() {
        // Star with 6 points → 12 verts.
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Star,
            ShapeParams {
                star_points: 6,
                ..Default::default()
            },
            [0.0, 0.0],
            PTW,
        );
        shape.on_drag(&mut scene, [4.0, 4.0]);
        shape.on_release(&mut scene);
        assert_eq!(scene.paths()[0].verts.len(), 12);

        // RoundRect with a real radius → 8 corner verts.
        let mut scene2 = VecScene::new();
        let mut shape2 = ShapeTool::new();
        shape2.on_press(
            &mut scene2,
            ShapeKind::RoundRect,
            ShapeParams {
                corner_radius_px: 20.0,
                ..Default::default()
            },
            [0.0, 0.0],
            PTW,
        );
        shape2.on_drag(&mut scene2, [4.0, 3.0]);
        shape2.on_release(&mut scene2);
        assert_eq!(scene2.paths()[0].verts.len(), 8);
    }

    #[test]
    fn fill_none_style_yields_stroke_only_shape() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.set_style(PenStyle {
            fill: ph2d_vec_scene::Rgba8::new(0, 0, 0, 0), // None
            ..PenStyle::default()
        });
        shape.on_press(
            &mut scene,
            ShapeKind::Rectangle,
            ShapeParams::default(),
            [0.0, 0.0],
            PTW,
        );
        shape.on_drag(&mut scene, [4.0, 3.0]);
        shape.on_release(&mut scene);
        let p = &scene.paths()[0];
        assert!(p.fill.is_none() && p.stroke.is_some());
    }

    #[test]
    fn cancel_removes_in_progress_shape() {
        let mut scene = VecScene::new();
        let mut shape = ShapeTool::new();
        shape.on_press(
            &mut scene,
            ShapeKind::Ellipse,
            ShapeParams::default(),
            [1.0, 1.0],
            PTW,
        );
        assert_eq!(scene.paths().len(), 1);
        shape.cancel(&mut scene);
        assert!(scene.is_empty() && !shape.is_active());
    }
}
