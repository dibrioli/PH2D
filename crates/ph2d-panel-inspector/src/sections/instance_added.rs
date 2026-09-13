//! ⭐⭐⭐ **O BLOCO DAS PEÇAS ACRESCENTADAS** do cartão de instância (ADR-0164 / F5.11).
//!
//! O *Added GameObject* do Unity: o artista pendura uma peça dentro de uma cópia, e o gesto que
//! falta é **dá-la à receita** para que todas as irmãs a recebam.
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::instance_removed`], e ele vem ANTES.** As duas listas são
//! diferenças VIVAS da cópia (desfazem-se as duas), e a ordem sai do que o artista está a olhar: a
//! peça acrescentada **está na tela**, a recusada não. *O bloco cujo sujeito se vê lê-se primeiro.*
//!
//! ⚠️ **A linha é um BOTÃO inteiro**, como a do *Put back*, e pela mesma razão: o verbo é
//! **construtivo** e o rótulo *é* a acção. ⛔ Não há `✕` — apagar uma peça acrescentada já é o
//! `Delete` da linha dela na Hierarquia, que a porta estreita
//! (`is_a_recipe_given_piece`) deixa passar exactamente por ela não ter elo. *Um segundo gesto
//! destrutivo aqui seria a segunda porta para o mesmo verbo.*

use super::*;
use ph2d_editor_core::screens::hero::InspectorInstanceInfo;

/// Quantas linhas de altura FIXA este bloco acrescenta.
pub(crate) fn rows(info: &InspectorInstanceInfo) -> usize {
    let painted = info
        .added_rows
        .len()
        .min(ids::INSP_INSTANCE_APPLY_ADDED.len());
    painted + usize::from(buttonless(info) > 0)
}

/// Quantas ficaram **sem botão** — a tabela de ids tem tecto.
fn buttonless(info: &InspectorInstanceInfo) -> usize {
    info.added_rows
        .len()
        .saturating_sub(ids::INSP_INSTANCE_APPLY_ADDED.len())
}

/// Pinta um botão por peça acrescentada. Devolve o `y` de baixo.
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
    for (i, row) in info.added_rows.iter().enumerate() {
        // ⚠️ A tabela de ids tem tecto, e o `get` é o que impede um índice fora dela — o que sobra
        // é CONTADO na linha seguinte, nunca truncado em silêncio.
        let Some(&id) = ids::INSP_INSTANCE_APPLY_ADDED.get(i) else {
            break;
        };
        let host = Rect::new(at.tx, ty, at.tw, at.line);
        hit_index.register(id, host);
        // ⚠️ **O rótulo sai do MODELO** (`AddedRow::label`), como o das duas irmãs: ele nomeia a
        // peça **e** a receita de destino, e com aninhamento essa receita não é a do topo do
        // cartão.
        let button = Button::new(id, row.label())
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id));
        paint_button(&button, host, scene, text_system, theme);
        ty += at.line;
    }
    // ⛔ **As que ficaram sem botão são DITAS, e a saída é NOMEADA** — acima do tecto o *Apply to
    // Master* do menu da linha alcança uma peça de cada vez. *Uma linha que perde o botão em
    // silêncio lê-se como um botão morto.*
    let left_out = buttonless(info);
    if left_out > 0 {
        paint_text(
            text_system,
            scene,
            &format!("+{left_out} more \u{2014} use Apply to Master on the row"),
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
