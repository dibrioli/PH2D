//! **A seção JOINTS** (W-JointTools) — o que o ponteiro faz a uma cadeia
//! ARTICULADA, com o relógio parado.
//!
//! Irmã de [`super::interact`], e separada dela por uma razão de PRODUTO e não
//! de tamanho: aquela seção descreve o que o ponteiro faz a uma cena que está
//! **rodando** (uma mola, um estouro, um campo) e esta descreve o que ele faz a
//! um **rig** que está parado. As duas famílias querem estados opostos do
//! transporte, e misturá-las obrigaria o artista a saber, chip a chip, qual
//! metade da lista pede Play e qual pede Pause.
//!
//! Cinco modos, e a lista inteira é [`JointTool::ALL`] — a ordem dos chips e a
//! ordem do modelo não podem divergir porque são a mesma.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_physics_ecs::{InteractionSettings, JointTool};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::interact::{IROWS, ISection};

/// Paint the Joints body (the section header is the caller's). Returns the `y`
/// it ended at.
pub(super) fn paint_joint(
    ctx: &mut PaintCtx,
    it: &InteractionSettings,
    x: f32,
    w: f32,
    y_in: f32,
) -> f32 {
    let mut y = super::interact::seg_row(
        ctx,
        x,
        w,
        y_in,
        tr("panel.physics.joint_tool"),
        ids::PHYSICS_JOINT_TOOL,
        &ids::PHYSICS_JOINT_TOOL_OPT,
        &JointTool::ALL.map(label),
        JointTool::ALL
            .iter()
            .position(|&t| t == it.joint)
            .unwrap_or(0),
    );

    // A ponta também obedece a um ÂNGULO? Só o IK tem ponta — os outros quatro
    // não resolvem nada, então a row seria um controle que ninguém lê.
    if it.joint == JointTool::Ik {
        y = super::interact::seg_row(
            ctx,
            x,
            w,
            y,
            tr("panel.physics.ik_angle"),
            ids::PHYSICS_IK_ANGLE,
            &ids::PHYSICS_IK_ANGLE_OPT,
            &[
                tr("panel.physics.ik_angle.free"),
                tr("panel.physics.ik_angle.match"),
            ],
            usize::from(it.ik_match_angle),
        );
    }

    // Os números do modo em mãos — da MESMA tabela da outra seção, filtrada
    // pelo campo `section` (ver [`ISection`] para por que não são duas listas).
    let row_gap = Spacing::Sm.px();
    for row in IROWS {
        if row.section != ISection::Joint || !(row.shown)(it) {
            continue;
        }
        let value = (row.get)(it);
        let used = super::paint_irow(ctx, row, value, x, w, y);
        y += used + row_gap;
    }

    // ⚠️ **Duas linhas de dica, e a segunda é constante de propósito.** A
    // primeira diz o que ESTE modo faz; a segunda diz que o Alt sempre carrega o
    // rig inteiro — um fato que vale nos cinco modos e que, escondido dentro da
    // frase de um deles, seria lido como propriedade daquele modo.
    y = hint(ctx, hint_key(it.joint), x, w, y);
    hint(ctx, "panel.physics.joint_hint.alt", x, w, y)
}

/// Uma linha de dica. Texto puro, hit-indexado por ninguém: é um fato, não um
/// controle, e uma affordance que ele não pode honrar seria pior que o texto.
fn hint(ctx: &mut PaintCtx, key: &str, x: f32, w: f32, y: f32) -> f32 {
    let font = TypeToken::Sm.px();
    let theme = ctx.host.theme();
    paint_text(
        ctx.text_system,
        ctx.scene,
        tr(key),
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y + ROW_H_PX
}

fn label(t: JointTool) -> &'static str {
    match t {
        JointTool::Body => tr("panel.physics.joint_tool.body"),
        JointTool::Rig => tr("panel.physics.joint_tool.rig"),
        JointTool::Links => tr("panel.physics.joint_tool.links"),
        JointTool::Ik => tr("panel.physics.joint_tool.ik"),
        JointTool::Fk => tr("panel.physics.joint_tool.fk"),
    }
}

/// A chave da dica de cada modo. Um `match` exaustivo e não um `format!` sobre
/// o `tag()`: uma chave montada em runtime não é encontrável por grep e some do
/// gate que varre as chaves de i18n.
fn hint_key(t: JointTool) -> &'static str {
    match t {
        JointTool::Body => "panel.physics.joint_hint.body",
        JointTool::Rig => "panel.physics.joint_hint.rig",
        JointTool::Links => "panel.physics.joint_hint.links",
        JointTool::Ik => "panel.physics.joint_hint.ik",
        JointTool::Fk => "panel.physics.joint_hint.fk",
    }
}
