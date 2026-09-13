//! **A grelha do mundo, a barra do Edit Prefab e o HUD de telemetria** — o que a `publish` do [`super`] (`snapshots`)
//! escreve no `HeroScreen` além da hierarquia, do gizmo e do Inspector. Filho por `#[path]` (OBRA 3 da
//! `line/render-bodies`), chamado no sítio do bloco.

use super::*;

/// A grelha (com as dims da cena sob o split do Motion), a barra da receita aberta, as estatísticas do quadro e a
/// contagem de componentes da Hierarquia.
#[allow(clippy::too_many_arguments)]
pub(super) fn publish(
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    frame_ms_ewma: f32,
    frame_cpu_ms_ewma: f32,
    input_events: u32,
    paint_stamps: u32,
    paint_ms: f32,
) {
    // M14.4b: publish the demo camera + window dims so the
    // hero paints its world grid overlay. `canvas` is a
    // placeholder — `paint_hero_screen` overrides it with
    // the layout-computed canvas rect.
    // Motion Nodes drift fix (2026-07-25): sob o split da tool Motion a CENA renderiza num
    // sub-retângulo (present.rs, via `CenterSplit::scene_viewport`), mas a grade do mundo
    // projetava a janela CHEIA — as linhas não pousavam sobre os sprites/instâncias do
    // Motion. A grade usa as MESMAS dims da cena (a porta única); fora do split é a janela
    // cheia, byte-idêntico.
    let (grid_w, grid_h) =
        ph2d_app_motion::field_gizmo::scene_window_wh(hero.view.center_split, window_size);
    hero.set_grid_view(Some(ph2d_editor_core::GridView {
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: grid_w,
        window_h: grid_h,
        canvas: ph2d_editor_core::zones::Rect::new(0.0, 0.0, 0.0, 0.0),
    }));
    // ⭐⭐⭐ **A BARRA DO MODO DE RECEITA** (o *Edit Prefab*) — o nome do que se está a editar,
    // quantas cópias seguem, e a saída. ⚠️ Publicada como o `grid_view` e pela mesma razão: quem
    // sabe que há uma receita aberta é o MUNDO, e a crate do chrome não o alcança. `None` fecha a
    // barra, e é o caminho de sempre.
    hero.set_prefab_edit(ph2d_app_components::master_editing::open_view(sim));
    // M14.4g Telemetry Phase A: publish real stats. Sprite
    // and entity counts come from PresentWorld (the source of
    // truth for "what we shipped to the GPU this frame"); fps
    // is derived from the EWMA frame_ms.
    let sprite_count = present
        .world_mut()
        .query::<&ph2d_render::RenderInstance>()
        .iter(present.world_mut())
        .count() as u32;
    // ⚠️ **Sem os quads de 9-slice.** Os nove quads de um sprite fatiado partilham o `SimRef`
    // da entidade (é o que faz o carimbo de `z_order` servir os nove), por isso contá-los aqui
    // faria UMA caixa de diálogo aparecer no HUD como NOVE entidades — um número que passaria a
    // mentir exatamente quando a cena fica interessante. A contagem de INSTÂNCIAS acima sobe de
    // propósito: nove quads são nove quads, e isso é o que um contador de desenho deve dizer.
    let entity_count = present
        .world_mut()
        .query_filtered::<&SimRef, bevy_ecs::query::Without<ph2d_render::nine_slice::SlicePatchMirror>>()
        .iter(present.world_mut())
        .count() as u32;
    let fps = if frame_ms_ewma > 0.001 {
        1000.0 / frame_ms_ewma
    } else {
        0.0
    };
    // M14.7 polish (10.1): raw fps = inverse of pure
    // CPU/command-encode time. Floored at 1 ms (1000 fps) so
    // a startup-edge measurement of 0 doesn't blow up to
    // `inf`; real workloads stabilize within a few frames.
    let raw_fps = 1000.0 / frame_cpu_ms_ewma.max(0.001);
    // Diagnostics: wall-clock NOT in the CPU-encode window = present/vsync acquire stall PLUS any
    // between-frames input work — the gap that makes "Raw" rise while FPS falls (HANDOFF §1.R).
    let present_stall_ms = (frame_ms_ewma - frame_cpu_ms_ewma).max(0.0);
    hero.stats = ph2d_editor_core::BottomHudStats {
        fps,
        frame_ms: frame_ms_ewma,
        draws: 1,
        sprite_count,
        entity_count,
        raw_fps,
        present_stall_ms,
        paint_ms,
        input_events,
        paint_stamps,
    };
    // Hierarchy counts use PresentWorld's archetype components
    // (Transform + Sprite + Visibility + ChildOf + Children).
    // It's a proxy — exactly the components the editor's
    // snapshot pipeline observes per entity. Multiplying by
    // entity count is a rough estimate; counting via archetype
    // walk is cheap enough at editor scales.
    let component_count = {
        let world = sim.world();
        let mut total = 0u32;
        for archetype in world.archetypes().iter() {
            let len = archetype.len();
            let comps = archetype.components().len() as u32;
            total = total.saturating_add(len.saturating_mul(comps));
        }
        total
    };
    #[cfg(feature = "panel-hierarchy")]
    ph2d_panel_hierarchy::set_live_component_count(component_count);
}
