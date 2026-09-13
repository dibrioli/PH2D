//! **A FILEIRA SEGMENTADA de um cartão** — a porta partilhada pelo cartão de Impasto, pelo rig de luz
//! (`paint_impasto_rig.rs`) e pela Wet Paint (`paint_wetpaint.rs`).
//!
//! ⚠️ Morava em `paint_impasto.rs` e os dois vizinhos chamavam-na por `crate::paint_impasto::…`: uma
//! porta de TRÊS consumidores com a morada de um deles. Saiu para aqui em 2026-09-13, por
//! responsabilidade, quando a migração dos rótulos para a tabela de strings (as chamadas `tr(…)`
//! longas partidas pelo `rustfmt`) levou aquele ficheiro a `604` contra o tecto de `600` de um
//! painel. O corpo das duas funções é o de lá, byte a byte; só o `seg_row` passou a `pub(crate)`.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::ROW_H_PX;

/// [`seg_row`] with OWNED labels — the lamp chips carry an on/off mark, so they cannot be `&'static str`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn seg_row_owned(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    group_id: ph2d_a11y::NodeId,
    a11y: &str,
    options: &[(ph2d_a11y::NodeId, String)],
    selected: usize,
) -> f32 {
    let opts: Vec<SegmentedOption> = options
        .iter()
        .map(|(id, label)| SegmentedOption::new(*id, label.as_str()))
        .collect();
    let seg = SegmentedAdaptive::new(group_id, a11y, opts).selected(selected);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y, content_w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + used + ph2d_tokens::control_gap_px()
}

/// A segmented option group as one card row (mirrors `paint_deform::seg_group`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn seg_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    group_id: ph2d_a11y::NodeId,
    a11y: &str,
    options: &[(ph2d_a11y::NodeId, &str)],
    selected: usize,
) -> f32 {
    let opts: Vec<SegmentedOption> = options
        .iter()
        .map(|(id, label)| SegmentedOption::new(*id, *label))
        .collect();
    let seg = SegmentedAdaptive::new(group_id, a11y, opts).selected(selected);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y, content_w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + used + ph2d_tokens::control_gap_px()
}
