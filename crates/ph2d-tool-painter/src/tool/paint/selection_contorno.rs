//! B20.11: o contorno da Seleção (nível 128) como polilinhas FECHADAS sobre as arestas entre texels,
//! em px de imagem. A shell transforma-as para o ecrã e traça as formigas com 1 px de ECRÃ a qualquer
//! zoom (o desenho na imagem perdia linhas com zoom < 1). Guardadas por identidade do `Arc` da
//! máscara e dos níveis do Offset, como o overlay (`selection_formigas`).

use super::PainterTool;
use std::cell::RefCell;
use std::sync::{Arc, Weak};

/// Uma polilinha fechada (o último ponto liga ao primeiro), cantos em px de imagem.
pub type Polilinha = Vec<[f32; 2]>;

/// Os contornos que a shell traça: o da máscara (formigas) e os dos níveis do Offset (ciano).
/// `no_raster`: a fronteira passou do orçamento — polilinhas vazias, as formigas saem pelo raster em
/// ecrã (`selection_formigas_raster`).
#[derive(Debug, Default, PartialEq)]
pub struct ContornosDaSelecao {
    pub mascara: Vec<Polilinha>,
    pub niveis: Vec<Polilinha>,
    pub no_raster: bool,
}

/// Orçamento do caminho vetorial em CANTOS (os pontos das polilinhas): custo de encode do Vello por
/// quadro. Tabela medida: `docs/Painter/missao_4k/BACKLOG.md` B20.11b. Múltiplo de 4 (gate).
pub const ORCAMENTO_DE_LADOS: usize = 20_000;
/// Orçamento em arestas unitárias: o custo do traçado (`contornos_da_mascara`), uma vez por máscara.
pub const ORCAMENTO_DE_ARESTAS: usize = 200_000;

/// `(cantos, arestas unitárias)` da fronteira 128 de `mask`, numa passada paralela por linhas de
/// vértices. Os cantos são EXATAMENTE os pontos que [`contornos_da_mascara`] emite (sela = 2).
#[must_use]
pub fn medida_da_fronteira(mask: &[u8], w: usize, h: usize) -> (usize, usize) {
    if w == 0 || h == 0 || mask.len() != w * h {
        return (0, 0);
    }
    let fila = |y: usize| (y < h).then(|| &mask[y * w..(y + 1) * w]);
    let linha = |y: usize| -> (u64, u64) {
        let (cima, baixo) = (y.checked_sub(1).and_then(fila), fila(y));
        let at = |r: Option<&[u8]>, x: usize| r.is_some_and(|r| x < w && r[x] >= 128);
        let (mut a, mut c) = (false, false);
        let (mut cantos, mut arestas) = (0u64, 0u64);
        for x in 0..=w {
            let (b, d) = (at(cima, x), at(baixo, x));
            let n = u8::from(a) + u8::from(b) + u8::from(c) + u8::from(d);
            cantos += match n {
                1 | 3 => 1,
                2 if a == d => 2,
                _ => 0,
            };
            arestas += u64::from(x < w && b != d) + u64::from(y < h && c != d);
            (a, c) = (b, d);
        }
        (cantos, arestas)
    };
    let mut buf = vec![0u8; (h + 1) * 16];
    let threads = crate::compositor::parallel_threads(w * h, h + 1);
    crate::compositor::em_faixas_de_linhas(&mut buf, 16, threads, |faixa, y0| {
        for (k, s) in faixa.as_chunks_mut::<16>().0.iter_mut().enumerate() {
            let (c, a) = linha(y0 + k);
            s[..8].copy_from_slice(&c.to_le_bytes());
            s[8..].copy_from_slice(&a.to_le_bytes());
        }
    });
    let u = |b: &[u8]| u64::from_le_bytes(b.try_into().unwrap_or([0; 8])) as usize;
    buf.as_chunks::<16>()
        .0
        .iter()
        .fold((0, 0), |(c, a), s| (c + u(&s[..8]), a + u(&s[8..])))
}

