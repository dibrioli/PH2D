//! **O JOURNAL POR TILE** — capturar o "antes" na hora da ESCRITA, em vez de o derivar de uma cópia
//! do documento (doc 28 §7, degrau S3).
//!
//! # O problema que ele existe para resolver
//!
//! Hoje o histórico obtém o lado `before` de um passo **diferenciando dois snapshots completos**, e
//! para ter o snapshot é preciso segurar um `Arc` de cada plano. Medido (§5.20), dentro de um gesto os
//! planos canvas-shaped têm **três donos** — o tool, o `cursor` do histórico e o `stroke_undo` do
//! pen-down — e `Arc::make_mut` **copia com qualquer coisa acima de um**. É daí que saem os três custos
//! que a §5.21 pôs num número só:
//!
//! ```text
//!   fold do relevo            11,9 ms  (forka os três planos no pen-up)
//!   fork do pen-down          11,7 ms  (a primeira escrita de todo gesto)
//!   `free` da geração velha  2,4-5,0   (a outra ponta do fork, cobrada no commit)
//! ```
//!
//! A alternativa é a que GIMP e Krita usam: **quem escreve avisa onde vai escrever, e os bytes velhos
//! daquela região são copiados antes**. Aí o plano tem um dono só, a escrita é in-place, e o histórico
//! guarda kilobytes em vez de documentos.
//!
//! # Por que TILE, e não retângulo
//!
//! Um passo escreve várias vezes, em regiões que crescem. Guardar UM retângulo obriga a capturar o
//! *complemento* quando ele cresce (subtração de retângulos, até quatro pedaços) — e errar isso perde
//! texels **em silêncio**, que é o modo de falha que o `diff_window` documenta e teme. Uma grade de
//! tiles com um bit de "já capturado" torna o crescimento trivial e idempotente: capture todo tile que
//! a região toca e que ainda não foi capturado. Não há aritmética a errar.
//!
//! ⚠️ **A primeira captura por tile é a que vale** — ela guarda o estado do início do passo. Uma
//! segunda escrita no mesmo tile não pode recapturar (guardaria bytes já modificados), e é por isso que
//! `capture` pula tile já tomado em vez de sobrescrever.
//!
//! # A regra do "não sei onde"
//!
//! Um sítio que não conhece a região passa `None` e o journal captura **o plano inteiro**. Isso custa
//! exatamente o que o fork custa hoje ⇒ **nunca é regressão**, e é correto por construção. O ganho é
//! incremental: cada sítio que aprende a sua região passa a pagar só a região.

/// O lado do tile, em ELEMENTOS do plano. 128 elementos × 128 linhas: a 4096² são 32×32 = 1024 tiles
/// por plano, e um tile de canvas RGBA são 128×128 = 16 KB.
///
/// ⚠️ Não precisa estar alinhado a pixel: a geometria do journal é sobre ÍNDICES do buffer, e a
/// conversão pixel→elemento é do chamador. Um tile que corta um pixel ao meio segue correto — ele só
/// captura mais ou menos bytes do que um humano desenharia.
pub(crate) const TILE: usize = 128;

/// Os bytes velhos de um plano, capturados tile a tile na primeira escrita de cada tile.
#[derive(Debug)]
pub(crate) struct TileJournal<T> {
    /// Elementos por linha do plano. Fixado na primeira captura do passo.
    stride: usize,
    /// Linhas do plano.
    rows: usize,
    /// Tiles por linha.
    tiles_x: usize,
    /// Os bytes velhos, por tile. `None` = ainda não capturado.
    data: Vec<Option<Box<[T]>>>,
}

impl<T> Default for TileJournal<T> {
    fn default() -> Self {
        Self {
            stride: 0,
            rows: 0,
            tiles_x: 0,
            data: Vec::new(),
        }
    }
}

