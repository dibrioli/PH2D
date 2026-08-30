#![forbid(unsafe_code)]
//! **O VOCABULÁRIO DE LADRILHO** do *Texture Pattern* (plano 33, W1) — e o assador que o resolve.
//!
//! # A ideia, numa frase
//!
//! *Tijolo*, *meia-queda*, *colmeia*, *espaçamento* e *sobreposição* são todos decisões sobre um
//! **reticulado** — e um reticulado de tijolo com meio passo **É** um reticulado rectangular de
//! duas linhas. ⇒ a lei de ladrilho resolve-se **ao ASSAR** (uma vez, memoizado) e a GPU faz o
//! único `Extend::Repeat` que sempre soube fazer.
//!
//! Isso é o que torna o preenchimento com padrão tão barato quanto uma cor chapada: **uma** `fill()`
//! por forma, sem camada de clip e sem rasterização por quadro. (Medido no plano 33 §0.1: o Vello
//! 0.8 empacota `x_extend`/`y_extend` em `sample_alpha` e o `fine.wgsl` honra-os, aplicando o extend
//! **antes** de somar o `atlas_offset` — o repeat dá a volta dentro do próprio ladrilho.)
//!
//! # Porque é uma folha ZERO-dep
//!
//! Dois donos que não se veem: a [`ph2d-vec-scene`] guarda o documento e é **pura** (o `Cargo.toml`
//! dela declara *"sem vello/kurbo/ph2d-color"*), e a `ph2d-vec-render` é quem alcança a stack
//! Linebender. Nenhuma pode depender da outra ⇒ o vocabulário mora aqui. É a MESMA razão, escrita
//! com as mesmas palavras, da `ph2d-warp-style` e da `ph2d-stroke-width`.
//!
//! ⛔ **Nada aqui conhece `peniko`, `kurbo` ou pixels de GPU.** O [`PatternMode`] é um enum nosso; é
//! a `ph2d-vec-render` que o traduz para `Extend`. Um `use peniko::…` neste crate é o começo do
//! caminho que a `ph2d-vec-scene` fechou de propósito.
//!
//! [`ph2d-vec-scene`]: https://docs.rs/ph2d-vec-scene

mod bake;
mod place;
mod seam;

#[cfg(test)]
mod bake_tests;
#[cfg(test)]
mod place_tests;
#[cfg(test)]
mod seam_tests;

pub use bake::{BakeError, MAX_TILE_EDGE_PX, Tile, bake};
pub use place::{HEX_ROW_RATIO, gap_px_from_world, hex_row_period, placement};
pub use seam::{SEAM_VISIBLE, seam_is_visible, tiles_cleanly, wrap_seam};

use serde::{Deserialize, Serialize};

/// **Como as cópias se arrumam** — o reticulado, e nada mais.
///
/// Espelha o menu *Tile Type* do Illustrator (Grid · Brick by Row · Brick by Column · Hex by
/// Row/Column), com uma redução medida: o *half-drop* têxtil **é** `BrickCol` com desfasamento
/// `1/2`, então ele não é um tipo — é um valor de [`TileLaw::offset_denom`]. Construí-lo à parte
/// seria dar duas respostas à mesma pergunta.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileKind {
    /// A grade: cada cópia debaixo da de cima. O ponto neutro.
    Grid,
    /// As LINHAS desfasam-se horizontalmente — o *Brick by Row*.
    BrickRow,
    /// As COLUNAS desfasam-se verticalmente — o *Brick by Column* (e o *half-drop* têxtil, com
    /// [`TileLaw::offset_denom`] = 2).
    BrickCol,
    /// A colmeia: `BrickRow` com meio passo **mais** a lei do espaçamento
    /// ([`HEX_ROW_RATIO`]) que põe os seis vizinhos à mesma distância.
    ///
    /// ⛔⛔ **NÃO é verdade que o assado de um `Hex` seja byte-idêntico ao de um `BrickRow`** — esta
    /// linha dizia-o e foi **medida falsa** em 2026-08-30, pela porta do produto. Dada a MESMA
    /// [`TileLaw`] ela é verdadeira, e é essa a leitura que a escreveu; mas quem constrói a lei é
    /// `PatternFill::law`, e é lá que as duas divergem: sobre a mesma arte `96x96`, o `BrickRow` dá
    /// `gap_px [0,0]` e um ladrilho `96x192` com o motivo **intacto**, e o `Hex` dá `gap_px [0,−13]`
    /// e um ladrilho `96x166` com **21 % do motivo reescrito pela cópia vizinha**.
    ///
    /// ⚠️ *Uma afirmação sobre uma função é verdadeira dos argumentos dela e falsa do produto, se
    /// for outro sítio que escolhe os argumentos.* A sobreposição é a colmeia a fazer as linhas
    /// **encaixarem** ([`hex_row_period`]) — é a definição dela, não um defeito.
    Hex,
}

