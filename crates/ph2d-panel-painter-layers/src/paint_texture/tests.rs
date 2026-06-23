//! Paint-level proof of the Texture section's relevance gate (DIRETIVA §2): with no texture
//! assigned, only the Kind picker + New register a hit rect; assigning a kind reveals the rest.
//! Drives the real `paint_texture_section` and reads the per-frame `HitIndex` (a row that is not
//! painted registers no hit rect, so no real pointer click can reach it).

use super::*;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

/// Paint the Texture section for `kind` (all else at the fallback defaults) and return every widget
/// id that registered a hit rect this frame.
fn painted_hit_ids_for(brush: BrushSettings) -> Vec<NodeId> {
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 360.0, 4000.0);
    let layout = HeroLayout::for_viewport(viewport);
    {
        let mut ctx = PaintCtx {
            host: &mut host,
            layout: &layout,
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        paint_texture_section(&mut ctx, Theme::default(), 0.0, 320.0, 0.0, brush);
    }
    host.hit_index_mut()
        .iter_registrations()
        .map(|(id, _)| id)
        .collect()
}

/// Hit ids for a brush with the given texture `kind` (default ViewPlane mapping).
fn painted_hit_ids(kind: u8) -> Vec<NodeId> {
    painted_hit_ids_for(BrushSettings {
        texture_kind: kind,
        ..crate::paint_brush::FALLBACK_BRUSH
    })
}

/// The always-shown controls (the picker + New) regardless of whether a texture is assigned.
const ALWAYS: [NodeId; 2] = [
    core_ids::PAINTER_BRUSH_TEXTURE_KIND,
    core_ids::PAINTER_BRUSH_TEXTURE_NEW,
];

/// The gated controls — only painted once a texture kind is assigned.
const GATED: [NodeId; 8] = [
    core_ids::PAINTER_BRUSH_TEXTURE_MAPPING,
    core_ids::PAINTER_BRUSH_TEXTURE_ANGLE,
    core_ids::PAINTER_BRUSH_TEXTURE_RAKE,
    core_ids::PAINTER_BRUSH_TEXTURE_RANDOM,
    core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
    core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
    core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
    core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
];

#[test]
fn none_kind_shows_only_picker_and_new() {
    let ids = painted_hit_ids(TextureKind::None.to_u8());
    for shown in ALWAYS {
        assert!(
            ids.contains(&shown),
            "the picker/New must always show ({shown:?}). painted = {ids:?}"
        );
    }
    for hidden in GATED {
        assert!(
            !ids.contains(&hidden),
            "no texture assigned, but {hidden:?} registered a hit rect (silent no-op). \
             painted = {ids:?}"
        );
    }
}

#[test]
fn assigned_kind_reveals_every_control() {
    let ids = painted_hit_ids(TextureKind::Noise.to_u8());
    for shown in ALWAYS.into_iter().chain(GATED) {
        assert!(
            ids.contains(&shown),
            "an assigned texture must show every control ({shown:?}). painted = {ids:?}"
        );
    }
}

#[test]
fn stencil_mapping_hides_rake_and_random_but_keeps_offset_size_angle() {
    // Stencil has a fixed frame driven by Offset/Size/Angle, so the per-dab Rake/Random rotations
    // do not apply and must not paint a hit rect (no silent no-op).
    let ids = painted_hit_ids_for(BrushSettings {
        texture_kind: TextureKind::Noise.to_u8(),
        texture_mapping: TextureMapping::Stencil.to_u8(),
        ..crate::paint_brush::FALLBACK_BRUSH
    });
    for hidden in [
        core_ids::PAINTER_BRUSH_TEXTURE_RAKE,
        core_ids::PAINTER_BRUSH_TEXTURE_RANDOM,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Stencil must hide {hidden:?} (its frame is fixed). painted = {ids:?}"
        );
    }
    for shown in [
        core_ids::PAINTER_BRUSH_TEXTURE_MAPPING,
        core_ids::PAINTER_BRUSH_TEXTURE_ANGLE,
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
    ] {
        assert!(
            ids.contains(&shown),
            "Stencil drives its frame via {shown:?} — it must show. painted = {ids:?}"
        );
    }
}

