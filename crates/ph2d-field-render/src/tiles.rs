//! ⭐⭐⭐ **A marcha por LADRILHO** (W56) — o consumidor da especialização por região.
//!
//! O irmão do [`crate`]: aqui mora tudo o que reparte a tela em pedaços e constrói uma árvore por
//! pedaço. Saiu do `lib.rs` quando ele passou o teto de LOC — e o corte é por responsabilidade, que
//! é o que o `architecture_workspace_file_loc_cap` pede: o `lib.rs` fica com **a marcha**, este
//! ficheiro com **o repartir**.

use crate::march::{Scene, march, march_slabs};
use crate::{Gbuffer, Orbit, Screen, T_MAX, resample_edges, slab};
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

/// ⭐⭐⭐ **Em quantas FATIAS DE PROFUNDIDADE o tubo de um ladrilho se reparte** (W56e) — medido.
///
/// ⚠️ **É o segundo eixo, e o único que sobra.** Ver [`crate::march::march_slabs`]: a região de um
/// ladrilho mede `lado + profundidade · |direcção|`, e encolher o **lado** não toca no segundo
/// termo — foi por isso que a varredura do [`TILE`] viu um vale e não uma descida.
///
/// A conta puxa para os dois lados: repartir **divide** o custo de avaliar (arestas guardadas por
/// região: `128,6` a `N = 1`, `67,2` a `N = 4`, `53,4` a `N = 8`) e **multiplica** o de montar (a
/// soma sobre as fatias: `1,00×`, `2,09×`, `3,32×`), que é 96% JIT.
///
/// ⭐ **A varredura (640×480, mediana de 5, `ms/pixels ≠ da marcha de linha`):**
///
/// | contorno | linha | N=1 | **N=2** | N=3 | N=4 | N=6 | N=8 |
/// |---|---:|---:|---:|---:|---:|---:|---:|
/// | círculo 56 | 63 | 38/0 | **34/1** | 41/2 | 44/1 | 57/2 | 68/2 |
/// | círculo 168 | 147 | 88/0 | **76/0** | 78/0 | 83/0 | 93/0 | 108/0 |
/// | estrela 168 | 220 | 195/0 | **181/1** | 187/2 | 182/2 | 199/2 | 213/4 |
/// | círculo 664 | 514 | 328/0 | **265/2** | 276/2 | 264/2 | 306/2 | 330/3 |
///
/// ⭐ **`2` é o melhor ou empata nas quatro**, e o vale é raso entre 2 e 4 — acima disso a montagem
/// domina. ⚠️ **E o ganho é `1,08×–1,24×`, não os `5×` que a nota da W56d prometia**: a nota lia o
/// mecanismo certo (a pegada) e errava o preço, porque a montagem é JIT e cresce com a soma.
pub(crate) const SLABS: usize = 2;

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
    slabs: usize,
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
        let tile_scene = Scene {
            shape: scene.shape,
            cam: scene.cam,
            basis: scene.basis,
            sharp: scene.sharp,
            clip: Some(bbox),
            step: scene.step,
        };
        // ⭐⭐⭐ **AS FRONTEIRAS DAS FATIAS** (W56e) — e as duas de FORA são o que torna isto
        // correcto sem uma premissa.
        //
        // ⚠️ A faixa `[t_lo, t_hi]` sai dos quatro raios de CANTO, e o `t` de entrada na caixa é
        // uma função **convexa** da posição de ecrã: o mínimo dela pode ser INTERIOR ao ladrilho,
        // e um raio interior pode portanto entrar antes de `t_lo`. ⇒ a 1ª e a última fatia vão de
        // `0` e até `T_MAX`, o que torna a cobertura das fronteiras **trivialmente** completa; e
        // como a montagem é preguiçosa, elas custam **zero** quando ninguém lá chega. *A cerca não
        // é uma afirmação sobre onde os raios entram, é a ausência de uma.*
        let Some((t_lo, t_hi)) = tile_t_range(scene.cam, plane, (x0, y0), (x1, y1), bbox) else {
            // Nenhum raio de canto alcança a peça — não há o que especializar, e desistir é a
            // resposta segura (um raio INTERIOR ainda pode acertar).
            let (hit, normal, _) = march(&tile_scene, &pts);
            return (idx, hit, normal);
        };
        // ⚠️ **O que mutação nenhuma mata aqui, e por quê.** Fazer este `shape_of` devolver
        // sempre `None` desliga a especialização inteira — e a imagem sai **idêntica**, porque o
        // documento não especializado é a resposta certa em todo o lado. É um defeito **só de
        // relógio**, a mesma família do «a região era a peça inteira» da W56d, e nenhum gate de
        // paridade o pode ver. Quem o defende é a tabela medida
        // (`the_table_of_how_many_depth_slabs`), que é relógio por natureza. *A afirmação encolhe
        // até ao que a máquina faz: a paridade prova a IMAGEM, a tabela prova o PREÇO.*
        let bounds = slab_bounds(t_lo, t_hi, slabs);
        let (hit, normal, _) = march_slabs(&tile_scene, &pts, &bounds, &mut |k| {
            let r = slab_region(
                scene.cam,
                plane,
                (x0, y0),
                (x1, y1),
                bbox,
                scene.sharp.normal,
                &bounds,
                k,
            )?;
            Some(ph2d_field_eval::hybrid::Hybrid::from_tree(
                rc.compile_at(doc, r.lo, r.hi, &r.pts),
            ))
        });
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

