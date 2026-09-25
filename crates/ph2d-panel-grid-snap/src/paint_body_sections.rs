//! Per-section paint helpers for the Grid Snap docked panel.
//!
//! Wave 11 §2.2 (ADR-0042 §6 #3) — split of the 300-LOC
//! `paint_body()` orchestrator into 4 section helpers (Grid Kind /
//! Target / Display / Inspect). Same recipe as the CEQ + BgRemoval
//! splits: each helper takes `&mut PaintCtx`, the live `GridSnapState`,
//! geometry params, and a `y_in: f32` cursor; returns `y_out: f32`.

use crate::layout::{ROW_H, row_gap};
use crate::paint_helpers::{
    paint_kind_button_grid, paint_labeled_segmented_row, paint_target_button_stack,
};
use crate::paint_kinds::paint_kind_config;
use crate::paint_rows::{
    paint_number_row_from_state, paint_opacity_slider_row, paint_show_overlay_row,
};
use crate::state::{length_unit, meters_to_display};
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{SectionFold, SectionHeader, paint_section_header};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;

/// Grid Kind label + 3×3 button grid + per-kind config rows.
fn paint_grid_kind_section_body(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        y = paint_kind_button_grid(
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y += row_gap();
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        y = paint_kind_config(
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y
}

/// Target section: label + button stack (Center / Intersection / …) +
/// Subdivisions + Magnetism Radius number rows.
fn paint_target_section_body(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        y = paint_target_button_stack(
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
        // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao`.
        let mag_label = tr("panel.grid_snap.sections.magnetism_radius");
        let sec = crate::paint_rows::seccao(
            ctx.text_system,
            &[tr("panel.grid_snap.sections.subdivisions"), mag_label],
        );
        y = paint_number_row_from_state(
            tr("panel.grid_snap.sections.subdivisions"),
            ph2d_editor_core::grid_snap::ids::GS_CFG_SNAP_SUBDIVISIONS,
            state.snap_subdivisions as f64,
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            sec,
            None,
        );
        y = paint_number_row_from_state(
            mag_label,
            ph2d_editor_core::grid_snap::ids::GS_CFG_SNAP_MAGNETISM_RADIUS,
            meters_to_display(state.snap_magnetism_radius),
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            sec,
            Some(length_unit()),
        );
    }
    y
}

/// Display section: label + Show Overlay toggle + Opacity slider +
/// Color swatch + Layer segmented (In front / Behind).
fn paint_display_section_body(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
) -> f32 {
    let theme = ctx.host.theme();

    let overlay_row = Rect::new(inner_x, y, inner_w, ROW_H);
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_show_overlay_row(
            overlay_row,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y += ph2d_tokens::row_pitch_px();

    let opacity_row = Rect::new(inner_x, y, inner_w, ROW_H);
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_opacity_slider_row(
            opacity_row,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y += ph2d_tokens::row_pitch_px();

    // ⛔⛔ **A fileira «Color» saiu em 2026-08-30 — ela era um controlo que MENTIA e uma SEGUNDA
    // porta para uma grandeza que já tinha a sua.**
    //
    // O RGB escolhido nunca alcançava o canvas: `grid_snap::render::grid_line_color` lê **só**
    // `color_rgba[3]` e deriva o R/G/B do fundo do canvas (`ColorToken::Bg0`, deslocado em
    // luminância). A lei é do Enio, 2026-07-02, e está escrita no doc daquela função — *«o grid
    // sempre lê como um contraste relativo subtil, seja qual for o tema»*. ⚠️ **Ela FICA
    // intocada:** uma cor escolhida à mão pode ser ilegível sobre o fundo, e ele escolheu a
    // robustez.
    //
    // O que sobrava do controlo era o ALFA — e o painel já tem um slider **Opacity** logo acima,
    // que o renderer multiplica (`base_alpha × opacity`). *Duas portas para uma grandeza é
    // exactamente o que este repo evita*, e a segunda vinha embrulhada num selector de cor cujo
    // quadradinho pintava o RGB escolhido: o artista via vermelho no painel e cinzento no canvas.
    //
    // ⚠️ **O `state.color_rgba` FICA no modelo e o renderer continua a multiplicar o alfa dele** —
    // tirá-lo mudaria a aparência de todo projecto já gravado com alfa ≠ 255, que é uma
    // regressão silenciosa. Ele passa a ser o que sempre foi de facto: uma constante do estilo.
    // Gate: `the_grid_panel_offers_no_colour_it_cannot_deliver`.

    let layer_idx = if state.grid_in_front { 0 } else { 1 };
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        let sec = crate::paint_rows::seccao_display(ctx.text_system);
        y = paint_labeled_segmented_row(
            tr("panel.grid_snap.sections.layer"),
            &[
                (
                    tr("panel.grid_snap.sections.in_front"),
                    ph2d_editor_core::grid_snap::ids::GS_LAYER_IN_FRONT,
                ),
                (
                    tr("panel.grid_snap.sections.behind"),
                    ph2d_editor_core::grid_snap::ids::GS_LAYER_BEHIND,
                ),
            ],
            layer_idx,
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            sec,
        );
    }
    y
}

/// Inspect section body: the shared inspector rows of `ph2d_editor_core::grid_snap::inspect`, WITHOUT
/// its header — the header is this panel's collapsible one ([`paint_inspect_section`]).
fn paint_inspect_section_body(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    ph2d_editor_core::grid_snap::inspect::paint_body(
        inner_x,
        inner_w,
        y,
        ctx.scene,
        ctx.text_system,
        theme,
        hit_index,
        store,
        state,
    )
}

/// ⭐⭐ **As quatro secções do painel DOBRAM** (ordem do dono, 2026-09-24, com foto: *«no Grid as
/// seções não fecham. Isso deve ser corrigido»*). Até esse dia cada uma pintava o título à mão
/// (`paint_section_label`: `TypeToken::Md` + uma linha azul POR BAIXO) e não tinha dobra nenhuma; o
/// título dela virou o estilo do [`SectionHeader`] da casa para o app inteiro, com a linha azul à
/// DIREITA do nome — e as secções daqui passaram a usá-lo.
///
/// ⚠️ **`Option<SectionFold>`, nunca um `bool`** — a lição da Física (F4b): o `is_collapsed` vira no
/// quadro do clique enquanto o `t` da dobra ainda desce, e um corpo gateado nele sumiria de uma vez
/// debaixo de uma seta que ainda roda.
#[allow(clippy::too_many_arguments)]
fn dobravel(
    ctx: &mut PaintCtx<'_>,
    id: ph2d_a11y::NodeId,
    title: &str,
    x: f32,
    w: f32,
    y: f32,
    depois: f32,
    corpo: impl FnOnce(&mut PaintCtx<'_>, f32) -> f32,
) -> f32 {
    let theme = ctx.host.theme();
    let h = ph2d_editor_core::widget::section_title_px() + ph2d_tokens::Spacing::Md.px();
    let collapsed = ctx.host.store().is_collapsed(id);
    let rect = Rect::new(x, y, w, h);
    let header = SectionHeader::new(id, title)
        .collapsible(!collapsed)
        .open_t(ctx.host.store().section_open_live(id));
    let body_top = y + h + ph2d_tokens::Spacing::Sm.px();
    let fold = {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_section_header(&header, rect, scene, text_system, theme);
        hit_index.register(id, rect);
        SectionFold::begin(store, id, x, w, body_top, scene, hit_index)
    };
    let Some(fold) = fold else {
        return body_top + depois;
    };
    let fim = corpo(ctx, body_top);
    let scene = &mut *ctx.scene;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    fold.finish(store, scene, hit_index, fim) + depois
}

/// Grid Kind: collapsible header + 3×3 button grid + per-kind config rows.
pub(crate) fn paint_grid_kind_section(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    dobravel(
        ctx,
        crate::ids::GS_SEC_KIND,
        tr("panel.grid_snap.sections.grid_kind"),
        inner_x,
        inner_w,
        y,
        row_gap() * 2.0,
        |ctx, y| paint_grid_kind_section_body(ctx, state, inner_x, inner_w, y),
    )
}

/// Target: collapsible header + the target stack + Subdivisions / Magnetism rows.
pub(crate) fn paint_target_section(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    dobravel(
        ctx,
        crate::ids::GS_SEC_TARGET,
        tr("panel.grid_snap.sections.target"),
        inner_x,
        inner_w,
        y,
        row_gap() * 2.0,
        |ctx, y| paint_target_section_body(ctx, state, inner_x, inner_w, y),
    )
}

/// Display: collapsible header + Show Grid · Opacity · Layer.
pub(crate) fn paint_display_section(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    dobravel(
        ctx,
        crate::ids::GS_SEC_DISPLAY,
        tr("panel.grid_snap.sections.display"),
        inner_x,
        inner_w,
        y,
        row_gap(),
        |ctx, y| paint_display_section_body(ctx, state, inner_x, inner_w, y),
    )
}

/// Inspect: collapsible header + the shared inspector rows.
pub(crate) fn paint_inspect_section(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    dobravel(
        ctx,
        ph2d_editor_core::grid_snap::ids::GS_INSPECT_HEADER,
        ph2d_i18n::tr("chrome.grid_snap.inspect"),
        inner_x,
        inner_w,
        y,
        0.0,
        |ctx, y| paint_inspect_section_body(ctx, state, inner_x, inner_w, y),
    )
}
