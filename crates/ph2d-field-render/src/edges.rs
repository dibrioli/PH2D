//! **A segunda passagem: a borda re-amostrada.**
//!
//! Irmão do [`crate`] por responsabilidade (teto de LOC): o `lib.rs` fica com a marcha, este com o
//! que decide **quais** pixels são silhueta e os re-amostra no padrão [`crate::ROOK`].

use crate::{EDGE_COS, EdgePixel, ROOK, Scene, Screen, march};
use rayon::prelude::*;

/// Quantos **pixels** de borda cada lote da segunda passagem leva (× 4 amostras cada).
///
/// ⚠️ **Pequeno de propósito, e o número saiu de um erro medido.** A primeira versão usava 4096, com
/// o raciocínio de "grande o bastante para o custo de montar um `tape` desaparecer" — e a medição
/// disse que um raio de borda custava **73×** um raio comum, o que é absurdo: os dois marcham o
/// mesmo campo. O 73 não era o preço do raio, era o preço de **três lotes numa máquina de 32
/// núcleos**: a segunda passagem corria com 3 threads enquanto a primeira usava todas.
///
/// *Um número de paralelismo dimensionado por uma intuição sobre overhead, e não pela contagem de
/// lotes que ele produz, é um `for` sequencial com um `par_` na frente.*
///
/// # ⛔⛔ DUAS RECUSAS MEDIDAS sobre esta passagem — não as reconstrua
///
/// 1. **Especializar a árvore por ladrilho aqui é neutro a pior** (W64): `29,0 → 36,4 ms` a 64
///    arestas. *A especialização paga-se por AMORTIZAÇÃO* — a marcha primária dilui a montagem por
///    `4 096` raios de ladrilho, esta por `~256`.
/// 2. **Reaproveitar o avaliador entre lotes é neutro** (W70): medido `0,97×`–`1,01×` a `640×360` e
///    a `1920×1080`, nas duas formas — o `map_init` da rayon (que **não** compra nada: ela parte um
///    `par_chunks` até *uma tarefa por lote*, e o `init` corre uma vez por tarefa) e a tarefa
///    dimensionada por `pixels / threads` (que corta as fitas de `1000` para `950` a `1920×1080` e
///    **não** move o relógio). ⚠️ *O quadro tem `917` regiões especializadas nesse tamanho: as
///    dezenas de fitas desta passagem são ruído ao lado delas.* A cura que **pagou** foi noutro
///    sítio — ver `Hybrid::grad` e a marcha por fatias.
const EDGE_CHUNK: usize = 64;

pub(crate) fn resample_edges(
    scene: &Scene<'_>,
    plane: Screen,
    hit: &[bool],
    normal: &[[f32; 3]],
    parallel: bool,
) -> Vec<EdgePixel> {
    let (w, h) = (plane.width() as usize, plane.height() as usize);
    let differs = |a: usize, b: usize| -> bool {
        if hit[a] != hit[b] {
            return true;
        }
        if !hit[a] {
            return false;
        }
        let (p, q) = (normal[a], normal[b]);
        p[0] * q[0] + p[1] * q[1] + p[2] * q[2] < EDGE_COS
    };

    let mut is_edge = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            // Só direita e baixo: a aresta é uma relação entre DOIS pixels, e marcar os dois quando
            // ela aparece cobre os quatro vizinhos sem os visitar duas vezes.
            if x + 1 < w && differs(i, i + 1) {
                is_edge[i] = true;
                is_edge[i + 1] = true;
            }
            if y + 1 < h && differs(i, i + w) {
                is_edge[i] = true;
                is_edge[i + w] = true;
            }
        }
    }

    let pixels: Vec<u32> = (0..w * h)
        .filter(|i| is_edge[*i])
        .map(|i| i as u32)
        .collect();
    if pixels.is_empty() {
        return Vec::new();
    }

    let chunk = |c: &[u32]| -> Vec<EdgePixel> {
        let mut pts = Vec::with_capacity(c.len() * 4);
        for &p in c {
            let (x, y) = ((p as usize % w) as f32, (p as usize / w) as f32);
            for (dx, dy) in ROOK {
                pts.push(plane.plane_at(x + dx, y + dy));
            }
        }
        let (hits, normals, _) = march(scene, &pts);
        c.iter()
            .enumerate()
            .map(|(k, &p)| {
                let b = k * 4;
                EdgePixel {
                    pixel: p,
                    hit: [hits[b], hits[b + 1], hits[b + 2], hits[b + 3]],
                    normal: [normals[b], normals[b + 1], normals[b + 2], normals[b + 3]],
                }
            })
            .collect()
    };

    // ⚠️ `chunks` preserva a ordem em `collect()` mesmo em paralelo — é isso que mantém `edges`
    // ordenado por `pixel` e a saída independente de como as threads se dividiram (ADR-0109).
    let out: Vec<Vec<EdgePixel>> = if parallel {
        pixels.par_chunks(EDGE_CHUNK).map(chunk).collect()
    } else {
        pixels.chunks(EDGE_CHUNK).map(chunk).collect()
    };
    out.concat()
}
