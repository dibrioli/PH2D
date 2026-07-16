//! Gates for **the Blob's EDGE** — where the grown form ends, and what the artist sees there.
//!
//! Born from Enio's smoke of the whole-layer filter (2026-07-16): the Filter Layer / Inflate itself was
//! *"muito bom!"*, and its **border** was rejected — the relief grows with a serrated, torn rim, and
//! *"essa irregularidade externa é imune ao filtro global e ao pincel smooth, nada pode corrigi-la."*
//!
//! ## The ball had TWO edges and they disagreed
//!
//! The post-pass in [`super::super::sculpt_blur::render_inflate`] fades the ball's **lift** to zero across
//! the outer 30% of its reach (the taper: a parabola is a sphere only near the apex). It did not fade the
//! **matter**: the advection copied `pre_cover` at FULL strength out to the last qualifying texel and then
//! stopped dead. The light weighs by coverage (`impasto_light::paint_body(cover) = cover`), so the
//! silhouette the artist sees was cut binary at `d² = reach2_s` — a bound read at the winner of a DISCRETE
//! argmax, which changes texel to texel in a Voronoi-like pattern. The border *was* that pattern,
//! binarised. That is the staircase in the photographs.
//!
//! The law these gates pin: **the height and the matter agree on where the form ends.** One taper, asked
//! once ([`super::super::sculpt_offset::ball_taper`]), answered the same way by both.
//!
//! ## The fixture is Enio's repro, not the probe's slab
//!
//! *A blob with high relief and a **curved** border.* The probe's rectangular slab hides this: its border is
//! axis-aligned, so the argmax pattern along it is regular and the staircase has nothing to climb. This line
//! has been bitten three times by a fixture that did not contain the phenomenon — the module docs of
//! [`super::inflate_support`] recount two of them.
//!
//! ## The numbers, measured (2026-07-16), on `diag_inflate_edge_profile`
//!
//! Radial coverage from the blob's centre outward:
//!
//! ```text
//! the deposit's own edge  255, 251, 222, 125,  11, 0        max step 114   (a real paint edge)
//! inflated, BEFORE        255, 255, 255, 255, 255, 0        max step 255   (a cookie cutter)
//! inflated, AFTER         255, 193, 193, 137,  88, 48, 0    max step  62   (a ramp)
//! ```

use super::super::sculpt_filter::FilterScope;
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Falloff};

const INFLATE: u8 = 7;

/// The PRE-EXISTING stale-whisper ghost, measured on the shipped kernel (`f8902dfc`) and unchanged by the
/// taper: 2 texels, 4/255 deep. See `a_knob_touched_mid_stroke_does_not_move_the_picture` — these are the
/// budget of a defect this fix inherited and must not grow, never a licence to add to it.
const GHOST_TEXELS: usize = 2;
const GHOST_DEPTH: u8 = 4;

/// The canvas the fixture paints on, and the blob's centre.
pub(super) const SIZE: u32 = 220;
pub(super) const CX: f32 = 110.0;
pub(super) const CY: f32 = 110.0;

/// **A round blob of THICK paint, laid by the real deposit** — Enio's repro.
///
/// Three taps of a big soft brush on one spot: strokes ADD, so the mound clears the report's "high relief"
/// (12 loads), and the `Smooth` falloff gives it the **soft, curved border** the slab has not got. Not a
/// synthetic dome: the failure this file exists to prevent is a gate measuring a canvas the product cannot
/// produce, and this line has already paid for that one twice.
pub(super) fn thick_round_blob() -> (PainterTool, crate::tool::RtLayerId) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    let b = BrushSpec {
        radius_px: 40.0,
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

    // A tap is Down+Up: one dab, one stroke. Three of them on the same centre, and strokes add.
    for _ in 0..3 {
        t.on_canvas_pointer(cp([CX, CY], PointerPhase::Down));
        t.on_canvas_pointer(cp([CX, CY], PointerPhase::Up));
    }
    (t, layer)
}

pub(super) fn covers_of(t: &PainterTool, layer: crate::tool::RtLayerId) -> Vec<u8> {
    t.covers
        .get(&layer)
        .map(|c| (**c).clone())
        .unwrap_or_default()
}

