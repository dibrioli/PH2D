//! ⭐ **Durante o Preview o painel MOSTRA e não EDITA** (F19 — decisão do dono, 18/10, opção A do
//! plano `docs/UI_New_and_Simple/vetor_nova_ui/30_plano_F19_o_painel_le_a_forma.md` §5.1).
//!
//! O Preview já barra o gesto do canvas e o undo, e ao sair RESTAURA as poses capturadas
//! (transform, opacidade, tinta, traço, forma, filtros). Uma edição feita no painel enquanto ele
//! corre perdia-se em silêncio ao sair — o «laranja» do Blender sem o aviso. O modo de protótipo
//! do Figma é a lei: o painel continua a mostrar os valores vivos, a edição fica fechada, e a
//! porta de saída (o botão *Preview*) continua aberta.
//!
//! A porta é o `Down`: sem ele não há foco, arrasto nem `SetValue`, e uma única pergunta cobre
//! os dez escritores que um painel tem. Os gestos de VISTA passam.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::WidgetStore;

/// Os painéis cuja edição o Preview perderia ao sair: o do vetor, o Inspector (Transform) e o
/// seletor de cor flutuante (o read-back dele restila a seleção).
const TRANCADOS: [NodeId; 3] = [
    ph2d_editor_core::ids::VECTOR_PANEL,
    ph2d_editor_core::ids::INSP_PANEL,
    ph2d_editor_core::ids::INSP_BLENDER_PICKER,
];

/// **Este `Down`, com o Preview ligado, é recusado?** `painel` e `alvo` são o painel e o widget
/// sob o ponto (`HeroScreen::chrome_panel_at` / `chrome_hit`).
///
/// Passam: fora dos painéis trancados, o corpo vazio, o botão *Preview* (a saída), as abas
/// Drawing/Interface, as abas do encaixe e os cabeçalhos de secção (dobrar é vista).
#[must_use]
pub fn recusa(store: &WidgetStore, painel: Option<NodeId>, alvo: Option<NodeId>) -> bool {
    let (Some(p), Some(a)) = (painel, alvo) else {
        return false;
    };
    TRANCADOS.contains(&p)
        && a != ph2d_panel_vector::ids::VECTOR_STATE_PREVIEW
        && a != ph2d_panel_vector::ids::VECTOR_TAB_DRAWING
        && a != ph2d_panel_vector::ids::VECTOR_TAB_INTERFACE
        && ph2d_editor_core::screens::hero::slot_tabs::panel_for_tab(a).is_none()
        && !store.is_collapsible_section(a)
}

#[cfg(test)]
mod tests {
    use super::recusa;
    use ph2d_editor_core::ids::{INSP_BLENDER_PICKER, INSP_PANEL, VECTOR_PANEL};
    use ph2d_editor_core::interaction::WidgetStore;
    use ph2d_panel_vector::ids as v;

    #[test]
    fn os_controlos_de_edicao_dos_tres_paineis_sao_recusados() {
        let s = WidgetStore::default();
        for p in [VECTOR_PANEL, INSP_PANEL, INSP_BLENDER_PICKER] {
            for a in [
                ph2d_tool_vector::ids::VECTOR_WIDTH,
                ph2d_tool_vector::ids::VECTOR_FILL_SWATCH,
                ph2d_tool_vector::ids::VECTOR_STROKE_OPACITY,
            ] {
                assert!(
                    recusa(&s, Some(p), Some(a)),
                    "{a:?} sobre {p:?} editaria no Preview"
                );
            }
        }
    }

    #[test]
    fn a_saida_e_a_vista_passam() {
        // ⚠️ A aba do encaixe (`panel_for_tab`) resolve-se pelo registo de painéis, que este binário
        // não instala (é do processo): o gate dela é o da shell, `a_aba_do_encaixe_passa_no_preview`.
        let mut s = WidgetStore::default();
        s.mark_collapsible_section(ph2d_tool_vector::ids::VECTOR_SECTION_FILL);
        for a in [
            v::VECTOR_STATE_PREVIEW,
            v::VECTOR_TAB_DRAWING,
            v::VECTOR_TAB_INTERFACE,
            ph2d_tool_vector::ids::VECTOR_SECTION_FILL,
        ] {
            assert!(
                !recusa(&s, Some(VECTOR_PANEL), Some(a)),
                "{a:?} é vista e foi recusado"
            );
        }
        assert!(
            !recusa(&s, Some(VECTOR_PANEL), None),
            "o corpo vazio do painel foi recusado"
        );
    }

    #[test]
    fn fora_dos_paineis_trancados_tudo_passa() {
        let s = WidgetStore::default();
        let a = Some(ph2d_tool_vector::ids::VECTOR_WIDTH);
        assert!(!recusa(&s, None, a));
        assert!(!recusa(&s, Some(ph2d_editor_core::NodeId(7)), a));
    }
}
