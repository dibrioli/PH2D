//! The Painter **Line** polyline editor overlay — the segments through the committed corner points, a dot
//! at each corner (the selected one emphasised), the whole-line transform gizmo, and the per-corner CAD
//! gizmos (a CIRCLE = Fillet, a SQUARE = Chamfer). Split from `painter_bridge_overlays` for the HR-18
//! file-LOC cap, mirroring `painter_bridge_curve_overlay`. Pure draw: reads the active `PainterTool`
//! snapshot + camera and writes guide geometry into the overlay `VectorScene`; mutates nothing.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_line_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    cursor: (f32, f32),
) {
    if let Some(bits) = hero.gizmo.selection
        && let Some(overlay) = painter.line_overlay()
    {
        let (iw, ih) = painter.canvas_size();
        let entity = ph2d_ecs::Entity::from_bits(bits);
        if iw > 0
            && ih > 0
            && let (Some(tr), Some(sprite)) = (
                sim.world().get::<crate::Transform>(entity),
                sim.world().get::<ph2d_render::Sprite>(entity),
            )
        {
            let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                camera,
                window_size,
            );
            use ph2d_vector::{
                Affine, BezPath, Brush, Circle, Color, Fill, Point, RoundedRect, Stroke,
            };
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            // Line stroke gizmo = fluorescent ORANGE (each stroke shape type gets a distinct accent).
            let pal = super::painter_bridge_gizmo::palette_accent(
                hero.theme,
                super::painter_bridge_gizmo::GIZMO_ACCENTS[3],
            );
            let scene = vector_scene.inner_mut();
            // Transform gizmo (editing phase) — drawn FIRST (under the segments + dots) so the editing
            // geometry stays visually dominant. Identical to the Curve gizmo (shared helper).
            if let Some(gz) = overlay.transform_gizmo.as_ref() {
                super::painter_bridge_gizmo::draw_transform_gizmo(scene, gz, affine, &pal, cursor);
            }
            // Segments through the committed corner points (themed frame colour, open or closed).
            let pts = &overlay.points;
            if pts.len() >= 2 {
                let sp: Vec<Point> = pts.iter().map(|&p| map(p)).collect();
                if overlay.closed && pts.len() >= 3 {
                    super::painter_bridge_gizmo::stroke_box(scene, &sp, &pal);
                } else {
                    super::painter_bridge_gizmo::stroke_open(scene, &sp, &pal);
                }
            }
            // A themed Sprite-gizmo handle at each committed corner; the SELECTED corner reads as a circle.
            for (i, &p) in overlay.points.iter().enumerate() {
                let sp = map(p);
                if overlay.selected == Some(i) {
                    super::painter_bridge_gizmo::circle_handle(scene, sp, &pal);
                } else {
                    super::painter_bridge_gizmo::square_handle(scene, sp, &pal);
                }
            }
            // Per-corner CAD gizmos: a CIRCLE (Fillet) + a SQUARE (Chamfer) handle at each real corner;
            // the active mod is accented (orange). Shapes carry the meaning — round = round the corner,
            // square = straight bevel.
            let handle = Color::new([0.80, 0.84, 0.92, 0.92]); // LITERAL-COLOR-OK: corner handle
            let hot = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: active corner mod
            let edge = Color::new([0.12, 0.13, 0.16, 0.9]); // LITERAL-COLOR-OK: handle outline
            let value_col = Color::new([0.62, 0.78, 1.0, 0.92]); // LITERAL-COLOR-OK: overlay value text (line hue)
            // Fillet / Chamfer amount labels (px), collected here + painted after the last `scene` use.
            let mut corner_labels: Vec<(String, Point)> = Vec::new();
            // Same visual size as the transform gizmo's square handles (half-extent 6 px → 12 px).
            for g in &overlay.corner_gizmos {
                let fc = if g.active == 1 { hot } else { handle };
                let fillet = Circle::new(map(g.fillet_handle), 6.0);
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(fc),
                    None,
                    &fillet,
                );
                scene.stroke(
                    &Stroke::new(1.0),
                    Affine::IDENTITY,
                    &Brush::Solid(edge),
                    None,
                    &fillet,
                );
                let s = map(g.chamfer_handle);
                let cc = if g.active == 2 { hot } else { handle };
                let sq = RoundedRect::new(s.x - 6.0, s.y - 6.0, s.x + 6.0, s.y + 6.0, 2.0);
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(cc),
                    None,
                    &sq,
                );
                scene.stroke(
                    &Stroke::new(1.0),
                    Affine::IDENTITY,
                    &Brush::Solid(edge),
                    None,
                    &sq,
                );
                // Fillet / Chamfer amount (px) beside the ACTIVE handle, placed just beyond it along the
                // edge from the vertex — same value language as the Line dimensions.
                if g.active != 0 {
                    let handle_img = if g.active == 1 {
                        g.fillet_handle
                    } else {
                        g.chamfer_handle
                    };
                    let vtx = map(g.vertex);
                    let hnd = map(handle_img);
                    let (ex, ey) = (hnd.x - vtx.x, hnd.y - vtx.y);
                    let l = (ex * ex + ey * ey).sqrt();
                    let (ux, uy) = if l > 1e-6 {
                        (ex / l, ey / l)
                    } else {
                        (0.0, -1.0)
                    };
                    corner_labels.push((
                        format!("{:.0}", g.amount),
                        Point::new(hnd.x + ux * 13.0, hnd.y + uy * 13.0),
                    ));
                }
            }
            // Live **dimensions** (drawing phase, "Dimensions" on): thin translucent CAD guides — the
            // dx/dy legs to the previous point + the px / corner-angle numbers, in the line-guide hue.
            if let Some(d) = overlay.dimensions.as_ref() {
                let dim = Color::new([0.55, 0.72, 1.0, 0.6]); // LITERAL-COLOR-OK: dimension guide (line hue, translucent)
                let (legs, labels) = dimension_geometry(d, &map);
                for (a, b) in &legs {
                    let mut path = BezPath::new();
                    path.move_to(*a);
                    path.line_to(*b);
                    scene.stroke(
                        &Stroke::new(0.75),
                        Affine::IDENTITY,
                        &Brush::Solid(dim),
                        None,
                        &path,
                    );
                }
                // Labels — AFTER the last `scene` use, so the VectorScene is free for text rendering.
                for (text, center) in &labels {
                    let r = ph2d_editor::zones::Rect::new(
                        center.x as f32 - 40.0,
                        center.y as f32 - 8.0,
                        80.0,
                        16.0,
                    );
                    ph2d_editor::paint::paint_text_centered(
                        text_system,
                        vector_scene,
                        text,
                        r,
                        11.0,
                        dim,
                    );
                }
            }
            // Fillet / Chamfer amount labels (editing phase) — after the last `scene` use, so the
            // VectorScene is free for text. `corner_labels` is empty while drawing (no gizmos then).
            for (text, center) in &corner_labels {
                let r = ph2d_editor::zones::Rect::new(
                    center.x as f32 - 40.0,
                    center.y as f32 - 8.0,
                    80.0,
                    16.0,
                );
                ph2d_editor::paint::paint_text_centered(
                    text_system,
                    vector_scene,
                    text,
                    r,
                    11.0,
                    value_col,
                );
            }
        }
    }
}

