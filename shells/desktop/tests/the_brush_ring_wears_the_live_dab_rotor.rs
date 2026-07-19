//! **The brush-cursor ring must draw the LIVE dab orientation, not the resting Angle.**
//!
//! With a slot following the stroke (Shape Rake / Flow, Grain Rake) the dab footprint turns onto the
//! stroke tangent every dab. The ring is the picture of that footprint, so it has to turn too — a
//! calligraphic nib aimed with a cursor that points somewhere else is worse than no cursor at all
//! (Enio 2026-07-19: *"permite que o círculo que representa o pincel rotacione em tempo real conforme
//! flow e rake"*).
//!
//! The tool publishes the composed rotor as `BrushSettings::dab_rotor`, and the ring must READ it rather
//! than re-derive an angle from `dab_angle_deg`. Two answers to "which way is this dab pointing?" drift,
//! and the one the artist sees would be the wrong one.
//!
//! ⚠️ **Why an arch-gate over the source:** the ring is pure draw in the shell. There is no unit test that
//! can observe it, and the behavioural gate that covers the rotor
//! (`the_brush_ring_rotor_turns_with_the_stroke_only_when_a_slot_follows`, in `ph2d-tool-painter`) stops at
//! the published value — mutating the SHELL to ignore that value leaves every test in the workspace green.
//! This is the gate that bleeds for that mutation.

use std::fs;
use std::path::PathBuf;

#[test]
fn the_brush_ring_wears_the_live_dab_rotor() {
    let src = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/render_loop/painter_bridge_brush_ring.rs"),
    )
    .expect("the brush ring source");
    // Strip line comments so the prose above the code cannot satisfy (or trip) the checks below.
    let code: String = src
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        code.contains("bs.dab_rotor"),
        "the ring must draw the LIVE orientation published by the tool (`BrushSettings::dab_rotor`), \
         so it turns with the stroke under Rake / Flow"
    );
    assert!(
        !code.contains("dab_angle_deg"),
        "the ring must NOT re-derive its angle from `dab_angle_deg`: that is the RESTING angle, and a \
         ring pinned to it stays still while the tip it depicts turns with the stroke"
    );
}
