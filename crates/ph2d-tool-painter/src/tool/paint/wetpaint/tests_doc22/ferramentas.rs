//! Doc 22 gates — the WET TOOLS (Wet, Dry, Blow, Smear, Blend, Erase pick) and the canvas actions (Wet
//! canvas, Dry canvas). Child of `tests_doc22.rs` (LOC cap): same fixture vocabulary.

use super::*;

/// The WET tool deposits water and ZERO pigment — the routing gate (the
/// mutation this kills: the tool arm falling through to the paint lanes).
#[test]
fn the_wet_tool_lays_water_without_pigment() {
    let mut t = wet_tool_fixture();
    t.pick_wet_tool(4);
    assert_eq!(t.paint.wetpaint.tool, WetTool::Wet);
    stroke(&mut t, 60.0);
    let (film, susp, sett, _) = grid_totals(&t);
    assert!(film > 0.0, "the wet tool must lay water");
    assert_eq!(susp, 0.0, "the wet tool must not lay pigment");
    assert_eq!(sett, 0.0);
}

/// The DRY tool shrinks the film and SEALS the paper (wetness zeroed at the
/// stroke's core).
#[test]
fn the_dry_tool_shrinks_the_film_and_seals() {
    let mut t = wet_tool_fixture();
    t.pick_wet_tool(4); // wet first
    stroke(&mut t, 60.0);
    let (film_before, ..) = grid_totals(&t);
    assert!(film_before > 0.0);
    t.pick_wet_tool(5); // dry
    stroke(&mut t, 60.0);
    let (film_after, ..) = grid_totals(&t);
    // The dry stamp bites through the sparse bristle TIPS (the felt floor is
    // 0.01), so one pass shrinks the total modestly — the oracle is a real
    // shrink, not a percentage of it.
    assert!(
        film_after < film_before * 0.95,
        "dry must shrink the film ({film_before} -> {film_after})"
    );
    let sess = t.paint.wetpaint.session.as_ref().expect("session");
    let g = &sess.engine.layers[0].grid;
    let i = 90 + 1 + (60 + 1) * g.s;
    assert_eq!(g.wet[i], 0, "dry must SEAL the paper under the stroke");
}

/// The BLOW tool injects velocity into the wet film (the only tool that
/// writes velocity).
#[test]
fn the_blow_tool_pushes_the_film() {
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0); // paint lays film + pigment
    let (_, _, _, vel_before) = grid_totals(&t);
    t.pick_wet_tool(6); // blow
    stroke(&mut t, 60.0);
    let (_, _, _, vel_after) = grid_totals(&t);
    assert!(
        vel_after > vel_before + 1e-3,
        "blow must inject velocity ({vel_before} -> {vel_after})"
    );
}