/// Build the on-screen geometry for the active Line segment's dimensions: the dx/dy **legs** (screen
/// point pairs to stroke) + the **labels** (text + screen centre) for the px distances and the two corner
/// angles. `map` is the image→screen affine application. Pure (no scene); the angle VALUE is the true
/// image-space angle, its LABEL is placed along the visible (screen) bisector. Degenerate legs (a purely
/// axis-aligned segment) are dropped.
/// The Line dimension overlay's screen geometry: the legs to stroke (start/end point pairs) + the labels
/// to paint (text + screen centre).
type DimGeometry = (
    Vec<(ph2d_vector::Point, ph2d_vector::Point)>,
    Vec<(String, ph2d_vector::Point)>,
);

fn dimension_geometry(
    d: &ph2d_tool_painter::LineDimensions,
    map: &impl Fn([f32; 2]) -> ph2d_vector::Point,
) -> DimGeometry {
    use ph2d_vector::Point;
    let prev = map(d.prev);
    let cur = map(d.cur);
    let corner = map([d.cur[0], d.prev[1]]); // right-angle corner of the two legs
    let mut legs = Vec::new();
    let mut labels = Vec::new();
    if d.dx >= 0.5 {
        legs.push((prev, corner));
        labels.push((
            format!("{:.0}", d.dx),
            leg_label_pos(prev, corner, cur, 12.0),
        ));
    }
    if d.dy >= 0.5 {
        legs.push((corner, cur));
        labels.push((
            format!("{:.0}", d.dy),
            leg_label_pos(corner, cur, prev, 12.0),
        ));
    }
    if let Some(af) = d.angle_from
        && let Some((smaller, larger, bis)) = corner_angle(d.prev, af, d.cur, prev, map)
    {
        let off = 24.0;
        labels.push((
            format!("{smaller:.0}\u{b0}"),
            Point::new(prev.x + bis.0 * off, prev.y + bis.1 * off),
        ));
        labels.push((
            format!("{larger:.0}\u{b0}"),
            Point::new(prev.x - bis.0 * off, prev.y - bis.1 * off),
        ));
    }
    (legs, labels)
}

