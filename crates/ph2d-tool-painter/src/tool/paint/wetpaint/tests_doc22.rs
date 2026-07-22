//! Doc 22 gates — the FULL tuning store, the tilt dial, the wet tools, the
//! canvas actions and the display flags. Sibling of `tests.rs` (same fixture
//! vocabulary); every gate names the mutation that bleeds it.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};
use ph2d_painter_brush::Falloff;
use ph2d_wet_paint::tuning::{KNOB_COUNT, KNOB_DEFS, Knob, KnobGroup};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn wet_tool_fixture() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 200 * 120 * 4], 200, 120);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("wetpaint");
    t
}

fn stroke(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([30.0, y], PointerPhase::Down));
    for k in 1..=20 {
        t.on_canvas_pointer(cp([30.0 + 6.0 * k as f32, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([150.0, y], PointerPhase::Up));
}

fn click(t: &mut PainterTool, id: ph2d_a11y::NodeId) {
    assert!(
        t.route_brush_wetpaint_event(&PanelEvent::Click(id)),
        "click on {id:?} was not consumed by the wet route"
    );
}

fn set(t: &mut PainterTool, id: ph2d_a11y::NodeId, v: f64) {
    assert!(
        t.route_brush_wetpaint_event(&PanelEvent::SetValue(id, v)),
        "SetValue on {id:?} was not consumed by the wet route"
    );
}

fn grid_totals(t: &PainterTool) -> (f64, f64, f64, f64) {
    let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
    let g = &sess.engine.layers[0].grid;
    let (mut film, mut susp, mut sett, mut vel) = (0.0f64, 0.0, 0.0, 0.0);
    for cy in 1..=g.h {
        for cx in 1..=g.w {
            let i = cx + cy * g.s;
            film += f64::from(g.film[i]);
            susp += f64::from(g.susp[i]);
            sett += f64::from(g.sett[i]);
            vel += f64::from(g.vel_x[i]).abs() + f64::from(g.vel_y[i]).abs();
        }
    }
    (film, susp, sett, vel)
}

/// The authored BOOT equals the engine's boot EXACTLY, knob by knob (f64
/// bits) — the reconcile of an untouched section must be a no-op. Mutation
/// that bleeds it: any drift between `WetKnobs::DEFAULT` / the tilt boot
/// and the engine's own boot values.
#[test]
fn the_boot_facts_equal_the_engine_boot_exactly() {
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0);
    let sess = t.paint.wetpaint.session.as_ref().expect("session");
    let e = &sess.engine;
    assert_eq!(e.sliders.water.to_bits(), 1.0f64.to_bits());
    assert_eq!(e.sliders.erase.to_bits(), 0.4f64.to_bits());
    for def in KNOB_DEFS.iter() {
        assert_eq!(
            e.tuning.get(def.knob).to_bits(),
            def.default.to_bits(),
            "knob {} drifted from its boot default",
            def.key
        );
    }
    assert!(e.sim.tilt_on);
    assert_eq!(e.sim.tilt_dir_x.to_bits(), 0.0f64.to_bits());
    assert_eq!(e.sim.tilt_dir_y.to_bits(), 1.0f64.to_bits());
    assert_eq!(e.sim.tilt_scale.to_bits(), 1.0f64.to_bits());
    assert!(!e.sim.km_mixing);
}

/// A Tuning-panel SetValue (dynamic id family) reaches the LIVE engine on
/// the next tick; NaN falls back to the default; the chip id routes too.
/// Mutation: the dynamic map losing a face, or `set` skipping the NaN law.
#[test]
fn a_tuning_knob_setvalue_reaches_the_live_engine() {
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0);
    set(&mut t, core_ids::wet_tuning_slider_id("leveling"), 1.7);
    t.wetpaint_tick(0.05);
    let sess = t.paint.wetpaint.session.as_ref().expect("session");
    assert_eq!(sess.engine.tuning.get(Knob::Leveling), 1.7);
    set(&mut t, core_ids::wet_tuning_chip_id("brake"), 3.25);
    t.wetpaint_tick(0.05);
    let sess = t.paint.wetpaint.session.as_ref().expect("session");
    assert_eq!(sess.engine.tuning.get(Knob::Brake), 3.25);
    set(&mut t, core_ids::wet_tuning_chip_id("brake"), f64::NAN);
    assert_eq!(
        t.paint.wetpaint.knobs.get(Knob::Brake),
        KNOB_DEFS[Knob::Brake as usize].default,
        "NaN must fall back to the default (the model's law)"
    );
    // Out-of-range clamps to the def's own bounds.
    set(&mut t, core_ids::wet_tuning_slider_id("waterCap"), 999.0);
    assert_eq!(
        t.paint.wetpaint.knobs.get(Knob::WaterCap),
        KNOB_DEFS[Knob::WaterCap as usize].max
    );
}

