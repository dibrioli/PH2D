//! **A lei de fusão de um registo** (ADR-0181 §1) — UMA, para o Quadro e para o Kanban (ADR-0195,
//! W5): ganha o carimbo de Lamport MAIOR; o contador primeiro, o par a desempatar. Comutativa,
//! associativa e idempotente: a ordem, o atraso e a duplicação não mudam o vencedor.

use crate::{Lamport, PeerId};

/// O maior contador de Lamport que um par aceita: um carimbo acima é LIXO (um par estragado ou mal
/// intencionado) e a escrita dele é recusada. `2^53` = 285 anos a um milhão de escritas por segundo
/// (cada escrita sobe o contador de 1), e deixa `2^64 - 2^53` de folga: o relógio nunca transborda.
pub const MAX_STAMP: u64 = 1 << 53;

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
        self.n = self.n.saturating_add(1);
        Lamport {
            n: self.n,
            peer: self.peer,
        }
    }

    /// Uma escrita que chegou com `at`.
    pub fn see(&mut self, at: Lamport) {
        if sane(at) {
            self.n = self.n.max(at.n);
        }
    }
}

/// O carimbo cabe no tecto ([`MAX_STAMP`])? Uma escrita com um carimbo acima não se funde.
#[must_use]
pub fn sane(at: Lamport) -> bool {
    at.n <= MAX_STAMP
}
