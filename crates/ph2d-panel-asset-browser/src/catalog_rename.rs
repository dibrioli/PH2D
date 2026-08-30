//! ⭐⭐ **O campo de renomear um catálogo** (plano 07, wave A3) — o molde é o
//! `marker_rename` da Timeline, e a razão de o copiar é uma só: ele é a única das três
//! versões desta forma no repo que **desiste quando o alvo desapareceu**.
//!
//! # ⚠️ Porque o alvo é o `CatalogId` e não o índice da linha
//!
//! A Timeline guarda o **índice** do marcador e por isso tem de reconferir o `snap` a cada
//! quadro. Aqui a taxonomia tem identidade própria (`CatalogId`), então o campo guarda-a — e
//! um catálogo criado ou apagado noutro sítio **reordena a coluna sem mover o alvo**. O
//! preço é o mesmo: se o catálogo deixar de existir, o campo abandona.
//!
//! # ⚠️ O campo ACOMPANHA a linha, e é preso ao corpo da lista
//!
//! Ele é pintado **por cima** da linha (depois do recorte, como a barra), e o `y` é
//! **cravado** dentro do corpo — a mesma lei do `x` do molde. Sem isso, rolar a lista com o
//! campo aberto deixaria um campo focado fora do painel a comer as teclas.

use ph2d_asset_index::CatalogId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, StrokeToken};

use crate::ids;
use crate::state::{AssetBrowserState, CatalogRename, with_catalogs};

/// Abre o campo sobre `id`. ⚠️ **`opened: false`** — quem semeia o buffer é o `paint`, uma vez
/// só; semear aqui obrigaria o despacho a conhecer o `WidgetStore` do texto.
pub(crate) fn open(state: &mut AssetBrowserState, id: CatalogId) {
    state.renaming = Some(CatalogRename { id, opened: false });
}

/// Pinta o campo aberto (nada a fazer sem um). `row_y` é o topo da linha do catálogo neste
/// quadro — `None` quando ela não foi pintada (rolada para fora), e aí o campo cola-se ao
/// bordo do corpo em vez de desaparecer.
pub(crate) fn paint(
    state: &mut AssetBrowserState,
    ctx: &mut PaintCtx,
    list_rect: Rect,
    row_h: f32,
    row_y: Option<f32>,
) {
    let Some(mut r) = state.renaming else {
        return;
    };
    // O catálogo pode ter desaparecido (apagado aqui ou noutra janela) — abandona em vez de
    // renomear um id que já não tem dono.
    let Some(seed) = with_catalogs(|t| t.get(r.id).map(|c| c.label().to_string())) else {
        state.renaming = None;
        return;
    };
    let theme = ctx.host.theme();
    let y = safe_clamp(
        row_y.unwrap_or(list_rect.y),
        list_rect.y,
        (list_rect.y + list_rect.h - row_h).max(list_rect.y),
    );
    let rect = Rect::new(list_rect.x, y, list_rect.w, row_h);

    // O 1.º quadro semeia o buffer com o nome actual, o cursor no fim, e toma o foco — UMA
    // vez (re-semear a cada quadro apagaria o que o artista escreveu). O `register` que
    // SUBSTITUI é seguro porque este id não tem tabelas laterais (ver `ids`).
    if !r.opened {
        ctx.host
            .store_mut()
            .register(ids::ASSET_CATALOG_RENAME, seed_state(&seed));
        ctx.host
            .store_mut()
            .set_focus(Some(ids::ASSET_CATALOG_RENAME));
        // O Esc aborta — quem o diz é o CAMPO, não uma lista de ids dentro do `dispatch_key`.
        ctx.host
            .store_mut()
            .mark_cancel_on_escape(ids::ASSET_CATALOG_RENAME);
        r.opened = true;
        state.renaming = Some(r);
    }

    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Xs.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        rect,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Accent, theme),
    );

    let (ti_state, text, caret, anchor) = match ctx.host.store().get(ids::ASSET_CATALOG_RENAME) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Focused, String::new(), 0, None),
    };
    let input = TextInput::new(ids::ASSET_CATALOG_RENAME, "").visual((
        ti_state,
        ctx.host.store().hover_live(ids::ASSET_CATALOG_RENAME),
    ));
    paint_text_input_with_buffer(
        &input,
        Some(text.as_str()),
        Some(caret),
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host
        .hit_index_mut()
        .register(ids::ASSET_CATALOG_RENAME, rect);
}

