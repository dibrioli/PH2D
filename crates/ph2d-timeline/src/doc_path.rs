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

use ph2d_anim::{AnimTarget, AnimValue, RationalTime};

use crate::doc::TimelineDoc;
use crate::path::PathAnchor;
use crate::prop::PropKind;

/// O estado RESOLVIDO do auto-orient de uma entidade — o que o apply honra e o que o
/// painel mostra, da mesma função.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoOrient {
    /// Não pedido (ou não há trajetória).
    Off,
    /// Pedido e honrado: o objeto gira para a tangente do caminho.
    Active,
    /// **Pedido e RECUSADO** porque a entidade tem uma track de Rotation. A recusa é
    /// nomeada de propósito: um toggle que fica ligado sem efeito é pior que um
    /// desligado, e o artista precisa saber o que apagar para o ter.
    BlockedByRotationTrack,
}

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
        // As distâncias novas, pela porta compartilhada — LIDAS do caminho que acabou de
        // ser reconstruído, nunca recalculadas aqui. Uma segunda aritmética é uma
        // segunda resposta, e o sintoma é o objeto a andar os números de outra curva.
        self.rewrite_path_key_values(target)
    }

    /// **O K do modo Path: acrescentar uma âncora onde o objeto está** ([ADR-0141] §3).
    ///
    /// Isto é o que "capturar a pose" significa numa trajetória, e é por isso que o
    /// `sample_prop_value` do shell **recusa** Position: capturar não é ler um escalar,
    /// é uma edição da GEOMETRIA — a âncora nova muda o percurso, e o percurso é o que
    /// as outras keys medem. Uma única porta faz as quatro coisas:
    ///
    /// 1. a âncora entra na posição que o TEMPO dela manda (âncora `i` é key `i`);
    /// 2. os vizinhos ainda `auto` re-suavizam (uma alça Auto Bezier é função dos
    ///    vizinhos, e o vizinho acabou de mudar);
    /// 3. as distâncias são relidas do caminho reconstruído;
    /// 4. **TODAS as keys** recebem a distância nova — inclusive as anteriores, se o
    ///    re-suavizar mexeu na curva antes do ponto novo.
    ///
    /// Re-keyar um instante que já tem key **move** aquela âncora em vez de empilhar
    /// uma segunda (o contrato do [`crate::TimelineDoc::upsert_key`]). Devolve `false`
    /// se `target` não é um binding Position.
    ///
    /// ⚠️ **A primeira âncora nasce sem percurso**, e isso é correto: um caminho de um
    /// ponto tem comprimento zero, e a track fica com uma key de valor `0`. O objeto
    /// não se move até haver a segunda — que é exatamente o que uma animação de uma
    /// key faz em qualquer canal.
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    pub fn add_path_key(&mut self, target: AnimTarget, t: RationalTime, at: [f32; 2]) -> bool {
        let Some(b) = self.bindings().iter().find(|b| b.target == target) else {
            return false;
        };
        if b.prop != PropKind::Position {
            return false;
        }
        // Onde a âncora entra: a contagem de keys ESTRITAMENTE antes de `t`. Num
        // instante já keyado isso dá o índice daquela key, e o passo abaixo a
        // sobrescreve em vez de duplicar — a mesma semântica do `upsert_key`.
        let (i, existing) = match self.active_clip().track(target) {
            Some(tr) => {
                let ks = tr.keys();
                (
                    ks.iter().filter(|k| k.t < t).count(),
                    ks.iter().any(|k| k.t == t),
                )
            }
            None => (0, false),
        };
        // A key primeiro (o valor certo é escrito no fim, quando a geometria existe).
        self.upsert_key(
            b.entity,
            PropKind::Position,
            t,
            AnimValue::Float(0.0),
            crate::Interp::Linear,
        );
        let Some(b) = self.bindings_mut().iter_mut().find(|b| b.target == target) else {
            return false;
        };
        let path = b
            .path
            .get_or_insert_with(|| crate::MotionPath::new(Vec::new()));
        let fresh = crate::MotionPath::auto_smooth(None, at, None);
        if existing {
            path.set_anchor(i, fresh);
        } else {
            path.insert_anchor(i, fresh);
        }
        // Os vizinhos `auto` (e a nova) ganham as alças que a vizinhança de AGORA pede.
        path.resmooth_auto();
        self.rewrite_path_key_values(target)
    }

    /// Reescreve o valor de TODA key com a distância que o caminho diz hoje.
    ///
    /// Extraída porque há dois autores de geometria ([`Self::move_path_anchor`] e
    /// [`Self::add_path_key`]) e **uma** resposta para "que número a key guarda". Duas
    /// cópias divergem, e o sintoma é o objeto a andar os números de uma curva que já
    /// não existe.
    fn rewrite_path_key_values(&mut self, target: AnimTarget) -> bool {
        let Some(b) = self.bindings().iter().find(|b| b.target == target) else {
            return false;
        };
        let Some(path) = b.path.as_ref() else {
            return false;
        };
        let lens: Vec<f64> = (0..path.len()).filter_map(|k| path.arclen_at(k)).collect();
        let Some(track) = self.active_clip_mut().track_mut(target) else {
            return true;
        };
        for (id, s) in track.ids().to_vec().into_iter().zip(lens) {
            track.set_value(id, AnimValue::Float(s as f32));
        }
        track.resolve_roving();
        true
    }

    /// [`Self::add_path_key`] por ENTIDADE, criando o binding Position se ainda não
    /// houver — a forma que o intent do K usa, e a única que o shell precisa conhecer.
    pub fn key_the_path(&mut self, entity: u64, t: RationalTime, at: [f32; 2]) -> bool {
        let target = self.bind(entity, PropKind::Position);
        self.add_path_key(target, t, at)
    }

    /// **O auto-orient desta entidade vale, e se não vale, por quê** ([ADR-0141] §6).
    ///
    /// UMA porta com DOIS consumidores: o apply pergunta para saber se escreve o
    /// ângulo, e o painel pergunta para saber o que MOSTRAR. Duas respostas divergem, e
    /// aí o toggle diz "ligado" enquanto nada gira — o modo de falha que faz o artista
    /// desconfiar da ferramenta inteira.
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    #[must_use]
    pub fn auto_orient(&self, entity: u64) -> AutoOrient {
        let Some(b) = self.binding_for(entity, PropKind::Position) else {
            return AutoOrient::Off;
        };
        if !b.auto_orient {
            return AutoOrient::Off;
        }
        // ⚠️ A RECUSA. Girar para a tangente escreve `Transform.rotation`, que é
        // exatamente o que uma track de Rotation escreve — dois autores do mesmo campo,
        // e o de trás vence em silêncio. O apply resolveria isso por ORDEM, que é a
        // pior maneira: funcionaria, ninguém saberia por quê, e inverter a ordem um dia
        // mudaria a animação de alguém sem nada no diff.
        if self.binding_for(entity, PropKind::Rotation).is_some() {
            return AutoOrient::BlockedByRotationTrack;
        }
        AutoOrient::Active
    }

    /// Liga/desliga o auto-orient do binding Position de `entity`. Devolve o estado
    /// RESOLVIDO (que pode ser a recusa), nunca só o que foi escrito.
    pub fn set_auto_orient(&mut self, entity: u64, on: bool) -> AutoOrient {
        if let Some(b) = self
            .bindings_mut()
            .iter_mut()
            .find(|b| b.entity == entity && b.prop == PropKind::Position)
        {
            b.auto_orient = on;
        }
        self.auto_orient(entity)
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