/// Enio's exact repro: SCULP → Inflate → **Filter Layer**, at full Depth.
pub(super) fn inflate_the_whole_layer(t: &mut PainterTool) {
    arm_sculpt(t, INFLATE, 0.5, 1.0);
    t.set_sculpt_depth(1.0);
    assert!(
        t.filter_sculpt_layer(FilterScope::Layer),
        "fixture: the filter refused to run"
    );
}

/// The coverage along the ray from the blob's centre out to `+x` — the blob is round, so a radius IS its
/// cross-section.
pub(super) fn radial_cover(cov: &[u8]) -> Vec<u8> {
    ((CX as u32)..SIZE - 2)
        .map(|x| cov[((CY as u32) * SIZE + x) as usize])
        .collect()
}

/// The biggest step between neighbours — how hard an edge is, in coverage units.
pub(super) fn max_step(profile: &[u8]) -> u8 {
    profile
        .windows(2)
        .map(|w| w[0].abs_diff(w[1]))
        .max()
        .unwrap_or(0)
}

/// **The rim the Inflate grows is FEATHERED — it is no harder than an edge the brush itself can lay.**
///
/// This is Enio's smoke, as a number. The oracle is the **deposit's own edge on the same fixture**, and that
/// is the point of it: the reference is not invented, it is what *paint* looks like in this product. The
/// Inflate grows paint; if the edge it grows is harder than any edge the brush can deposit, what it grew is
/// not paint — it is a cookie cutter, which is exactly the report.
///
/// Measured: the deposit's rim steps by **114**; the Blob's rim stepped by **255** (255 → 0 in one texel)
/// and now steps by **62**. The moat is on both sides of the deposit's number.
///
/// **Mutation that must bleed:** drop the taper from the advection — deliver `pre_cover[si]` verbatim
/// (`let v = pre_cover[si];`) — and the step goes back to 255. Equally: make [`ball_taper`] return `1.0`
/// whenever it returns non-zero.
#[test]
fn the_inflated_rim_is_feathered_not_cut() {
    let (mut t, layer) = thick_round_blob();
    let deposit = radial_cover(&covers_of(&t, layer));
    let deposit_step = max_step(&deposit);

    // The fixture must CONTAIN the phenomenon: a hard-edged brush would lay a 255-step of its own and make
    // the comparison below vacuous. Zero does not fail unless you make it fail.
    assert!(
        deposit_step < 128,
        "fixture: the deposit's own rim already steps by {deposit_step} — this brush lays a hard edge, so \
         'no harder than the deposit' is not a claim about anything. Soften the fixture's falloff."
    );

    inflate_the_whole_layer(&mut t);
    let grown = radial_cover(&covers_of(&t, layer));
    let grown_step = max_step(&grown);

    assert!(
        grown_step <= deposit_step,
        "the Inflate left the paint's edge stepping by {grown_step} coverage per texel; the DEPOSIT's own \
         edge on this very fixture steps by {deposit_step}. So the Blob cut a rim harder than any rim this \
         brush can paint.\n\n  deposit: {deposit:?}\n  grown:   {grown:?}\n\nThe light weighs by coverage, \
         so that step IS the silhouette the artist sees — and the place it is cut is read at the winner of \
         a discrete argmax, which is a Voronoi pattern. Binarised, it is the staircase of Enio's smoke \
         (2026-07-16)."
    );
}