/// Direções: 0 = +x, 1 = +y, 2 = −x, 3 = −y (y para baixo).
const PASSO: [(i64, i64); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

/// Os contornos de `mask` (dentro = ≥ 128): cada componente 4-conexo e cada buraco vira uma polilinha
/// fechada exatamente sobre as arestas entre texels dentro/fora, com os colineares juntos. O dentro
/// fica à DIREITA de quem anda; num vértice em sela vira-se à direita (os dois blobs ficam separados).
#[must_use]
pub fn contornos_da_mascara(mask: &[u8], w: usize, h: usize) -> Vec<Polilinha> {
    if w == 0 || h == 0 || mask.len() != w * h {
        return Vec::new();
    }
    let dentro = |x: usize, y: usize| mask[y * w + x] >= 128;
    let vw = (w + 1) as u64;
    let v = |x: usize, y: usize| (y as u64) * vw + x as u64;
    // Arestas dirigidas: (vértice de partida << 2) | direção.
    let mut arestas: Vec<u64> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if !dentro(x, y) {
                continue;
            }
            if y == 0 || !dentro(x, y - 1) {
                arestas.push(v(x, y) << 2);
            }
            if x + 1 == w || !dentro(x + 1, y) {
                arestas.push((v(x + 1, y) << 2) | 1);
            }
            if y + 1 == h || !dentro(x, y + 1) {
                arestas.push((v(x + 1, y + 1) << 2) | 2);
            }
            if x == 0 || !dentro(x - 1, y) {
                arestas.push((v(x, y + 1) << 2) | 3);
            }
        }
    }
    arestas.sort_unstable();
    let mut usada = vec![false; arestas.len()];
    let achar = |a: u64| arestas.binary_search(&a).ok();
    let mut saida = Vec::new();
    for i0 in 0..arestas.len() {
        if usada[i0] {
            continue;
        }
        let d0 = (arestas[i0] & 3) as usize;
        let mut i = i0;
        let mut d = d0;
        let mut pontos: Polilinha = Vec::new();
        let ponto = |vert: u64| [(vert % vw) as f32, (vert / vw) as f32];
        pontos.push(ponto(arestas[i0] >> 2));
        loop {
            usada[i] = true;
            let vert = arestas[i] >> 2;
            let (dx, dy) = PASSO[d];
            let fim = (vert as i64 + dx + dy * vw as i64) as u64;
            // A regra escolhe a aresta; já usada = o laço fechou (numa sela a outra é de outro laço).
            let seguinte = [(d + 1) % 4, d, (d + 3) % 4]
                .into_iter()
                .find_map(|nd| achar((fim << 2) | nd as u64).map(|j| (j, nd)));
            let Some((j, nd)) = seguinte.filter(|&(j, _)| !usada[j]) else {
                break;
            };
            if nd != d {
                pontos.push(ponto(fim));
            }
            i = j;
            d = nd;
        }
        if d == d0 && pontos.len() > 1 {
            pontos.remove(0); // o início caiu no meio de um lado
        }
        saida.push(pontos);
    }
    saida
}

struct Chave {
    tamanho: (u32, u32),
    mascara: Weak<Vec<u8>>,
    niveis: Vec<Weak<Vec<u8>>>,
}

impl PartialEq for Chave {
    fn eq(&self, o: &Self) -> bool {
        self.tamanho == o.tamanho
            && self.mascara.ptr_eq(&o.mascara)
            && self.niveis.len() == o.niveis.len()
            && self.niveis.iter().zip(&o.niveis).all(|(a, b)| a.ptr_eq(b))
    }
}

thread_local! {
    static GUARDADO: RefCell<Option<(Chave, Arc<ContornosDaSelecao>)>> = const { RefCell::new(None) };
}

impl PainterTool {
    /// As máscaras dos níveis do Offset do tamanho da tela (o ciano).
    pub(super) fn niveis_do_offset_vivos(&self) -> Vec<Arc<Vec<u8>>> {
        let n = self.source_size.0 as usize * self.source_size.1 as usize;
        self.selection_offset_contour_levels()
            .iter()
            .filter_map(|&l| self.selection_offset_level_mask(l).cloned())
            .filter(|m| m.len() == n)
            .collect()
    }

    /// Os contornos da Seleção viva (máscara + níveis do Offset), guardados até a máscara ou os níveis
    /// trocarem de `Arc`. Acima de [`ORCAMENTO_DE_LADOS`] / [`ORCAMENTO_DE_ARESTAS`] não se traçam
    /// (`no_raster`). `None` sem seleção viva.
    #[must_use]
    pub fn selection_contornos(&self) -> Option<Arc<ContornosDaSelecao>> {
        if !self.paint.selection_active {
            return None;
        }
        let (w, h) = self.source_size;
        let (wu, hu) = (w as usize, h as usize);
        let mask = &self.paint.selection_mask;
        if wu == 0 || hu == 0 || mask.len() != wu * hu {
            return None;
        }
        let niveis = self.niveis_do_offset_vivos();
        let chave = Chave {
            tamanho: (w, h),
            mascara: Arc::downgrade(mask),
            niveis: niveis.iter().map(Arc::downgrade).collect(),
        };
        GUARDADO.with_borrow_mut(|g| {
            if g.as_ref().is_none_or(|(c, _)| *c != chave) {
                let (cantos, arestas) = std::iter::once(mask)
                    .chain(&niveis)
                    .map(|m| medida_da_fronteira(m, wu, hu))
                    .fold((0, 0), |(c, a), (dc, da)| (c + dc, a + da));
                let c = if cantos > ORCAMENTO_DE_LADOS || arestas > ORCAMENTO_DE_ARESTAS {
                    ContornosDaSelecao {
                        no_raster: true,
                        ..Default::default()
                    }
                } else {
                    ContornosDaSelecao {
                        mascara: contornos_da_mascara(mask, wu, hu),
                        niveis: niveis
                            .iter()
                            .flat_map(|m| contornos_da_mascara(m, wu, hu))
                            .collect(),
                        no_raster: false,
                    }
                };
                *g = Some((chave, Arc::new(c)));
            }
            g.as_ref().map(|(_, c)| Arc::clone(c))
        })
    }
}

#[cfg(test)]
#[path = "selection_contorno_tests.rs"]
mod tests;
