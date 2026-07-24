//! **A porta única do motion path**, no nível do DOCUMENTO ([ADR-0141] §2).
//!
//! [`crate::MotionPath::edit`] mantém a geometria e a tabela de arco em acordo. Mas a
//! distância de cada âncora está guardada num **segundo lugar** — o *valor* da key
//! correspondente na track —, e mover uma âncora move a distância de todas as
//! seguintes. Fechar só o primeiro lugar deixa o sistema **estável e errado**: a curva
//! nova na tela, e o objeto ainda percorrendo os números da curva velha.
//!
//! Então quem move uma âncora move as duas coisas, aqui, numa chamada. Duas portas
//! divergem — é a lição que este módulo já pagou três vezes noutro eixo
//! ([[feedback_derived_coordinate_seed_must_match_sample]]).
//!
//! Split de `doc.rs` sob o teto de 700 LOC, e uma unidade por direito próprio: é o
//! único lugar do documento que sabe que uma track pode ter geometria do lado.
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use ph2d_anim::{AnimTarget, AnimValue};

use crate::doc::TimelineDoc;
use crate::path::PathAnchor;
use crate::prop::PropKind;

impl TimelineDoc {
    /// **Move (ou remodela) a âncora `i` da trajetória de `target`, e reescreve as
    /// distâncias que as keys guardam** — as duas metades, numa operação.
    ///
    /// Devolve `false` se `target` não nomeia um binding Position com caminho, ou se
    /// não há âncora `i`; nesse caso nada é tocado.
    ///
    /// ⚠️ **Os TEMPOS das keys não se movem.** Arrastar uma âncora é uma edição
    /// ESPACIAL: o objeto passa a fazer um caminho diferente *no mesmo tempo*, que é o
    /// que se espera de puxar uma curva na tela. Quem quer mudar o *quando* arrasta a
    /// key no dope-sheet, e esse é o outro gesto.
    ///
    /// ⚠️ **E o pareamento é por ÍNDICE.** Âncora `i` é key `i`, na ordem em que a
    /// track as guarda (ordenada por tempo). Uma track com menos keys do que o caminho
    /// tem âncoras é escrita até onde alcança — o resto do caminho fica lá, e a
    /// autoria (Fatia 4) é quem mantém os dois do mesmo tamanho.
    pub fn move_path_anchor(&mut self, target: AnimTarget, i: usize, to: PathAnchor) -> bool {
        let Some(b) = self.bindings_mut().iter_mut().find(|b| b.target == target) else {
            return false;
        };
        if b.prop != PropKind::Position {
            return false;
        }
        let Some(path) = b.path.as_mut() else {
            return false;
        };
        if !path.set_anchor(i, to) {
            return false;
        }
        // As distâncias novas, LIDAS do caminho que acabou de ser reconstruído — nunca
        // recalculadas aqui. Uma segunda aritmética é uma segunda resposta.
        let lens: Vec<f64> = (0..path.len()).filter_map(|k| path.arclen_at(k)).collect();
        let Some(track) = self.active_clip_mut().track_mut(target) else {
            return true; // caminho sem track ainda: a geometria vale, não há o que rever
        };
        let ids: Vec<_> = track.ids().to_vec();
        for (id, s) in ids.into_iter().zip(lens) {
            track.set_value(id, AnimValue::Float(s as f32));
        }
        // Uma key roving tem o TEMPO derivado do valor; os valores acabaram de mudar.
        track.resolve_roving();
        true
    }

    /// A âncora `i` da trajetória de `target`, se houver.
    #[must_use]
    pub fn path_anchor(&self, target: AnimTarget, i: usize) -> Option<PathAnchor> {
        self.binding(target)?
            .path
            .as_ref()?
            .anchors()
            .get(i)
            .copied()
    }
}

#[cfg(test)]
#[path = "doc_path_tests.rs"]
mod tests;
