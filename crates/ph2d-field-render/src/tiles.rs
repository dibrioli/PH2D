//! ⭐⭐⭐ **A marcha por LADRILHO** (W56) — o consumidor da especialização por região.
//!
//! O irmão do [`crate`]: aqui mora tudo o que reparte a tela em pedaços e constrói uma árvore por
//! pedaço. Saiu do `lib.rs` quando ele passou o teto de LOC — e o corte é por responsabilidade, que
//! é o que o `architecture_workspace_file_loc_cap` pede: o `lib.rs` fica com **a marcha**, este
//! ficheiro com **o repartir**.

use crate::{Gbuffer, Orbit, Scene, Screen, T_MAX, march, resample_edges, slab};
use ph2d_field::FieldDoc;
use rayon::prelude::*;

/// O que um ladrilho devolve: os índices dos pixels dele, a máscara e as normais.
type TileResult = (Vec<usize>, Vec<bool>, Vec<[f32; 3]>);

/// O lado de um ladrilho, em pixels — **medido, não escolhido**.
///
/// ⚠️ **É o vale entre duas contas que puxam para lados opostos:** a fita especializada custa menos
/// quanto **menor** for a pegada do ladrilho, e a montagem dela paga-se uma vez **por ladrilho**.
/// A varredura (`the_table_of_what_the_tiled_march_buys`, 640×480, mediana de 7, o quadro inteiro em
/// ms):
///
/// | arestas | 32 | 48 | **64** | 96 | 128 | 192 |
/// |---:|---:|---:|---:|---:|---:|---:|
/// | 56 | 61 | **43** | 44 | 47 | 50 | 139 |
/// | 168 | 143 | 102 | **90** | 93 | 130 | 216 |
/// | 664 | 523 | 362 | **330** | 355 | 487 | 838 |
///
/// ⭐ O vale é raso entre **48 e 96** e fundo fora dele: a 32 px a montagem domina (400 fitas por
/// quadro), a 192 px a pegada volta a trazer o contorno quase inteiro. Da tabela sai também o
/// modelo — **≈ 0,29 ms de montagem por ladrilho**, e o resto é avaliação.
pub(crate) const TILE: usize = 64;

/// ⭐⭐⭐ **A marcha por ladrilho, com uma árvore por região** — ver o `TILE`.
///
/// ⚠️ **A região é a caixa do FRUSTUM do ladrilho intersectada com a da peça**, e a marcha de cada
/// raio é presa à caixa da peça (`Scene::clip`). As duas metades juntas são o que torna a árvore
/// especializada válida em **todo** ponto avaliado: nenhuma amostra cai fora da região para que ela
/// foi construída.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tiled_trace(
    doc: &FieldDoc,
    rc: &ph2d_field_eval::RegionCompiler,
    scene: &Scene<'_>,
    plane: Screen,
    bbox: ([f32; 3], [f32; 3]),
    parallel: bool,
    antialias: bool,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    tile: usize,
) -> Gbuffer {
    let (w, h) = (plane.width() as usize, plane.height() as usize);
    let (out_w, out_h) = (plane.width() as u32, plane.height() as u32);
    let tiles: Vec<(usize, usize)> = (0..h.div_ceil(tile))
        .flat_map(|ty| (0..w.div_ceil(tile)).map(move |tx| (tx, ty)))
        .collect();
    let one = |&(tx, ty): &(usize, usize)| -> TileResult {
        let (x0, y0) = (tx * tile, ty * tile);
        let (x1, y1) = ((x0 + tile).min(w), (y0 + tile).min(h));
        let mut idx = Vec::with_capacity((x1 - x0) * (y1 - y0));
        let mut pts = Vec::with_capacity(idx.capacity());
        for y in y0..y1 {
            for x in x0..x1 {
                idx.push(y * w + x);
                pts.push(plane.plane_at(x as f32 + 0.5, y as f32 + 0.5));
            }
        }
        let empty = (
            idx.len(),
            vec![false; idx.len()],
            vec![[0.0f32; 3]; idx.len()],
        );
        if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            return (idx, empty.1, empty.2);
        }
        // A caixa do frustum do ladrilho: os quatro raios de canto, de `t = 0` a `T_MAX`.
        let Some(region) = tile_region(
            scene.cam,
            plane,
            (x0, y0),
            (x1, y1),
            bbox,
            scene.sharp.normal,
        ) else {
            // A caixa do ladrilho não cruza a da peça ⇒ nenhum raio dele acerta em nada.
            return (idx, empty.1, empty.2);
        };
        let tree = rc.compile(doc, region.0, region.1);
        let local = ph2d_field_eval::hybrid::Hybrid::from_tree(tree);
        let tile_scene = Scene {
            shape: &local,
            cam: scene.cam,
            basis: scene.basis,
            sharp: scene.sharp,
            clip: Some(bbox),
        };
        let (hit, normal, _) = march(&tile_scene, &pts);
        (idx, hit, normal)
    };
    let done: Vec<TileResult> = if parallel {
        tiles.par_iter().map(one).collect()
    } else {
        tiles.iter().map(one).collect()
    };
    let mut hit = vec![false; w * h];
    let mut normal = vec![[0.0f32; 3]; w * h];
    for (idx, th, tn) in done {
        for (k, &i) in idx.iter().enumerate() {
            hit[i] = th[k];
            normal[i] = tn[k];
        }
    }
    let edges =
        if antialias && !cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            resample_edges(scene, plane, &hit, &normal, parallel)
        } else {
            Vec::new()
        };
    Gbuffer {
        width: out_w,
        height: out_h,
        hit,
        normal,
        edges,
    }
}