impl<T: Copy> TileJournal<T> {
    /// Esquece tudo: o passo fechou, e o próximo captura do zero.
    pub(crate) fn reset(&mut self) {
        self.stride = 0;
        self.rows = 0;
        self.tiles_x = 0;
        self.data.clear();
    }

    /// `true` se nada foi capturado neste passo. Consumidor: os gates deste módulo.
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.data.iter().all(Option::is_none)
    }

    /// Quantos bytes o journal retém. Consumidor: os gates deste módulo.
    #[cfg(test)]
    pub(crate) fn heap_bytes(&self) -> usize {
        self.data
            .iter()
            .flatten()
            .map(|t| t.len() * size_of::<T>())
            .sum()
    }

    /// Prepara a grade para um plano de `stride × rows`. Se as dimensões mudaram, o que estava
    /// capturado descrevia OUTRO plano e é descartado — um journal com meia grade velha misturaria
    /// duas geometrias, que é pior que não ter journal.
    fn arm(&mut self, stride: usize, rows: usize) {
        if self.stride == stride && self.rows == rows {
            return;
        }
        self.stride = stride;
        self.rows = rows;
        self.tiles_x = stride.div_ceil(TILE);
        let tiles_y = rows.div_ceil(TILE);
        self.data.clear();
        self.data.resize_with(self.tiles_x * tiles_y, || None);
    }

    /// A largura, em elementos, do tile da coluna `tx` (a última coluna pode ser curta).
    fn tile_w(&self, tx: usize) -> usize {
        ((tx + 1) * TILE).min(self.stride) - tx * TILE
    }

    /// **Captura os bytes velhos de todo tile que `area` toca e que ainda não foi capturado.**
    ///
    /// `area` é `(x0, y0, x1, y1)` em ELEMENTOS, meio-aberta; `None` = o plano inteiro (a resposta de
    /// quem não sabe onde vai escrever, e ela é sempre correta).
    pub(crate) fn capture(
        &mut self,
        buf: &[T],
        stride: usize,
        area: Option<(usize, usize, usize, usize)>,
    ) {
        if stride == 0 || buf.is_empty() || !buf.len().is_multiple_of(stride) {
            return; // um plano que o stride não mede: o chamador cai no caminho completo
        }
        let rows = buf.len() / stride;
        self.arm(stride, rows);
        let (x0, y0, x1, y1) = area.unwrap_or((0, 0, stride, rows));
        let (x0, y0) = (x0.min(stride), y0.min(rows));
        let (x1, y1) = (x1.min(stride), y1.min(rows));
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for ty in (y0 / TILE)..=((y1 - 1) / TILE) {
            for tx in (x0 / TILE)..=((x1 - 1) / TILE) {
                let t = ty * self.tiles_x + tx;
                if self.data[t].is_some() {
                    continue; // ⚠️ a PRIMEIRA captura é a que vale — ver o cabeçalho
                }
                let w = self.tile_w(tx);
                let ry = (ty * TILE)..((ty + 1) * TILE).min(rows);
                let mut v: Vec<T> = Vec::with_capacity(w * ry.len());
                for y in ry {
                    let base = y * stride + tx * TILE;
                    v.extend_from_slice(&buf[base..base + w]);
                }
                self.data[t] = Some(v.into_boxed_slice());
            }
        }
    }

    /// O valor que o elemento `i` tinha no início do passo, se o tile dele foi capturado.
    pub(crate) fn get(&self, i: usize) -> Option<T> {
        if self.stride == 0 {
            return None;
        }
        let (y, x) = (i / self.stride, i % self.stride);
        if y >= self.rows {
            return None;
        }
        let (tx, ty) = (x / TILE, y / TILE);
        let tile = self.data.get(ty * self.tiles_x + tx)?.as_ref()?;
        tile.get((y % TILE) * self.tile_w(tx) + (x % TILE)).copied()
    }
}

#[cfg(test)]
#[path = "undo_journal_tests.rs"]
mod tests;
