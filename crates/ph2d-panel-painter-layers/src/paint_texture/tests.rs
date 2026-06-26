//! Paint-level proof of the Texture section's relevance gate (DIRETIVA §2): with no texture
//! assigned, only the Kind picker registers a hit rect; assigning a kind reveals the rest.
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
        paint_texture_section(&mut ctx, Theme::default(), 0.0, 320.0, 0.0, brush, false);
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

/// Like [`painted_hit_ids_for`] but expands the **Color Ramp** section first (it defaults collapsed),
/// so the "Use Color Ramp" checkbox + (when enabled) the ramp editor controls actually paint.
fn painted_hit_ids_ramp_open(brush: BrushSettings) -> Vec<NodeId> {
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    host.store_mut()
        .set_collapsed(core_ids::PAINTER_BRUSH_COLOR_RAMP_SECTION, false);
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
        paint_texture_section(&mut ctx, Theme::default(), 0.0, 320.0, 0.0, brush, false);
    }
    host.hit_index_mut()
        .iter_registrations()
        .map(|(id, _)| id)
        .collect()
}

/// The always-shown control (the Kind picker) regardless of whether a texture is assigned.
const ALWAYS: [NodeId; 1] = [core_ids::PAINTER_BRUSH_TEXTURE_KIND];

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
fn none_kind_shows_only_picker() {
    let ids = painted_hit_ids(TextureKind::None.to_u8());
    for shown in ALWAYS {
        assert!(
            ids.contains(&shown),
            "the Kind picker must always show ({shown:?}). painted = {ids:?}"
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
fn stencil_mapping_swaps_texture_transform_for_the_stencil_card() {
    // Stencil's frame is its OWN `stencil_*` placement (Enio 2026-06-25): the per-dab Rake/Random
    // rotations don't apply, AND the texture's own Offset/Size/Angle are hidden — replaced by the
    // Stencil card's Size/Offset/Rotation boxes — so the gizmo and the tiling never share a field.
    let ids = painted_hit_ids_for(BrushSettings {
        texture_kind: TextureKind::Noise.to_u8(),
        texture_mapping: TextureMapping::Stencil.to_u8(),
        ..crate::paint_brush::FALLBACK_BRUSH
    });
    for hidden in [
        core_ids::PAINTER_BRUSH_TEXTURE_RAKE,
        core_ids::PAINTER_BRUSH_TEXTURE_RANDOM,
        core_ids::PAINTER_BRUSH_TEXTURE_ANGLE,
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Stencil must hide the texture transform {hidden:?} (the card owns placement). painted = {ids:?}"
        );
    }
    for shown in [
        core_ids::PAINTER_BRUSH_TEXTURE_MAPPING,
        core_ids::PAINTER_BRUSH_STENCIL_SIZE_X,
        core_ids::PAINTER_BRUSH_STENCIL_SIZE_Y,
        core_ids::PAINTER_BRUSH_STENCIL_OFFSET_X,
        core_ids::PAINTER_BRUSH_STENCIL_OFFSET_Y,
        core_ids::PAINTER_BRUSH_STENCIL_ANGLE,
    ] {
        assert!(
            ids.contains(&shown),
            "Stencil card must show {shown:?}. painted = {ids:?}"
        );
    }
}

#[test]
fn param_sliders_register_exactly_the_kind_s_specs() {
    use ph2d_tool_painter::{MAX_TEX_PARAMS, param_specs};
    // For each kind, exactly its `param_specs().len()` param sliders register a hit rect — the first
    // `n` slots paint, and the next slot does NOT (a painted-but-meaningless row is the silent-no-op
    // trap). Voronoi uses all 6 slots; Clouds 5; Diamonds 3.
    for kind in [
        TextureKind::Voronoi,
        TextureKind::Clouds,
        TextureKind::Diamonds,
    ] {
        let ids = painted_hit_ids(kind.to_u8());
        let n = param_specs(kind).len();
        for i in 0..n {
            assert!(
                ids.contains(&core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[i]),
                "{kind:?} param slot {i} must register a hit rect. painted = {ids:?}"
            );
        }
        if n < MAX_TEX_PARAMS {
            assert!(
                !ids.contains(&core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[n]),
                "{kind:?} uses {n} slots — slot {n} must not register. painted = {ids:?}"
            );
        }
    }
}

#[test]
fn color_ramp_controls_gate_on_the_enable_toggle() {
    // Section expanded but ramp OFF (default): only the enable checkbox registers; Mode / Interp / + / −
    // stay hidden.
    let off = painted_hit_ids_ramp_open(BrushSettings {
        texture_kind: TextureKind::Noise.to_u8(),
        ..crate::paint_brush::FALLBACK_BRUSH
    });
    assert!(
        off.contains(&core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE),
        "the 'Use Color Ramp' enable checkbox shows when the section is expanded. painted = {off:?}"
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
    let on = painted_hit_ids_ramp_open(BrushSettings {
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
        "only 2 stops -> no 3rd handle"
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
    // The Alpha-action dropdown below the ramp registers a hit rect.
    assert!(
        on.contains(&core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE),
        "the ramp alpha-action dropdown must register a hit rect. painted = {on:?}"
    );
}