#[test]
fn param_sliders_register_exactly_the_kind_s_specs() {
    use ph2d_tool_painter::param_specs;
    // Clouds exposes 3 params (Contrast / Brightness / Detail) → first 3 param sliders paint a hit
    // rect; the unused 4th must NOT (a painted-but-meaningless row is the silent-no-op trap).
    let ids = painted_hit_ids(TextureKind::Clouds.to_u8());
    let n = param_specs(TextureKind::Clouds).len();
    assert_eq!(n, 3, "Clouds exposes Contrast + Brightness + Detail");
    for i in 0..n {
        assert!(
            ids.contains(&core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[i]),
            "param slot {i} must register a hit rect for Clouds. painted = {ids:?}"
        );
    }
    assert!(
        !ids.contains(&core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[3]),
        "Clouds uses 3 slots — slot 3 must not register"
    );

    // Diamonds exposes only Contrast + Brightness → slot 2 must stay hidden.
    let ids2 = painted_hit_ids(TextureKind::Diamonds.to_u8());
    assert_eq!(param_specs(TextureKind::Diamonds).len(), 2);
    assert!(ids2.contains(&core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[1]));
    assert!(
        !ids2.contains(&core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[2]),
        "Diamonds has no shape param — slot 2 must not register. painted = {ids2:?}"
    );
}

#[test]
fn color_ramp_controls_gate_on_the_enable_toggle() {
    // Ramp OFF (default): only the enable toggle registers; Mode / Interp / + / − stay hidden.
    let off = painted_hit_ids(TextureKind::Noise.to_u8());
    assert!(
        off.contains(&core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE),
        "the Color Ramp enable toggle always shows. painted = {off:?}"
    );
    for hidden in [
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE,
    ] {
        assert!(
            !off.contains(&hidden),
            "ramp {hidden:?} must be hidden when the ramp is off"
        );
    }
    // Ramp ON: 2 stops with distinct stable ids (slot 5) so the handles key apart.
    let mut stops = [[0.0; 6]; ph2d_tool_painter::PANEL_RAMP_STOPS];
    stops[0] = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0]; // id 0
    stops[1] = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0]; // id 1
    let on = painted_hit_ids_for(BrushSettings {
        texture_kind: TextureKind::Noise.to_u8(),
        texture_ramp_enabled: true,
        texture_ramp_stop_count: 2,
        texture_ramp_stops: stops,
        ..crate::paint_brush::FALLBACK_BRUSH
    });
    for shown in [
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE,
    ] {
        assert!(
            on.contains(&shown),
            "enabled ramp must register {shown:?}. painted = {on:?}"
        );
    }
    // Each of the 2 stops gets a draggable bar handle (position); the 3rd does not.
    assert!(
        on.contains(&core_ids::painter_brush_texture_ramp_handle_id(0)),
        "stop 0 drag handle"
    );
    assert!(
        on.contains(&core_ids::painter_brush_texture_ramp_handle_id(1)),
        "stop 1 drag handle"
    );
    assert!(
        !on.contains(&core_ids::painter_brush_texture_ramp_handle_id(2)),
        "only 2 stops → no 3rd handle"
    );
    // The bottom colour box (edits the selected stop) registers a hit rect.
    assert!(
        on.contains(&core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH),
        "the colour box must register a hit rect. painted = {on:?}"
    );
    // The editable index + position chips register so a click can focus + type into them.
    assert!(
        on.contains(&core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_INDEX),
        "the stop-index selector chip must register a hit rect. painted = {on:?}"
    );
    assert!(
        on.contains(&core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS),
        "the stop-position chip must register a hit rect. painted = {on:?}"
    );
}

#[test]
fn offset_and_size_tracks_are_inverse_of_the_tool_clamp() {
    // The panel maps the stored value back onto the 0..1 slider track; the centre/identity values
    // must land where the tool's setters put them (offset 0 → mid-track, size 1 → near-low track).
    assert!(
        (offset_track(0.0) - 0.5).abs() < 1e-6,
        "offset 0 is mid-track"
    );
    assert_eq!(offset_track(TEX_OFFSET_MIN), 0.0);
    assert_eq!(offset_track(TEX_OFFSET_MAX), 1.0);
    assert_eq!(size_track(TEX_SIZE_MIN), 0.0);
    assert_eq!(size_track(TEX_SIZE_MAX), 1.0);
}
