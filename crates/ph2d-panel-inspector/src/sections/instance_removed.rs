//! ⭐⭐⭐ **O BLOCO DAS PEÇAS RECUSADAS** do cartão de instância (ADR-0164 / F5.10).
//!
//! O *Removed GameObject* do Unity: uma cópia pode apagar uma peça que a receita tem, e só ela a
//! perde. ⚠️ **É a única diferença de uma cópia que não se vê na cópia** — as outras leem-se no
//! objecto (a cor mudou, a peça mexeu-se); esta lê-se numa **ausência**, e uma ausência não tem
//! onde ser clicada. *Sem esta lista o gesto seria irreversível pelo painel.*
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::instance`]** e do [`super::instance_orphans`], e a fronteira
//! entre os dois últimos é o que cada linha SABE: um órfão perdeu a peça no mestre (o nome e os
//! bytes viajam com ele); uma peça recusada **continua viva na receita**, e o nome dela lê-se de lá.
//!
//! ⚠️ **A linha é um BOTÃO inteiro, e não texto + `✕` como a dos órfãos.** As duas escolhas seguem
//! do verbo: largar um órfão é **destruir**, e um `✕` pequeno ao lado do que se vai perder é a forma
//! de o dizer; devolver uma peça é **construir**, e aí o rótulo *é* a acção (`Put back "Arm"`).
//! ⇒ nenhuma delas embrulha, porque um botão não embrulha — e é por isso que este bloco não precisa
//! da medição que o dos órfãos paga.

use super::*;
use ph2d_editor_core::screens::hero::InspectorInstanceInfo;

/// Quantas linhas de altura FIXA este bloco acrescenta.
pub(crate) fn rows(info: &InspectorInstanceInfo) -> usize {
    let painted = info
        .removed_rows
        .len()
        .min(ids::INSP_INSTANCE_RESTORE_PIECE.len());
    painted + usize::from(buttonless(info) > 0)
}

/// Quantas ficaram **sem botão** — a tabela de ids tem tecto.
fn buttonless(info: &InspectorInstanceInfo) -> usize {
    info.removed_rows
        .len()
        .saturating_sub(ids::INSP_INSTANCE_RESTORE_PIECE.len())
}

/// Pinta um botão por peça recusada. Devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorInstanceInfo,
    at: super::instance::CardMetrics,
    mut ty: f32,
) -> f32 {
    for (i, row) in info.removed_rows.iter().enumerate() {
        // ⚠️ A tabela de ids tem tecto, e o `get` é o que impede um índice fora dela — o que sobra
        // é CONTADO na linha seguinte, nunca truncado em silêncio.
        let Some(&id) = ids::INSP_INSTANCE_RESTORE_PIECE.get(i) else {
            break;
        };
        let host = Rect::new(at.tx, ty, at.tw, at.line);
        hit_index.register(id, host);
        // ⚠️ **O rótulo sai do MODELO** (`RemovedRow::label`), como o da escada do *Aplicar*:
        // escrever a frase num pintor põe a lei num sítio que nenhum gate de modelo alcança.
        let button = Button::new(id, row.label())
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id));
        paint_button(&button, host, scene, text_system, theme);
        ty += at.line;
    }
    // ⛔ **As que ficaram sem botão são DITAS, e a saída é NOMEADA** — acima do tecto o *Revert* da
    // raiz devolve todas de uma vez. *Uma linha que perde o botão em silêncio lê-se como um botão
    // morto.*
    let left_out = buttonless(info);
    if left_out > 0 {
        paint_text(
            text_system,
            scene,
            &format!("+{left_out} more \u{2014} Revert on the copy puts every piece back"),
            at.tx + Spacing::Sm.px(),
            ty,
            at.small,
            at.list_tw,
            resolve(ColorToken::Text2, theme),
        );
        ty += at.line;
    }
    ty
}