/// ⭐⭐⭐ **Como o campo NASCE: com o nome inteiro SELECCIONADO.**
///
/// ⛔⛔ **O molde da Timeline põe o cursor no fim e não selecciona nada, e aqui isso é um defeito** —
/// apanhado pelo smoke de 2026-08-30: renomear *«Catalog»* para *«Heroes»* dava **«CatalogHeroes»**.
/// *Um campo que abre pelo item «Rename…» já sabe que o artista quer OUTRO nome*; é o que o Finder,
/// o Blender e o Unity fazem, e escolher o contrário obriga a um Ctrl+A que ninguém adivinha.
///
/// ⚠️ **Acrescentar continua a um passo** (uma seta), e substituir passa a estar a zero — a
/// assimetria é deliberada e é a que o gesto pede.
#[must_use]
fn seed_state(label: &str) -> InteractiveState {
    InteractiveState::TextInput {
        state: TextInputState::Focused,
        text: label.to_string(),
        caret: label.len(),
        // ⚠️ **Âncora em `0` com o cursor no FIM** — a selecção é o nome todo, e a primeira tecla
        // escrita apaga-a (`delete_selection_if_any`, o «type to overwrite» do despachante).
        selection_anchor: Some(0),
    }
}

/// Fecha o campo e devolve o novo nome, se houver um que valha a pena mandar.
///
/// ⚠️ **O `take` torna o par Enter→(`Submit`,`Blur`) idempotente** — o segundo evento não acha
/// nada. ⚠️ E um nome **igual ao actual** não levanta acção: ela marcaria o projecto sujo por
/// nada. ⛔ Um nome **vazio ou com `/`** vai na mesma — quem recusa e **fala** é o dreno, e
/// duplicar a regra aqui daria duas respostas à mesma pergunta.
pub(crate) fn commit(state: &mut AssetBrowserState, store: &WidgetStore) -> Option<(u128, String)> {
    let r = state.renaming.take()?;
    let text = match store.get(ids::ASSET_CATALOG_RENAME) {
        Some(InteractiveState::TextInput { text, .. }) => text.trim().to_string(),
        _ => return None,
    };
    let current = with_catalogs(|t| t.get(r.id).map(|c| c.label().to_string()))?;
    if text == current {
        return None;
    }
    Some((r.id.0, text))
}

/// ⭐⭐ **Abandona o campo E LARGA O FOCO** — a metade que o `cancel` não pode fazer.
///
/// ⛔⛔ Sem ela, fechar a coluna (o botão *só-grade*) ou estreitá-la até ela colapsar deixava um
/// campo **focado e invisível a comer as teclas**: o `paint` da coluna sai cedo quando a largura é
/// zero, então o campo deixa de ser pintado e registado, mas o `WidgetStore` continua com o foco
/// nele — e a partir daí escrever no app não faz nada em lado nenhum.
///
/// ⚠️ **O foco só se larga se for NOSSO** — pisar o foco de outro widget seria trocar um defeito
/// por outro.
pub(crate) fn abandon(state: &mut AssetBrowserState, store: &mut WidgetStore) {
    if state.renaming.take().is_some() && store.focus_id() == Some(ids::ASSET_CATALOG_RENAME) {
        store.set_focus(None);
    }
}

/// Abandona sem gravar (Esc).
pub(crate) fn cancel(state: &mut AssetBrowserState) {
    state.renaming = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐⭐ **O campo abre com o nome INTEIRO seleccionado.**
    ///
    /// ⚠️ Este gate nasceu de um defeito que **nenhum dos 24 gates de costura via**: eles medem o
    /// que o `Submit` manda, e a semente é escrita pelo pintor. O smoke apanhou-o em produto —
    /// *«Catalog» + «Heroes» = «CatalogHeroes»* — e a cura mudou-se para uma função com nome para
    /// deixar de depender de alguém correr a cena.
    ///
    /// **Mutação que deve sangrar:** `selection_anchor: None`.
    #[test]
    fn the_field_opens_with_the_whole_name_selected() {
        let InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        } = seed_state("Catalog")
        else {
            panic!("a semente deixou de ser um campo de texto");
        };
        assert_eq!(text, "Catalog");
        assert_eq!(caret, text.len(), "o cursor tem de estar no fim");
        assert_eq!(
            selection_anchor,
            Some(0),
            "sem selecção, escrever ACRESCENTA ao nome antigo — foi o defeito de 2026-08-30"
        );
    }

    /// ⚠️ **Um nome vazio não produz uma selecção inválida** — o caso existe (um catálogo com
    /// rótulo vazio é recusado pelo modelo, mas a semente lê o que estiver lá).
    #[test]
    fn an_empty_name_seeds_an_empty_field() {
        let InteractiveState::TextInput {
            caret,
            selection_anchor,
            ..
        } = seed_state("")
        else {
            panic!("a semente deixou de ser um campo de texto");
        };
        assert_eq!(caret, 0);
        assert_eq!(selection_anchor, Some(0));
    }
}
