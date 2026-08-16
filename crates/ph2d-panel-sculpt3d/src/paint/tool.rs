//! **QUE FERRAMENTA ESTÁ NA MÃO, E QUE REFERÊNCIA ELA SEGUE** — irmão do
//! `body.rs`, cortado por ASSUNTO.
//!
//! As duas rows viajam juntas porque respondem à MESMA pergunta em dois níveis:
//! *qual verbo* e *segundo quem*. A `Reference` fica logo abaixo da lista de
//! ferramentas por isso, e não por caber ali.
//!
//! O resto do corpo do painel (pincel, espelho, topologia, sombreamento, cena,
//! entrega) fica no `body.rs`.

use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_sculpt3d::{RefMode, Verb, kelvinlet::Scales};
use ph2d_tokens::Spacing;

use super::widgets::{self, command, header, labelled_seg, seg};
use crate::state::{Sculpt3dSnapshot, UiLevel};

/// **A FERRAMENTA** — os dezesseis verbos numa faixa que REFLUI.
///
/// É a mesma decisão que a lista de dez ferramentas do Impasto tomou: um grupo
/// segmentado com muitas opções quebra em linhas, e a alternativa (um dropdown)
/// esconde quinze ferramentas atrás de um clique para mostrar uma.
pub(super) fn paint_tool(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (fold, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_TOOL,
        tr("panel.sculpt3d.section.tool"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    let selected = Verb::ALL
        .iter()
        .position(|&v| v == snap.ui.brush.verb)
        .unwrap_or(0);
    let labels: Vec<&str> = Verb::ALL.iter().map(|v| v.label()).collect();
    y = seg(
        ctx,
        ids::SCULPT3D_SEC_TOOL,
        &ids::SCULPT3D_VERB,
        &labels,
        selected,
        x,
        w,
        y,
    );
    y = paint_reference_row(ctx, snap, x, w, y);
    widgets::end_fold(ctx, fold, y + Spacing::Md.px())
}

/// **A REFERÊNCIA que este verbo segue** — a row `S` · `B` · `L`.
///
/// ⚠️ **Ela pinta só os modos OFERECIDOS** (`RefMode::offered_for`), e o id de
/// cada chip vem da posição no `RefMode::ALL` — nunca da posição na fileira. É a
/// lei anti-chip-morto do §3 do plano: hoje o `L` não declara nada de seu, e um
/// chip que produz o que o vizinho produz é um botão que o artista descobre
/// vazio clicando.
///
/// ⚠️ **E ela fica no BASIC**, com o seletor de ferramenta: o doc 20 mede que a
/// escolha muda o pincel em `1,08× a 1,44×` ao longo do raio e a lei do kernel em
/// `1,7e-3` — é o achado mais consequente do estudo, e escondê-lo atrás de um
/// interruptor Pro seria esconder a decisão que mais importa.
fn paint_reference_row(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let verb = snap.ui.brush.verb;
    let offered: Vec<RefMode> = RefMode::offered_for(verb).collect();
    if offered.len() < 2 {
        // Um modo só não é uma escolha; pintar a fileira seria um rádio de um
        // botão. Não acontece hoje e o gate cobra os dois.
        return y;
    }
    let mode_ids: Vec<_> = offered
        .iter()
        .map(|m| ids::SCULPT3D_REF_MODE[*m as usize])
        .collect();
    let labels: Vec<&str> = offered.iter().map(|m| m.label()).collect();
    let selected = offered
        .iter()
        .position(|&m| m == snap.ui.brush.mode)
        .unwrap_or(0);
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.reference"),
        ids::SCULPT3D_SEC_TOOL,
        &mode_ids,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let y = command(
        ctx,
        ids::SCULPT3D_REF_MODE_ALL,
        tr("panel.sculpt3d.reference_all"),
        x,
        w,
        y,
    );
    paint_elastic_scales_row(ctx, snap, x, w, y)
}

/// **QUÃO LARGO é o campo elástico** — a família de escalas do kernel.
///
/// ⚠️ **A row é oferecida pela MESMA porta que o motor pergunta**
/// (`RefMode::field(verb)`), e não por uma lista de verbos ao lado: o
/// `stroke_target` consome o kernel exatamente onde essa porta devolve `Some`,
/// então uma fileira desenhada em qualquer outro lugar seriam três chips que não
/// movem um vértice. Duas cópias da mesma pergunta é como a próxima nasce
/// desalinhada.
///
/// ⚠️ **PRO, pela regra do [`UiLevel`]:** o valor foi ARMADO — a `Tight` é a
/// família que a medição do resíduo de borda escolheu —, então em Basic o
/// artista está com a largura que o kernel escolheu por ele, não com um vazio.
fn paint_elastic_scales_row(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.ui_level.shows(UiLevel::Pro) {
        return y;
    }
    let verb = snap.ui.brush.verb;
    if snap.ui.brush.mode.field(verb).is_none() {
        return y;
    }
    let labels: Vec<&str> = Scales::ALL.iter().map(|s| s.label()).collect();
    let selected = Scales::ALL
        .iter()
        .position(|&s| s == snap.ui.brush.elastic_scales)
        .unwrap_or(0);
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.elastic_scales"),
        ids::SCULPT3D_SEC_TOOL,
        &ids::SCULPT3D_ELASTIC_SCALES,
        &labels,
        selected,
        x,
        w,
        y,
    )
}