/// The SMEAR tool drags SUSPENDED pigment along the stroke without adding
/// mass — the centre of mass moves, the total does not grow.
#[test]
fn the_smear_tool_drags_pigment_without_adding_any() {
    let mut t = wet_tool_fixture();
    // A short patch on the left half.
    t.on_canvas_pointer(cp([40.0, 60.0], PointerPhase::Down));
    for k in 1..=6 {
        t.on_canvas_pointer(cp([40.0 + 4.0 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([64.0, 60.0], PointerPhase::Up));
    let com_x = |t: &PainterTool| -> (f64, f64) {
        let sess = t.paint.wetpaint.session.as_ref().expect("session");
        let g = &sess.engine.layers[0].grid;
        let (mut m, mut mx) = (0.0f64, 0.0f64);
        for cy in 1..=g.h {
            for cx in 1..=g.w {
                let v = f64::from(g.susp[cx + cy * g.s]);
                m += v;
                mx += v * cx as f64;
            }
        }
        (m, if m > 0.0 { mx / m } else { 0.0 })
    };
    let (mass_before, com_before) = com_x(&t);
    assert!(mass_before > 0.0, "fixture: the patch must hold pigment");
    t.pick_wet_tool(2); // smear
    // Drag from inside the patch rightward, past its edge.
    t.on_canvas_pointer(cp([50.0, 60.0], PointerPhase::Down));
    for k in 1..=15 {
        t.on_canvas_pointer(cp([50.0 + 5.0 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([125.0, 60.0], PointerPhase::Up));
    let (mass_after, com_after) = com_x(&t);
    assert!(
        com_after > com_before + 0.5,
        "smear must drag the pigment along the stroke ({com_before} -> {com_after})"
    );
    // The model's smear is a COPY-based drag (lerp from a snapshot — it may
    // duplicate mass a little, "do not fix this"), while the PAINT route
    // adds a full pigment load per dab. The mutation this kills (the tool
    // arm falling through to the lanes) shows as a paint-sized gain.
    let twin_gain = {
        let mut p = wet_tool_fixture();
        p.on_canvas_pointer(cp([40.0, 60.0], PointerPhase::Down));
        for k in 1..=6 {
            p.on_canvas_pointer(cp([40.0 + 4.0 * k as f32, 60.0], PointerPhase::Move));
        }
        p.on_canvas_pointer(cp([64.0, 60.0], PointerPhase::Up));
        let (m0, _) = com_x(&p);
        p.on_canvas_pointer(cp([50.0, 60.0], PointerPhase::Down));
        for k in 1..=15 {
            p.on_canvas_pointer(cp([50.0 + 5.0 * k as f32, 60.0], PointerPhase::Move));
        }
        p.on_canvas_pointer(cp([125.0, 60.0], PointerPhase::Up));
        let (m1, _) = com_x(&p);
        m1 - m0
    };
    let smear_gain = mass_after - mass_before;
    assert!(
        smear_gain < twin_gain * 0.5,
        "smear gained mass like a paint stroke ({smear_gain} vs paint {twin_gain}) — the tool arm fell through to the lanes"
    );
}

/// The BLEND tool remixes without depositing: total pigment mass stays put
/// while the colour field changes (the paint route would ADD ~600/dab).
#[test]
fn the_blend_tool_remixes_without_depositing() {
    let mut t = wet_tool_fixture();
    // TWO colours side by side (a uniform patch would relax toward its own
    // mean — zero change by construction; the fixture must CONTAIN the
    // phenomenon): red on the left, blue on the right, then settled.
    t.on_canvas_pointer(cp([40.0, 60.0], PointerPhase::Down));
    for k in 1..=8 {
        t.on_canvas_pointer(cp([40.0 + 4.0 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([72.0, 60.0], PointerPhase::Up));
    t.paint.brush.color = [0.1, 0.2, 0.9];
    t.on_canvas_pointer(cp([80.0, 60.0], PointerPhase::Down));
    for k in 1..=8 {
        t.on_canvas_pointer(cp([80.0 + 4.0 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([112.0, 60.0], PointerPhase::Up));
    click(&mut t, crate::ids::PAINTER_WETPAINT_DRYCANVAS); // settle it
    let (_, susp0, sett0, _) = grid_totals(&t);
    assert!(sett0 > 0.0, "fixture: dried paint to remix");
    // Mass-weighted BLUE of the red half — remixing across the boundary
    // must pull blue into it (the sparse mask makes single cells lie; the
    // regional mean cannot).
    let sett_probe = |t: &PainterTool| {
        let sess = t.paint.wetpaint.session.as_ref().expect("session");
        let g = &sess.engine.layers[0].grid;
        let (mut acc, mut m) = (0.0f64, 0.0f64);
        for cy in 50..=70 {
            for cx in 40..=72 {
                let i = cx + 1 + (cy + 1) * g.s;
                acc += f64::from(g.sett_rgb[i][2]) * f64::from(g.sett[i]);
                m += f64::from(g.sett[i]);
            }
        }
        if m > 0.0 { acc / m } else { 0.0 }
    };
    let probe_before = sett_probe(&t);
    t.pick_wet_tool(3); // blend
    stroke(&mut t, 60.0);
    let (_, susp1, sett1, _) = grid_totals(&t);
    let total0 = susp0 + sett0;
    let total1 = susp1 + sett1;
    assert!(
        total1 < total0 * 1.05,
        "blend must not deposit pigment ({total0} -> {total1})"
    );
    assert!(
        sett_probe(&t) > probe_before + 1e-6,
        "blend must pull the neighbour's blue into the red half"
    );
}

/// Picking Erase in the tool list is the rail eraser's OTHER VIEW: the mode
/// flips to the eraser wire; picking Paint returns to the fluid.
#[test]
fn erase_pick_is_the_rail_erasers_other_view() {
    let mut t = wet_tool_fixture();
    t.pick_wet_tool(1);
    assert!(t.paint.eraser, "Erase pick must land on the eraser wire");
    assert!(
        t.paint.wetpaint.armed,
        "the ARM survives the tool view swap"
    );
    t.pick_wet_tool(0);
    assert!(!t.paint.eraser);
    assert!(
        matches!(t.paint.paint_mode, PaintMode::WetPaint),
        "Paint pick must return to the fluid while armed"
    );
}

/// Wet canvas: born with no session, it CREATES one and wets the whole
/// sheet (wetness raised everywhere, zero pigment, canvas bytes untouched).
#[test]
fn wet_canvas_wets_the_sheet_without_painting() {
    let mut t = wet_tool_fixture();
    let before = Arc::clone(&t.canvas_rgba);
    assert!(t.paint.wetpaint.session.is_none());
    click(&mut t, crate::ids::PAINTER_WETPAINT_WETCANVAS);
    let sess = t.paint.wetpaint.session.as_ref().expect("session born");
    let g = &sess.engine.layers[0].grid;
    let far = 150 + 1 + (30 + 1) * g.s; // far from anything
    assert!(g.wet[far] > 0, "the whole sheet must read damp");
    let (_, susp, sett, _) = grid_totals(&t);
    assert_eq!(susp + sett, 0.0);
    assert_eq!(
        *before, *t.canvas_rgba,
        "wetting alone must not move a canvas byte"
    );
}

/// Dry canvas settles every suspended grain; Fast dry drains the film.
#[test]
fn dry_canvas_settles_and_fast_dry_drains() {
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0);
    let (_, susp0, ..) = grid_totals(&t);
    assert!(susp0 > 0.0);
    click(&mut t, crate::ids::PAINTER_WETPAINT_DRYCANVAS);
    let (film1, susp1, sett1, _) = grid_totals(&t);
    assert_eq!(susp1, 0.0, "dry canvas must settle ALL suspension");
    assert!(sett1 > 0.0);
    assert_eq!(film1, 0.0, "dry canvas must zero the water");
    // Fast dry on a fresh wet stroke.
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0);
    let (film0, ..) = grid_totals(&t);
    assert!(film0 > 0.0);
    click(&mut t, crate::ids::PAINTER_WETPAINT_FASTDRY);
    let (film1, ..) = grid_totals(&t);
    assert!(
        film1 < film0 * 0.05,
        "fast dry must drain the fluid ({film0} -> {film1})"
    );
}