/// **A lei de ladrilho**: o reticulado + o vão, em pixels da arte.
///
/// ⚠️ **O vão é assinado**: negativo é a **sobreposição** (o *Overlap* do Illustrator), e ele custa
/// zero porque a mesma máquina de dar-a-volta que o tijolo precisa já o exprime.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileLaw {
    /// O reticulado.
    pub kind: TileKind,
    /// O desfasamento é **1/`offset_denom`** de uma célula. `0` e `1` significam **nenhum**, e por
    /// isso qualquer tipo com `offset_denom <= 1` assa exactamente como a [`TileKind::Grid`] — o
    /// ponto neutro tem gate.
    ///
    /// ⚠️ **Não há tecto escrito à mão aqui, e a ausência é a decisão** (`CLAUDE.md` §0.0): o
    /// recurso é o ATLAS de imagem do Vello, e o tecto sai dele — um `offset_denom` de `n`
    /// multiplica **uma** aresta do ladrilho por `n`, então o limite legítimo é
    /// `MAX_TILE_EDGE_PX / aresta_da_célula`, que a UI deriva e o [`bake`] impõe.
    pub offset_denom: u8,
    /// O vão acrescentado a cada célula, em pixels da arte. Negativo = sobreposição.
    pub gap_px: [i32; 2],
}

impl TileLaw {
    /// A lei mais simples: a grade encostada. O ponto neutro do assador.
    #[must_use]
    pub fn grid() -> Self {
        Self {
            kind: TileKind::Grid,
            offset_denom: 1,
            gap_px: [0, 0],
        }
    }

    /// **Quantas células o ladrilho precisa antes de se repetir.**
    ///
    /// É `1` para a grade (e para todo desfasamento de `1/1`, que é nenhum), `2` para a colmeia, e
    /// o próprio `offset_denom` para os tijolos: depois de `n` linhas o desfasamento acumulado é
    /// `n/n = 1` célula inteira, que é a identidade.
    #[must_use]
    pub fn period(&self) -> u32 {
        match self.kind {
            TileKind::Grid => 1,
            TileKind::Hex => 2,
            TileKind::BrickRow | TileKind::BrickCol => u32::from(self.offset_denom.max(1)),
        }
    }

    /// O ladrilho em CÉLULAS, `[colunas, linhas]`.
    ///
    /// ⚠️ O eixo que cresce é o **perpendicular** ao do desfasamento: linhas que deslizam na
    /// horizontal precisam de `n` LINHAS para fechar.
    #[must_use]
    pub fn cells(&self) -> [u32; 2] {
        let n = self.period();
        match self.kind {
            TileKind::Grid => [1, 1],
            TileKind::BrickRow | TileKind::Hex => [1, n],
            TileKind::BrickCol => [n, 1],
        }
    }

    /// O desfasamento em pixels da célula `(col, row)` dentro do ladrilho, `[dx, dy]`.
    ///
    /// ⚠️ **O arredondamento por célula não parte a periodicidade**, e é por isso que ele é honesto:
    /// a célula `k + n` é a célula `k` do ladrilho seguinte, deslocada por exactamente uma célula —
    /// que módulo a largura do ladrilho é zero. O erro visual fica em meio pixel; o fecho é exacto,
    /// e tem gate (`the_lattice_closes_on_itself`).
    #[must_use]
    pub fn shift_px(&self, cell: [u32; 2], col: u32, row: u32) -> [u32; 2] {
        let n = u64::from(self.period());
        if n <= 1 {
            return [0, 0];
        }
        match self.kind {
            TileKind::Grid => [0, 0],
            TileKind::BrickRow | TileKind::Hex => {
                let dx = (u64::from(row) * u64::from(cell[0])) / n;
                #[allow(clippy::cast_possible_truncation)]
                [dx as u32, 0]
            }
            TileKind::BrickCol => {
                let dy = (u64::from(col) * u64::from(cell[1])) / n;
                #[allow(clippy::cast_possible_truncation)]
                [0, dy as u32]
            }
        }
    }
}

/// **Como o ladrilho preenche o que sobra da forma.** Traduzido para o `Extend` do peniko pela
/// `ph2d-vec-render` — este crate não conhece peniko.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternMode {
    /// Repete (`Extend::Repeat`). O caminho comum.
    Tile,
    /// Espelha a cada repetição (`Extend::Reflect`) — costura invisível mesmo em arte não-periódica.
    Mirror,
    /// Estica a borda (`Extend::Pad`): uma cópia só, e o resto é a orla dela.
    Clamp,
}
