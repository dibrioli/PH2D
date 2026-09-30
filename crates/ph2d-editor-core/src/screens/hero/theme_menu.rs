//! ⭐ **A tabela `id da linha ⇄ tema` do menu de tema** — os oito, das duas famílias.
//!
//! Uma tabela, três leitores: o despacho (`chrome::theme`), a marca de estado do menu
//! (`context_menu_overlay`) e as rows (`menu_rows`). Um `match` em cada um seria o trio que
//! envelhece em separado — e a família moderna (2026-09-04) foi exactamente o dia em que os três
//! teriam de mudar juntos.
//!
//! ⚠️ **Mora num ficheiro PRÓPRIO, e não em `menu_rows.rs`, por causa do gate
//! `every_menu_row_reaches_a_handler`:** ele exige que todo id pintado seja nomeado num sítio de
//! despacho *fora* da tabela de rows, e mede-o lendo o fonte. A tabela que o handler consulta É
//! o sítio de despacho — e aqui ela é visível ao gate.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;

/// `(id da linha do menu de tema, tema)` — os oito, na ordem do menu.
pub const THEME_MENU: [(NodeId, Theme); 8] = [
    (ids::CTX_MENU_THEME_FORGE, Theme::Forge),
    (ids::CTX_MENU_THEME_PAINT, Theme::Workshop),
    (ids::CTX_MENU_THEME_SUNSTONE, Theme::Sunstone),
    (ids::CTX_MENU_THEME_BLUEPRINT, Theme::Blueprint),
    (ids::CTX_MENU_THEME_DARK, Theme::Dark),
    (ids::CTX_MENU_THEME_GRAY, Theme::Gray),
    (ids::CTX_MENU_THEME_LIGHT, Theme::Light),
    (ids::CTX_MENU_THEME_OLED, Theme::Oled),
];

/// O id da linha de menu que escolhe `theme`.
#[must_use]
pub fn theme_menu_id(theme: Theme) -> NodeId {
    THEME_MENU
        .iter()
        .find(|(_, t)| *t == theme)
        .map(|(id, _)| *id)
        .expect("todo tema tem uma linha no menu")
}

/// O tema que a linha de menu `id` escolhe, se for uma.
#[must_use]
pub fn theme_of_menu_id(id: NodeId) -> Option<Theme> {
    THEME_MENU
        .iter()
        .find(|(mid, _)| *mid == id)
        .map(|(_, t)| *t)
}

/// ⭐⭐ **O menu de tema de uma SECÇÃO** — `(id da linha, tema)`, com `None` = *«o tema do app»*.
///
/// Os mesmos oito do seletor do topo, com ids próprios (ver [`crate::ids::CTX_MENU_SECTION_THEME_APP`]).
/// ⚠️ É a tabela de despacho do gate `every_menu_row_reaches_a_handler`: o clique numa destas linhas
/// é resolvido por [`section_theme_of_menu_id`], e o marcador de estado lê a mesma tabela.
pub const SECTION_THEME_MENU: [(NodeId, Option<Theme>); 9] = [
    (ids::CTX_MENU_SECTION_THEME_APP, None),
    (ids::CTX_MENU_SECTION_THEME_FORGE, Some(Theme::Forge)),
    (ids::CTX_MENU_SECTION_THEME_PAINT, Some(Theme::Workshop)),
    (ids::CTX_MENU_SECTION_THEME_SUNSTONE, Some(Theme::Sunstone)),
    (
        ids::CTX_MENU_SECTION_THEME_BLUEPRINT,
        Some(Theme::Blueprint),
    ),
    (ids::CTX_MENU_SECTION_THEME_DARK, Some(Theme::Dark)),
    (ids::CTX_MENU_SECTION_THEME_GRAY, Some(Theme::Gray)),
    (ids::CTX_MENU_SECTION_THEME_LIGHT, Some(Theme::Light)),
    (ids::CTX_MENU_SECTION_THEME_OLED, Some(Theme::Oled)),
];

