//! **De quem são os campos de parâmetro que o painel desenha** — e quando eles não são de
//! ninguém. Módulo irmão de `state` (que hospeda os thread-locals; o teto de 600 LOC por
//! arquivo de painel está a poucas linhas dele).
//!
//! # O bug que este módulo existe para matar
//!
//! A regra era `published.unwrap_or(active)`: sem forma VIVA em foco, os campos caíam na
//! forma ATIVA do catálogo. Isso está certo para UM caso — nada selecionado, mostre o
//! default do próximo traço — e errado para todos os outros. Selecionar um **conector** (ou
//! uma curva comum, já convertida — é a mesma classe de bug, não um caso) deixava o painel
//! exibindo "Star: Points 5": os parâmetros da última forma do catálogo, pendurados sobre um
//! objeto que não tem parâmetro nenhum. A UI mentia sobre o que estava selecionado.
//!
//! # As três respostas, nesta ordem
//!
//! 1. **Há forma VIVA em foco** ⇒ os campos são dela (e a editam, mesmo na ferramenta
//!    Select — é o ciclo Live Shape).
//! 2. **A seleção está VAZIA** ⇒ os campos são os da forma ATIVA do catálogo: o default do
//!    **próximo** traço, que é a única coisa que eles poderiam significar sem alvo.
//! 3. **Senão** ⇒ a seção **SOME**. Foi selecionado algo que não é forma viva, e um campo
//!    editável que não edita nada é pior que campo nenhum.
//!
//! ## A exceção que a regra precisa: o modo Shape
//!
//! No modo **Shape** o usuário está armado para DESENHAR — os campos são o default do
//! próximo traço, exatamente como no caso 2, e valem mesmo com algo selecionado (armar a
//! estrela com um conector ainda selecionado é um gesto legítimo: escolher a forma no
//! catálogo *já* põe a tool em Shape, sem limpar a seleção). Sem esta exceção, a correção
//! do bug tiraria do usuário o jeito de configurar a forma que ele acabou de escolher.

use crate::state;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::DrawMode;
use ph2d_vec_scene::ShapeKind;

/// A forma cujos parâmetros o painel desenha — `None` ⇒ **a seção não existe**.
///
/// Função pura (os quatro fatos entram como argumento) para o gate poder varrer a tabela
/// inteira de combinações sem montar um painel.
#[must_use]
pub(crate) fn shape_focus(
    published: Option<ShapeKind>,
    active: ShapeKind,
    selection: usize,
    drawing_shape: bool,
) -> Option<ShapeKind> {
    published.or_else(|| (selection == 0 || drawing_shape).then_some(active))
}

/// A forma em foco NESTE frame, a partir do que a shell publicou (forma viva + contagem da
/// seleção) e do que a tool diz (forma ativa + modo). A porta única: o `paint` (que decide
/// desenhar) e o `event` (que decide ciclar uma escolha) leem daqui — se cada um resolvesse
/// o foco à sua maneira, um botão pintado por um seria recusado pelo outro.
#[must_use]
pub(crate) fn resolved(snap: &VectorStyleSnapshot) -> Option<ShapeKind> {
    shape_focus(
        state::current_shape_focus(),
        snap.shape,
        state::current_selection_count(),
        snap.mode == DrawMode::Shape,
    )
}

#[cfg(test)]
mod tests {
    use super::shape_focus;
    use ph2d_vec_scene::ShapeKind;

    /// O caso que define a feature: em Select (que não tem forma própria), a forma viva
    /// selecionada traz os campos dela — é assim que se edita um polígono já desenhado.
    #[test]
    fn a_selected_live_shape_shows_its_fields_even_in_select() {
        assert_eq!(
            shape_focus(Some(ShapeKind::Polygon), ShapeKind::Rectangle, 1, false),
            Some(ShapeKind::Polygon)
        );
    }

    /// Sem NADA selecionado, os campos são os da forma ATIVA do catálogo — o default do
    /// próximo traço.
    #[test]
    fn without_a_selection_the_fields_are_the_active_shapes() {
        assert_eq!(
            shape_focus(None, ShapeKind::Star, 0, false),
            Some(ShapeKind::Star)
        );
    }

    /// **O BUG (Enio):** selecionado algo que NÃO é forma viva (um conector, uma curva
    /// comum), a seção some. Antes ela caía no catálogo e pendurava "Star: Points 5" sobre
    /// um objeto que não tem parâmetro nenhum.
    #[test]
    fn a_selected_non_shape_hides_the_section_instead_of_falling_back_to_the_catalog() {
        assert_eq!(
            shape_focus(None, ShapeKind::Star, 1, false),
            None,
            "sem forma viva E com algo selecionado, os campos nao sao de ninguem"
        );
        // E não é um caso: vale para qualquer seleção não-vazia sem forma viva.
        assert_eq!(shape_focus(None, ShapeKind::Star, 7, false), None);
    }

    /// A exceção: no modo **Shape** o usuário está armado para desenhar, e os campos são o
    /// default do PRÓXIMO traço — mesmo com algo selecionado (escolher a forma no catálogo
    /// já põe a tool em Shape, sem limpar a seleção).
    #[test]
    fn arming_a_shape_shows_the_catalog_fields_even_with_something_selected() {
        assert_eq!(
            shape_focus(None, ShapeKind::Star, 1, true),
            Some(ShapeKind::Star)
        );
    }

    /// Conflito (catálogo em Polygon, estrela selecionada): a SELEÇÃO manda — o painel
    /// mostra o que está na tela, não o que a caneta faria a seguir. Inclusive no modo
    /// Shape: a forma viva em foco vence o catálogo.
    #[test]
    fn the_selection_wins_over_the_active_shape() {
        assert_eq!(
            shape_focus(Some(ShapeKind::Star), ShapeKind::Polygon, 1, false),
            Some(ShapeKind::Star)
        );
        assert_eq!(
            shape_focus(Some(ShapeKind::Star), ShapeKind::Polygon, 1, true),
            Some(ShapeKind::Star)
        );
    }
}
