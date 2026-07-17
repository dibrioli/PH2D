//! **What two OVERLAPPING strokes do to each other** — the probes born from Enio's 3rd and 4th smokes of
//! the Blob's border (2026-07-16). Sibling of [`super::inflate_edge_probes`], which measures the rim of a
//! LONE form; the phenomena here need a junction, and a lone-form fixture cannot contain them.
//!
//! ## *"why does the inflate not apply at the LATERAL of the junction between
//! two overlapping strokes?"* — it is the OTHER WAY ROUND, and that is the whole bug
//!
//! [`diag_why_the_junction_does_not_inflate`] walks the union's right silhouette of a capsule crossed by a
//! blob (his photograph) and measures how far the COVER edge — the silhouette the light actually weighs —
//! moved under one Filter Layer at Depth 1 (`rho = 16 px`):
//!
//! ```text
//! rows  55- 82  capsule's plain flank    124 -> 124   +0
//! rows 107-112  blob's plain flank       137 -> 137   +0
//! rows  83- 90  the ARMPIT               124 -> 128   +4
//! ```
//!
//! **The Blob does not grow a form. It fills armpits.** A ball of radius 16 moves a plain flank by ZERO.
//!
//! [`diag_is_the_winner_out_of_its_own_reach`] prints why, texel by texel. Two gates stand between the ball
//! and the canvas, and on a flank each one is enough on its own:
//!
//! * **`v = pre_cover[si] * t <= pre_cover[gi]`.** The taper's ramp and the deposit's own soft rim decay
//!   over the SAME few texels (`t` = .76 .54 .35 .19 → `v` = 193 137 88 48, against a rim of 255 251 222
//!   125). The arriving ghost is always fainter than the paint already there, so the `max` keeps what is
//!   there and the form does not move. **A form cannot grow through its own falloff.**
//! * **`own_v >= lifted`.** Filter Layer fills `amount` uniformly — *bare canvas included* — so every empty
//!   texel claims a self-floor of `|Depth| * 1` = one whole load. The self-floor was written for a BRUSH,
//!   where `amount` outside the footprint is 0 and the floor cannot compete; under the filter it is a load
//!   of relief on paint that does not exist, and it beats the tapered ball at exactly the rim (x=138: lift
//!   3.49 vs floor 3.93 → `sbuf = 0`, nothing advects).
//!
//! Under both sits ONE fact: **the argmax picks the TALLEST source, not the nearest** (`a = 1/512` is so
//! flat that a peak 20 texels away at 10 loads beats a neighbour 1 texel away at 4.6). So `d_win` at the rim
//! is the distance to the form's PEAK — i.e. **`t` at the rim is a function of the form's SIZE**, not of how
//! far the matter had to travel. A form wider than the ball taper-fades its own rim to nothing. And because
//! `d_win` jumps across the argmax's Voronoi seams, `t` jumps, `v` jumps: that is the staircase.
//!
//! The armpit is the one place both gates open — the wedge is BARE (`pre_cover` 42, 2, 0) while a tall
//! source is still near (`d_win = 17`, `t = .75`, `v = 191`). It fills, its neighbours do not, and the
//! shoulders of that lone lobe are what the arrows in the photograph point at.
//!
//! ## *"nada pode ser feito?"* — the root cause, named
//!
//! Two perpendicular strokes, four white gashes in the armpits. [`diag_does_the_inflate_un_paint_the_armpit`]
//! answers the first question and clears the advection: **0 texels lose coverage, 0 lose pigment.** The gash
//! is coverage that never ARRIVED, framed by neighbours that got 255.
//!
//! Its coverage map draws the real thing — a **hard line at x=151, straight across a uniform stroke**:
//!
//! ```text
//! 124 |##################+++++++++++++++++++++###########|
//! 125 |#################+---..................###########|
//!                      ^ vertical arm         ^ the Inflate switches back ON here
//! ```
//!
//! Left of it the Blob does nothing at all; right of it it grows the arm to 255. Nothing in the paint
//! changes at x=151. What changes is **who wins the envelope**: the cross's CENTRE, or the local spine.
//!
//! [`diag_the_parabola_captures_what_it_cannot_serve`] names the law. A source of height `H` above a texel
//! wins that texel's envelope out to `sqrt(H/a)`, but the budget only lets it deliver out to `rho*sqrt(2)`.
//! With `a = 1/(2*D*unit^2)` and `rho = D*unit`, those two numbers are not the same number:
//!
//! ```text
//! d_capture / d_reach = sqrt(H / D)          measured on the cross: 47.5 vs 22.6 texels = 2.1x at Depth 1
//! ```
//!
//! **The parabola claims a region `sqrt(H/D)` times wider than its own ball and hands every texel in the gap
//! NOTHING** — its winner is disqualified, and a lower envelope has no second place. On paint thicker than
//! the Depth (i.e. all real impasto) the gap is always there; a **junction is the tallest point on the
//! canvas**, so its dead zone is the widest, and the boundary of its Voronoi cell is a visible hard edge
//! across paint that is otherwise uniform. That is the white gash, and the arcs rippling out of it are that
//! boundary curving.
//!
//! Every remedy in `render_inflate` — the per-source budget, the taper, the self-floor, the coverage guard —
//! exists to contain a structuring element with **unbounded support**, and each remedy is a thing the artist
//! can see. A **bounded** ball has `capture == reach == rho` identically, by construction: it cannot claim
//! what it cannot serve, so none of the four remedies has anything to do. That is not a mitigation of this
//! bug; it is the removal of its cause.
//!

