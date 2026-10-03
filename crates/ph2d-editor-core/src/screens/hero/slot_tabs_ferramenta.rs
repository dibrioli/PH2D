//! ⭐⭐ **A ABA DIZ QUE MODO SE QUER** — irmão do [`super::slot_tabs`], pela quarta pergunta da fila:
//! *o que se vê*, *o que uma aba diz*, *como se chega ao que não se vê*, e **o que ESCOLHER uma aba
//! significa**.
//!
//! Report do dono (29/09): *«não consigo voltar para o modo sculpt (faça voltar ao abrir/selecionar
//! o painel sculpt e vice-versa)»*. O painel da escultura e o do Painter partilham o encaixe como
//! abas; escolher uma troca entre esculpir e pintar a peça.
//!
//! ⭐ **Desde a F3 do Sculpt (spec/06) a troca é um PEDIDO DE MODO** — Sculpt ▸ Sculpt ⇄ Sculpt ▸
//! Paint, pela porta única do quadro do modo —, e não uma ferramenta pedida à parte: dois caminhos
//! para o mesmo módulo divergem (spec/06 §5). O IMG deixou de ser porta.
//!
//! ⚠️ **A regra lê o CLIQUE e não «quem está à frente»:** a aba da frente muda sozinha quando um
//! painel abre ou fecha, e tomar isso por intenção trocava o modo sem o artista pedir.

use crate::action_bus::EditorAction;
use crate::object_mode::{ModeRequest, ModeState, ObjectMode};
use ph2d_a11y::NodeId;

/// ⭐ **Clicar na aba `panel` pede o quê?** — `None` quando a aba não fala de modo, ou quando o
/// objecto activo já está no que ela pede (ou não o tem).
///
/// - a aba da **escultura** em Paint ⇒ o Sculpt; em Sculpt com uma ferramenta de imagem na mão ⇒
///   larga-a (`CancelActiveTool`).
/// - a aba do **Painter** em Sculpt ⇒ o Paint.
#[must_use]
pub fn intencao_da_aba(
    panel: NodeId,
    modo: &ModeState,
    ferramenta_de_imagem: Option<&str>,
) -> Option<EditorAction> {
    let pede = |m: ObjectMode| {
        modo.available()
            .contains(&m)
            .then_some(EditorAction::ObjectMode(ModeRequest::Enter(m)))
    };
    match modo.current() {
        ObjectMode::Paint if panel == crate::ids::SCULPT3D_PANEL => pede(ObjectMode::Sculpt),
        ObjectMode::Sculpt if panel == crate::ids::PAINTER_LAYERS_PANEL => pede(ObjectMode::Paint),
        ObjectMode::Sculpt
            if panel == crate::ids::SCULPT3D_PANEL && ferramenta_de_imagem.is_some() =>
        {
            Some(EditorAction::CancelActiveTool)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{HIER_PANEL, PAINTER_LAYERS_PANEL, SCULPT3D_PANEL};

    const PECA: u64 = 7;
    const IMAGEM: u64 = 8;

    fn em(entidade: u64, modos: &[ObjectMode], modo: ObjectMode) -> ModeState {
        let mut s = ModeState::default();
        s.publish(Some(entidade), modos);
        if modo != ObjectMode::Object {
            s.enter(entidade, modo);
        }
        s
    }

    fn entra(m: ObjectMode) -> Option<EditorAction> {
        Some(EditorAction::ObjectMode(ModeRequest::Enter(m)))
    }

    /// ⭐⭐ **GATE — as duas abas trocam o MODO da peça, e nenhuma pede o que ela já tem.** Report
    /// do dono (29/09), pela porta do modo (spec/06 F3). ⚠️ O CONTROLO é uma aba qualquer: ela nunca
    /// pede nada.
    #[test]
    fn a_aba_escolhida_diz_que_modo_se_quer() {
        let peca = [ObjectMode::Sculpt, ObjectMode::Paint];
        let pintar = em(PECA, &peca, ObjectMode::Paint);
        let esculpir = em(PECA, &peca, ObjectMode::Sculpt);
        assert_eq!(
            intencao_da_aba(SCULPT3D_PANEL, &pintar, Some("painter")),
            entra(ObjectMode::Sculpt),
            "a aba da escultura em Paint não volta ao Sculpt (o report do dono)"
        );
        assert_eq!(
            intencao_da_aba(PAINTER_LAYERS_PANEL, &esculpir, None),
            entra(ObjectMode::Paint),
            "a aba do Painter em Sculpt não pinta a peça (o «vice-versa» do dono)"
        );
        assert_eq!(
            intencao_da_aba(PAINTER_LAYERS_PANEL, &pintar, Some("painter")),
            None
        );
        assert_eq!(intencao_da_aba(SCULPT3D_PANEL, &esculpir, None), None);
        assert_eq!(
            intencao_da_aba(SCULPT3D_PANEL, &esculpir, Some("bgremoval")),
            Some(EditorAction::CancelActiveTool),
            "a aba da escultura não larga a ferramenta de imagem que tapa o barro"
        );
        assert_eq!(
            intencao_da_aba(HIER_PANEL, &pintar, Some("painter")),
            None,
            "o CONTROLO: uma aba que não fala de modo pediu um"
        );
    }

    /// ⭐ **GATE — a aba da escultura sobre uma IMAGEM em Paint não pede um modo que ela não tem**:
    /// o pedido seria uma recusa com aviso por um clique numa aba.
    #[test]
    fn a_aba_nao_pede_um_modo_que_o_activo_nao_tem() {
        let imagem = em(IMAGEM, &[ObjectMode::Paint], ObjectMode::Paint);
        assert_eq!(intencao_da_aba(SCULPT3D_PANEL, &imagem, Some("painter")), None);
        let objecto = em(PECA, &[ObjectMode::Sculpt, ObjectMode::Paint], ObjectMode::Object);
        assert_eq!(intencao_da_aba(PAINTER_LAYERS_PANEL, &objecto, None), None);
    }
}
