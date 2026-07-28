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
//! Um sítio que não conhece a região passa `None` e o journal captura **o plano inteiro**. É correto por
//! construção, e o ganho é incremental: cada sítio que aprende a sua região passa a pagar só a região.
//!
//! ⚠️ **"Custa o que o fork custa" era uma AFIRMAÇÃO minha, e a medição a derrubou** (doc 28 §5.25).
//! Um plano inteiro em tiles são 1024 alocações de 16 KB montadas em série, contra **uma** cópia
//! paralela: medido a 4096², **12,08 ms contra 3,24** do fork — o fallback era **3,73× pior** que a
//! coisa que ele substitui, e a §5.20 conta **39 dos 47 sítios** que passariam `None`. Por isso a
//! captura é **paralela por tile** acima do limiar de [`crate::plane_copy`]: os tiles são **disjuntos**
//! e a leitura do buffer é **pura**, que é a forma que o ADR-0109 sanciona e que esta crate já usa em
//! quatro lugares. Com ela o fallback volta a ser o que a política do S1 promete — *lento nunca, errado
//! jamais*.

use rayon::prelude::*;

/// **Os bytes de UM tile, copiados do plano.**
///
/// Porta única: o caminho serial e o paralelo terminam aqui, então eles não podem divergir sobre o que
/// um tile é — e o gate de paridade entre os dois compara essa resposta, não duas implementações.
fn tile_bytes<T: Copy>(buf: &[T], stride: usize, rows: usize, ty: usize, tx: usize) -> Box<[T]> {
    let w = ((tx + 1) * TILE).min(stride) - tx * TILE;
    let ry = (ty * TILE)..((ty + 1) * TILE).min(rows);
    let mut v: Vec<T> = Vec::with_capacity(w * ry.len());
    for y in ry {
        let base = y * stride + tx * TILE;
        v.extend_from_slice(&buf[base..base + w]);
    }
    v.into_boxed_slice()
}

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
    /// Quantos tiles já foram tomados — o que decide se uma captura de plano inteiro pode ir pelo
    /// caminho contíguo. `usize` e não uma varredura de `data`: a pergunta é feita a cada captura.
    taken: usize,
    /// **O plano INTEIRO, copiado de uma vez** — a resposta de quem não sabe onde vai escrever e chega
    /// antes de qualquer tile.
    ///
    /// ⚠️ Ele existe por MEDIÇÃO, não por elegância: montar o mesmo plano em 1024 caixas de 16 KB custa
    /// **2,2× um memcpy** a 2048², onde a cópia fica abaixo do limiar do rayon e o paralelo não salva
    /// (doc 28 §5.25). Com ele o fallback custa exatamente o que o fork custa **em toda tela**, que é o
    /// que a política *"quem não sabe passa o plano inteiro"* promete.
    whole: Option<Box<[T]>>,
}

impl<T> Default for TileJournal<T> {
    fn default() -> Self {
        Self {
            stride: 0,
            rows: 0,
            tiles_x: 0,
            data: Vec::new(),
            taken: 0,
            whole: None,
        }
    }
}

impl<T: Copy + Send + Sync> TileJournal<T> {
    /// Esquece tudo: o passo fechou, e o próximo captura do zero.
    pub(crate) fn reset(&mut self) {
        self.stride = 0;
        self.rows = 0;
        self.tiles_x = 0;
        self.data.clear();
        self.taken = 0;
        self.whole = None;
    }

    /// `true` se nada foi capturado neste passo. Consumidor: os gates deste módulo.
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.whole.is_none() && self.taken == 0
    }

    /// Quantos bytes o journal retém. Consumidor: os gates deste módulo.
    #[cfg(test)]
    pub(crate) fn heap_bytes(&self) -> usize {
        let tiles: usize = self.data.iter().flatten().map(|t| t.len()).sum();
        (tiles + self.whole.as_ref().map_or(0, |w| w.len())) * size_of::<T>()
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
        self.taken = 0;
        self.whole = None;
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
        if self.whole.is_some() {
            return; // o plano inteiro já está guardado: não há tile novo a tomar
        }
        // **O plano inteiro, contíguo** — só quando nenhum tile foi tomado ainda, porque a primeira
        // captura de cada tile é a que vale e uma cópia do buffer de AGORA já traria bytes escritos.
        if area.is_none() && self.taken == 0 {
            self.whole = Some(crate::plane_copy::par_clone(buf).into_boxed_slice());
            return;
        }
        let (x0, y0, x1, y1) = area.unwrap_or((0, 0, stride, rows));
        let (x0, y0) = (x0.min(stride), y0.min(rows));
        let (x1, y1) = (x1.min(stride), y1.min(rows));
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let (ty0, ty1) = (y0 / TILE, (y1 - 1) / TILE);
        let (tx0, tx1) = (x0 / TILE, (x1 - 1) / TILE);
        let tiles_x = self.tiles_x;
        // Vale espalhar por threads? A porta de fork faz a MESMA pergunta antes de copiar um plano, e a
        // resposta vem da mesma função — duas cópias dela divergiriam sobre a mesma tela. O `span` conta
        // tiles INTEIROS (superestima a última coluna/linha), o que é o certo para um limiar.
        let span = (ty1 - ty0 + 1) * (tx1 - tx0 + 1) * TILE * TILE;
        if crate::plane_copy::worth_parallel::<T>(span) {
            self.taken += self
                .data
                .par_iter_mut()
                .enumerate()
                .filter(|(t, slot)| {
                    // ⚠️ a PRIMEIRA captura é a que vale — ver o cabeçalho
                    slot.is_none()
                        && (ty0..=ty1).contains(&(t / tiles_x))
                        && (tx0..=tx1).contains(&(t % tiles_x))
                })
                .map(|(t, slot)| {
                    *slot = Some(tile_bytes(buf, stride, rows, t / tiles_x, t % tiles_x));
                })
                .count();
            return;
        }
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                let t = ty * tiles_x + tx;
                if self.data[t].is_some() {
                    continue; // ⚠️ a PRIMEIRA captura é a que vale — ver o cabeçalho
                }
                self.data[t] = Some(tile_bytes(buf, stride, rows, ty, tx));
                self.taken += 1;
            }
        }
    }

    /// O valor que o elemento `i` tinha no início do passo, se o tile dele foi capturado.
    pub(crate) fn get(&self, i: usize) -> Option<T> {
        if let Some(w) = &self.whole {
            return w.get(i).copied();
        }
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
