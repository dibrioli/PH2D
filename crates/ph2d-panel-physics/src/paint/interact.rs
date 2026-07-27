//! **A seção INTERACTION** (W-Hand) — o único bloco deste painel que não descreve
//! o mundo, e sim o PONTEIRO.
//!
//! Arquivo próprio pela mesma razão que o `body.rs` é irmão do `paint.rs`: as
//! partes crescem por motivos diferentes, e esta tem uma forma que as outras não
//! têm (dois rádios + knobs que aparecem por ferramenta).

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_physics_ecs::{HoldMode, InteractionSettings, InteractionTool};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::interact::{IROWS, ISection};

/// Paint the Interaction body (the section header is the caller's). Returns the
/// `y` it ended at.
pub(super) fn paint_interact(
    ctx: &mut PaintCtx,
    it: &InteractionSettings,
    x: f32,
    w: f32,
    y_in: f32,
) -> f32 {
    let mut y = y_in;

    // Which tool. The list IS `InteractionTool::ALL`, so the chip order and the
    // model's order cannot drift.
    y = seg_row(
        ctx,
        x,
        w,
        y,
        tr("panel.physics.tool"),
        ids::PHYSICS_INTERACT_TOOL,
        &ids::PHYSICS_INTERACT_TOOL_OPT,
        &InteractionTool::ALL.map(tool_label),
        InteractionTool::ALL
            .iter()
            .position(|&t| t == it.tool)
            .unwrap_or(0),
    );

    // How it holds — the HAND only. `needs_a_body()` would be the wrong door
    // here even though it reads right: the Pose also takes a body, and it does
    // not hold it with a spring — it SOLVES for it. Two tools, two questions.
    if it.tool == InteractionTool::Hand {
        y = seg_row(
            ctx,
            x,
            w,
            y,
            tr("panel.physics.hold"),
            ids::PHYSICS_HOLD_MODE,
            &ids::PHYSICS_HOLD_MODE_OPT,
            &HoldMode::ALL.map(hold_label),
            HoldMode::ALL
                .iter()
                .position(|&m| m == it.hold)
                .unwrap_or(0),
        );
    }

    // The numbers of whichever tool is in hand — from the ONE table, asking each
    // row whether it is live.
    let row_gap = Spacing::Sm.px();
    for row in IROWS {
        if row.section != ISection::Sim || !(row.shown)(it) {
            continue;
        }
        let value = (row.get)(it);
        let used = super::paint_irow(ctx, row, value, x, w, y);
        y += used + row_gap;
    }

    // How the tool is USED. A hint and not a control, because the gesture lives on
    // the canvas: without it the section reads like settings for something that
    // never happens (the tool is inert with the clock stopped — the law is in
    // `body_grab`).
    //
    // As TRÊS ferramentas desta seção empurram o solver, então a dica é uma só.
    // ⚠️ Ela já foi condicional, e deixou de ser quando a Pose saiu daqui para a
    // seção Joints (W-JointTools): a pergunta *"este gesto quer Play ou Pause?"*
    // passou a ser respondida pela SEÇÃO em que o controle mora, que é a forma de
    // não haver resposta a esquecer.
    super::paint_hint(ctx, "panel.physics.interact_hint", x, w, y)
}

fn tool_label(t: InteractionTool) -> &'static str {
    match t {
        InteractionTool::Hand => tr("panel.physics.tool.hand"),
        InteractionTool::Explode => tr("panel.physics.tool.explode"),
        InteractionTool::Attract => tr("panel.physics.tool.attract"),
    }
}

fn hold_label(m: HoldMode) -> &'static str {
    match m {
        HoldMode::Spring => tr("panel.physics.hold.spring"),
        HoldMode::Rigid => tr("panel.physics.hold.rigid"),
        HoldMode::Rope => tr("panel.physics.hold.rope"),
    }
}

/// A labelled segmented control. Same shape as the Inspector's `seg_row` —
/// deliberately, so the two physics surfaces look like one thing — but a local
/// copy because that one is private to its crate and the shape is six lines.
#[allow(clippy::too_many_arguments)]
pub(super) fn seg_row(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    group: ph2d_a11y::NodeId,
    options: &[ph2d_a11y::NodeId],
    labels: &[&str],
    selected: usize,
) -> f32 {
    let theme = ctx.host.theme();
    let label_font = TypeToken::Sm.px();
    // ⚠️ **`Md` e não `Xs`, e o número saiu de um smoke** (Enio, 2026-07-27:
    // *"ajuste apenas os espaçamentos das labels que ficaram muito apertados"*).
    // O texto é pintado CENTRADO nesta faixa, então o respiro que sobra abaixo
    // dele é metade da folga: com `Xs` (4 px) o rótulo encostava nos chips com
    // 2 px, e o olho lia a palavra como parte do primeiro botão.
    let label_h = label_font + Spacing::Md.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let seg = SegmentedAdaptive::new(
        group,
        label,
        options
            .iter()
            .zip(labels)
            .map(|(&id, &l)| SegmentedOption::new(id, l))
            .collect(),
    )
    .selected(selected.min(labels.len().saturating_sub(1)));
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let seg_h = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y + label_h, w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + label_h + seg_h + Spacing::Sm.px()
}
