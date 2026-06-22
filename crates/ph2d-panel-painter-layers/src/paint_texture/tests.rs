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
