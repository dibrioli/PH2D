//! Gates for **Inflate's SUPPORT** — where the Blob is allowed to write at all.
//!
//! Born from Enio's 4th smoke of the Inflate (2026-07-15): *"ainda levanta área retangular abaixo do
//! brush"* — a hard, axis-aligned shelf around the dome, on THICK paint with a BIG brush. The reach-cap of
//! `c8ea47de` killed the parabola's unbounded runaway, and the rectangle survived it, because the runaway
//! was only the mechanism's LOUD half.
//!
//! ## The quiet half, with arithmetic
//!
//! The peak field the engine dilates is `g = pre ± |Depth|·amount` — so a texel the brush never touched
//! (`amount = 0`) still stands in the envelope as a parabola rooted at its own ground, `pre`. On sloped
//! terrain that source LIFTS its downhill neighbours: within the (correct, circular) reach cap, the lift at
//! a texel with local slope `m` is `max over d ≤ ρ√2 of (m·d − a·d²)` — for the thick-blob flanks of the
//! screenshot (`m ≈ 0.15 loads/px`, Depth 1) that is **≈ 1.2 loads over the ENTIRE write window**, ending
//! exactly at `kr`'s edge. `kr` is a rectangle. That is the shelf: not the runaway — the *ambient terrain
//! dilation*, applied wherever the window reaches, gated by nothing the artist did.
//!
//! The fixtures the previous gates ran on could not show it: flat ground beside the form (the plateau), or
//! ground fully covered by a Constant-falloff brush. *A gate only proves what its fixture contains* —
//! so these run on the handoff's product numbers: brush ≥ 60 px, `pre` ≥ 10 loads, laid by the REAL
//! deposit, overlapping, ≥ 2 dabs.
//!
//! The law these gates pin: **the Blob's support is the brush's touched set dilated by each toucher's own
//! ball — never the write window.** Outside it, the canvas is not "close to" `pre`; it is `pre`, bit for bit.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Falloff};
use std::sync::Arc;

const INFLATE: u8 = 7;

/// The light's height→pixel gain, re-read from the product so this file cannot drift from it.
const UNIT: f32 = super::super::impasto_light::DEPTH_UNIT_PX;

/// Thick, sloped, PRODUCT-laid paint: two overlapping strokes from the real deposit with a big brush.
///
/// `derive_height` scales the deposit by `radius/10`, so a 60-px brush lays ~6 loads per pass and the
/// overlap band clears the handoff's "≥ 10 loads" — with settled, gently-sloping flanks, which are the
/// terrain the ambient dilation feeds on. A synthetic cliff here would repeat the exact fixture mistake
/// this file's module docs recount.
fn thick_overlapping_paint(size: u32) -> (PainterTool, crate::tool::RtLayerId, Vec<f32>) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 60.0,
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
    let layer = t.layers.active().expect("a layer");

    // Two horizontal strokes, 30 px apart: the band between them is paint OVER paint (strokes ADD).
    for y in [113.0f32, 143.0] {
        t.on_canvas_pointer(cp([70.0, y], PointerPhase::Down));
        let mut x = 74.0;
        while x <= 186.0 {
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            x += 4.0;
        }
        t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    }

    let relief = heights_of(&t, layer);
    let peak = relief.iter().copied().fold(f32::MIN, f32::max);
    assert!(
        peak >= 10.0,
        "fixture: the overlap band peaks at {peak:.1} loads — the handoff's repro says ≥ 10 (thick,
         built-up paint); the deposit or the brush size drifted"
    );
    (t, layer, relief)
}

