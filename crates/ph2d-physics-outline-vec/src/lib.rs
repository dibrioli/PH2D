//! **«Convert to Collision Outline»** (Hierarchy context menu, dono 09/11): a Vector drawing is
//! REPLACED by a «Collision Shape» whose `ColliderShape::Outline` covers what the drawing paints.
//!
//! The area is what the renderer paints: the FILL of the closed contours (by the path's fill rule) and
//! the STROKE band of every contour, open ones included (an open line with a stroke is a platform).
//! A path with neither paint counts its closed contours as filled. The area is rasterised in the
//! drawing's own local frame and traced by the same door as a picture (`ph2d_physics_outline::trace`,
//! i.e. `trace_fitted`: Detail rises to fit `MAX_POINTS`/`MAX_PIECES`, never cutting the art), so
//! overlapping contours, holes and fill rules need no polygon boolean. `custom = true`: there is no
//! picture to retrace.
//!
//! The shape takes the vector object's place: same parent, same slot among the siblings, same
//! `Transform`; the drawing's children move under it. The vector entity and its path leave together,
//! in the same frame, so the editor's per-frame diff makes ONE undo step.

use ph2d_ecs::{
    ChildOf, Entity, Name, RootOrder, Transform, VecPathRef, World,
    ordered_children, set_sibling_order,
};
use ph2d_outline_trace::Mask;
use ph2d_physics_ecs::{Collider, ColliderOutline, ColliderShape};
use ph2d_vec_scene::{FillRule, VecPath, VecScene};

/// The long side of the area raster, in px. One px of error over the drawing's size (≤ 0,2 %), at
/// 1 MB of RGBA for the trace's door; the outline's Detail (2 px) dominates the error anyway.
pub const RASTER_LONG_SIDE: f64 = 512.0;

/// Why a row cannot be converted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The object is not a Vector drawing (or its path is gone).
    NotAVector,
    /// The drawing paints no area (an open path without a stroke).
    NoArea,
}

/// The `Collider` + `ColliderOutline` that cover `path`'s painted area, in its LOCAL frame.
pub fn shape_of(path: &VecPath) -> Result<(Collider, ColliderOutline), Refusal> {
    let cooked = path.cooked();
    let has_fill = cooked.fill.is_some() || cooked.stroke.is_none();
    let r = cooked
        .stroke
        .as_ref()
        .map_or(0.0, |s| s.width.max(0.0) * 0.5);
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    let mut any_closed = false;
    for c in 0..cooked.contour_count() {
        let Some((verts, closed)) = cooked.contour(c) else {
            continue;
        };
        any_closed |= closed && verts.len() >= 2;
        for v in verts {
            for p in [v.anchor, v.in_handle, v.out_handle] {
                for a in 0..2 {
                    lo[a] = lo[a].min(p[a] - r);
                    hi[a] = hi[a].max(p[a] + r);
                }
            }
        }
    }
    let fills = has_fill && any_closed;
    if !(fills || r > 0.0) || !(lo[0] < hi[0] || lo[1] < hi[1]) {
        return Err(Refusal::NoArea);
    }
    let span = (hi[0] - lo[0]).max(hi[1] - lo[1]);
    let s = RASTER_LONG_SIDE / span;
    // One empty px around the art: the march closes every ring inside the picture.
    let pad = 1.0 / s;
    let (x0, y1) = (lo[0] - pad, hi[1] + pad);
    let w = ((hi[0] - lo[0]) * s).ceil() as u32 + 2;
    let h = ((hi[1] - lo[1]) * s).ceil() as u32 + 2;
    let tol = 0.25 / s;
    let mut lines: Vec<(Vec<[f64; 2]>, bool)> = Vec::new();
    for c in 0..cooked.contour_count() {
        if let Some((verts, closed)) = cooked.contour(c) {
            lines.push((flatten_contour(verts, closed, tol), closed));
        }
    }
    let px = |p: [f64; 2]| [(p[0] - x0) * s, (y1 - p[1]) * s];
    let lines: Vec<(Vec<[f64; 2]>, bool)> = lines
        .into_iter()
        .map(|(l, c)| (l.into_iter().map(px).collect(), c))
        .collect();
    let mut on = vec![false; (w * h) as usize];
    if fills {
        fill(&mut on, w, h, &lines, cooked.fill_rule);
    }
    if r > 0.0 {
        stroke(&mut on, w, h, &lines, r * s);
    }
    if !on.iter().any(|&b| b) {
        return Err(Refusal::NoArea);
    }
    let rgba: Vec<u8> = on
        .iter()
        .flat_map(|&b| [255, 255, 255, if b { 255 } else { 0 }])
        .collect();
    let mask = Mask::from_rgba8(w, h, &rgba, 128);
    let mut outline = ph2d_physics_outline::trace(&mask, &ColliderOutline::default()).outline;
    if outline.rings.is_empty() {
        return Err(Refusal::NoArea);
    }
    outline.custom = true;
    let half = [f64::from(w) / (2.0 * s), f64::from(h) / (2.0 * s)];
    let collider = Collider {
        shape: ColliderShape::Outline {
            half_x: half[0] as f32,
            half_y: half[1] as f32,
        },
        offset: [(x0 + half[0]) as f32, (y1 - half[1]) as f32],
        ..Collider::default()
    };
    Ok((collider, outline))
}

