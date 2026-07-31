//! **O que uma fórmula DEVE de volta** — para cada canal que uma expressão dirige, a pose
//! a devolver quando ela some.
//!
//! ⚠️ **Estado de display, nunca de documento.** Nada aqui é serializado, nada aqui pode
//! ser desfeito, e nada aqui escreve num [`crate::TimelineDoc`].
//!
//! ⚠️ **Este módulo já foi o `expr_live`, e a METADE de preview foi removida em
//! 2026-07-30** junto com o card de expressão (o único produtor dela). O que sobrou não é
//! resíduo: é a metade que a auditoria de 2026-07-29 (§4 D-I) provou ser a MAIOR, e que
//! tem gate próprio (`clearing_a_formula_hands_the_pose_back`) — ela é alcançável pela
//! porta de autoria (`TimelineIntent::SetBindingExpr`), que segue viva. O arquivo mudou de
//! nome porque um `expr_live` sem preview seria um nome que mente.

use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    /// **O LIVRO-RAZÃO da direção por fórmula** — para cada canal que uma fórmula dirige, a
    /// pose a devolver se ela parar: `target -> valor pré-expressão`.
    ///
    /// ⚠️ Isto existe por causa de um gate que estava CERTO e de um comentário meu que estava
    /// ERRADO. Eu escrevi que parar a fórmula é o undo inteiro — o passe keyado reescreve a
    /// propriedade a partir das curvas todo frame — e isso é verdade para um canal KEYADO e
    /// falso para o mais comum: uma binding **pelada** (sem keys, sem fórmula) é esparsa de
    /// propósito, o `solo_source_value` devolve `None` para ela justamente para que uma
    /// binding recém-criada nunca seja forçada a um default, e portanto **ninguém** a
    /// escreve. Medido: apagar a fórmula e o objeto ficava onde ela o tinha deixado.
    ///
    /// ⚠️ Medido (auditoria 2026-07-29, §4 D-I): uma binding pelada dirigida por
    /// `value + 250` ficava em **250.0000** depois de DELETE + Apply, e em todo frame
    /// seguinte. Isso é *"mesmo deletando as expressões, elas ficam atuando"*, literalmente.
    /// Por isso é um MAPA, preenchido por todo sítio que dirige uma fórmula (o post-pass
    /// global, o blend per-clip) e drenado pelo único lugar que sabe que ninguém respondeu
    /// por um canal.
    ///
    /// ⚠️ O valor é o **`value` pré-expressão do próprio driver**, atualizado a cada frame
    /// dirigido — não um snapshot tirado quando a fórmula foi instalada. Um snapshot seria
    /// uma segunda resposta a *o que esta propriedade é* (e ficaria velho no primeiro
    /// scrub); o `value` é o número que o driver já computa para alimentar a fórmula, então
    /// devolvê-lo é, por construção, *"o que ela teria sido"*.
    static OWED: RefCell<BTreeMap<u64, f32>> = const { RefCell::new(BTreeMap::new()) };
}

/// Lembra o que uma propriedade DIRIGIDA teria sido sem a fórmula dela, para poder ser
/// devolvida se a fórmula sumir. Chamado por todo driver, em todo frame dirigido
/// (ver [`OWED`]).
pub(crate) fn remember(target: u64, pre_expression_value: f32) {
    OWED.with(|c| {
        c.borrow_mut().insert(target, pre_expression_value);
    });
}

/// **Toda pose devida**, e tomá-la a limpa.
///
/// `still_driven` responde *"uma fórmula está dirigindo este canal agora?"* — um canal cuja
/// fórmula segue instalada não deve nada, e devolver a pose dele brigaria com o driver. O
/// chamador fornece o predicado porque é ele quem conhece o frame: as fórmulas do documento
/// e — criticamente — se qualquer OUTRA coisa escreveu a propriedade neste frame.
///
/// A drenagem é exatamente-uma-vez por entrada, então uma mudança autorada posterior nunca é
/// atropelada por uma devolução velha.
pub(crate) fn drain_owed(still_driven: &dyn Fn(u64) -> bool) -> Vec<(u64, f32)> {
    OWED.with(|c| {
        let mut owed = c.borrow_mut();
        let handing: Vec<(u64, f32)> = owed
            .iter()
            .filter(|(t, _)| !still_driven(**t))
            .map(|(t, v)| (*t, *v))
            .collect();
        for (t, _) in &handing {
            owed.remove(t);
        }
        handing
    })
}

/// Se há alguma pose devida — o apply pergunta, para que o frame que a devolve não seja
/// pulado pelo caminho rápido sem-fórmula (`frame_solve::any_formula`).
///
/// ⚠️ Verdadeiro enquanto uma fórmula ainda dirige, também, e isso é deliberado: manter o
/// passe agendado é o que torna o `composed` disponível no frame em que a devolução precisa
/// dele, e um documento com fórmula já ia por esse caminho de qualquer jeito.
#[must_use]
pub fn has_pending_restore() -> bool {
    OWED.with(|c| !c.borrow().is_empty())
}

/// Esquece toda pose devida — para um host instalando outro documento.
///
/// ⚠️ Sem isto, carregar o projeto B entregaria as poses do projeto A a quaisquer bindings
/// que reusassem aqueles targets. O load já esquece o relógio, a fila de undo e a timeline
/// exatamente por isso.
pub fn forget_owed_poses() {
    OWED.with(|c| c.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Uma pose é devolvida só uma vez, e só depois de o driver ter ido embora.**
    ///
    /// ⚠️ A pergunta *"ainda está dirigido?"* é do CHAMADOR — o livro-razão não pode
    /// respondê-la, porque uma fórmula pode dirigir de dois lugares (o canal global, qualquer
    /// clip) e só o passe vê os dois. Então o predicado é passado, e este teste faz o papel
    /// do chamador.
    #[test]
    fn the_pose_is_owed_back_once_and_only_after_the_driver_goes_away() {
        forget_owed_poses();
        remember(3, 7.5);
        assert!(has_pending_restore(), "the ledger has a note");
        assert!(
            drain_owed(&|t| t == 3).is_empty(),
            "nothing is handed back while that channel is still driven"
        );
        assert!(
            has_pending_restore(),
            "and a refused drain must not consume the note"
        );

        assert_eq!(drain_owed(&|_| false), vec![(3, 7.5)]);
        assert!(
            drain_owed(&|_| false).is_empty() && !has_pending_restore(),
            "and it is handed back exactly ONCE — a stale restore would clobber a later edit"
        );
    }

    /// **Vários canais podem dever ao mesmo tempo**, o que a versão de um slot só não sabia
    /// representar: duas linhas limpas num gesto são um Apply.
    #[test]
    fn every_channel_that_owes_is_handed_back_and_the_driven_ones_are_left_alone() {
        forget_owed_poses();
        remember(1, 10.0);
        remember(2, 20.0);
        remember(3, 30.0);
        let handed = drain_owed(&|t| t == 2);
        assert_eq!(handed, vec![(1, 10.0), (3, 30.0)]);
        assert!(
            has_pending_restore(),
            "channel 2 is still driven, so its note stays"
        );
        forget_owed_poses();
        assert!(
            !has_pending_restore(),
            "and a host can drop the whole ledger"
        );
    }
}