/// **Where the ball's lift has faded, the matter it carried has faded too** — the two edges agree.
///
/// The sibling of the gate above, and the one that names the *mechanism* rather than the symptom. The height
/// fades across the ball's outer flank (`reach − equator = ρ(√2−1)` ≈ 6.6 texels at Depth 1); the matter used
/// to arrive at full strength across that whole flank and vanish at the end of it. So the form's edge was
/// described **twice, differently**: the relief said *"I fade out over six texels"* and the paint said
/// *"I stop, here"* — and the light believes the paint.
///
/// Two assertions, both dead: the transition has **partial** texels at all (a binary rim has none), and the
/// last paint the ball delivers is a **whisper** — because the lift that delivered it is all but spent.
///
/// Measured: partial texels **0 → 5**; the outermost delivered coverage **255 → 48**.
///
/// **Mutation that must bleed:** the same one — deliver `pre_cover[si]` verbatim. The rim goes binary: zero
/// partial texels, and the outermost coverage is a full 255.
#[test]
fn the_matter_fades_where_the_ball_fades() {
    let (mut t, layer) = thick_round_blob();
    inflate_the_whole_layer(&mut t);
    let grown = radial_cover(&covers_of(&t, layer));

    // The transition: from the last fully-covered texel to the first bare one.
    let last_full = grown.iter().rposition(|c| *c == 255).expect("solid paint");
    let first_bare = grown
        .iter()
        .skip(last_full)
        .position(|c| *c == 0)
        .map(|p| p + last_full)
        .expect("bare canvas past the blob");
    let transition = &grown[last_full..=first_bare];
    let partial = transition.iter().filter(|c| **c > 0 && **c < 255).count();

    assert!(
        partial >= 3,
        "the grown rim goes from solid paint to bare canvas through {partial} partly-covered texels: \
         {transition:?}. The ball's lift fades over its outer flank — about {:.1} texels at this Depth — so \
         the matter it carries has to fade over the same flank. A rim with no partial coverage is a rim the \
         ball did not fade at all: the height says the form fades out and the paint says it stops dead, and \
         the light believes the paint.",
        (super::super::impasto_light::DEPTH_UNIT_PX) * (std::f32::consts::SQRT_2 - 1.0)
    );

    let outermost = transition
        .iter()
        .rev()
        .find(|c| **c > 0)
        .copied()
        .expect("some paint in the transition");
    assert!(
        outermost < 128,
        "the outermost texel the ball painted got a coverage of {outermost} — a full coat, laid at the very \
         distance where the ball's lift has run out. What arrives has to arrive at the strength that got \
         there."
    );
}

/// **A faster mouse does not grow a different rim** — the matter's half of
/// [`super::a_faster_mouse_does_not_sculpt_deeper`].
///
/// That gate says it for the height, in the sentence this line already owns: *the same geometry, delivered
/// coarsely and delivered finely, must leave the same relief* — byte for byte, because the dab list is
/// identical either way (the stroke engine spaces dabs **by DISTANCE**; only the batching differs), so
/// anything less than equality is the batching leaking into the result. A 1000 Hz mouse must not sculpt
/// deeper than a 125 Hz one.
///
/// The advection is the one write the height's gate cannot see, and it is now a composite — so it can leak
/// the batching in its own way: read the LIVE pixel for the destination and each batch lays another coat,
/// so the rim darkens with the polling rate. Same law, same sentence, one plane over.
///
/// A LIGHT touch, for the reason the sibling spells out: at saturation `amount` is 1 from the first dab, `v`
/// never grows, each texel composites exactly once, and the bug hides. (Too light and the ball never beats
/// its self-floor and nothing advects at all — the window is real but it is a window.)
///
/// **Mutation that must bleed:** compose against `rgba[gi*4+k]` (the live pixel) instead of
/// `pre_rgba[gi*4+k]` (the frozen plane) in `render_inflate`'s advection.
#[test]
fn a_faster_mouse_does_not_grow_a_different_rim() {
    let run = |samples: u32| -> Vec<u8> {
        let (mut t, _) = thick_round_blob();
        arm_sculpt(&mut t, INFLATE, 0.5, 0.4);
        t.set_sculpt_depth(1.0);
        // The SAME straight line, reported by a lazy mouse and by a frantic one. The dab list is identical
        // (spacing is by distance); only the number of batches — and therefore of renders — changes.
        let (a, b) = ([70.0f32, CY], [150.0f32, CY]);
        let mut path = vec![a];
        for k in 1..=samples {
            let f = k as f32 / samples as f32;
            path.push([a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]);
        }
        drag(&mut t, &path);
        (*t.canvas_rgba).clone()
    };

    let coarse = run(3);
    let fine = run(40);
    let worst = coarse
        .iter()
        .zip(&fine)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert_eq!(
        worst, 0,
        "the same line, delivered in 3 pointer batches and in 40, left pixels differing by up to \
         {worst}/255. The dab list is identical either way — the engine spaces dabs by DISTANCE — so this \
         is the batching leaking into the picture: an advection that composes onto the LIVE canvas takes one \
         coat per batch, and the rim darkens with the polling rate of the mouse. The artist would feel it \
         constantly and never once be able to describe it."
    );
}

