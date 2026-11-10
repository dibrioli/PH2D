//! Larguras que não pulam: enquanto um quadro vive, cada coluna fica na maior largura natural já
//! vista (um número vivo que encolhe não puxa as colunas da direita).

use crate::{Column, Key};

/// ⭐ As maiores larguras naturais vistas, por coluna. Recomeça quando as colunas mudam (outras
/// chaves, ou outro número delas) e em [`StableWidths::reset`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StableWidths {
    seen: Vec<(Key, f32)>,
}

impl StableWidths {
    /// As larguras naturais de agora (`natural[i]` é a de `cols[i]`), nunca abaixo das já vistas.
    pub fn widths(&mut self, cols: &[Column], natural: &[f32]) -> Vec<f32> {
        let same = self.seen.len() == cols.len()
            && self.seen.iter().zip(cols).all(|((k, _), c)| *k == c.key);
        if !same {
            self.seen = cols.iter().map(|c| (c.key.clone(), 0.0)).collect();
        }
        for ((_, seen), now) in self.seen.iter_mut().zip(natural) {
            *seen = seen.max(*now);
        }
        self.seen.iter().map(|(_, w)| *w).collect()
    }

    /// Esquece tudo (o quadro fechou ou mudou de assunto).
    pub fn reset(&mut self) {
        self.seen.clear();
    }
}