/// Per-knob reset and per-group reset restore the defaults — and ONLY their
/// own scope. Mutation: a reset arm resolving to the wrong knob/group.
#[test]
fn tuning_resets_restore_their_own_scope_only() {
    let mut t = wet_tool_fixture();
    set(&mut t, core_ids::wet_tuning_slider_id("leveling"), 1.9);
    set(&mut t, core_ids::wet_tuning_slider_id("drag"), 0.9);
    click(&mut t, core_ids::wet_tuning_reset_id("leveling"));
    assert_eq!(
        t.paint.wetpaint.knobs.get(Knob::Leveling),
        KNOB_DEFS[Knob::Leveling as usize].default
    );
    assert_eq!(t.paint.wetpaint.knobs.get(Knob::Drag), 0.9, "wrong scope");
    // Group reset: PHYSICS resets leveling's group, never PAINT's drag.
    set(&mut t, core_ids::wet_tuning_slider_id("leveling"), 1.9);
    click(&mut t, core_ids::WET_TUNING_GROUP_RESETS[2]);
    assert_eq!(
        t.paint.wetpaint.knobs.get(Knob::Leveling),
        KNOB_DEFS[Knob::Leveling as usize].default
    );
    assert_eq!(t.paint.wetpaint.knobs.get(Knob::Drag), 0.9, "wrong group");
    let _ = KnobGroup::Paint; // (vocabulary anchor)
}

/// The tilt dial: ring+spoke SetValues drive the sim's vector on the next
/// tick (ring 8 = scale 2, spoke 0 = +x exact); the toggle flips WITHOUT
/// losing the direction; touching the dial turns the tilt on.
#[test]
fn the_tilt_dial_drives_the_sim_vector() {
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0);
    set(&mut t, core_ids::PAINTER_WETPAINT_TILT_RING, 8.0);
    set(&mut t, core_ids::PAINTER_WETPAINT_TILT_SPOKE, 0.0);
    t.wetpaint_tick(0.05);
    {
        let sess = t.paint.wetpaint.session.as_ref().expect("session");
        assert_eq!(sess.engine.sim.tilt_scale, 2.0);
        assert_eq!(sess.engine.sim.tilt_dir_x.to_bits(), 1.0f64.to_bits());
        assert_eq!(sess.engine.sim.tilt_dir_y.to_bits(), 0.0f64.to_bits());
        assert!(sess.engine.sim.tilt_on);
    }
    click(&mut t, core_ids::PAINTER_WETPAINT_TILT_TOGGLE);
    t.wetpaint_tick(0.05);
    {
        let sess = t.paint.wetpaint.session.as_ref().expect("session");
        assert!(!sess.engine.sim.tilt_on, "toggle must reach the sim");
        assert_eq!(
            sess.engine.sim.tilt_dir_x.to_bits(),
            1.0f64.to_bits(),
            "the toggle must NOT lose the dial's direction"
        );
    }
    assert_eq!(t.paint.wetpaint.tilt_ring, 8);
    assert_eq!(t.paint.wetpaint.tilt_spoke, 0);
}

/// The WET tool deposits water and ZERO pigment — the routing gate (the
/// mutation this kills: the tool arm falling through to the paint lanes).
#[test]
fn the_wet_tool_lays_water_without_pigment() {
    let mut t = wet_tool_fixture();
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[4]);
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
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[4]); // wet first
    stroke(&mut t, 60.0);
    let (film_before, ..) = grid_totals(&t);
    assert!(film_before > 0.0);
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[5]); // dry
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
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[6]); // blow
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
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[2]); // smear
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
    click(&mut t, core_ids::PAINTER_WETPAINT_DRYCANVAS); // settle it
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
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[3]); // blend
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
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[1]);
    assert!(t.paint.eraser, "Erase pick must land on the eraser wire");
    assert!(
        t.paint.wetpaint.armed,
        "the ARM survives the tool view swap"
    );
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[0]);
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
    click(&mut t, core_ids::PAINTER_WETPAINT_WETCANVAS);
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
    click(&mut t, core_ids::PAINTER_WETPAINT_DRYCANVAS);
    let (film1, susp1, sett1, _) = grid_totals(&t);
    assert_eq!(susp1, 0.0, "dry canvas must settle ALL suspension");
    assert!(sett1 > 0.0);
    assert_eq!(film1, 0.0, "dry canvas must zero the water");
    // Fast dry on a fresh wet stroke.
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0);
    let (film0, ..) = grid_totals(&t);
    assert!(film0 > 0.0);
    click(&mut t, core_ids::PAINTER_WETPAINT_FASTDRY);
    let (film1, ..) = grid_totals(&t);
    assert!(
        film1 < film0 * 0.05,
        "fast dry must drain the fluid ({film0} -> {film1})"
    );
}