/// A escolha que a linha de menu `id` faz para uma secção: `Some(None)` = o tema do app,
/// `Some(Some(t))` = o tema `t`, `None` = não é uma linha deste menu.
#[must_use]
pub fn section_theme_of_menu_id(id: NodeId) -> Option<Option<Theme>> {
    SECTION_THEME_MENU
        .iter()
        .find(|(mid, _)| *mid == id)
        .map(|(_, t)| *t)
}

/// ⭐⭐ **O clique numa linha de tema de SECÇÃO** — a linha diz o tema, o pedido diz a secção.
///
/// ⚠️ Mora aqui, ao lado da tabela, e não na galeria de widgets onde vive o do contorno: a galeria é
/// `widget` e a tabela é `screens`, e o DAG dos módulos da fundação não deixa o primeiro ler o
/// segundo. Devolve se consumiu o evento.
pub fn apply_section_theme_click(
    store: &mut crate::interaction::WidgetStore,
    event: crate::interaction::WidgetEvent,
) -> bool {
    let crate::interaction::WidgetEvent::Click(id) = event else {
        return false;
    };
    let Some(escolha) = section_theme_of_menu_id(id) else {
        return false;
    };
    if let Some(req) = store.consume_last_context_menu()
        && let crate::interaction::ContextMenuKind::SectionOutline { section } = req.kind
    {
        store.set_section_theme(section, escolha);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Os oito temas têm linha, e `theme_of_menu_id` é o inverso exacto de `theme_menu_id`.
    #[test]
    fn every_theme_has_one_row_and_the_map_round_trips() {
        for theme in Theme::ALL {
            let id = theme_menu_id(theme);
            assert_eq!(theme_of_menu_id(id), Some(theme));
        }
        assert_eq!(THEME_MENU.len(), Theme::ALL.len());
        assert_eq!(theme_of_menu_id(ids::TOOL_UNDO), None);
        // ⭐ e o menu de SECÇÃO cobre os oito mais o do app, sem partilhar um id com o do topo
        for theme in Theme::ALL {
            assert!(SECTION_THEME_MENU.iter().any(|(_, t)| *t == Some(theme)));
        }
        for (id, _) in SECTION_THEME_MENU {
            assert_eq!(
                theme_of_menu_id(id),
                None,
                "a linha de seccao mudaria o tema do APP"
            );
        }
    }

    /// ⭐⭐ **Escolher uma linha de tema no menu do título dá o tema À SECÇÃO do pedido, e «App
    /// Theme» devolve-a ao app.** O pedido é o que o Down no item fotografa antes do clique (o
    /// mesmo caminho do contorno). *Mutação: o braço a ignorar o pedido ⇒ o tema não é escrito.*
    #[test]
    fn a_linha_de_tema_escreve_o_tema_da_seccao_do_pedido() {
        use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent, WidgetStore};
        let seccao = ids::INSP_LIVE_TRANSFORM_SECTION;
        let mut store = WidgetStore::with_capacity(8);
        let abre = |store: &mut WidgetStore| {
            store.open_context_menu(ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: ContextMenuKind::SectionOutline { section: seccao },
            });
            store.close_context_menu();
        };
        abre(&mut store);
        assert!(apply_section_theme_click(
            &mut store,
            WidgetEvent::Click(ids::CTX_MENU_SECTION_THEME_LIGHT)
        ));
        assert_eq!(store.section_theme(seccao), Some(Theme::Light));
        abre(&mut store);
        assert!(apply_section_theme_click(
            &mut store,
            WidgetEvent::Click(ids::CTX_MENU_SECTION_THEME_APP)
        ));
        assert_eq!(store.section_theme(seccao), None);
        // e um clique que não é deste menu não é consumido
        assert!(!apply_section_theme_click(
            &mut store,
            WidgetEvent::Click(ids::CTX_MENU_THEME_LIGHT)
        ));
    }
}