/// ⭐⭐⭐ **AS FRONTEIRAS DAS FATIAS DE UM LADRILHO** (W56e) — e as duas de FORA são o que torna a
/// marcha correcta **sem uma premissa**.
///
/// ⚠️ A faixa `[t_lo, t_hi]` sai dos quatro raios de CANTO, e o `t` de entrada na caixa é `max` de
/// funções afins da posição de ecrã ⇒ **convexo** ⇒ o mínimo dele pode ser **interior** ao
/// ladrilho. ⛔ **Medido:** um raio interior entra até `7,4e-2` **antes** de `t_lo` e sai até
/// `1,2e-1` depois de `t_hi`, sobre uma peça que mede `1,0`. ⇒ a 1.ª fatia começa em `0` e a última
/// acaba em `T_MAX`, o que torna a cobertura **trivialmente** completa; e como a montagem é
/// preguiçosa, elas custam **zero** quando ninguém lá chega. *A cerca não é uma afirmação sobre
/// onde os raios entram: é a ausência de uma.*
///
/// ⚠️ **Uma função só, e o gate chama ESTA.** A 1.ª versão do
/// `every_sample_lies_inside_the_region_that_built_its_tape` reconstruía as fronteiras dentro do
/// teste — e três mutações que apagavam as fronteiras de fora **sobreviveram**, porque mexiam na
/// cópia do produto enquanto o gate media a dele. *Duas cópias de uma lei é uma lei que gate nenhum
/// defende.*
pub(crate) fn slab_bounds(t_lo: f32, t_hi: f32, slabs: usize) -> Vec<f32> {
    let mut bounds: Vec<f32> = Vec::with_capacity(slabs + 3);
    bounds.push(0.0);
    for k in 0..=slabs {
        bounds.push(t_lo + (t_hi - t_lo) * k as f32 / slabs as f32);
    }
    bounds.push(T_MAX);
    bounds
}

/// A região da fatia `k` — a mesma porta para o produto e para o gate, pela mesma razão.
#[allow(clippy::too_many_arguments)]
pub(crate) fn slab_region(
    cam: &Orbit,
    plane: Screen,
    lo_px: (usize, usize),
    hi_px: (usize, usize),
    bbox: ([f32; 3], [f32; 3]),
    margin: f32,
    bounds: &[f32],
    k: usize,
) -> Option<Region> {
    let (a, b) = (*bounds.get(k)?, *bounds.get(k + 1)?);
    (b > a).then(|| region_between(cam, plane, lo_px, hi_px, bbox, margin, a, b))?
}

/// ⭐⭐ **A faixa de `t` que o ladrilho inteiro ocupa dentro da caixa da peça**, ou `None` se
/// nenhum raio de canto a alcança. Ver [`tile_region`] — e é ela que a marcha por FATIA reparte.
pub(crate) fn tile_t_range(
    cam: &Orbit,
    plane: Screen,
    lo_px: (usize, usize),
    hi_px: (usize, usize),
    bbox: ([f32; 3], [f32; 3]),
) -> Option<(f32, f32)> {
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
    (t_lo.is_finite() && t_lo <= t_hi).then_some((t_lo, t_hi))
}