/// **The grown rim is made of the PAINT'S material, and it fades with the paint.**
///
/// The third plane the ball carries, and the one **no gate reached at all** until a surviving mutant said
/// so — nothing had ever read `mats` after a sculpt, so the material's whole advection was unproven. A
/// `Material` is an IDENTITY — what the paint IS — so it composes `over` at the opacity that arrived,
/// exactly as the pigment does, and for the same reason the deposit's own merge gives
/// (`impasto_live::commit_stroke_height`): the paint on top is the paint you see.
///
/// The claim is stated as an EQUALITY, which is what makes it sharp. Over a ground of 0 with paint of 255,
/// the arriving coverage is `255·t` and the arriving material is `over(0, 255, t) = 255·t` — **the same
/// number**. So the two planes must agree texel by texel: they are one ball, faded by one taper. And the rim
/// must actually fade across its width, or the ball is copying its source verbatim and the outermost breath
/// of paint wears a full mirror.
///
/// The fixture paints with a deliberately EXTREME material (mirror-metal) precisely so the ground's neutral
/// and the paint's cannot be confused: on a `NEUTRAL` brush both ends read 0 and the asserts pass no matter
/// what the code does.
///
/// **Mutation that must bleed:** copy the source's material verbatim (`mat[gi] = pre_mats[si];`) — measured:
/// a rim texel covered 22/255 wearing 255/255 metal. Or compose by a constant `a8 = 255`, which is the same
/// thing.
///
/// **A mutant that survives, and honestly should:** rounding the arriving opacity down
/// (`a8 = (t * 255.0) as u32`) shifts the material by at most 1/255 at the fringe — where the coverage is
/// near zero and the light weighs material BY coverage. It is below the resolution of the data and of any
/// oracle that models appearance; a gate tuned to catch it would be modelling the arithmetic instead of the
/// picture, which is the thing this line's oracles are forbidden to do.
#[test]
fn the_grown_rim_wears_the_paints_material_and_fades_with_it() {
    let (mut t, layer) = thick_round_blob();
    // A mirror-metal paint on a neutral ground — the two ends of the material axis, so "which material is
    // this?" has an answer the assert can read.
    let mut b = t.paint.brush;
    b.impasto_metallic = 1.0;
    b.impasto_roughness = 0.0;
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.on_canvas_pointer(cp([CX, CY], PointerPhase::Down));
    t.on_canvas_pointer(cp([CX, CY], PointerPhase::Up));

    let metal = ph2d_painter_brush::material::Material {
        shine: b.impasto_shine,
        roughness: 0.0,
        metallic: 1.0,
        wax: b.impasto_wax,
        wax_color: b.impasto_wax_color,
    }
    .to_bytes();
    // The ground's metallic, read at a corner texel the blob never reached.
    let ground = t.mats.get(&layer).map(|m| m[0][2]).unwrap_or(0);
    let cov0 = covers_of(&t, layer);
    assert_ne!(
        metal[2], ground,
        "fixture: the paint's material and the ground's are the same, so this gate cannot fail"
    );

    inflate_the_whole_layer(&mut t);
    let cov1 = covers_of(&t, layer);
    let mats = t.mats.get(&layer).expect("a material plane").clone();

    // The rim: texels the ball painted onto canvas the deposit had left bare.
    let mut grown: Vec<(u8, u8)> = (0..cov0.len())
        .filter(|i| cov0[*i] == 0 && cov1[*i] > 0)
        .map(|i| (cov1[i], mats[i][2]))
        .collect();
    assert!(
        grown.len() > 100,
        "fixture: the ball grew onto only {} bare texels",
        grown.len()
    );
    grown.sort_by_key(|(c, _)| *c);

    // ONE taper, two planes. The ground's metallic is 0 and the paint's is 255, so the arriving coverage is
    // `255·t` and the arriving material is `over(0, 255, t) = 255·t` — the SAME number. They cannot disagree
    // unless the matter has stopped asking the taper how much of it got here.
    for (cov, met) in &grown {
        assert!(
            met.abs_diff(*cov) <= 1,
            "a rim texel is covered {cov}/255 but its metallic is {met}/255. Over a ground of 0 with paint of \
             255 both are the same `255·t` — the coverage and the material are the same ball, faded by the \
             same taper, so a gap between them means one of them stopped asking."
        );
    }
    // …and it really does FADE across the rim: a whisper of metal over bare canvas is not a mirror.
    let (thick_cov, thick_metal) = *grown.last().expect("a thickly-painted rim texel");
    let (thin_cov, thin_metal) = grown[0];
    assert!(
        thin_metal * 2 < thick_metal,
        "the rim's material runs {thin_metal}..{thick_metal}/255 (coverage {thin_cov}..{thick_cov}) — it is \
         not fading at all. The paint it grew from is {}/255 metal; if the whole rim wears that, the ball is \
         copying its source verbatim and the outermost breath of paint is a full mirror.",
        metal[2]
    );
}

