//! ⭐ **A tabela `id da linha ⇄ tema` do menu de tema** — os doze, das duas famílias.
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

/// `(id da linha do menu de tema, tema)` — os doze, na ordem do menu.
pub const THEME_MENU: [(NodeId, Theme); 12] = [
    (ids::CTX_MENU_THEME_FORGE, Theme::Forge),
    (ids::CTX_MENU_THEME_PAINT, Theme::Workshop),
    (ids::CTX_MENU_THEME_SUNSTONE, Theme::Sunstone),
    (ids::CTX_MENU_THEME_BLUEPRINT, Theme::Blueprint),
    (ids::CTX_MENU_THEME_DARK, Theme::Dark),
    (ids::CTX_MENU_THEME_GRAY, Theme::Gray),
    (ids::CTX_MENU_THEME_LIGHT, Theme::Light),
    (ids::CTX_MENU_THEME_OLED, Theme::Oled),
    (ids::CTX_MENU_THEME_PLUMBER_RED, Theme::PlumberRed),
    (ids::CTX_MENU_THEME_PLUMBER_GREEN, Theme::PlumberGreen),
    (ids::CTX_MENU_THEME_SUNSET, Theme::Sunset),
    (ids::CTX_MENU_THEME_CANDY, Theme::Candy),
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
/// Os mesmos doze do seletor do topo, com ids próprios (ver [`crate::ids::CTX_MENU_SECTION_THEME_APP`]).
/// ⚠️ É a tabela de despacho do gate `every_menu_row_reaches_a_handler`: o clique numa destas linhas
/// é resolvido por [`section_theme_of_menu_id`], e o marcador de estado lê a mesma tabela.
pub const SECTION_THEME_MENU: [(NodeId, Option<Theme>); 13] = [
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
    (
        ids::CTX_MENU_SECTION_THEME_PLUMBER_RED,
        Some(Theme::PlumberRed),
    ),
    (
        ids::CTX_MENU_SECTION_THEME_PLUMBER_GREEN,
        Some(Theme::PlumberGreen),
    ),
    (ids::CTX_MENU_SECTION_THEME_SUNSET, Some(Theme::Sunset)),
    (ids::CTX_MENU_SECTION_THEME_CANDY, Some(Theme::Candy)),
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

/// ⭐⭐ **O tema que uma linha de menu MOSTRA** — o do seletor do topo, o de uma linha de secção, ou
/// (na linha *«o tema do app»*) o `do_app`. `None` = a linha não é de tema.
#[must_use]
pub fn tema_da_linha(id: NodeId, do_app: Theme) -> Option<Theme> {
    theme_of_menu_id(id).or_else(|| section_theme_of_menu_id(id).map(|t| t.unwrap_or(do_app)))
}

/// ⭐⭐ **As quatro cores que identificam um tema na amostra do menu** (ordem do dono, 2026-09-30:
/// *«em vez de uma única cor no retângulo do theme, melhor 4 retângulos pequenos com as principais
/// cores de cada theme»*) — painel · cartão · acento · texto, na ordem de leitura de um painel.
///
/// ⚠️ **Saem do TEMA, não de uma tabela de hex:** a amostra de uma cor só era uma cópia escrita à
/// mão (e o Candy mudou de base na mesma semana — a cópia teve de ser emendada nos dois menus). O
/// cartão sai da porta que o PINTA ([`crate::widget::section_cards::CardDepth::fill`]), senão no
/// Black a amostra mostraria o cartão da cor do painel, que é o defeito que ela curou.
#[must_use]
pub fn cores_do_tema(tema: Theme) -> [ph2d_vector::Color; 4] {
    use crate::paint::resolve;
    use ph2d_tokens::ColorToken;
    [
        resolve(ColorToken::PanelBg, tema),
        crate::widget::section_cards::CardDepth::Section.fill(tema),
        resolve(ColorToken::Accent, tema),
        resolve(ColorToken::Text1, tema),
    ]
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
    if apply_custom_theme_click(store, id) {
        return true;
    }
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

/// ⭐⭐ **Os três verbos do TEMA CUSTOM** — *Reset All Card Themes*, *Save Custom Theme* e *Load
/// Custom Theme* (ordem do dono, 2026-09-30). Agem sobre o PAINEL inteiro, logo o pedido do menu só
/// é consumido (fecha-o) e a secção dele não é lida. Devolve se `id` era um deles.
fn apply_custom_theme_click(store: &mut crate::interaction::WidgetStore, id: NodeId) -> bool {
    let verbo: fn(&mut crate::interaction::WidgetStore) =
        if id == ids::CTX_MENU_SECTION_THEMES_RESET {
            crate::interaction::WidgetStore::reset_section_themes
        } else if id == ids::CTX_MENU_SECTION_THEMES_SAVE_CUSTOM {
            crate::interaction::WidgetStore::save_custom_section_themes
        } else if id == ids::CTX_MENU_SECTION_THEMES_LOAD_CUSTOM {
            |s| {
                let _ = s.load_custom_section_themes();
            }
        } else {
            return false;
        };
    let _ = store.consume_last_context_menu();
    verbo(store);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Os doze temas têm linha, e `theme_of_menu_id` é o inverso exacto de `theme_menu_id`.
    #[test]
    fn every_theme_has_one_row_and_the_map_round_trips() {
        for theme in Theme::ALL {
            let id = theme_menu_id(theme);
            assert_eq!(theme_of_menu_id(id), Some(theme));
        }
        assert_eq!(THEME_MENU.len(), Theme::ALL.len());
        assert_eq!(theme_of_menu_id(ids::TOOL_UNDO), None);
        // ⭐ e o menu de SECÇÃO cobre os doze mais o do app, sem partilhar um id com o do topo
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

    /// ⭐⭐ **Toda linha de tema mostra o tema DELA, e a «App Theme» mostra o do app.**
    /// *Mutação: o braço do `section_theme_of_menu_id` a devolver o do app sempre ⇒ as linhas de
    /// secção pintam todas as mesmas quatro cores.*
    #[test]
    fn cada_linha_de_tema_mostra_o_proprio_tema() {
        for (id, t) in THEME_MENU {
            assert_eq!(tema_da_linha(id, Theme::Dark), Some(t));
        }
        for (id, t) in SECTION_THEME_MENU {
            assert_eq!(
                tema_da_linha(id, Theme::Candy),
                Some(t.unwrap_or(Theme::Candy))
            );
        }
        assert_eq!(tema_da_linha(ids::TOOL_UNDO, Theme::Dark), None);
    }

    /// ⭐⭐ **As quatro cores de um tema são QUATRO** — em todo tema moderno o painel, o cartão, o
    /// acento e o texto são cores diferentes, o que é a razão de a amostra ter quatro ladrilhos.
    /// ⚠️ O Black é o caso que pede o cartão da PORTA do cartão: pelo `Bg1` ele seria o preto do
    /// painel. *Mutação: o 2.º ladrilho a ler `resolve(Bg1, ..)` ⇒ o Black repete o painel.*
    #[test]
    fn as_quatro_cores_de_um_tema_sao_quatro() {
        for tema in Theme::MODERN {
            let c = cores_do_tema(tema);
            for i in 0..4 {
                for j in (i + 1)..4 {
                    assert_ne!(
                        c[i].to_rgba8(),
                        c[j].to_rgba8(),
                        "{tema:?}: os ladrilhos {i} e {j} da amostra são a mesma cor"
                    );
                }
            }
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

    /// ⭐⭐ **As três linhas do TEMA CUSTOM chegam à lei pelo MESMO despacho do clique** — gravar,
    /// limpar tudo e voltar a pôr, em duas secções ao mesmo tempo. *Mutação: trocar o braço do
    /// `Reset` pelo do `Save` ⇒ as secções não voltam ao app.*
    #[test]
    fn as_linhas_do_tema_custom_chegam_a_lei() {
        use crate::interaction::{WidgetEvent, WidgetStore};
        let (a, b) = (
            ids::INSP_LIVE_TRANSFORM_SECTION,
            ids::INSP_LIVE_VISIBILITY_SECTION,
        );
        let mut store = WidgetStore::with_capacity(8);
        store.set_section_theme(a, Some(Theme::Oled));
        store.set_section_theme(b, Some(Theme::Sunset));
        let clica = |store: &mut WidgetStore, id| {
            assert!(apply_section_theme_click(store, WidgetEvent::Click(id)));
        };
        clica(&mut store, ids::CTX_MENU_SECTION_THEMES_SAVE_CUSTOM);
        clica(&mut store, ids::CTX_MENU_SECTION_THEMES_RESET);
        assert_eq!(store.section_theme(a), None);
        assert_eq!(store.section_theme(b), None);
        clica(&mut store, ids::CTX_MENU_SECTION_THEMES_LOAD_CUSTOM);
        assert_eq!(store.section_theme(a), Some(Theme::Oled));
        assert_eq!(store.section_theme(b), Some(Theme::Sunset));
    }

    /// ⭐⭐ **Load sem nada gravado: o clique é engolido, nada muda e o menu fica ABERTO** — o
    /// comportamento de uma linha apagada. ⚠️ O despacho não precisa de guarda própria: o verbo já
    /// recusa sem gravação e o clique nunca fecha o menu VIVO (quem o fecha é o Down, e o Down numa
    /// linha indisponível não o fecha — gate `the_down_on_an_unavailable_row_keeps_the_menu_open`).
    /// Uma guarda aqui foi escrita, medida por mutação e APAGADA: nenhum gesto a observava.
    #[test]
    fn load_sem_gravacao_nao_age_nem_fecha_o_menu() {
        use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent, WidgetStore};
        let seccao = ids::INSP_LIVE_TRANSFORM_SECTION;
        let mut store = WidgetStore::with_capacity(8);
        store.set_section_theme(seccao, Some(Theme::Candy));
        store.open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::SectionOutline { section: seccao },
        });
        assert!(apply_section_theme_click(
            &mut store,
            WidgetEvent::Click(ids::CTX_MENU_SECTION_THEMES_LOAD_CUSTOM)
        ));
        assert!(store.context_menu().is_some(), "o menu continua aberto");
        assert_eq!(
            store.section_theme(seccao),
            Some(Theme::Candy),
            "nada mudou"
        );
    }
}
