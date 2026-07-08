use crate::tool::PainterTool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_painter_brush::{TextureKind, TextureMapping};

/// The full panel→tool seam EFFECT (the other half of the panel's `tests/seam.rs` forward proof):
/// the exact `PanelEvent`s the panel forwards, fed to `handle_panel_event`, mutate the observable
/// brush state (read back through the published `BrushSettings` snapshot). Also pins the clamps.
#[test]
fn panel_events_drive_watercolor_state() {
    let mut t = PainterTool::default();
    assert!(!t.brush_settings().watercolor, "default off");

    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_ENABLE));
    assert!(t.brush_settings().watercolor, "Wet edges toggled on");

    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_EDGE, 3.0));
    assert_eq!(t.brush_settings().edge_gain, 3.0, "Edge slider set");

    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_GRANULATION,
        0.5,
    ));
    assert_eq!(t.brush_settings().granulation, 0.5, "Granulation set");

    // Pigment: the merged slider (Mix id) drives BOTH the amount and the on/off gate.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_MIX, 0.75));
    let b = t.brush_settings();
    assert!(b.pigment, "Pigment slider > 0 enables the gate");
    assert_eq!(b.pigment_mix, 0.75, "Pigment amount set");
    // Sliding to 0 turns the gate off but REMEMBERS the amount (zero-loss merge).
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_MIX, 0.0));
    let b = t.brush_settings();
    assert!(!b.pigment, "Pigment slider 0 disables the gate");
    assert_eq!(
        b.pigment_mix, 0.75,
        "amount remembered while the gate is off"
    );
    // Re-enable for the rest of the sweep (so the reset assertion at the end has a gate to clear).
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_MIX, 0.75));

    // Paper COLOUR from the shared picker's read-back (the document ground; "r,g,b" 8-bit).
    assert_eq!(
        t.brush_settings().paper_color,
        [1.0, 1.0, 1.0],
        "paper defaults to white"
    );
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        "239,233,220".into(),
    ));
    let pc = t.brush_settings().paper_color;
    assert_eq!(
        [
            (pc[0] * 255.0 + 0.5) as u8,
            (pc[1] * 255.0 + 0.5) as u8,
            (pc[2] * 255.0 + 0.5) as u8
        ],
        [239, 233, 220],
        "paper colour routed from the picker read-back"
    );

    // Render-path optics: Fill / Depth / Warp drive the same seam.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_FILL, 0.4));
    assert_eq!(t.brush_settings().fill, 0.4, "Fill set");
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_DEPTH,
        2.0,
    ));
    assert_eq!(t.brush_settings().depth, 2.0, "Depth set");
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_WARP,
        10.0,
    ));
    assert_eq!(t.brush_settings().warp, 10.0, "Warp set");

    // Wet Mix: the Smudge + Wet sliders drive the same seam (clamped 0..1).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_SMUDGE,
        0.8,
    ));
    assert!(
        (t.brush_settings().wet_smudge - 0.8).abs() < 1e-6,
        "Smudge set"
    );
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_WET, 9.0));
    assert_eq!(t.brush_settings().wet_rewet, 1.0, "Wet clamped to 1");

    // Wet Mix mixer knobs: Charge / Dilution / Pull drive the same seam (clamped 0..1).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_CHARGE,
        0.3,
    ));
    assert!(
        (t.brush_settings().wet_charge - 0.3).abs() < 1e-6,
        "Charge set"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_DILUTION,
        0.6,
    ));
    assert!(
        (t.brush_settings().wet_dilution - 0.6).abs() < 1e-6,
        "Dilution set"
    );
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_PULL, 2.0));
    assert_eq!(t.brush_settings().wet_pull, 1.0, "Pull clamped to 1");

    // Paper + Granulation slots: kind picker, Size, Angle, and the "Same as Paper" toggle.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_WATERCOLOR_PAPER_KIND,
        (TextureKind::PaperRough.to_u8()).to_string(),
    ));
    assert_eq!(
        t.paint.brush.paper.kind,
        TextureKind::PaperRough,
        "Paper kind picked"
    );
    assert_eq!(
        t.paint.brush.paper.mapping,
        TextureMapping::Tiled,
        "paper forced canvas-anchored"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_X,
        50.0,
    ));
    assert_eq!(
        t.paint.brush.paper.size[0], 50.0,
        "Paper Size X set (0.1..100)"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_PAPER_ANGLE,
        45.0,
    ));
    assert_eq!(t.paint.brush.paper.angle_deg, 45, "Paper Angle set");
    assert!(
        t.brush_settings().granulation_use_paper,
        "Same as Paper default on"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_GRAN_SAME));
    assert!(
        !t.brush_settings().granulation_use_paper,
        "Same as Paper toggled off"
    );

    // Full Paper slot: Mapping / Rake / Random / Offset / Depth / param.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_WATERCOLOR_PAPER_MAPPING,
        (TextureMapping::Random.to_u8()).to_string(),
    ));
    assert_eq!(
        t.paint.brush.paper.mapping,
        TextureMapping::Random,
        "Paper mapping picked"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_PAPER_RAKE));
    assert!(t.paint.brush.paper.rake, "Paper Rake toggled");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_PAPER_RANDOM));
    assert!(t.paint.brush.paper.random_angle, "Paper Random toggled");
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_PAPER_OFFSET_X,
        0.3,
    ));
    assert!(
        (t.paint.brush.paper.offset[0] - 0.3).abs() < 1e-6,
        "Paper Offset X set"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_PAPER_DEPTH,
        0.7,
    ));
    assert!(
        (t.paint.brush.paper_depth - 0.7).abs() < 1e-6,
        "Paper Depth set"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_PAPER_PARAMS[2],
        0.8,
    ));
    assert!(
        (t.paint.brush.paper.params[2] - 0.8).abs() < 1e-6,
        "Paper param slot 2 set"
    );
    // Reset clears the Paper slot back to None.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_PAPER_RESET));
    assert_eq!(
        t.paint.brush.paper.kind,
        TextureKind::None,
        "Paper reset to empty"
    );

    // Clamp: Edge caps at 8, Spread at 48.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_EDGE,
        99.0,
    ));
    assert_eq!(t.brush_settings().edge_gain, 8.0, "Edge clamped to 8");
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_SPREAD,
        99.0,
    ));
    assert_eq!(t.brush_settings().edge_spread, 48.0, "Spread clamped to 48");

    // Reset returns the whole section to defaults — the `watercolor`/`pigment` gates OFF (which is
    // what makes a brush neutral); the params go back to their sensible when-enabled defaults.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_RESET));
    let b = t.brush_settings();
    assert!(
        !b.watercolor && !b.pigment,
        "reset turned the Watercolor + Pigment gates off"
    );
    assert_eq!(b.edge_gain, 1.5, "reset restored the default Edge gain");
}

