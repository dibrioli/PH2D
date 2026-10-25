//! **Re-chavear um binding** (F31 §A): o mesmo `target` — e com ele as trilhas, trajetórias e
//! expressões de cada clip — passa a nomear outra propriedade do MESMO objecto. Quem o usa é a
//! migração das chaves posicionais do Vetor para o id (`ph2d-timeline-persist`).

use super::*;

impl TimelineDoc {
    /// Troca a propriedade do binding `target` por `prop`. Recusa (`false`) se `target` não existe
    /// ou se o objecto dele já tem `prop` ligada — não funde duas trilhas.
    pub fn rechaveia(&mut self, target: AnimTarget, prop: PropKind) -> bool {
        let Some(pos) = self.bindings.iter().position(|b| b.target == target) else {
            return false;
        };
        let (entity, wire) = (self.bindings[pos].entity, self.bindings[pos].wire_id);
        let ocupada = self.bindings.iter().any(|o| {
            o.target != target && o.prop == prop && o.entity == entity && o.wire_id == wire
        });
        if ocupada {
            return false;
        }
        self.bindings[pos].prop = prop;
        true
    }
}
