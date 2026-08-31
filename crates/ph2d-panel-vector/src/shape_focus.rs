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
//! 2. **A ferramenta na mão DESENHA uma forma** (Forma ou Moldura) ⇒ os campos são os da forma
//!    ARMADA: o default do **próximo** traço.
//! 3. **Senão** ⇒ a seção **SOME**. Um campo editável que não edita nada é pior que campo nenhum.
//!
//! ⚠️⚠️ **A resposta 2 era *"a seleção está VAZIA"*, e isso pôs os campos em toda ferramenta.**
//! Report do Enio de 2026-08-31, com foto: na ferramenta **Select** e sem nada selecionado, o
//! painel oferecia *"ROUND / Radius"* — os parâmetros do próximo traço numa ferramenta que não
//! desenha traço nenhum. *A pergunta certa nunca foi quantos objetos estão selecionados; é se a
//! ferramenta na mão vai desenhar uma forma* — e a resposta disso é a mesma porta que a semente da
//! shell já usava (`DrawMode::shape_kind`).
//!
//! ## A exceção que a regra precisa: o modo Shape
//!
//! No modo **Shape** o usuário está armado para DESENHAR — os campos são o default do
//! próximo traço, exatamente como no caso 2, e valem mesmo com algo selecionado (armar a
//! estrela com um conector ainda selecionado é um gesto legítimo: escolher a forma no
//! catálogo *já* põe a tool em Shape, sem limpar a seleção). Sem esta exceção, a correção
//! do bug tiraria do usuário o jeito de configurar a forma que ele acabou de escolher.
//!
//! ⚠️⚠️ **A exceção estava escrita aqui e NÃO ERA ALCANÇÁVEL**, e é o report do Enio de
//! 2026-08-31 (*"troco de Shape na tool Shape e as propriedades não trocam imediatamente"*): o
//! `or_else` põe o `published` PRIMEIRO, então com uma forma viva selecionada — que é o estado
//! normal logo depois de desenhar — o `drawing_shape` nunca chegava a ser lido. Quem a torna real
//! é a shell, que no modo Shape **não publica foco vivo nenhum**
//! (`vec_shape_params::shape_field_target`). A lei mora lá, e não aqui, porque a **ESCRITA** da
//! caixa numérica precisa exactamente da mesma resposta — e ela vive no shell.

use crate::state;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_vec_scene::ShapeKind;

/// A forma cujos parâmetros o painel desenha — `None` ⇒ **a seção não existe**.
///
/// A forma VIVA em foco, ou a forma ARMADA, ou ninguém. Função pura (os dois fatos entram como
/// argumento) para o gate poder varrer a tabela inteira sem montar um painel.
#[must_use]
pub(crate) fn shape_focus(live: Option<ShapeKind>, armed: Option<ShapeKind>) -> Option<ShapeKind> {
    live.or(armed)
}

/// A forma em foco NESTE frame, a partir do que a shell publicou (forma viva + contagem da
/// seleção) e do que a tool diz (forma ativa + modo). A porta única: o `paint` (que decide
/// desenhar) e o `event` (que decide ciclar uma escolha) leem daqui — se cada um resolvesse
/// o foco à sua maneira, um botão pintado por um seria recusado pelo outro.
#[must_use]
pub(crate) fn resolved(snap: &VectorStyleSnapshot) -> Option<ShapeKind> {
    shape_focus(state::current_shape_focus(), armed_kind(snap))
}

/// **A forma que o GESTO desta ferramenta vai desenhar** — `None` quando ela não desenha forma
/// nenhuma, e é isso que faz a seção sumir fora da Forma e da Moldura.
///
/// ⚠️ **É a mesma porta que a semente da shell usa** (`DrawMode::shape_kind`, via
/// `vector_bridge::shape_catalog`), e tem de ser: sem alvo vivo, a shell escreve nas caixas os
/// VALORES do kind efectivo enquanto esta secção pintava os CAMPOS do botão aceso do catálogo. No
/// modo **Moldura** — o único em que os dois divergem — o painel oferecia *"Estrela: Pontas"* com o
/// raio de quina do `RoundRect` dentro, e nenhum gate os comparava porque cada metade estava certa
/// sozinha.
#[must_use]
fn armed_kind(snap: &VectorStyleSnapshot) -> Option<ShapeKind> {
    snap.mode.shape_kind(snap.shape)
}