/// Unit vector, or `None` if `v` is ~zero.
fn unit(v: [f64; 2]) -> Option<[f64; 2]> {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (l > 1e-6).then(|| [v[0] / l, v[1] / l])
}

/// Place a leg's label at the leg midpoint, nudged `offset` px along the leg's normal to the side AWAY
/// from `away` (the opposite vertex), so the number sits outside the triangle.
fn leg_label_pos(
    a: ph2d_vector::Point,
    b: ph2d_vector::Point,
    away: ph2d_vector::Point,
    offset: f64,
) -> ph2d_vector::Point {
    use ph2d_vector::Point;
    let mid = Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    let Some(dir) = unit([b.x - a.x, b.y - a.y]) else {
        return mid;
    };
    let (nx, ny) = (-dir[1], dir[0]); // perpendicular
    let s = if nx * (away.x - mid.x) + ny * (away.y - mid.y) > 0.0 {
        -1.0
    } else {
        1.0
    };
    Point::new(mid.x + nx * s * offset, mid.y + ny * s * offset)
}

/// The corner angle at `vertex_img` between the edges to `from_img` and `to_img`: `(smaller, larger)`
/// degrees (summing to 360) from the TRUE image-space vectors, plus a SCREEN-space interior bisector for
/// label placement. `None` if either edge is degenerate.
fn corner_angle(
    vertex_img: [f32; 2],
    from_img: [f32; 2],
    to_img: [f32; 2],
    vertex_s: ph2d_vector::Point,
    map: &impl Fn([f32; 2]) -> ph2d_vector::Point,
) -> Option<(f64, f64, (f64, f64))> {
    let u1 = unit([
        f64::from(from_img[0] - vertex_img[0]),
        f64::from(from_img[1] - vertex_img[1]),
    ])?;
    let u2 = unit([
        f64::from(to_img[0] - vertex_img[0]),
        f64::from(to_img[1] - vertex_img[1]),
    ])?;
    let theta = (u1[0] * u2[0] + u1[1] * u2[1])
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees();
    let smaller = theta;
    let larger = 360.0 - theta;
    // Interior bisector in SCREEN space (where the wedge is visually), so a non-uniform sprite scale
    // doesn't misplace the label. Straight corner → fall back to a perpendicular.
    let a = map(from_img);
    let b = map(to_img);
    let e1 = unit([a.x - vertex_s.x, a.y - vertex_s.y]);
    let e2 = unit([b.x - vertex_s.x, b.y - vertex_s.y]);
    let bis = match (e1, e2) {
        (Some(e1), Some(e2)) => unit([e1[0] + e2[0], e1[1] + e2[1]]).unwrap_or([-e2[1], e2[0]]),
        _ => [0.0, -1.0],
    };
    Some((smaller, larger, (bis[0], bis[1])))
}

#[cfg(test)]
mod tests {
    use super::dimension_geometry;
    use ph2d_tool_painter::LineDimensions;
    use ph2d_vector::Point;

    fn identity(p: [f32; 2]) -> Point {
        Point::new(f64::from(p[0]), f64::from(p[1]))
    }

    #[test]
    fn right_angle_corner_labels_the_dy_and_both_angles() {
        // Vertical segment (10,0)→(10,25) after a horizontal one from (0,0): a 90° corner at (10,0).
        let d = LineDimensions {
            prev: [10.0, 0.0],
            cur: [10.0, 25.0],
            dx: 0.0,
            dy: 25.0,
            angle_from: Some([0.0, 0.0]),
        };
        let (legs, labels) = dimension_geometry(&d, &identity);
        assert_eq!(
            legs.len(),
            1,
            "only the dy leg; the zero-length dx leg is dropped"
        );
        let texts: Vec<&str> = labels.iter().map(|(t, _)| t.as_str()).collect();
        assert!(texts.contains(&"25"), "dy distance labelled: {texts:?}");
        assert!(
            texts.contains(&"90\u{b0}"),
            "smaller corner angle 90°: {texts:?}"
        );
        assert!(
            texts.contains(&"270\u{b0}"),
            "larger corner angle 270°: {texts:?}"
        );
    }

    #[test]
    fn first_segment_has_no_angle_only_distances() {
        let d = LineDimensions {
            prev: [0.0, 0.0],
            cur: [30.0, 40.0],
            dx: 30.0,
            dy: 40.0,
            angle_from: None,
        };
        let (legs, labels) = dimension_geometry(&d, &identity);
        assert_eq!(legs.len(), 2, "both dx and dy legs");
        let texts: Vec<&str> = labels.iter().map(|(t, _)| t.as_str()).collect();
        assert!(
            texts.contains(&"30") && texts.contains(&"40"),
            "dx/dy distances: {texts:?}"
        );
        assert!(
            texts.iter().all(|t| !t.contains('\u{b0}')),
            "no angle on the first segment: {texts:?}"
        );
    }
}