/// Show wet: the veil is VISIBLE (bytes move) and NEVER bakes — ending the
/// session recomposites clean first. Mutation: the end-session door baking
/// the veiled composite.
#[test]
fn the_show_wet_veil_shows_and_never_bakes() {
    let mut t = wet_tool_fixture();
    click(&mut t, core_ids::PAINTER_WETPAINT_WETCANVAS);
    let clean = t.canvas_rgba.as_ref().clone();
    click(&mut t, core_ids::PAINTER_WETPAINT_SHOWWET);
    assert!(t.paint.wetpaint.show_wet);
    assert_ne!(
        clean, *t.canvas_rgba,
        "the damp sheet must be VISIBLE under show wet"
    );
    t.wetpaint_end_session();
    assert_eq!(
        clean, *t.canvas_rgba,
        "the veil must never bake — ending the session recomposites clean"
    );
}

/// Paper checkbox: grain prints into the pigment (bytes move where paint
/// is) and BAKES on purpose — it is part of the painting.
#[test]
fn paper_visual_prints_grain_into_the_paint_and_bakes() {
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0);
    let plain = t.canvas_rgba.as_ref().clone();
    click(&mut t, core_ids::PAINTER_WETPAINT_PAPER_VISUAL);
    assert!(t.paint.wetpaint.paper_visual);
    let grained = t.canvas_rgba.as_ref().clone();
    assert_ne!(plain, grained, "the tooth must print into the pigment");
    t.wetpaint_end_session();
    assert_eq!(
        grained, *t.canvas_rgba,
        "the paper look is part of the painting — it bakes"
    );
}

/// The K–M flags reach their homes: mixing lands in the sim on the next
/// reconcile; glaze changes the composite of film-over-dried-paint.
#[test]
fn km_flags_reach_the_sim_and_the_composite() {
    let mut t = wet_tool_fixture();
    stroke(&mut t, 60.0);
    click(&mut t, core_ids::WET_TUNING_KM_MIXING);
    t.wetpaint_tick(0.05);
    {
        let sess = t.paint.wetpaint.session.as_ref().expect("session");
        assert!(sess.engine.sim.km_mixing, "mixing must reach the sim");
    }
    // Glaze: dried paint + a fresh wash over it, then flip the flag.
    click(&mut t, core_ids::PAINTER_WETPAINT_DRYCANVAS);
    stroke(&mut t, 60.0);
    let plain = t.canvas_rgba.as_ref().clone();
    click(&mut t, core_ids::WET_TUNING_KM_GLAZE);
    assert_ne!(
        plain, *t.canvas_rgba,
        "glaze must change the film-over-dried stacking"
    );
}

/// The Tuning checkbox flips the snapshot fact the bridge reads.
#[test]
fn the_tuning_checkbox_flips_the_snapshot() {
    let mut t = wet_tool_fixture();
    assert!(!t.brush_settings().wet_tuning_open);
    click(&mut t, core_ids::PAINTER_WETPAINT_TUNING);
    assert!(t.brush_settings().wet_tuning_open);
}

/// The section reset restores EVERYTHING doc 22 added: knobs, tool, tilt,
/// flags — and disarms.
#[test]
fn the_section_reset_restores_the_whole_section() {
    let mut t = wet_tool_fixture();
    set(&mut t, core_ids::wet_tuning_slider_id("leveling"), 1.9);
    click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[6]);
    set(&mut t, core_ids::PAINTER_WETPAINT_TILT_RING, 8.0);
    click(&mut t, core_ids::PAINTER_WETPAINT_SHOWWET);
    click(&mut t, core_ids::PAINTER_WETPAINT_TUNING);
    click(&mut t, core_ids::WET_TUNING_KM_MIXING);
    click(&mut t, core_ids::PAINTER_WETPAINT_RESET);
    let w = &t.paint.wetpaint;
    assert!(!w.armed);
    assert_eq!(w.knobs, WetKnobs::DEFAULT);
    assert_eq!(w.tool, WetTool::Paint);
    assert!(w.tilt_on);
    assert_eq!((w.tilt_ring, w.tilt_spoke), (4, 3));
    assert!(!w.show_wet && !w.paper_visual && !w.km_mixing && !w.km_glaze);
    assert!(!w.tuning_open);
    let _ = KNOB_COUNT; // vocabulary anchor
}
