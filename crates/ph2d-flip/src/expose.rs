//! **Exposição** — quantos quadros uma chave fica na tela (W3.T3.1).
//!
//! É a operação central da tira: "segura esse desenho por mais 2 quadros". A
//! semântica é a da tira de exposição (TVPaint/Harmony): **esticar EMPURRA** o que
//! vem depois; encolher puxa. O tempo é relativo — o animador não quer que mexer no
//! quadro 3 apague o 4.
//!
//! Na ÚLTIMA chave não há o que empurrar (o hold implícito é infinito), então a
//! exposição vira uma **sentinela de fim**: é ela que fecha o vão da camada. O que
//! aparece DEPOIS do vão é decisão do [`crate::LayerCycle`] — no default (`Hold`) o
//! último desenho continua na tela, então fixar a exposição da última chave não
//! apaga nada; só define onde o ciclo recomeça.

use crate::ids::{Frame, LayerId};
use crate::object::FlipObject;

impl FlipObject {
    /// A chave `key` da camada passa a expor `frames` quadros (`≥ 1`). Devolve
    /// `true` se mudou alguma coisa.
    ///
    /// - chave com sucessora → as chaves seguintes são **deslocadas** pelo delta;
    /// - última chave → cria/move a **sentinela de fim** em `key + frames`.
    pub fn set_exposure(&mut self, layer_id: LayerId, key: Frame, frames: u32) -> bool {
        let n = frames.max(1);
        let Some(layer) = self.layer(layer_id) else {
            return false;
        };
        // Só chave REAL tem exposição (sentinela não é desenho).
        if layer.frames().get(&key).is_none_or(|f| f.drawing.is_none()) {
            return false;
        }
        let cur = layer.duration_at(key);
        if cur == n {
            return false;
        }
        let end = key.saturating_add(n as i32);

        if cur == 0 {
            // Última chave: a exposição É a sentinela.
            let Some(l) = self.layer_mut(layer_id) else {
                return false;
            };
            return l.set_end_sentinel(key, end);
        }

        // Desloca as chaves seguintes pelo delta. Ao ESTICAR move-se do fim para o
        // começo (senão a primeira mudança cai em cima da vizinha ainda parada); ao
        // encolher, do começo para o fim. Mesma disciplina do reorder de listas.
        let delta = n as i32 - cur as i32;
        let after: Vec<Frame> = layer.frames().range((key + 1)..).map(|(&f, _)| f).collect();
        if after.iter().any(|&f| f + delta <= key) {
            return false; // encolher tanto que colidiria com a própria chave
        }
        let seq: Vec<Frame> = if delta > 0 {
            after.into_iter().rev().collect()
        } else {
            after
        };
        let Some(l) = self.layer_mut(layer_id) else {
            return false;
        };
        let mut moved = false;
        for f in seq {
            moved |= l.relocate_frame(f, f + delta);
        }
        moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Hold, KeyKind};
    use crate::ids::FlipObjectId;

    fn obj(keys: &[Frame]) -> (FlipObject, LayerId) {
        let mut o = FlipObject::new(FlipObjectId(1), "O");
        let l = o.add_layer("L");
        for &k in keys {
            o.insert_frame(l, k, Hold::Implicit, KeyKind::Keyframe);
        }
        (o, l)
    }

    fn cells(o: &FlipObject, l: LayerId) -> Vec<(Frame, u32)> {
        o.layer(l)
            .unwrap()
            .cells()
            .into_iter()
            .map(|(k, _, e)| (k, e))
            .collect()
    }

    /// Esticar a 1ª chave EMPURRA as seguintes (o tempo é relativo — mexer no
    /// quadro 1 não pode comer o quadro 2).
    #[test]
    fn stretching_a_key_pushes_the_rest() {
        let (mut o, l) = obj(&[0, 2, 4]);
        assert_eq!(cells(&o, l), vec![(0, 2), (2, 2), (4, 1)]);
        assert!(o.set_exposure(l, 0, 5));
        assert_eq!(
            cells(&o, l),
            vec![(0, 5), (5, 2), (7, 1)],
            "as seguintes andaram +3, mantendo as PRÓPRIAS exposições"
        );
        // E o desenho de cada chave veio junto (não trocou de dono).
        let d0 = o.drawing_at(l, 0);
        let d5 = o.drawing_at(l, 5);
        assert_ne!(d0, d5);
    }

    /// Encolher puxa de volta.
    #[test]
    fn shrinking_a_key_pulls_the_rest_back() {
        let (mut o, l) = obj(&[0, 6, 8]);
        assert!(o.set_exposure(l, 0, 2));
        assert_eq!(cells(&o, l), vec![(0, 2), (2, 2), (4, 1)]);
        // `0` é clampado a 1 (uma chave sempre ocupa ao menos um quadro) — e
        // encolher até 1 é legítimo: as seguintes vêm junto.
        assert!(o.set_exposure(l, 0, 0));
        assert_eq!(cells(&o, l), vec![(0, 1), (1, 2), (3, 1)]);
    }

    /// A ÚLTIMA chave não tem o que empurrar: a exposição vira a sentinela de fim —
    /// e no ciclo default (`Hold`) o desenho **continua na tela** depois dela.
    #[test]
    fn the_last_key_gets_an_end_sentinel_and_still_holds_on_screen() {
        let (mut o, l) = obj(&[0, 4]);
        assert!(o.set_exposure(l, 4, 3));
        assert_eq!(cells(&o, l), vec![(0, 4), (4, 3)]);
        assert_eq!(o.layer(l).unwrap().span(), Some((0, 7)), "o vão fecha em 7");
        // O ciclo default é `post = Hold`: depois do vão, o último desenho segura.
        let last = o.drawing_at(l, 6);
        assert!(last.is_some());
        assert_eq!(o.drawing_at(l, 99), last, "Hold: a arte não some");
        // Re-expor move a sentinela (não empilha).
        assert!(o.set_exposure(l, 4, 6));
        assert_eq!(o.layer(l).unwrap().span(), Some((0, 10)));
        assert_eq!(cells(&o, l), vec![(0, 4), (4, 6)]);
    }

    #[test]
    fn a_key_that_is_not_there_has_no_exposure() {
        let (mut o, l) = obj(&[0, 4]);
        assert!(!o.set_exposure(l, 7, 3), "não há chave em 7");
        assert!(!o.set_exposure(l, 0, 4), "já expõe 4 — nada muda");
    }
}