/// `touched ⊕ disc(reach)` — every texel within `reach` px of a texel the brush touched.
///
/// Brute-force disc stamping: O(touched · reach²), run on a 256² canvas — a fraction of the deposit the
/// fixture already paid for.
fn dilate_touched(touched: &[f32], size: u32, reach: i64) -> Vec<bool> {
    let s = size as i64;
    let mut mask = vec![false; touched.len()];
    for (i, a) in touched.iter().enumerate() {
        if *a <= 0.0 {
            continue;
        }
        let (cx, cy) = ((i as i64) % s, (i as i64) / s);
        for dy in -reach..=reach {
            let y = cy + dy;
            if y < 0 || y >= s {
                continue;
            }
            for dx in -reach..=reach {
                let x = cx + dx;
                if x < 0 || x >= s || dx * dx + dy * dy > reach * reach {
                    continue;
                }
                mask[(y * s + x) as usize] = true;
            }
        }
    }
    mask
}

/// **Beyond the ball's reach the canvas is `pre`, BIT FOR BIT — on thick paint, under a big brush.**
///
/// This is the residual rectangle of Enio's 4th smoke, as a gate, on the handoff's product numbers
/// (brush 64 px ≥ 60, `pre` ≥ 10 loads and overlapping, ≥ 2 dabs). It also *decides the handoff's A-vs-B
/// question* by construction: it probes the `heights` buffer, so if it is red the shelf is HEIGHT (A),
/// not a relight seam (B).
///
/// The oracle is **byte-identity outside `touched ⊕ (ρ√2 + smooth + 1)`** — no threshold, because the
/// domain there is empty: a ball pressed along the brush's falloff has nothing to say about texels its
/// radius cannot span. Any write at all is the window's edge showing through, and the window is a
/// rectangle.
///
/// **The red this gate was born with:** the pre-fix `render_inflate` itself — untouched sources rooted at
/// their own ground AND winners held only to the global ρ√2 — measured **11 830 texels / 7.09 loads** of
/// shelf on this very fixture. The support is now held by TWO independent layers (the sentinel keeps
/// untouched sources from contributing; the per-winner budget disqualifies over-reachers), so no single
/// mutation reopens it — remove BOTH (`g = pre` where `amount = 0`, and `t := 1.0` unconditionally) and
/// this bleeds exactly the smoke report. Each layer alone is pinned by its own gate: the sentinel by
/// [`an_untouched_wall_does_not_shadow_the_balls_lift`], the budget by
/// [`a_weak_dabs_ball_is_small_and_its_reach_shrinks_with_it`] and the taper by
/// [`the_balls_edge_meets_the_ground_without_a_cliff`].
#[test]
fn the_inflate_writes_nothing_beyond_the_balls_reach_on_thick_paint() {
    let size = 256u32;
    let (mut t, layer, before) = thick_overlapping_paint(size);

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.radius_px = 64.0; // the handoff's "pincel GRANDE"
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(1.0); // Depth +1.0 ⇒ ρ = 16 px, ρ√2 ≈ 22.6

    // ≥ 2 dabs, on the thick overlap band — and the touched set is captured BEFORE pen-up, because the
    // session (and its `amount`) dies at commit.
    // Captured and measured BEFORE pen-up: the Up stamps the stroke-smoothing window's held-back TAIL
    // dabs (`paint_end` → `stroke.finish` → `stamp_dabs`) and then kills the session — so `amount` read
    // after it describes neither the whole stroke nor anything at all. The support law is a property of
    // EVERY render, so mid-stroke is not a concession; the tail batch runs the same kernel.
    t.on_canvas_pointer(cp([110.0, 128.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([146.0, 128.0], PointerPhase::Move));
    let touched: Vec<f32> = (*t.paint.sculpt.amount).clone();
    let after = heights_of(&t, layer);
    t.on_canvas_pointer(cp([146.0, 128.0], PointerPhase::Up));
    assert!(
        touched.iter().any(|a| *a > 0.0),
        "fixture: the sculpt stroke touched nothing"
    );

    // ρ√2 for Depth 1.0, plus the smooth (0 here), plus one texel so the boundary itself is not argued.
    let rho = (1.0f32 * UNIT).ceil() as i64;
    let reach = ((rho * rho * 2) as f64).sqrt().ceil() as i64 + 1;
    let inside = dilate_touched(&touched, size, reach);

    // The PRESENCE sibling: the ball genuinely lifted the paint — so the byte-identity below proves the
    // reach is BOUNDED, not that the tool is dead. (It does NOT demand growth beyond the touched set:
    // with a SOFT falloff the ball's radius dies at the brush edge with the amount, so the support IS
    // the touched set — "na borda nada se move", the dome gate's own law. Growth past the footprint is
    // the Constant-falloff story, pinned by the plateau gate.)
    let grew = (0..before.len())
        .filter(|i| inside[*i])
        .any(|i| after[i] - before[i] > 0.25);
    assert!(
        grew,
        "fixture: the ball lifted nothing at all — the absence check below is vacuous"
    );

    // THE gate: outside the ball's reach, not one bit moved.
    let mut worst = 0.0f32;
    let mut worst_at = (0u32, 0u32);
    let mut violations = 0usize;
    for i in 0..before.len() {
        if inside[i] || after[i].to_bits() == before[i].to_bits() {
            continue;
        }
        violations += 1;
        let d = (after[i] - before[i]).abs();
        if d > worst {
            worst = d;
            worst_at = (i as u32 % size, i as u32 / size);
        }
    }
    assert_eq!(
        violations,
        0,
        "Inflate wrote {violations} texels BEYOND the ball's reach (worst: {worst:.3} loads at \
         {worst_at:?}). A ball pressed along the brush's falloff cannot span there; these writes are the \
         compute window showing through — and the window is a rectangle, which is exactly the residual \
         shelf of Enio's 4th smoke (2026-07-15). The ambient mechanism: an untouched texel's parabola is \
         rooted at its own ground (`g = pre`), so on sloped thick paint it dilates its downhill \
         neighbours — terrain the artist never touched, lifted by terrain the artist never touched, \
         cropped at `kr`."
    );
}

/// **An untouched wall does not SHADOW the ball's lift — the sentinel's own gate.**
///
/// The post-pass disqualifies any winner the brush never touched, and that alone kills the ambient
/// shelf — so *why the sentinel?* Because the envelope keeps ONE winner per texel: let untouched terrain
/// compete (`g = pre`) and a tall bare wall OUTWINS the touched source beside it (its ground beats
/// `peak − a·d²`), gets disqualified in the post-pass, and the texel falls back to `pre` — while the
/// touched source that legitimately reaches it goes unheard. The fattening ring develops HOLES wherever
/// the terrain around the stroke is interesting, which is the same bug as the shelf with the sign
/// flipped: terrain the artist never touched, deciding what the brush does.
///
/// With the sentinel, untouched texels never enter the race, so the winner is always the best TOUCHED
/// source and the lift beside a wall is the same as the lift beside nothing.
///
/// **Mutation that must bleed:** root untouched texels at their own ground again (`g = pre` where
/// `amount = 0` in `render_inflate`'s peak-field build) — the wall wins, is disqualified, and the
/// receiver's lift vanishes.
#[test]
fn an_untouched_wall_does_not_shadow_the_balls_lift() {
    let size = 160u32;
    let n = (size * size) as usize;
    let (mut t, layer, _) = sculpt_canvas(size);

    // Thin paint everywhere (0.5 loads), with a TALL bare wall from x = 100 on — thick built-up paint
    // the sculpt brush will NOT touch.
    let field: Vec<f32> = (0..n)
        .map(|i| if (i as u32 % size) >= 100 { 20.0 } else { 0.5 })
        .collect();
    t.heights.insert(layer, Arc::new(field.clone()));
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.radius_px = 12.0; // touches x ∈ [68, 92] — never the wall
    b.falloff = Falloff::Constant;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(1.0); // ρ = 16 px: the ball spans the gap to the receiver easily

    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Down));
    let touched: Vec<f32> = (*t.paint.sculpt.amount).clone();
    let after = heights_of(&t, layer); // before Up: the tail batch would grow `amount` past this capture

    let wall_i = (80 * size + 105) as usize;
    assert!(
        touched[wall_i] <= 0.0,
        "fixture: the brush touched the wall — the shadow this gate stages cannot form"
    );
    // The receiver: bare floor 3 px past the brush's edge, 5 px shy of the wall — inside the touched
    // sources' own reach, and inside the wall's shadow if the wall is allowed to compete.
    let recv = (80 * size + 95) as usize;
    assert!(touched[recv] <= 0.0, "fixture: the receiver must be UNtouched");

    let lift = after[recv] - field[recv];
    assert!(
        lift > 0.05,
        "the texel between the brush and the wall rose only {lift:.3} loads. A touched source 3 px away \
         reaches it; if this is ~0 the WALL won the envelope and was then disqualified — terrain the \
         artist never touched, silencing the stroke they made. That is what the untouched-source \
         sentinel exists to prevent."
    );
}

