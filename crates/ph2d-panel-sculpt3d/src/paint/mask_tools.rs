//! **O QUE SE FAZ COM UMA MÁSCARA PINTADA** — as quatro operações, o extract e
//! o transform.
//!
//! Irmão (`#[path]`-livre, `mod` normal) do [`super::body`], e o corte é de
//! ASSUNTO: o `body` responde *como o pincel se comporta* (o verbo, a curva, o
//! padrão, os knobs) e isto responde *o que fazer com o que ele já pintou*. As
//! duas metades crescem por motivos diferentes — foi o transform que cruzou o
//! teto de LOC do pai, e a wave anterior já tinha empurrado o extract para
//! dentro dele.
//!
//! ⚠️ E as seis moram na seção do PINCEL, não numa própria: um artista que
//! acabou de pintar máscara procura o que fazer com ela **onde ele a pintou**.

use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_sculpt3d::TransformKind;
use ph2d_tokens::Spacing;

use super::body::{MASK_LABELS, paint_one_row};
use super::widgets::{command, labelled_seg};
use crate::rows;
use crate::state::Sculpt3dSnapshot;

/// As seis coisas que se faz com uma máscara já pintada.
pub(super) fn paint_mask_tools(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    // As quatro operações de máscara moram AQUI, ao lado do verbo que a pinta —
    // um artista que acabou de pintar máscara procura o que fazer com ela onde
    // ele a pintou, não numa seção própria três rolagens abaixo.
    let mask: Vec<&str> = MASK_LABELS.iter().map(|k| tr(k)).collect();
    let mut y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.mask"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_MASK_OP,
        &mask,
        // ⚠️ Nenhum fica aceso, e `usize::MAX` é como se diz isso: as quatro são
        // GESTOS (executam e acabam), não um modo escolhido. Acender uma delas
        // afirmaria um estado que não existe.
        usize::MAX,
        x,
        w,
        y,
    );
    // **O EXTRACT**, e os dois números que ele lê, logo abaixo das quatro
    // operações — a quinta coisa que se faz com uma máscara pintada, e a única
    // que produz uma PEÇA. Ela aparece na seção da cena, no número de peças; o
    // gesto fica onde a máscara foi pintada.
    //
    // ⚠️ **Os knobs vêm DEPOIS do botão**, e não antes: eles são os argumentos
    // dele, e um argumento acima do verbo lê como um knob solto do pincel.
    y = command(
        ctx,
        ids::SCULPT3D_EXTRACT,
        tr("panel.sculpt3d.extract"),
        x,
        w,
        y,
    ) + Spacing::Sm.px();
    for row in rows::rows().filter(|r| r.place == rows::Place::AfterExtract && r.visible(&snap.ui))
    {
        y = paint_one_row(ctx, snap, row, x, w, y);
    }
    // **O TRANSFORM** — a SEXTA coisa que se faz com uma máscara pintada, e a
    // segunda que não é um traço: mover, girar e escalar a parte LIVRE.
    //
    // ⚠️ **Aceso ou apagado, ao contrário das quatro operações acima:** elas
    // executam e acabam (`usize::MAX`, nenhuma acesa), estes três ARMAM o botão
    // esquerdo — então um deles fica aceso enquanto vale, e clicar o aceso
    // desarma. Um rádio que nunca acende esconderia o único estado em que o
    // esquerdo deixa de esculpir.
    let labels: Vec<&str> = TransformKind::ALL.iter().map(|k| k.label()).collect();
    let armed = snap
        .transform
        .and_then(|k| TransformKind::ALL.iter().position(|&a| a == k))
        .unwrap_or(usize::MAX);
    y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.transform"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_TRANSFORM,
        &labels,
        armed,
        x,
        w,
        y,
    );
    y
}
