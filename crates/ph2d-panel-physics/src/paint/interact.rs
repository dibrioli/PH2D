//! **A seção INTERACTION** (W-Hand) — o único bloco deste painel que não descreve
//! o mundo, e sim o PONTEIRO.
//!
//! Arquivo próprio pela mesma razão que o `body.rs` é irmão do `paint.rs`: as
//! partes crescem por motivos diferentes, e esta tem uma forma que as outras não
//! têm (dois rádios + knobs que aparecem por ferramenta).

use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_physics_ecs::{HoldMode, InteractionSettings, InteractionTool};
use ph2d_tokens::Spacing;

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
        &crate::ids::PHYSICS_INTERACT_TOOL_OPT,
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
            &crate::ids::PHYSICS_HOLD_MODE_OPT,
            &HoldMode::ALL.map(hold_label),
            HoldMode::ALL
                .iter()
                .position(|&m| m == it.hold)
                .unwrap_or(0),
        );
    }

    // The numbers of whichever tool is in hand — from the ONE table, asking each
    // row whether it is live.
    let row_gap = Spacing::Xs.px();
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

/// Uma escolha com nome — **pela porta da ESCOLHA** ([`ph2d_editor_core::property_row::paint_choice_row`]).
///
/// ⛔⛔ Até 2026-09-23 ela pintava o nome POR CIMA (numa faixa `Sm + Md`, do smoke de 27/07) e o
/// grupo a toda a largura; a varredura geométrica `nenhum_nome_por_cima_do_controlo` acusava as
/// três escolhas deste painel. Hoje o nome fica AO LADO quando o grupo cabe numa fileira da coluna
/// do valor, e só vira PALETA quando não cabe — a lei vive na porta e não aqui.
///
/// ⚠️ A coluna é a de omissão (`Seccao::apenas_campos(1)`), a MESMA `property_label_col_w` das
/// linhas numéricas desta secção. ⚠️ O id do GRUPO saiu da assinatura: ele é da acessibilidade (o
/// `populate` regista-o) e nunca entrou na pintura.
#[allow(clippy::too_many_arguments)]
pub(super) fn seg_row(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    options: &[ph2d_a11y::NodeId],
    labels: &[&str],
    selected: usize,
) -> f32 {
    let theme = ctx.host.theme();
    let selected = selected.min(labels.len().saturating_sub(1));
    let segs: Vec<(&str, bool, ph2d_a11y::NodeId)> = options
        .iter()
        .zip(labels)
        .enumerate()
        .map(|(i, (&id, &l))| (l, i == selected, id))
        .collect();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        &segs,
        ph2d_editor_core::widget::Seccao::apenas_campos(1),
    )
}