#[cfg(test)]
mod tests {
    use super::shape_focus;
    use crate::section_scope::DRAWS_A_SHAPE;
    use ph2d_tool_vector::VectorStyleSnapshot;
    use ph2d_tool_vector::params::DrawMode;
    use ph2d_vec_scene::ShapeKind;

    /// O caso que define a feature: em Select (que não desenha forma nenhuma), a forma viva
    /// selecionada traz os campos dela — é assim que se edita um polígono já desenhado.
    #[test]
    fn a_selected_live_shape_shows_its_fields_even_in_select() {
        assert_eq!(
            shape_focus(Some(ShapeKind::Polygon), None),
            Some(ShapeKind::Polygon)
        );
    }

    /// Sem forma viva, os campos são os da forma ARMADA — o default do próximo traço.
    #[test]
    fn without_a_live_shape_the_fields_are_the_armed_ones() {
        assert_eq!(
            shape_focus(None, Some(ShapeKind::Star)),
            Some(ShapeKind::Star)
        );
    }

    /// **O BUG (Enio):** selecionado algo que NÃO é forma viva (um conector, uma curva
    /// comum), a seção some. Antes ela caía no catálogo e pendurava "Star: Points 5" sobre
    /// um objeto que não tem parâmetro nenhum.
    #[test]
    fn a_selected_non_shape_hides_the_section_instead_of_falling_back_to_the_catalog() {
        assert_eq!(shape_focus(None, None), None);
    }

    /// Conflito (armada a Estrela, um Polígono vivo em foco): a forma VIVA manda — o painel mostra
    /// o que está na tela.
    ///
    /// ⚠️⚠️ **E no modo Forma a shell nunca publica um foco vivo enquanto o artista está ARMADO**
    /// (`vec_shape_params::shape_field_target`): armar é dizer *"vou desenhar isto"*, e a caixa que
    /// ele digita move o default do próximo traço. A lei do latch mora na shell porque a ESCRITA
    /// também precisa dela, e a escrita não alcança um `pub(crate)` deste crate.
    #[test]
    fn the_live_shape_wins_over_the_armed_one() {
        assert_eq!(
            shape_focus(Some(ShapeKind::Star), Some(ShapeKind::Polygon)),
            Some(ShapeKind::Star)
        );
    }

    /// ⭐⭐ **O CENSO da resposta 2: quem ARMA uma forma é exactamente [`DRAWS_A_SHAPE`].**
    ///
    /// ⚠️ Sem isto, a tabela de escopo e a lei de foco poderiam discordar sobre quais ferramentas
    /// desenham uma forma — e o sintoma seria uma seção pintada com os campos de ninguém, ou uma
    /// escondida onde o gesto de facto arma. *Duas listas que respondem a mesma pergunta divergem
    /// no dia em que um modo novo entra numa e não na outra.*
    #[test]
    fn the_modes_that_arm_a_shape_are_exactly_the_ones_the_scope_table_names() {
        let todos = [
            DrawMode::Select,
            DrawMode::Node,
            DrawMode::Pen,
            DrawMode::Pencil,
            DrawMode::Shape,
            DrawMode::Text,
            DrawMode::Build,
            DrawMode::Connect,
            DrawMode::PickBlend,
            DrawMode::Fillet,
            DrawMode::Chamfer,
            DrawMode::Width,
            DrawMode::Cut,
            DrawMode::Frame,
        ];
        // A fixture CONTÉM o fenômeno: se a lista acima encolher, o censo deixa de varrer o que diz.
        assert_eq!(
            todos.len(),
            14,
            "o vocabulario de modos mudou — reveja a tabela"
        );
        for m in todos {
            let snap = VectorStyleSnapshot {
                mode: m,
                shape: ShapeKind::Star,
                ..VectorStyleSnapshot::default()
            };
            assert_eq!(
                super::armed_kind(&snap).is_some(),
                DRAWS_A_SHAPE.contains(&m),
                "o modo {m:?} discorda entre a lei de foco e a tabela de escopo"
            );
        }
    }
}