/// The Preset dropdown seam: `SelectOption(PAINTER_BRUSH_PRESET, idx)` reconfigures the whole brush.
/// Watercolor Basic turns the render-path on with the wet_edges knobs; Digital Basic turns it back
/// off — both PRESERVING the user's colour + radius (a preset is a look, not a what/where reset).
#[test]
fn preset_dropdown_reconfigures_the_brush() {
    let mut t = PainterTool::default();
    // Give the brush a distinctive colour + size the preset must preserve.
    t.paint.brush.color = [0.2, 0.6, 0.9];
    t.paint.brush.radius_px = 40.0;

    // Watercolor Basic (idx 1): render-path on + wet_edges optics.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_PRESET,
        "1".into(),
    ));
    let b = t.brush_settings();
    assert!(b.watercolor, "Watercolor Basic turns the render-path on");
    assert_eq!(b.edge_gain, 3.0, "wet_edges edge gain");
    assert_eq!(b.fill, 0.12, "wet_edges fill");
    assert_eq!(b.depth, 1.2, "wet_edges depth");
    assert_eq!(
        b.color,
        [0.2, 0.6, 0.9],
        "colour preserved across the preset"
    );
    assert_eq!(
        t.paint.brush.radius_px, 40.0,
        "radius preserved across the preset"
    );
    // Paper slot wired to a canvas-anchored cold-press paper (the substrate the wash sits on).
    assert_eq!(
        t.paint.brush.paper.kind,
        TextureKind::PaperCold,
        "Paper = cold-press"
    );
    assert_eq!(
        t.paint.brush.paper.mapping,
        TextureMapping::Tiled,
        "paper is canvas-anchored"
    );

    // Digital Basic (idx 0): back to the plain brush, colour + size still preserved.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_PRESET,
        "0".into(),
    ));
    let b = t.brush_settings();
    assert!(!b.watercolor, "Digital Basic turns the render-path off");
    assert_eq!(b.color, [0.2, 0.6, 0.9], "colour still preserved");
    assert_eq!(t.paint.brush.radius_px, 40.0, "radius still preserved");
}

/// A tagged layer installs into the RIGHT slot: "Use as Paper" → the Paper slot; "Use as Granulation"
/// → the Granulation slot (Same-as-Paper off, its own map). The two are distinct destinations, not the
/// same Grain slot (the bug Enio caught).
#[test]
fn use_layers_routes_paper_and_granulation_to_separate_slots() {
    let lum = vec![128u8; 8 * 8];

    let mut t = PainterTool::default();
    t.use_layers_as_watercolor_paper(lum.clone(), 8, 8);
    let b = &t.paint.brush;
    assert_eq!(b.paper.kind, TextureKind::Image, "paper → Paper slot Image");
    assert_eq!(b.paper.mapping, TextureMapping::Tiled, "canvas-anchored");
    assert!(b.watercolor, "render-path on");
    // The Grain slot is untouched (Paper is its own slot now).
    assert_eq!(
        b.texture.kind,
        TextureKind::None,
        "the per-dab Grain slot is not touched"
    );

    let mut t2 = PainterTool::default();
    t2.use_layers_as_granulation(lum, 8, 8);
    let b2 = &t2.paint.brush;
    // Granulation = the GRAIN slot (its own section); "Same as Paper" turned off so the map is used.
    assert_eq!(
        b2.texture.kind,
        TextureKind::Image,
        "granulation → Grain slot Image"
    );
    assert!(
        !b2.granulation_use_paper,
        "granulation uses the Grain map, not the paper"
    );
    assert!(
        (b2.granulation - 0.65).abs() < 1e-6,
        "pronounced mineral-settling amount"
    );
    assert_eq!(
        b2.paper.kind,
        TextureKind::None,
        "the Paper slot is not touched by the granulation tag"
    );
}