/// **The ball's edge MEETS the ground — the support's boundary is not a cliff.**
///
/// Holding every texel to its winner's own reach bounds WHERE the ball writes; this gate bounds HOW the
/// writing ends. A cap alone leaves the last reachable texel carrying the slope-advance in full
/// (`≈ m·R` loads — on the thick fixture's flanks, a circular wall standing exactly on the support's
/// edge: the residual shelf again, rounded). The taper fades the lift to zero ACROSS the sphere's
/// outer flank — squared, so the gradient lands at zero WITH the lift — and the boundary stops being
/// drawable at all.
///
/// Both sides measured on this fixture before the bar was set (the limiar-no-chute lesson): with the
/// squared taper the outer 3 px of the support carry ≤ ~0.15 loads; with the taper deleted (`t := 1`
/// inside the reach) they carry ~2–3 loads. The bar sits in the empty middle, and it also convicts the
/// half-regression (the LINEAR taper, ~0.6 loads).
///
/// **Mutation that must bleed:** in `render_inflate`'s post-pass, replace the squared taper with `1.0`
/// inside the reach (delete the fade) — the ring lift jumps an order of magnitude.
#[test]
fn the_balls_edge_meets_the_ground_without_a_cliff() {
    let size = 256u32;
    let (mut t, layer, before) = thick_overlapping_paint(size);

    // CONSTANT falloff, dwelled: amount stays HIGH right up to the touched set's hard edge, so sources
    // at the boundary own a full-size ball and the ring below sits at THEIR reach. (A soft falloff
    // cannot stage this gate at all: its edge sources have `amount → 0`, the per-source budget already
    // strangles their reach, and the taper never gets a wall to fade — verified while mutating.)
    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.radius_px = 64.0;
    b.falloff = Falloff::Constant;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(1.0);

    t.on_canvas_pointer(cp([110.0, 128.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([146.0, 128.0], PointerPhase::Move));
    // Before Up — the tail batch would grow `amount` past this capture (see the sibling gate's note).
    let touched: Vec<f32> = (*t.paint.sculpt.amount).clone();
    let after = heights_of(&t, layer);
    t.on_canvas_pointer(cp([146.0, 128.0], PointerPhase::Up));

    // The ring sits at the strongest toucher's OWN reach — the support's true edge on this stroke.
    let a_max = touched.iter().copied().fold(0.0f32, f32::max).min(1.0);
    let rho = 1.0f32 * UNIT;
    let reach = f64::from(rho * (2.0 * a_max).sqrt()).ceil() as i64;
    let outer = dilate_touched(&touched, size, reach);
    let core = dilate_touched(&touched, size, reach - 1);

    // The OUTER TEXEL of the support — the step the eye would see at the boundary, since one texel
    // further out is `pre` bit-for-bit (the sibling gate). Not a 3-px band: further in, the lift is the
    // secant law doing its pinned-correct job on a steep flank, and it is legitimately large there.
    let mut ring_max = 0.0f32;
    let mut ring_n = 0usize;
    for i in 0..before.len() {
        if outer[i] && !core[i] {
            ring_n += 1;
            ring_max = ring_max.max((after[i] - before[i]).abs());
        }
    }
    assert!(ring_n > 300, "fixture: the support ring is empty — vacuous");
    assert!(
        ring_max < 0.1,
        "the ball's outermost written texel still carries {ring_max:.3} loads — a drawable wall at the \
         support's boundary, standing on every thick flank (rounded, but the same shelf of Enio's 4th \
         smoke). The lift must FADE to zero at the reach: the squared taper zeroes value AND gradient \
         there. (Both sides measured on this fixture: squared taper = 0.0024, taper deleted = 1.269; \
         the bar also convicts the half-regression to a LINEAR fade, ≈ 0.3.)"
    );
}

/// **A weak dab's ball is SMALL — its reach shrinks with its falloff, not just its height.**
///
/// The Blob's radius follows the falloff (`peak = |Depth|·amount`), and the reach must follow the SAME
/// number: a parabola of peak `p` has spent itself after `√(p/a)` — that is where its ball ends. Capping
/// every source at the full-strength `ρ√2` instead lets a barely-touched source keep lifting sloped ground
/// far past its own ball, which is the residual shelf again, wearing the stroke's soft edge.
///
/// The fixture is a RAMP (slope 0.05 loads/px — well inside what the settle leaves on a thick flank) under
/// one weak dab, so the uphill subsidy is real and the per-source bound is the only thing standing.
///
/// **Mutation that must bleed:** in `render_inflate`'s post-pass, read the reach from the full Depth
/// instead of the winner's own `amount` (`reach²_s = 2ρ²` for every source) — the weak dab's lift runs to
/// the full-strength radius and the far probe moves.
#[test]
fn a_weak_dabs_ball_is_small_and_its_reach_shrinks_with_it() {
    let size = 160u32;
    let n = (size * size) as usize;
    let (mut t, layer, _) = sculpt_canvas(size);

    // A ramp: pre rises 0.05 loads per texel of x. Covered everywhere (it is all paint).
    let ramp: Vec<f32> = (0..n)
        .map(|i| (i as u32 % size) as f32 * 0.05)
        .collect();
    t.heights.insert(layer, Arc::new(ramp.clone()));
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();

    arm_sculpt(&mut t, INFLATE, 0.5, 0.25); // a WEAK touch: strength 0.25
    let mut b = t.paint.brush;
    b.radius_px = 24.0;
    b.falloff = Falloff::Constant;
    b.strength = 0.25;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(1.0); // Depth +1.0 ⇒ full-strength ρ√2 ≈ 22.6 px

    let c = 80.0f32;
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    let touched: Vec<f32> = (*t.paint.sculpt.amount).clone();
    let after = heights_of(&t, layer); // before Up: the tail batch would grow `amount` past this capture

    let a_max = touched.iter().copied().fold(0.0f32, f32::max);
    assert!(
        a_max > 0.0 && a_max <= 0.5,
        "fixture: the dab was supposed to be WEAK (amount ≤ 0.5), got {a_max:.2} — at full strength the \
         per-source reach equals the global one and this gate proves nothing"
    );

    // Each source's own ball: reach²(s) = 2ρ²·amount(s). The mask uses the strongest touch, + 1 texel.
    let rho = 1.0f32 * UNIT;
    let reach = (rho * (2.0 * a_max).sqrt()).ceil() as i64 + 1;
    let inside = dilate_touched(&touched, size, reach);

    let before = &ramp;
    let grew = (0..n).any(|i| inside[i] && after[i] > before[i] + 0.05);
    assert!(grew, "fixture: the weak dab did nothing at all — vacuous");

    let mut violations = 0usize;
    let mut worst = 0.0f32;
    for i in 0..n {
        if !inside[i] && after[i].to_bits() != before[i].to_bits() {
            violations += 1;
            worst = worst.max((after[i] - before[i]).abs());
        }
    }
    assert_eq!(
        violations, 0,
        "a dab of amount {a_max:.2} wrote {violations} texels beyond ITS OWN ball (reach {reach} px; \
         worst lift {worst:.3} loads). The falloff sets the ball's radius — `peak = |Depth|·amount` — so \
         the reach must be `ρ·√(2·amount)`, per source. A global ρ√2 cap keeps the shelf alive at every \
         soft stroke edge, which is where every real stroke ends."
    );
}