/// A caixa de mundo que contém tudo o que os raios deste ladrilho podem amostrar **dentro da peça**.
///
/// ⚠️ **Os quatro raios de CANTO bastam, e não é aproximação:** o frustum de um ladrilho é o casco
/// convexo dos quatro segmentos de canto (a lente é convergente ou paralela, e nas duas o ladrilho é
/// um quadrilátero plano), então a caixa dos oito extremos contém todo raio interior.
/// ⚠️ **`margin` não é folga de conforto — é a SONDA DA NORMAL.** Ela é uma diferença central em
/// `ponto ± ε`, e um `ε` que saia da região faria a árvore especializada responder onde ela não vale.
/// O sintoma medido: 90 pixels a **apagarem-se** (o gradiente saía nulo e a marcha desistia do
/// acerto), num quadro em que a máscara devia ser idêntica. *Uma região tem de conter tudo o que é
/// avaliado — inclusive o que é avaliado DEPOIS de o raio parar.*
pub(crate) fn tile_region(
    cam: &Orbit,
    plane: Screen,
    lo_px: (usize, usize),
    hi_px: (usize, usize),
    bbox: ([f32; 3], [f32; 3]),
    margin: f32,
) -> Option<([f32; 3], [f32; 3])> {
    // ⭐⭐ **A faixa de `t` é a da CAIXA, não `[0, T_MAX]`.**
    //
    // ⛔ **Medido:** com `T_MAX` o tubo do ladrilho é tão comprido que a caixa dele engole a peça
    // inteira — a região de **todo** ladrilho saía sendo a peça, e a especialização comprava `1,3×`
    // em vez dos `5×` que a tabela do §57.12 prometia. *Uma região que não é menor que a peça não é
    // uma região.*
    //
    // ⚠️ **E os extremos estão nos cantos**, o que torna quatro raios suficientes: a entrada e a
    // saída da caixa são máximo e mínimo de funções lineares sobre o quadrilátero do ladrilho.
    let corners = [
        (lo_px.0 as f32, lo_px.1 as f32),
        (hi_px.0 as f32, lo_px.1 as f32),
        (lo_px.0 as f32, hi_px.1 as f32),
        (hi_px.0 as f32, hi_px.1 as f32),
    ];
    let (mut t_lo, mut t_hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for (px, py) in corners {
        let (sx, sy) = plane.plane_at(px, py);
        let (o, d) = cam.ray_at_plane(sx, sy);
        if let Some((a, b)) = slab(o, d, bbox.0, bbox.1) {
            t_lo = t_lo.min(a.max(0.0));
            t_hi = t_hi.max(b.min(T_MAX));
        }
    }
    if !t_lo.is_finite() || t_lo > t_hi {
        // Nenhum raio de canto alcança a caixa. ⚠️ Um raio INTERIOR ainda pode, então o que se faz é
        // **desistir da especialização** — nunca dar o ladrilho por vazio.
        return Some(bbox);
    }
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for (px, py) in corners {
        let (sx, sy) = plane.plane_at(px, py);
        let (o, d) = cam.ray_at_plane(sx, sy);
        for t in [t_lo, t_hi] {
            for k in 0..3 {
                let v = d[k].mul_add(t, o[k]);
                lo[k] = lo[k].min(v);
                hi[k] = hi[k].max(v);
            }
        }
    }
    // …intersectada com a da peça: fora dela não há superfície nenhuma.
    let mut out = ([0.0f32; 3], [0.0f32; 3]);
    let pad = margin * 4.0;
    for k in 0..3 {
        out.0[k] = lo[k].max(bbox.0[k]);
        out.1[k] = hi[k].min(bbox.1[k]);
        if out.0[k] > out.1[k] {
            return None;
        }
        out.0[k] -= pad;
        out.1[k] += pad;
    }
    Some(out)
}
