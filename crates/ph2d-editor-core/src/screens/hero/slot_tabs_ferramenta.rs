//! ⭐⭐ **A ABA DIZ QUE FERRAMENTA SE QUER** — irmão do [`super::slot_tabs`], pela quarta pergunta
//! da fila: *o que se vê*, *o que uma aba diz*, *como se chega ao que não se vê*, e **o que ESCOLHER
//! uma aba significa**.
//!
//! Report do dono (29/09): *«não consigo voltar para o modo sculpt (faça voltar ao abrir/selecionar
//! o painel sculpt e vice-versa)»*. O painel da escultura e o do Painter partilham o encaixe como
//! abas, e clicar numa só a levantava — a ferramenta em mãos ficava a mesma, logo a aba da escultura
//! à frente com o Painter na mão continuava a pintar.
//!
//! ⚠️ **A regra lê o CLIQUE e não «quem está à frente»:** a aba da frente muda sozinha quando um
//! painel abre ou fecha, e tomar isso por intenção trocava a ferramenta sem o artista pedir.
//!
//! ⭐ **Zero variantes novas no barramento** (o tecto medido do `action_bus.rs`): as duas intenções
//! já existem — `CancelActiveTool` devolve a mão à ferramenta de omissão, que é a da escultura, e
//! `ActivateTool` pega no Painter pelo mesmo portão do pill `PNTR` (o modo IMG ligado).

use crate::action_bus::EditorAction;
use ph2d_a11y::NodeId;

/// O id canónico do Painter — o mesmo que o pill `PNTR` pede.
const PAINTER: &str = "painter";

/// ⭐ **Clicar na aba `panel` pede que ferramenta?** — `None` quando a aba não fala de ferramenta ou
/// quando a mão já tem o que ela pede.
///
/// - a aba da **escultura** com uma ferramenta de imagem na mão ⇒ larga-a (`CancelActiveTool`).
/// - a aba do **Painter** com o modo IMG ligado e outra coisa na mão ⇒ pega nele (`ActivateTool`).
///
/// ⛔ **O Painter já na mão NÃO pede nada**, e é load-bearing: o `ActivateTool` de uma ferramenta
/// já activa ALTERNA (o pill volta à de omissão), logo clicar na aba do Painter com ele na mão
/// largava-o — o contrário do que o clique diz.
#[must_use]
pub fn intencao_da_aba(
    panel: NodeId,
    ferramenta_de_imagem: Option<&str>,
    modo_img: bool,
) -> Option<EditorAction> {
    if panel == crate::ids::SCULPT3D_PANEL && ferramenta_de_imagem.is_some() {
        return Some(EditorAction::CancelActiveTool);
    }
    if panel == crate::ids::PAINTER_LAYERS_PANEL
        && modo_img
        && ferramenta_de_imagem != Some(PAINTER)
    {
        return Some(EditorAction::ActivateTool { tool_id: PAINTER });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{PAINTER_LAYERS_PANEL, SCULPT3D_PANEL};

    /// ⭐⭐ **GATE — as duas abas trocam a ferramenta, e nenhuma pede o que a mão já tem.** Report
    /// do dono (29/09). ⚠️ O CONTROLO é uma aba qualquer: ela nunca pede ferramenta nenhuma.
    #[test]
    fn a_aba_escolhida_diz_que_ferramenta_se_quer() {
        assert_eq!(
            intencao_da_aba(SCULPT3D_PANEL, Some(PAINTER), true),
            Some(EditorAction::CancelActiveTool),
            "a aba da escultura com o Painter na mão não volta à escultura (o report do dono)"
        );
        assert_eq!(
            intencao_da_aba(SCULPT3D_PANEL, None, true),
            None,
            "a aba da escultura sem ferramenta de imagem pediu uma troca que não há"
        );
        assert_eq!(
            intencao_da_aba(PAINTER_LAYERS_PANEL, None, true),
            Some(EditorAction::ActivateTool { tool_id: PAINTER }),
            "a aba do Painter não pega nele (o «vice-versa» do dono)"
        );
        assert_eq!(
            intencao_da_aba(PAINTER_LAYERS_PANEL, Some(PAINTER), true),
            None,
            "a aba do Painter com ele na mão pede-o outra vez — e o pedido ALTERNA, largando-o"
        );
        assert_eq!(
            intencao_da_aba(PAINTER_LAYERS_PANEL, None, false),
            None,
            "com o modo IMG desligado o portão recusa o Painter: pedi-lo seria um clique sem efeito"
        );
        assert_eq!(
            intencao_da_aba(crate::ids::HIER_PANEL, Some(PAINTER), true),
            None,
            "o CONTROLO: uma aba que não fala de ferramenta trocou a ferramenta"
        );
    }
}