/// **The conversion**: `e` (a Vector object of `scene`) is replaced by a «Collision Shape» with its
/// outline. Returns the shape (to select). Nothing changes on a refusal.
pub fn convert(world: &mut World, scene: &mut VecScene, e: Entity) -> Result<Entity, Refusal> {
    let id = world.get::<VecPathRef>(e).map(|r| r.0);
    let path = id
        .and_then(|id| scene.path(id))
        .ok_or(Refusal::NotAVector)?;
    let (collider, outline) = shape_of(path)?;
    let transform = world.get::<Transform>(e).copied().unwrap_or_default();
    let parent = world.get::<ChildOf>(e).map(|c| c.0);
    let name = ph2d_unique_name::unique_names_in(world, ph2d_physics_ecs::NOME_DA_FORMA, 1).pop().unwrap_or_default();
    let shape = world
        .spawn((Name::new(name), transform, collider, outline))
        .id();
    match parent {
        Some(p) => {
            let order: Vec<Entity> = ordered_children(world, p)
                .into_iter()
                .map(|c| if c == e { shape } else { c })
                .collect();
            world.entity_mut(shape).insert(ChildOf(p));
            set_sibling_order(world, p, &order);
        }
        None => {
            if let Some(o) = world.get::<RootOrder>(e).copied() {
                world.entity_mut(shape).insert(o);
            }
        }
    }
    let kids = ordered_children(world, e);
    for &k in &kids {
        world.entity_mut(k).insert(ChildOf(shape));
    }
    set_sibling_order(world, shape, &kids);
    world.despawn(e);
    // The path leaves the document in the SAME frame: a later `sync` would split the gesture in two
    // undo steps (the shape without the path, then the path gone).
    if let Some(id) = id {
        scene.remove_path(id);
    }
    if parent.is_none() && world.get::<RootOrder>(shape).is_none() {
        ph2d_ecs::assign_missing_root_order(world);
    }
    ph2d_ecs::assign_missing_stable_ids(world);
    Ok(shape)
}

/// A contour (cubic verts) as a polyline within `tol` of the curve.
fn flatten_contour(verts: &[ph2d_vec_scene::VecVertex], closed: bool, tol: f64) -> Vec<[f64; 2]> {
    let Some(first) = verts.first() else {
        return Vec::new();
    };
    let mut out = vec![first.anchor];
    let n = verts.len();
    let last = if closed { n } else { n - 1 };
    for i in 0..last {
        let (a, b) = (&verts[i], &verts[(i + 1) % n]);
        cubic([a.anchor, a.out_handle, b.in_handle, b.anchor], tol, 0, &mut out);
    }
    out
}

