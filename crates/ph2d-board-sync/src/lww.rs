//! **A lei de fusão de um registo** (ADR-0181 §1) — UMA, para o Quadro e para o Kanban (ADR-0195,
//! W5): ganha o carimbo de Lamport MAIOR; o contador primeiro, o par a desempatar. Comutativa,
//! associativa e idempotente: a ordem, o atraso e a duplicação não mudam o vencedor.

use crate::{Lamport, PeerId};

/// `at` vence o registo que hoje tem o carimbo `current`? (Igual = a mesma escrita: não reescreve.)
#[must_use]
pub fn wins(at: Lamport, current: Lamport) -> bool {
    at > current
}

/// O relógio de Lamport de um par: cada escrita minha sobe-o, cada escrita que chega puxa-o.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Clock {
    pub n: u64,
    pub peer: PeerId,
}

impl Clock {
    #[must_use]
    pub fn new(peer: PeerId) -> Self {
        Self { n: 0, peer }
    }

    /// Um carimbo novo, maior do que tudo o que este par já viu.
    pub fn tick(&mut self) -> Lamport {
        self.n += 1;
        Lamport {
            n: self.n,
            peer: self.peer,
        }
    }

    /// Uma escrita que chegou com `at`.
    pub fn see(&mut self, at: Lamport) {
        self.n = self.n.max(at.n);
    }
}
