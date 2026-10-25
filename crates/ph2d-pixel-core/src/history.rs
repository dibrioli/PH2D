//! O histórico de um sprite: pilhas de inversos com tecto em BYTES (uma contagem é um
//! multiplicador, não um tecto — ADR-0117).

use crate::op::{Op, OpError, apply};
use crate::sprite::Sprite;

/// O tecto por omissão: 64 MiB de RAM por sprite — o do MVP web (`docs/Pixel Art/app/engine/ops.js`,
/// `DEFAULT_CAP_BYTES`). O recurso é a memória do processo; o mais velho cai primeiro.
pub const DEFAULT_CAP_BYTES: usize = 64 * 1024 * 1024;

/// Desfazer/refazer de um sprite. Um gesto do artista = um op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    undo: Vec<Op>,
    redo: Vec<Op>,
    bytes: usize,
    cap_bytes: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::with_cap(DEFAULT_CAP_BYTES)
    }
}

impl History {
    #[must_use]
    pub fn with_cap(cap_bytes: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            bytes: 0,
            cap_bytes,
        }
    }

    /// Aplica `op` e guarda o inverso; o refazer esvazia-se.
    pub fn commit(&mut self, sprite: &mut Sprite, op: Op) -> Result<(), OpError> {
        let inverse = apply(sprite, op)?;
        for o in self.redo.drain(..) {
            self.bytes -= o.bytes();
        }
        self.bytes += inverse.bytes();
        self.undo.push(inverse);
        while self.bytes > self.cap_bytes && self.undo.len() > 1 {
            let dropped = self.undo.remove(0);
            self.bytes -= dropped.bytes();
        }
        Ok(())
    }

    /// Desfaz o último passo. `false` = nada a desfazer.
    pub fn undo(&mut self, sprite: &mut Sprite) -> bool {
        Self::swap(sprite, &mut self.undo, &mut self.redo, &mut self.bytes)
    }

    /// Refaz o último passo desfeito. `false` = nada a refazer.
    pub fn redo(&mut self, sprite: &mut Sprite) -> bool {
        Self::swap(sprite, &mut self.redo, &mut self.undo, &mut self.bytes)
    }

    fn swap(sprite: &mut Sprite, from: &mut Vec<Op>, to: &mut Vec<Op>, bytes: &mut usize) -> bool {
        let Some(op) = from.pop() else {
            return false;
        };
        *bytes -= op.bytes();
        match apply(sprite, op) {
            Ok(inv) => {
                *bytes += inv.bytes();
                to.push(inv);
                true
            }
            // Um inverso que o próprio `apply` produziu não falha; se falhar o documento não mudou.
            Err(_) => false,
        }
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    #[must_use]
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}