/// **The Inflate grows the form's rim; it does not repaint its interior — byte for byte.**
///
/// The guard on the composite. The matter's arrival became an `over` (it has an opacity now, so it must
/// compose), and an `over` that is wrong in the interior would wash or darken the paint the artist already
/// approved — silently, since the core is where the eye goes. Where the taper is 1 and the source is opaque
/// the composite reduces to the verbatim copy that shipped, and this pins that reduction on the real path,
/// as bytes rather than as a tolerance.
///
/// **Mutation that must bleed:** compose against the LIVE pixel instead of the frozen plane
/// (`rgba[gi*4+k]` for the destination in `render_inflate`'s advection) — the core stays put here but the
/// fringe accumulates per render; see `the_rim_is_the_same_however_slowly_it_was_dragged`. Or scale the
/// arriving alpha by anything other than the source's own: the core shifts and this reddens.
#[test]
fn the_inflate_does_not_repaint_the_forms_interior() {
    let (mut t, layer) = thick_round_blob();
    let before = (*t.canvas_rgba).clone();
    let cov0 = covers_of(&t, layer);
    inflate_the_whole_layer(&mut t);
    let after = (*t.canvas_rgba).clone();

    // The interior: every texel the deposit covered solidly. The rim is expected to move — that is the verb.
    let mut checked = 0usize;
    for i in 0..cov0.len() {
        if cov0[i] < 255 {
            continue;
        }
        checked += 1;
        assert_eq!(
            &after[i * 4..i * 4 + 4],
            &before[i * 4..i * 4 + 4],
            "the Inflate repainted a solidly-covered texel at {i}: {:?} → {:?}. It grows the form's rim; \
             the paint the artist already laid is not its business.",
            &before[i * 4..i * 4 + 4],
            &after[i * 4..i * 4 + 4]
        );
    }
    assert!(
        checked > 1000,
        "fixture: only {checked} solidly-covered texels to check — the blob did not paint"
    );
}