use super::inflate_edge::{CX, CY, SIZE, covers_of, inflate_the_whole_layer};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Falloff};

/// The two brushes of [`capsule_crossed_by_blob`], and the geometry the armpit rows fall out of.
const CAP_R: f32 = 22.0;
const BLOB_R: f32 = 40.0;

fn arm_brush(t: &mut PainterTool, r: f32) {
    let b = BrushSpec {
        radius_px: r,
        hardness: 0.3,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
}

/// **Enio's junction** (2026-07-16, 3rd smoke): a vertical CAPSULE crossed by a fat round BLOB — the middle
/// of his photograph, and the topology his four arrows point at.
///
/// Two OVERLAPPING strokes, so the union's right silhouette has a **reflex corner** (an armpit) where the
/// blob's arc runs back into the capsule's straight side: at `|y - CY| = sqrt(BLOB_R^2 - CAP_R^2)`, i.e.
/// rows ~77 and ~143. Everything else on that side is a plain convex flank — the control.
fn capsule_crossed_by_blob() -> (PainterTool, crate::tool::RtLayerId) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    let layer = t.layers.active().expect("a layer");
    arm_brush(&mut t, CAP_R);
    t.on_canvas_pointer(cp([CX, 45.0], PointerPhase::Down));
    for k in 1..=26u8 {
        t.on_canvas_pointer(cp([CX, 45.0 + 5.0 * f32::from(k)], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([CX, 175.0], PointerPhase::Up));
    arm_brush(&mut t, BLOB_R);
    for _ in 0..2 {
        t.on_canvas_pointer(cp([CX, CY], PointerPhase::Down));
        t.on_canvas_pointer(cp([CX, CY], PointerPhase::Up));
    }
    (t, layer)
}

/// The rightmost texel on row `y` whose coverage clears `thr` — the silhouette the LIGHT draws.
fn right_cover_edge(cov: &[u8], y: u32, thr: u8) -> i32 {
    (0..SIZE)
        .rev()
        .find(|x| cov[(y * SIZE + x) as usize] >= thr)
        .map_or(-1, |x| x as i32)
}

/// The rightmost texel on row `y` whose relief clears `thr` — the silhouette the BUFFER holds.
fn right_height_edge(h: &[f32], y: u32, thr: f32) -> i32 {
    (0..SIZE)
        .rev()
        .find(|x| h[(y * SIZE + x) as usize] >= thr)
        .map_or(-1, |x| x as i32)
}

/// DIAGNOSTIC (Enio, 2026-07-16) — *"descubra porque o inflate nao se aplica na lateral da juncao entre
/// dois tracos sobrepostos"*.
///
/// Walks the union's right silhouette row by row and prints how far it MOVED. A filter applied to the whole
/// layer at one strength should grow every row of a convex flank by the same amount; the armpit rows (~77
/// and ~143) are where two strokes meet. Prints the COVER edge (what the light weighs) beside the HEIGHT
/// edge (what the buffer holds), because this line's every bug so far has been those two disagreeing.
#[test]
#[ignore]
fn diag_why_the_junction_does_not_inflate() {
    let (mut t, layer) = capsule_crossed_by_blob();
    let (cov0, h0) = (covers_of(&t, layer), heights_of(&t, layer));
    inflate_the_whole_layer(&mut t);
    let (cov1, h1) = (covers_of(&t, layer), heights_of(&t, layer));

    println!("  y | cover: pre -> post (grow) | height: pre -> post (grow) |  h@pre_edge");
    for y in 55..=165 {
        let (c0, c1) = (
            right_cover_edge(&cov0, y, 100),
            right_cover_edge(&cov1, y, 100),
        );
        let (e0, e1) = (
            right_height_edge(&h0, y, 0.5),
            right_height_edge(&h1, y, 0.5),
        );
        let hp = h1[(y * SIZE + c0.max(0) as u32) as usize];
        let mark = if (76..=78).contains(&y) || (142..=144).contains(&y) {
            " <-- ARMPIT"
        } else {
            ""
        };
        println!(
            "{y:3} | {c0:3} -> {c1:3} ({:+3}) | {e0:3} -> {e1:3} ({:+3}) | {hp:6.2}{mark}",
            c1 - c0,
            e1 - e0
        );
    }
}

/// DIAGNOSTIC — **the parabola CAPTURES farther than it can SERVE, and the gap is the artefact.**
///
/// A source of height `H` above a texel wins that texel's envelope out to `d_capture = sqrt(H/a)` — but the
/// budget only lets it deliver out to `d_reach = rho*sqrt(2)`. With `a = 1/(2*D*unit^2)` and `rho = D*unit`:
///
/// ```text
/// d_capture / d_reach = sqrt(H / D)
/// ```
///
/// So on paint THICKER than the Depth — which is all real impasto — the tallest point on the canvas claims a
/// region `sqrt(H/D)` times wider than its ball, and hands every texel in the gap NOTHING. A junction is the
/// tallest point there is, so its dead zone is the biggest, and the boundary of its Voronoi cell is a hard
/// line across otherwise uniform paint. This prints the prediction beside the measurement.
#[test]
#[ignore]
fn diag_the_parabola_captures_what_it_cannot_serve() {
    let (t, layer) = the_cross();
    let pre = heights_of(&t, layer);
    let unit = super::super::impasto_light::DEPTH_UNIT_PX;
    for depth in [0.5f32, 1.0] {
        let a = 1.0 / (2.0 * depth * unit * unit);
        let rho = depth * unit;
        let reach = rho * std::f32::consts::SQRT_2;
        let at = |x: u32, y: u32| pre[(y * SIZE + x) as usize];
        let (h_cross, h_arm) = (at(110, 110), at(150, 110));
        let delta = h_cross - h_arm;
        // Where the local spine finally overtakes the junction: pre_c - a*(dx^2 + dy^2) = pre_s - a*dy^2.
        let line = 110.0 + (delta / a).sqrt();
        println!(
            "Depth {depth}: rho {rho:.0}  reach {reach:.1}  |  junction {h_cross:.2} vs arm {h_arm:.2} \
             loads (delta {delta:.2})"
        );
        println!(
            "   capture radius of the junction = sqrt(H/a) = {:.1} texels  -> {:.1}x its reach (sqrt(H/D) = {:.2})",
            (h_cross / a).sqrt(),
            (h_cross / a).sqrt() / reach,
            (h_cross / depth).sqrt()
        );
        println!(
            "   its Voronoi cell ends at x = {line:.0}  <- the hard line in the coverage map\n"
        );
    }
}

/// **Enio's CROSS** (2026-07-16, 4th smoke): two PERPENDICULAR strokes — the tightest junction there is,
/// and the one whose four armpits came back with WHITE GASHES in them.
/// Re-exports so the ball workshop ([`super::inflate_ball_candidate`]) can drive Enio's cross fixture
/// against a candidate kernel without duplicating it.
pub(super) fn the_cross_pub() -> (PainterTool, crate::tool::RtLayerId) {
    the_cross()
}
pub(super) fn inflate_cross(t: &mut PainterTool) {
    inflate_the_whole_layer(t);
}

fn the_cross() -> (PainterTool, crate::tool::RtLayerId) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    let layer = t.layers.active().expect("a layer");
    arm_brush(&mut t, CAP_R);
    t.on_canvas_pointer(cp([CX, 55.0], PointerPhase::Down));
    for k in 1..=22u8 {
        t.on_canvas_pointer(cp([CX, 55.0 + 5.0 * f32::from(k)], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([CX, 165.0], PointerPhase::Up));
    t.on_canvas_pointer(cp([55.0, CY], PointerPhase::Down));
    for k in 1..=22u8 {
        t.on_canvas_pointer(cp([55.0 + 5.0 * f32::from(k), CY], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([165.0, CY], PointerPhase::Up));
    (t, layer)
}

/// DIAGNOSTIC (Enio, 2026-07-16, the cross) — **does the Inflate UN-PAINT?**
///
/// The advection's contract is that coverage is a PRESENCE: it maxes, the paint extends, it never thins.
/// The four white gashes in the photograph say otherwise, so this asks the canvas directly: did any texel
/// end with LESS coverage — or a more transparent pigment — than it started with?
#[test]
#[ignore]
fn diag_does_the_inflate_un_paint_the_armpit() {
    let (mut t, layer) = the_cross();
    let cov0 = covers_of(&t, layer);
    let rgba0 = t.canvas_rgba.to_vec();
    inflate_the_whole_layer(&mut t);
    let cov1 = covers_of(&t, layer);
    let rgba1 = t.canvas_rgba.to_vec();

    let (mut lost, mut worst, mut at) = (0usize, 0i32, (0u32, 0u32));
    let (mut a_lost, mut a_worst, mut a_at) = (0usize, 0i32, (0u32, 0u32));
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = (y * SIZE + x) as usize;
            let d = i32::from(cov0[i]) - i32::from(cov1[i]);
            if d > 0 {
                lost += 1;
                if d > worst {
                    worst = d;
                    at = (x, y);
                }
            }
            let da = i32::from(rgba0[i * 4 + 3]) - i32::from(rgba1[i * 4 + 3]);
            if da > 0 {
                a_lost += 1;
                if da > a_worst {
                    a_worst = da;
                    a_at = (x, y);
                }
            }
        }
    }
    println!("coverage LOST : {lost:5} texels, worst -{worst} at {at:?}");
    println!("pigment  LOST : {a_lost:5} texels, worst -{a_worst} alpha at {a_at:?}");

    let _ = (rgba0, rgba1);
    // The armpit, DRAWN. `#`=255 `+`>192 `-`>96 `.`>0 ` `=bare. A white gash is a run of ` ` with paint on
    // both sides — a texel the ball never reached while its neighbours did.
    let glyph = |v: u8| match v {
        255 => '#',
        193..=254 => '+',
        97..=192 => '-',
        1..=96 => '.',
        0 => ' ',
    };
    for (label, cov) in [("BEFORE", &cov0), ("AFTER", &cov1)] {
        println!("\n--- {label} --- armpit quadrant, x 112..162, y 112..152");
        for y in 112..152u32 {
            let row: String = (112..162u32)
                .map(|x| glyph(cov[(y * SIZE + x) as usize]))
                .collect();
            println!("{y:3} |{row}|");
        }
    }
}

/// DIAGNOSTIC — **is the envelope's winner out of its OWN reach?**
///
/// The lower envelope returns exactly ONE source per texel: the argmax of `pre[q] - a*d^2`. The post-pass
/// then judges that winner against its budget (`d^2 < 2*rho^2*amount`) and, if it fails, the texel keeps its
/// own floor and advects NOTHING. There is no second place to fall back to.
///
/// So this prints, for one row across the blob's rim: how far the winner travelled, whether it was legal,
/// and — brute force over the legal disc — whether a source that WAS in reach existed at that texel.
#[test]
#[ignore]
fn diag_is_the_winner_out_of_its_own_reach() {
    let (t, layer) = capsule_crossed_by_blob();
    let pre = heights_of(&t, layer);
    let cov = covers_of(&t, layer);
    let depth = 1.0f32;
    let unit = super::super::impasto_light::DEPTH_UNIT_PX;
    let rho = (depth * unit).ceil() as i64;
    // Filter Layer fills `amount` UNIFORMLY at the brush strength — every texel, bare canvas included.
    let s = 1.0f32;
    let g: Vec<f32> = pre.iter().map(|p| p + depth * s).collect();
    let a = 1.0 / (2.0 * depth * unit * unit);
    let (hbuf, sbuf) = super::super::sculpt_offset::blob_dilate(&g, SIZE, SIZE, a, true);
    let full_reach2 = (2 * rho * rho) as f32;
    println!(
        "rho = {rho} px   reach = sqrt(2*rho^2) = {:.1} texels   a = 1/{:.0}",
        full_reach2.sqrt(),
        1.0 / a
    );
    println!("  x | pre  cov | d_win |   t   | v=255t vs cov | lift vs floor | VERDICT");
    for (label, y) in [("ARMPIT row 86", 86u32), ("FLANK  row 110", 110u32)] {
        println!("--- {label} ---");
        for x in 118..=145u32 {
            let i = (y * SIZE + x) as usize;
            let (dx, dy) = super::super::sculpt_offset::unpack_src(sbuf[i]);
            let d2 = (dx * dx + dy * dy) as f32;
            let t = super::super::sculpt_offset::ball_taper(d2, full_reach2);
            let p0 = pre[i];
            let lifted = p0 + t * (hbuf[i].max(p0) - p0);
            let floor = p0 + depth * s;
            // Brute force: the best source whose travel is INSIDE its own ball.
            let r = full_reach2.sqrt().floor() as i64;
            let (mut best, mut bd) = (f32::MIN, 0.0f32);
            for oy in -r..=r {
                for ox in -r..=r {
                    let q2 = (ox * ox + oy * oy) as f32;
                    if q2 >= full_reach2 {
                        continue;
                    }
                    let (qx, qy) = (x as i64 + ox, y as i64 + oy);
                    if qx < 0 || qy < 0 || qx >= i64::from(SIZE) || qy >= i64::from(SIZE) {
                        continue;
                    }
                    let v = g[(qy * i64::from(SIZE) + qx) as usize] - a * q2;
                    if v > best {
                        best = v;
                        bd = q2;
                    }
                }
            }
            let _ = (best, bd);
            let v = (255.0 * t) as u8;
            let floor_won = floor >= lifted;
            let verdict = if floor_won {
                "floor wins -> sbuf=0, NO MATTER"
            } else if v <= cov[i] {
                "ball wins, but v <= cov -> SKIPPED"
            } else {
                "GROWS"
            };
            println!(
                "{x:3} | {p0:4.1} {:3} | {:5.1} | {t:.3} | {v:5} vs {:3}   | {lifted:5.2} vs {floor:5.2} | {verdict}",
                cov[i],
                d2.sqrt(),
                cov[i]
            );
        }
    }
}