/// A caixa de mundo varrida pelo ladrilho **entre `t0` e `t1`** — ver [`tile_region`].
///
/// ⭐⭐⭐ **É a porta da marcha por FATIA DE PROFUNDIDADE** (W56e): a mesma conta, com a faixa
/// repartida. Uma fatia varre menos `(u, v)` do que o tubo inteiro, e é exactamente essa varredura
/// que decide quantas arestas a árvore especializada tem de guardar.
///
/// # ⛔ Os quatro cantos NÃO bastam na lente convergente — e a prova que os salva
///
/// O doc do [`tile_region`] dizia *"e não é aproximação"*. É, e só na **paralela** é exacta: lá a
/// direcção é constante, o ponto de entrada é bilinear na posição de ecrã, e um raio interior é
/// combinação convexa dos quatro cantos. Na convergente a direcção é **normalizada**, então
/// `d̂(s)` percorre um quadrilátero **esférico** que abaúla para fora da corda dos quatro cantos.
/// ⛔ **Medido** (`a_tile_region_contains_the_rays_inside_it`, câmera de frente): a fuga vai de
/// `2,80e-4` com uma fatia a `4,03e-4` com oito — e **passa a folga** (`4e-4`) exactamente quando
/// a fatia aperta. *A premissa não mordia porque o tubo era grande; fatiar é o que a acorda.*
///
/// ⭐ **A cura tem prova, e custa dois produtos internos.** Todo ponto a parâmetro `t` está sobre a
/// esfera de raio `t` em torno do olho, dentro do cone do ladrilho; a distância dele à **corda**
/// dos quatro cantos no mesmo `t` é no máximo a **flecha** `t · (1 − cos α)`, com `α` o ângulo
/// entre o raio central e o canto mais afastado. E `hull{p_j(t)} ⊆ hull{p_j(t₀), p_j(t₁)}` porque
/// `p_j(t)` é linear em `t` ⇒ a caixa dos oito pontos contém a corda em toda a faixa. Inflar essa
/// caixa por `t₁ · (1 − cos α)` contém, portanto, **todo** raio interior. ⚠️ Na paralela `α = 0` e
/// a inflação é **exactamente zero** — a lente sem o defeito não paga por ele.
#[allow(clippy::too_many_arguments)]
pub(crate) fn region_between(
    cam: &Orbit,
    plane: Screen,
    lo_px: (usize, usize),
    hi_px: (usize, usize),
    bbox: ([f32; 3], [f32; 3]),
    margin: f32,
    t_lo: f32,
    t_hi: f32,
) -> Option<Region> {
    let corners = [
        (lo_px.0 as f32, lo_px.1 as f32),
        (hi_px.0 as f32, lo_px.1 as f32),
        (lo_px.0 as f32, hi_px.1 as f32),
        (hi_px.0 as f32, hi_px.1 as f32),
    ];
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    // O raio CENTRAL do ladrilho — o eixo do cone contra o qual a flecha se mede.
    let (cx, cy) = plane.plane_at(
        (lo_px.0 + hi_px.0) as f32 * 0.5,
        (lo_px.1 + hi_px.1) as f32 * 0.5,
    );
    let (_, axis) = cam.ray_at_plane(cx, cy);
    let mut cos_a = 1.0f32;
    for (px, py) in corners {
        let (sx, sy) = plane.plane_at(px, py);
        let (o, d) = cam.ray_at_plane(sx, sy);
        cos_a = cos_a.min(d[0] * axis[0] + d[1] * axis[1] + d[2] * axis[2]);
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
    // ⭐⭐ **Os cantos CRUS do tubo desta fatia** (W59) — é deles que sai o casco em `(u, v)`, e é
    // por isso que eles viajam ao lado da caixa em vez de serem redescobertos do outro lado: quem
    // especializa recebe o MUNDO, não a câmera.
    let mut pts: Vec<[f32; 3]> = Vec::with_capacity(8);
    for (px, py) in corners {
        let (sx, sy) = plane.plane_at(px, py);
        let (o, d) = cam.ray_at_plane(sx, sy);
        for t in [t_lo, t_hi] {
            pts.push([
                d[0].mul_add(t, o[0]),
                d[1].mul_add(t, o[1]),
                d[2].mul_add(t, o[2]),
            ]);
        }
    }
    // ⭐ A folga da sonda da normal **mais** a flecha do cone — ver o doc acima.
    let pad = margin * 4.0 + t_hi.abs() * (1.0 - cos_a).max(0.0);
    for k in 0..3 {
        out.0[k] = lo[k].max(bbox.0[k]);
        out.1[k] = hi[k].min(bbox.1[k]);
        if out.0[k] > out.1[k] {
            return None;
        }
        out.0[k] -= pad;
        out.1[k] += pad;
    }
    Some(Region {
        lo: out.0,
        hi: out.1,
        pts,
    })
}

/// ⭐⭐ **A região de uma fatia** (W59) — a caixa **e** a forma real.
///
/// ⚠️ **As duas, e não uma:** a caixa é o que o recorte da marcha, o `Revolve` e o sinal consomem;
/// os pontos são a pegada real do tubo, e só a **distância** de um `Extrude` a usa. Trocar a caixa
/// pelos pontos obrigaria os outros três a redescobri-la.
#[derive(Clone, Debug)]
pub(crate) struct Region {
    pub(crate) lo: [f32; 3],
    pub(crate) hi: [f32; 3],
    /// Os oito cantos do tubo, **crus** — a folga é somada por `ph2d_field_eval::hull_uv`.
    pub(crate) pts: Vec<[f32; 3]>,
}