/// **The stroke is what its amount says, not what its render history says** — so touching a knob
/// mid-stroke does not move a pixel.
///
/// The property the whole no-restore path rests on, and the reason the composite reads the FROZEN plane for
/// its destination rather than the live one. The freehand path re-renders the stroke from `pre` on **every
/// pointer-move batch** with no restore in between (`sculpt_session`), and consecutive batches' windows
/// overlap by the ball's whole reach — so a texel is rendered again every time `amount` grows under it. That
/// is only sound because every write in the advection is an ASSIGNMENT sourced from frozen state: a render
/// answers *what does this stroke, at its current amount, do to the canvas it started from*, exactly as the
/// height's `target[gi] = f(pre, amount)` does.
///
/// The matter's arrival has an opacity now, so it must compose — and an `over` onto the LIVE pixel takes a
/// fresh coat on every batch. That is the sequential accumulation this line has already paid for twice (the
/// bite's share that had to telescope; the relief's capsule —
/// [[feedback_a_sequential_accumulation_is_sampling_dependent]]).
///
/// `refresh_live_sculpt` is the **product's own** way of asking the question: it restores the frozen window
/// and re-renders the stroke ONCE at its current settings, which is what every Sculpt knob does under the
/// artist's finger. Nudge nothing and the picture must not move. If the incremental path had been
/// accumulating coats, the rim would visibly JUMP the instant a knob was touched — and the artist would have
/// no word for it.
///
/// ## Its sibling, and what it cost to find the reachable state
///
/// [`super::a_faster_mouse_does_not_sculpt_deeper`] is this same law for the **height**, and it was written
/// off the same mutation (`pre` → live) surviving the shape-editor gate. This is the **matter's** half: the
/// advection is the one write in the sculpt that the height gate cannot see.
///
/// Reaching the divergence took three tries, all of which passed against the broken code:
/// * *"rendering the same state twice paints it once"* — the guard (`v <= pre_cover[gi]`) skips the second
///   render outright, so it is idempotent whichever destination the composite reads.
/// * the same drive at `strength = 0.25` — **zero advection happens at all**: the ball never beats its own
///   self-floor (`own = |Depth|·amount`), so `sbuf` is zeroed and the loop `continue`s on every texel. The
///   gate measured nothing and said so with a tick.
/// * at `strength = 1.0` the first dab saturates `amount`, so `v` never grows and each texel composites
///   exactly once — the same blind spot, and the same one the height's gate documents ("a LIGHT touch, on
///   purpose").
///
/// The divergence needs `amount` to GROW between two renders of the same texel. Measured at
/// `strength = 0.4`: 4090 composites over 1346 texels — 2744 re-composites. That is why the number is that
/// number, and it was found by **instrumenting the product and counting**, after three guesses missed.
///
/// ## The residual, named — a PRE-EXISTING ghost this fix does not own
///
/// Two texels disagree by 4/255, and they did **before this fix too** (measured against the shipped kernel
/// at `f8902dfc`: 6 bytes). They sit at the sculpt brush's own rim (x = 103, y = 78 and y = 141 on this
/// fixture — symmetric about the stroke): an early render advected a whisper there, a later one no longer
/// does, and **the advection has no way to un-paint** — it writes where the ball delivers and `continue`s
/// everywhere else, so a stale whisper outlives the render that made it. Making the render TOTAL (assign
/// every texel of `kr`, restoring the frozen planes where the ball delivers nothing) would close it and is
/// the honest shape; it also writes the whole window on every pointer move, which is a perf decision that
/// needs its own measurement against the kill criterion — not a drive-by inside a border fix. It is in the
/// handoff.
///
/// What this gate therefore guards is the thing this change IS responsible for: **the taper must not make
/// the render history matter more than it already did.** With the live-pixel destination it does, hugely —
/// 2520 bytes against the same 6.
///
/// **Mutation that must bleed:** compose against `rgba[gi*4+k]` (the live pixel) instead of
/// `pre_rgba[gi*4+k]` (the frozen plane) in `render_inflate`'s advection. Equally: read the guard's
/// destination live (`v <= cov[gi]`) instead of frozen (`v <= pre_cover[gi]`).
#[test]
fn a_knob_touched_mid_stroke_does_not_move_the_picture() {
    let (mut t, _) = thick_round_blob();
    // Low strength: `amount` climbs across the dabs instead of saturating on the first, so the ball keeps
    // growing under texels it already reached — the only state in which a re-render has anything to say.
    arm_sculpt(&mut t, INFLATE, 0.5, 0.4);
    t.set_sculpt_depth(1.0);

    // A live stroke, held mid-drag: Down + Moves and NO Up. The Up stamps tail dabs and kills the session
    // ([[feedback_capture_stroke_session_before_pen_up]]), and a dead session makes the refresh below a
    // silent no-op — the gate would pass by measuring nothing.
    t.on_canvas_pointer(cp([70.0, CY], PointerPhase::Down));
    let mut x = 74.0;
    while x <= 150.0 {
        t.on_canvas_pointer(cp([x, CY], PointerPhase::Move));
        x += 4.0;
    }
    assert!(
        t.paint.sculpt.layer.is_some(),
        "fixture: the session died, so the refresh below would do nothing at all"
    );
    let incremental = (*t.canvas_rgba).clone();

    // The product's own restore-and-re-render, at the settings that are already set: a knob touched, and
    // nudged nowhere.
    t.refresh_live_sculpt();
    let refreshed = (*t.canvas_rgba).clone();

    let texels = (0..incremental.len() / 4)
        .filter(|i| incremental[i * 4..i * 4 + 4] != refreshed[i * 4..i * 4 + 4])
        .count();
    let worst = incremental
        .iter()
        .zip(&refreshed)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert!(
        texels <= GHOST_TEXELS && worst <= GHOST_DEPTH,
        "the incremental stroke and the same stroke re-rendered once from the frozen source disagree on \
         {texels} texels, by up to {worst}/255 — the known ghost is {GHOST_TEXELS} texels at \
         {GHOST_DEPTH}/255. A composite that reads the LIVE canvas takes one coat per pointer-move batch, so \
         the rim darkens with how slowly the hand moved and JUMPS the moment a knob is touched. Measured \
         with the live-pixel destination: 2520 bytes."
    );
}
