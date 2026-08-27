//! **O ASSADOR** — de uma arte + uma lei de reticulado para **um** rectângulo que a GPU repete.
//!
//! # Porque isto existe, e porque não é um passo do quadro
//!
//! O Vello sabe repetir uma imagem (`Extend::Repeat`) e **só** sabe repetir num reticulado
//! rectangular. Todo o resto que o artista quer — tijolo, meia-queda, colmeia, vão, sobreposição —
//! é uma arrumação **dentro** de um rectângulo maior. ⇒ este módulo faz essa arrumação **uma vez**,
//! e o quadro fica a custar uma `fill()`.
//!
//! # A conta, em três linhas
//!
//! - a **célula** mede `arte + vão` (com o vão assinado: negativo é sobreposição);
//! - o **ladrilho** mede `célula x [colunas, linhas]`, onde a contagem vem de
//!   [`TileLaw::cells`] — e é `[1,1]` para a grade;
//! - cada célula recebe uma cópia da arte na origem dela **mais** o desfasamento do reticulado, e a
//!   cópia **dá a volta** nas duas bordas (é a volta que faz o ladrilho fechar).

use crate::TileLaw;

/// **O tecto de uma aresta do ladrilho assado, e o recurso dele é o ATLAS DE IMAGEM DO VELLO.**
///
/// Medido em `vello_encoding-0.8.0/src/image_cache.rs:9-10`: o atlas nasce em `1024`, **dobra** até
/// `MAX_ATLAS_SIZE = 8192`, e é **um só para todas as imagens do quadro** — as nossas sprites, os
/// FX raster, os padrões, tudo. Um recurso que não consegue vaga é **descartado em silêncio**
/// (`resolve.rs:296`), e o próprio `fine.wgsl` avisa que esse caso *"isn't robust"*.
///
/// ⇒ `4096` é **metade** do máximo absoluto, de propósito: um ladrilho a `8192` tomaria o atlas
/// inteiro e evictaria tudo o resto. ⛔ Não é um número "por segurança": é o recurso, dito com nome,
/// e a metade é o que deixa um padrão coexistir com o resto do quadro.
///
/// ⚠️ E é ele que limita o [`TileLaw::offset_denom`] — que por isso **não tem tecto próprio**.
pub const MAX_TILE_EDGE_PX: u32 = 4096;

/// Porque o assado não saiu.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BakeError {
    /// Arte de dimensão zero, ou um buffer que não bate com `largura x altura x 4`.
    Empty,
    /// O ladrilho passaria de [`MAX_TILE_EDGE_PX`]. Carrega as medidas que ele teria — recusar em
    /// voz alta é a única alternativa a um recurso descartado em silêncio pelo atlas.
    TooBig { width: u32, height: u32 },
}

/// O ladrilho assado: RGBA **reto** (não pré-multiplicado, como o `StableImage::from_rgba` espera).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tile {
    /// `width * height * 4` bytes, RGBA reto.
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Quantas células cabem no ladrilho, `[colunas, linhas]` — o [`crate::placement`] precisa
    /// disto para saber quantos PERÍODOS o rectângulo cobre.
    pub cells: [u32; 2],
}

