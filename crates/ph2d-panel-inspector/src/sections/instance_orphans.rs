//! ⭐⭐⭐ **O BLOCO DAS EXCEPÇÕES SEM ALVO** do cartão de instância (ADR-0164 / F5.3 e F5.3-ter).
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::instance`]**, e o corte foi imposto pelo
//! `panel_functions_under_loc_cap` quando o `✕` por linha entrou (244 de 200). A lei da casa é
//! pagar um tecto com um **CORTE**, nunca com uma linha de isenção — e o assunto estava à mão: lá
//! mora *«o que esta cópia tem de diferente da receita»*, aqui *«o que sobrou de peças que já não
//! existem»*. São dois estados diferentes, e o cartão já os pintava em cores diferentes.
//!
//! # ⚠️ MEDIR e PINTAR são duas funções, e as duas vivem aqui
//!
//! O fundo do cartão tem de ser desenhado **antes** do conteúdo (senão cobre-o), logo a altura é
//! medida numa passagem separada. ⇒ as duas metades têm de concordar sobre o orçamento de quebra,
//! e a maneira de garantir isso é serem vizinhas: uma largura que mudasse só numa delas põe o botão
//! de baixo por cima do texto — que é exactamente o defeito que este bloco pagou em 2026-09-05.

use super::*;
use ph2d_editor_core::screens::hero::InspectorInstanceInfo;

/// **A altura das linhas de órfão**, com o orçamento de quebra que o pintor vai usar.
pub(crate) fn rows_height(
    text_system: &mut TextSystem,
    info: &InspectorInstanceInfo,
    font: f32,
    orphan_tw: f32,
    line: f32,
) -> f32 {
    info.orphan_rows
        .iter()
        .map(|r| super::text_h(text_system, &row_text(r), font, orphan_tw, line))
        .sum()
}

/// Quantas linhas ficaram **sem `✕`** — a tabela de ids tem tecto e a lista não.
pub(crate) fn dropless(info: &InspectorInstanceInfo) -> usize {
    info.orphan_rows
        .len()
        .saturating_sub(ids::INSP_INSTANCE_DROP_ORPHAN.len())
}

/// Quantas linhas de altura FIXA este bloco acrescenta: o aviso do tecto e o botão de limpar.
pub(crate) fn fixed_rows(info: &InspectorInstanceInfo) -> usize {
    usize::from(dropless(info) > 0) + usize::from(info.orphans() > 0)
}

fn row_text(row: &ph2d_editor_core::screens::hero::OrphanRow) -> String {
    format!("\u{2022} {}", row.label())
}

/// Pinta as linhas, o `✕` de cada uma, o aviso do tecto e o botão de limpar. Devolve o `y` de baixo.
///
/// ⚠️ **Elas vêm em `Text2`, e as vivas em `Text1`:** as de cima são o que esta peça TEM, estas são
/// o que sobrou de peças que já não existem. Pintá-las iguais faria o artista ler uma lista só, e o
/// botão de baixo apaga **apenas** estas.
///
/// ⚠️ **A ordem é a do mapa, que agrupa por PEÇA por construção** (a chave ordena `piece` antes de
/// `type_id`) — é o agrupamento que torna a lista legível quando duas peças morreram.
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
    for (i, row) in info.orphan_rows.iter().enumerate() {
        let text = row_text(row);
        let h = super::text_h(text_system, &text, at.font, at.orphan_tw, at.line);
        paint_text(
            text_system,
            scene,
            &text,
            at.tx + Spacing::Sm.px(),
            ty,
            at.font,
            at.orphan_tw,
            resolve(ColorToken::Text2, theme),
        );
        // ⭐⭐⭐ **Cada uma tem o `✕` DELA** (2026-09-05). Até aqui a lista dizia *quais* são e o
        // único gesto apagava **todas**: quem quisesse largar uma de cinco largava as cinco.
        // *Uma lista que se lê item a item pede um gesto item a item* — e o botão de baixo passa a
        // ser o atalho, não a única saída.
        //
        // ⚠️ O `✕` fica na PRIMEIRA linha do texto, e não centrado no bloco: numa linha embrulhada
        // ele desceria para o meio do parágrafo e deixaria de se ler como o botão daquela entrada.
        if let Some(&id) = ids::INSP_INSTANCE_DROP_ORPHAN.get(i) {
            let host = Rect::new(at.right - at.line, ty, at.line, at.line);
            hit_index.register(id, host);
            paint_icon_button(
                host,
                IconGlyph::Builtin(IconId::Close),
                IconButtonStyle::Compact,
                store.button_visual(id),
                scene,
                theme,
            );
        }
        ty += h;
    }
    // ⛔ **As que ficaram sem `✕` são DITAS** — uma linha que perde o botão em silêncio lê-se como
    // um botão morto. Ver [`ids::MAX_INSTANCE_ORPHAN_ROWS`].
    let left_out = dropless(info);
    if left_out > 0 {
        paint_text(
            text_system,
            scene,
            &format!("+{left_out} without a button \u{2014} Clear removes those too"),
            at.tx + Spacing::Sm.px(),
            ty,
            at.small,
            at.list_tw,
            resolve(ColorToken::Text2, theme),
        );
        ty += at.line;
    }
    // ⭐ O gesto que larga TODAS — e ele **só aparece quando existem**: um botão permanentemente
    // inerte é ruído que o artista aprende a ignorar.
    if info.orphans() > 0 {
        let host = Rect::new(at.tx, ty, at.tw, at.line);
        hit_index.register(ids::INSP_INSTANCE_CLEAR_ORPHANS, host);
        let button = Button::new(
            ids::INSP_INSTANCE_CLEAR_ORPHANS,
            format!("Clear {} unused override(s)", info.orphans()),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_INSTANCE_CLEAR_ORPHANS));
        paint_button(&button, host, scene, text_system, theme);
        ty += at.line;
    }
    ty
}