fn cubic(p: [[f64; 2]; 4], tol: f64, depth: u32, out: &mut Vec<[f64; 2]>) {
    let (dx, dy) = (p[3][0] - p[0][0], p[3][1] - p[0][1]);
    let len = dx.hypot(dy);
    let flat = [p[1], p[2]].iter().all(|q| {
        if len < f64::EPSILON {
            (q[0] - p[0][0]).hypot(q[1] - p[0][1]) < tol
        } else {
            ((q[0] - p[0][0]) * dy - (q[1] - p[0][1]) * dx).abs() / len < tol
        }
    });
    if depth >= 12 || flat {
        out.push(p[3]);
        return;
    }
    let mid = |a: [f64; 2], b: [f64; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let (ab, bc, cd) = (mid(p[0], p[1]), mid(p[1], p[2]), mid(p[2], p[3]));
    let (abc, bcd) = (mid(ab, bc), mid(bc, cd));
    let m = mid(abc, bcd);
    cubic([p[0], ab, abc, m], tol, depth + 1, out);
    cubic([m, bcd, cd, p[3]], tol, depth + 1, out);
}

/// Scanline fill of the CLOSED polylines (px space), sampled at pixel centres.
fn fill(on: &mut [bool], w: u32, h: u32, lines: &[(Vec<[f64; 2]>, bool)], rule: FillRule) {
    let mut xs: Vec<(f64, i32)> = Vec::new();
    for row in 0..h {
        let y = f64::from(row) + 0.5;
        xs.clear();
        for (l, closed) in lines {
            if !closed || l.len() < 2 {
                continue;
            }
            for i in 0..l.len() {
                let (a, b) = (l[i], l[(i + 1) % l.len()]);
                if (a[1] <= y) == (b[1] <= y) {
                    continue;
                }
                let t = (y - a[1]) / (b[1] - a[1]);
                xs.push((a[0] + t * (b[0] - a[0]), if b[1] > a[1] { 1 } else { -1 }));
            }
        }
        xs.sort_by(|p, q| p.0.total_cmp(&q.0));
        let mut wind = 0;
        for k in 0..xs.len() {
            wind += xs[k].1;
            let inside = match rule {
                FillRule::NonZero => wind != 0,
                FillRule::EvenOdd => (k + 1) % 2 == 1,
            };
            if !inside || k + 1 >= xs.len() {
                continue;
            }
            let c0 = (xs[k].0 - 0.5).ceil().max(0.0) as u32;
            let c1 = ((xs[k + 1].0 - 0.5).ceil().max(0.0) as u32).min(w);
            for c in c0..c1 {
                on[(row * w + c) as usize] = true;
            }
        }
    }
}

/// The stroke band: every pixel centre within `r` px of a segment (round caps and joins).
fn stroke(on: &mut [bool], w: u32, h: u32, lines: &[(Vec<[f64; 2]>, bool)], r: f64) {
    let r = r.max(0.5);
    for (l, closed) in lines {
        let n = l.len();
        let segs = if *closed { n } else { n.saturating_sub(1) };
        let segs: Vec<([f64; 2], [f64; 2])> = if n == 1 {
            vec![(l[0], l[0])]
        } else {
            (0..segs).map(|i| (l[i], l[(i + 1) % n])).collect()
        };
        for (a, b) in segs {
            let bx0 = (a[0].min(b[0]) - r - 0.5).floor().max(0.0) as u32;
            let bx1 = ((a[0].max(b[0]) + r).ceil().max(0.0) as u32).min(w);
            let by0 = (a[1].min(b[1]) - r - 0.5).floor().max(0.0) as u32;
            let by1 = ((a[1].max(b[1]) + r).ceil().max(0.0) as u32).min(h);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let l2 = dx * dx + dy * dy;
            for row in by0..by1 {
                for c in bx0..bx1 {
                    let p = [f64::from(c) + 0.5, f64::from(row) + 0.5];
                    let t = if l2 > 0.0 {
                        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let q = [a[0] + t * dx, a[1] + t * dy];
                    if (p[0] - q[0]).hypot(p[1] - q[1]) <= r {
                        on[(row * w + c) as usize] = true;
                    }
                }
            }
        }
    }
}
