//! ⭐⭐ **As duas grelhas que NÃO são um ladrilhado: a *quadtree* e a *Voronoi*.**
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, não por tamanho** (tecto de 600 LOC do painel, 2026-09-15,
//! quando cada secção passou a declarar a coluna dela). As seis irmãs do
//! [`super::paint_kinds`] são **ladrilhados**: perguntam um tamanho de célula, uma origem e uma
//! vizinhança, e a grelha estende-se por todo o plano. Estas duas perguntam outra coisa — uma
//! **CAIXA** que as delimita (`min`/`max`) e uma regra de subdivisão (profundidade, sementes) —, e
//! nenhuma delas tem origem nem vizinhança.
//!
//! ⛔ *Um ficheiro que cresce até ao tecto cura-se pelo assunto que já estava lá dentro, nunca por
//! uma entrada nova na lista de folgas* (`CLAUDE.md` §5.0).

use crate::paint_helpers::button_state;
use crate::paint_rows::{paint_aabb_rows, paint_number_row, paint_number_row_from_state};
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::{Button, ButtonKind, paint_button};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_quadtree_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao` e o report do dono de 2026-09-15.
    let caixa = crate::paint_rows::aabb_labels(tr("panel.grid_snap.bounded.qt_bounds"));
    let sec = crate::paint_rows::seccao(
        text_system,
        &[
            tr("panel.grid_snap.bounded.max_leaf"),
            tr("panel.grid_snap.bounded.max_depth"),
            tr("panel.grid_snap.bounded.demo_points"),
            tr("panel.grid_snap.bounded.demo_seed"),
            &caixa[0],
            &caixa[1],
            &caixa[2],
            &caixa[3],
        ],
    );
    y = paint_number_row(
        tr("panel.grid_snap.bounded.max_leaf"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_QT_MAX_PER_LEAF,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
        None,
    );
    y = paint_number_row(
        tr("panel.grid_snap.bounded.max_depth"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_QT_MAX_DEPTH,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
        None,
    );
    y = paint_aabb_rows(
        tr("panel.grid_snap.bounded.qt_bounds"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_QT_BOUNDS_MIN_X,
        ph2d_editor_core::grid_snap::ids::GS_CFG_QT_BOUNDS_MIN_Y,
        ph2d_editor_core::grid_snap::ids::GS_CFG_QT_BOUNDS_MAX_X,
        ph2d_editor_core::grid_snap::ids::GS_CFG_QT_BOUNDS_MAX_Y,
        state.quadtree_cfg.bounds.min,
        state.quadtree_cfg.bounds.max,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_number_row_from_state(
        tr("panel.grid_snap.bounded.demo_points"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_QT_DEMO_POINTS,
        state.quadtree_cfg.demo_point_count as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
        None,
    );
    paint_number_row_from_state(
        tr("panel.grid_snap.bounded.demo_seed"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_QT_DEMO_SEED,
        state.quadtree_cfg.demo_rng_seed as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_voronoi_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao` e o report do dono de 2026-09-15.
    let caixa = crate::paint_rows::aabb_labels(tr("panel.grid_snap.bounded.voronoi_bounds"));
    let sec = crate::paint_rows::seccao(
        text_system,
        &[
            tr("panel.grid_snap.bounded.seed_count"),
            tr("panel.grid_snap.bounded.rng_seed"),
            tr("panel.grid_snap.bounded.lloyd_iters"),
            &caixa[0],
            &caixa[1],
            &caixa[2],
            &caixa[3],
        ],
    );
    y = paint_number_row(
        tr("panel.grid_snap.bounded.seed_count"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_SEED_COUNT,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
        None,
    );
    y = paint_number_row(
        tr("panel.grid_snap.bounded.rng_seed"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_RNG_SEED,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
        None,
    );
    y = paint_number_row(
        tr("panel.grid_snap.bounded.lloyd_iters"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_LLOYD_ITERS,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
        None,
    );
    y = paint_aabb_rows(
        tr("panel.grid_snap.bounded.voronoi_bounds"),
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_BOUNDS_MIN_X,
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_BOUNDS_MIN_Y,
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_BOUNDS_MAX_X,
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_BOUNDS_MAX_Y,
        state.voronoi_cfg.bounds.min,
        state.voronoi_cfg.bounds.max,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    // Reseed button.
    let reseed_label = tr("panel.grid_snap.bounded.reseed_next_rng");
    let reseed_rect =
        ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, y, reseed_label);
    let btn = Button {
        id: ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_RESEED,
        label: reseed_label.to_string(),
        state: button_state(
            store,
            ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_RESEED,
        ),
        kind: ButtonKind::Default,
        // Neutro: este sítio ainda não adere ao eixo do hover (ver `ph2d_editor_core::motion`).
        hover_t: 1.0,
        // Neutro: um botão sozinho arredonda os quatro cantos (a lei do grupo, wave 20).
        cell: ph2d_editor_core::widget::GroupCell {
            col: ph2d_editor_core::widget::GroupPos::Only,
            row: ph2d_editor_core::widget::GroupPos::Only,
        },
    };
    paint_button(&btn, reseed_rect, scene, text_system, theme);
    hit_index.register(
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_RESEED,
        reseed_rect,
    );
    y + ph2d_tokens::row_pitch_px()
}