/// Assa `law` sobre `art` (RGBA reto, `aw x ah`) e devolve o rectângulo que a GPU repete.
///
/// ⚠️ **A grade encostada devolve a arte ao BYTE** — inclusive o RGB debaixo de alfa zero (ver
/// [`over`]). O ponto neutro é identidade, e tem gate.
pub fn bake(art: &[u8], aw: u32, ah: u32, law: &TileLaw) -> Result<Tile, BakeError> {
    if aw == 0 || ah == 0 || art.len() != (aw as usize) * (ah as usize) * 4 {
        return Err(BakeError::Empty);
    }
    let cell = cell_px([aw, ah], law.gap_px);
    let cells = law.cells();
    let (tw, th) = (
        u64::from(cell[0]) * u64::from(cells[0]),
        u64::from(cell[1]) * u64::from(cells[1]),
    );
    if tw > u64::from(MAX_TILE_EDGE_PX) || th > u64::from(MAX_TILE_EDGE_PX) {
        return Err(BakeError::TooBig {
            width: u32::try_from(tw).unwrap_or(u32::MAX),
            height: u32::try_from(th).unwrap_or(u32::MAX),
        });
    }
    // Depois da guarda os dois cabem em `u32` por construção (<= MAX_TILE_EDGE_PX).
    let (tw, th) = (
        u32::try_from(tw).unwrap_or(MAX_TILE_EDGE_PX),
        u32::try_from(th).unwrap_or(MAX_TILE_EDGE_PX),
    );
    let mut rgba = vec![0u8; (tw as usize) * (th as usize) * 4];
    for row in 0..cells[1] {
        for col in 0..cells[0] {
            let s = law.shift_px(cell, col, row);
            let origin = [col * cell[0] + s[0], row * cell[1] + s[1]];
            blit_wrapped(&mut rgba, [tw, th], art, [aw, ah], origin);
        }
    }
    Ok(Tile {
        rgba,
        width: tw,
        height: th,
        cells,
    })
}

/// A célula em pixels: a arte mais o vão assinado, **nunca abaixo de um pixel**.
///
/// ⚠️ O piso não é conforto: um ladrilho de largura zero é uma divisão por zero no amostrador, e um
/// vão negativo maior que a arte é um pedido legítimo do artista (sobreposição total).
fn cell_px(art: [u32; 2], gap: [i32; 2]) -> [u32; 2] {
    let one = |a: u32, g: i32| -> u32 {
        let v = i64::from(a)
            .saturating_add(i64::from(g))
            .clamp(1, i64::from(u32::MAX));
        u32::try_from(v).unwrap_or(1)
    };
    [one(art[0], gap[0]), one(art[1], gap[1])]
}

/// Copia `src` para `dst` com a origem em `origin`, **dando a volta nas duas bordas**.
///
/// ⭐ A volta é o que faz o ladrilho fechar: a parte da cópia que passa da borda direita reaparece
/// à esquerda, e é por isso que um tijolo desfasado continua a encaixar consigo próprio. É também a
/// mesma máquina que dá a sobreposição de graça — com a célula menor que a arte, a cópia sobrepõe-se
/// a si mesma pela volta.
fn blit_wrapped(dst: &mut [u8], tile: [u32; 2], src: &[u8], art: [u32; 2], origin: [u32; 2]) {
    let (tw, th) = (tile[0], tile[1]);
    for y in 0..art[1] {
        let dy = (origin[1] + y) % th;
        for x in 0..art[0] {
            let dx = (origin[0] + x) % tw;
            let so = ((y as usize) * (art[0] as usize) + x as usize) * 4;
            let dof = ((dy as usize) * (tw as usize) + dx as usize) * 4;
            over(&mut dst[dof..dof + 4], &src[so..so + 4]);
        }
    }
}

/// *Source-over* em alfa RETO, com as duas pontas tratadas à mão de propósito.
///
/// ⚠️⚠️ **`dst` transparente COPIA, não compõe** — e esta linha é a diferença entre a grade ser
/// byte-idêntica e não ser. A fórmula geral divide por `out_a`; com um destino vazio e um texel de
/// fonte **transparente mas colorido** (o que todo PNG comum tem) daria `0/0`, e o RGB debaixo da
/// alfa zero seria apagado. É a família do [Bug #4 do Motion](../../../docs/Motion%20Nodes/BUGS_motion_nodes.md):
/// o defeito vive fora de `alpha = 1`, que é onde quase nenhum gate olha.
fn over(dst: &mut [u8], src: &[u8]) {
    if dst[3] == 0 {
        dst.copy_from_slice(src);
        return;
    }
    if src[3] == 0 {
        return;
    }
    let (sa, da) = (f32::from(src[3]) / 255.0, f32::from(dst[3]) / 255.0);
    let keep = da * (1.0 - sa);
    let out_a = sa + keep;
    for i in 0..3 {
        let c = (f32::from(src[i]) * sa + f32::from(dst[i]) * keep) / out_a;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            dst[i] = c.round().clamp(0.0, 255.0) as u8;
        }
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    }
}
