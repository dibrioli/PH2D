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
///
/// ⛔⛔ **E o `64` ERA o óptimo enquanto um ladrilho pequeno pagava uma COMPILAÇÃO a mais — reconferido
/// na W88.**
///
/// A W82 pôs uma cache de fitas entre quadros: a fita de um ladrilho pequeno passou a ser **reusada**,
/// e o termo que castigava os pequenos quase desapareceu. ⚠️ E a 1.ª reconferência (§88) ainda deu
/// `64`, porque o tecto FIXO da cache (`2 048` fitas) estrangulava exactamente os tamanhos pequenos —
/// *o «óptimo» era o maior ladrilho que ainda cabia no meu tecto*. Com o tecto **derivado do que o
/// quadro pede**, a resposta inverteu-se.
///
/// ⭐ **A varredura nova** (`measure_the_tile_size_now_that_the_cache_exists`, **intercalada** ×3,
/// com a cache aquecida por um arrasto em cada tamanho, quadro de movimento sem anti-serrilhado):
///
/// | | 16 | **24** | 32 | 48 | 64 | 96 |
/// |---|---:|---:|---:|---:|---:|---:|
/// | `640×360` 168 ar | `12,5` | **`13,3`** | `13,8` | `15,8` | `19,1` | `32,8` |
/// | `640×360` 672 ar | `38,2` | **`41,1`** | `43,8` | `53,3` | `62,1` | `111,5` |
/// | `1600×900` 168 ar | `62,6` | **`60,8`** | `58,2` | `61,8` | `66,5` | `76,7` |
///
/// ⭐⭐⭐ **`24` ship**, e o ganho contra o `64` é **`1,44×`** no caso do preview e **`1,51×`** na peça
/// pesada. O `16` ganha por `6 %` a `640×360` e **perde** a `1600×900`, e guarda o dobro das fitas —
/// *uma diferença de `6 %` não paga o dobro da memória, e a linha de baixo diz que ela nem é uma
/// diferença.*
///
/// ⚠️ **A tabela ANTIGA fica registada porque ela não estava errada** — ela media um mundo em que
/// montar uma fita se pagava uma vez por ladrilho **por quadro**. *Uma constante que se move é uma
/// medição a acontecer.*
pub(crate) const TILE: usize = 24;

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
/// ⛔⛔ **O `2` ERA o óptimo enquanto uma região custava o DOBRO — reconferido na W71.**
///
/// A W70 tirou a cada especialização a fita de gradiente que ninguém avalia e o `fork` que
/// recompilava o par: **metade do preço de montar**. Como repartir *multiplica* o custo de montar e
/// *divide* o de avaliar, baixar o preço de montar move o vale para **mais** fatias — e move mesmo.
/// *Quem move o número que sustenta uma nota tem de reconferir a nota* (CLAUDE.md §0.0).
///
/// ⭐ **A varredura nova** (`measure_where_the_frame_goes_and_how_many_slabs_it_wants`, tile `64`,
/// **intercalada** `N=2,3,4,6` × 3 rondas × mediana de 5, máquina a `load < 5`, quadro inteiro em
/// ms com anti-serrilhado):
///
/// | tamanho | arestas | N=2 | N=3 | **N=4** | N=6 |
/// |---|---:|---:|---:|---:|---:|
/// | `640×360` | 168 | `37,6` | `35,0` | **`34,6`** | `35,0` |
/// | `640×360` | 672 | `131,2` | `126,6` | `116,1` | **`114,9`** |
/// | `1920×1080` | 168 | `157,9` | `149,0` | `146,7` | **`143,5`** |
/// | `1920×1080` | 672 | `504,1` | `447,7` | `424,8` | **`415,9`** |
///
/// ⭐ **`4` ship porque o caso que o artista SENTE é o primeiro** — o quadro de movimento a
/// `640×360` na resolução de omissão, onde `4` ganha e `6` já volta a subir. Nos outros três `6` é
/// melhor por `1 %`–`2 %`, que é a largura do vale. Ganho: `1,09×` no caso do preview e **`1,19×`**
/// no mais pesado.
///
/// ⚠️ **A tabela ANTIGA (W56e) fica registada porque ela não estava errada** — ela media outro
/// preço: `círculo 168` dava `88/76/78/83/93/108` para `N=1..8`, com `2` a ganhar. *Uma constante
/// que se move é uma medição a acontecer.*
///
/// ⚠️ E o `TILE` **não** se moveu: a varredura por ladrilho a `640×360` com `N=4` dá
/// `32 → 48,8` · `48 → 37,8` · **`64 → 33,8`** · `96 → 39,9` · `128 → 55,6`.
pub(crate) const SLABS: usize = 4;

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
    cache: Option<&crate::TapeCache>,
) -> Gbuffer {
    let (w, h) = (plane.width() as usize, plane.height() as usize);
    let (out_w, out_h) = (plane.width() as u32, plane.height() as u32);
    // ⛔⛔ **RECUSA MEDIDA (W87): ordenar os ladrilhos CAROS PRIMEIRO é neutro a pior.**
    //
    // Um ladrilho é a unidade **indivisível** de trabalho, e o quadro não acaba antes do mais caro.
    // A §82.5 mediu que o mais caro vale `1,52×` a fatia perfeita, e a §89 mediu a perda de facto:
    // `32` quadros **independentes** (um por thread, cada um serial) escalam `17,0×` e **um** quadro
    // repartido pelas mesmas `32` threads escala `11,65×` ⇒ **a decomposição custa `1,47×`**, e o
    // trabalho existe: está mal repartido.
    //
    // ⛔ **Mas pôr os gordos à frente da fila não o cura.** A/B no MESMO processo (`tiles::LPT`):
    //
    // | threads | ordem natural | caros primeiro |
    // |---:|---:|---:|
    // | 4 | `81,87` | `81,34` |
    // | 16 | `30,78` | **`32,30`** |
    // | 32 | `25,62` | **`26,66`** |
    //
    // ⚠️ **A régua que eu usei — a PROFUNDIDADE da peça sob o ladrilho (`t_hi − t_lo`) — está
    // provavelmente ANTI-correlacionada com o custo:** um ladrilho no meio da peça é fundo e os
    // raios dele **acertam cedo**; o caro é o da **silhueta**, onde eles passam rasantes e dão
    // dezenas de passos. *Um escalonamento por um palpite de custo escalona pelo palpite.*
    //
    // ⏳ Fica aberto: uma régua de custo que sirva (a do quadro anterior, medida, é a candidata) —
    // e antes dela, saber se é mesmo a ordem que falta.
    let tiles: Vec<(usize, usize)> = (0..h.div_ceil(tile))
        .flat_map(|ty| (0..w.div_ceil(tile)).map(move |tx| (tx, ty)))
        .collect();
    let body = |&(tx, ty): &(usize, usize)| -> TileResult {
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
            stencil: scene.stencil,
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
            // ⭐⭐⭐ **A CACHE ENTRE QUADROS** (W82) — ver [`crate::TapeCache`]. Uma fita
            // construída para uma caixa serve toda a sub-caixa dela, então a pergunta não é *«qual
            // é a chave desta região?»* mas *«há alguma fita cuja caixa a contenha?»*.
            if let Some(c) = cache
                && let Some(t) = c.get(r.lo, r.hi)
            {
                return Some(ph2d_field_eval::hybrid::Hybrid::from_region_tape(&t));
            }
            SPECIALISED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            SLAB_SPEC[k.min(crate::SLABS_COUNTED - 1)]
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let t0 = std::time::Instant::now();
            let tape = match cache {
                // ⚠️ **Com cache a região é a CAIXA INFLADA, e não o casco do tubo.** Duas razões,
                // e as duas são sobre a cache e não sobre a marcha: a caixa é a forma que se testa
                // depressa e sem ambiguidade, e a inflação é o que faz a fita sobreviver ao quadro
                // seguinte (a `f = 1` a cache acerta `9 %`). O preço está medido no doc do módulo.
                Some(c) => {
                    // ⚠️ A semente é a IDENTIDADE da região (ladrilho × fatia), **estável entre
                    // quadros** — ver [`crate::tape_cache::PHASE`]. Uma semente que mudasse por
                    // quadro punha a caixa noutro sítio a cada compilação, e a dispersão das
                    // coortes virava ruído.
                    let seed = ((x0 as u64) << 40) ^ ((y0 as u64) << 20) ^ (k as u64);
                    let (lo, hi) = crate::tape_cache::inflate_phased(
                        r.lo,
                        r.hi,
                        c.inflate_of(),
                        seed,
                        c.phase_of(),
                    );
                    let t = ph2d_field_eval::hybrid::RegionTape::compile(rc.compile(doc, lo, hi));
                    c.insert(lo, hi, t.clone());
                    ph2d_field_eval::hybrid::Hybrid::from_region_tape(&t)
                }
                None => ph2d_field_eval::hybrid::Hybrid::from_tree(
                    rc.compile_at(doc, r.lo, r.hi, &r.pts),
                ),
            };
            SPECIALISE_NS.fetch_add(
                u64::try_from(t0.elapsed().as_nanos()).unwrap_or(u64::MAX),
                std::sync::atomic::Ordering::Relaxed,
            );
            Some(tape)
        });
        (idx, hit, normal)
    };
    // ⭐ **O ladrilho mais caro** — ver [`TILE_MAX`]. Dois carregamentos e um `fetch_max` por
    // ladrilho, contra os milhares de amostras que ele acabou de dar.
    let one = |t: &(usize, usize)| -> TileResult {
        let spent = || {
            crate::STEP_SAMPLES.load(std::sync::atomic::Ordering::Relaxed)
                + crate::NORMAL_SAMPLES.load(std::sync::atomic::Ordering::Relaxed)
        };
        let before = spent();
        let r = body(t);
        let custo = spent() - before;
        TILE_MAX.fetch_max(custo, std::sync::atomic::Ordering::Relaxed);
        if RECORD_TILE_COSTS.load(std::sync::atomic::Ordering::Relaxed)
            && let Ok(mut v) = TILE_COSTS.lock()
        {
            v.push((t.1 * w.div_ceil(tile) + t.0, custo));
        }
        r
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

/// ⭐⭐⭐ **Quanto TEMPO a especialização custou, em nanossegundos** (W71) — o numerador da fracção
/// que decide se ainda vale a pena atacar a montagem.
///
/// ⚠️ **Ele existe porque a mesma tabela admitia duas leituras que diferem por `3×`.** O A/B da W70
/// removeu `132` fitas float **e** `293` de gradiente e ganhou `27,2 ms`: dividir por `132` diz que
/// a montagem que sobra é `79 %` do quadro; dividir por `425` diz `25 %`. As duas mandam em waves
/// diferentes — *cache entre quadros* contra *atacar a marcha* — e nenhuma delas é uma medição.
///
/// ⚠️ Lê-se num traçado **serial**: a soma é de CPU, e só contra um relógio de parede serial é que
/// ela é uma fracção. Um `Instant::now()` por região custa ~25 ns contra `~1,3 ms` de trabalho.
#[doc(hidden)]
pub static SPECIALISE_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// ⭐ **Quantas ÁRVORES o traçado especializou** — o contador que uma sonda lê.
///
/// ⚠️ **Ele existe porque a contagem foi ADIVINHADA e a adivinha estava errada.** Uma sonda mediu
/// `60` especializações por quadro (a contagem de ladrilhos a `D=6`) e concluiu que a montagem
/// custava `245 ms`; o produto compila **preguiçosamente**, e só as fatias que algum raio alcança.
/// *Uma sonda que assume a contagem mede a sua própria suposição.*
///
/// ⚠️ Um incremento atómico por especialização é ruído ao lado dos milissegundos que ela custa.
#[doc(hidden)]
pub static SPECIALISED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// ⭐⭐⭐ **As amostras do LADRILHO MAIS CARO do quadro** (W81) — e ele é o chão do relógio.
///
/// ⚠️ **Um ladrilho é a unidade indivisível de trabalho**: ele monta a própria fita e marcha os
/// próprios raios, e nenhuma thread o pode partir. ⇒ por mais threads que existam, o quadro **não
/// pode acabar antes** deste ladrilho. Com `T` threads o limite inferior do relógio é
///
/// ```text
/// makespan >= max(total / T, mais_caro)
/// ```
///
/// e a fronteira de cima de um roubo de trabalho é `total / T + mais_caro`.
///
/// ⭐⭐ **É a grandeza que uma varredura SERIAL não pode ver**, e as três constantes deste módulo
/// (`TILE`, [`SLABS`] e a decisão de especializar) foram todas escolhidas em varreduras seriais: ali
/// só o **trabalho total** conta, e desequilíbrio nenhum existe. *Uma constante escolhida num modelo
/// de máquina que o produto não corre é um palpite com tabela ao lado.*
///
/// ⚠️ **Lê-se num traçado SERIAL**, como a [`SPECIALISE_NS`]: a diferença dos contadores globais em
/// torno de um ladrilho só é a conta dele quando ninguém mais escreve entretanto. As contagens não
/// dependem do escalonamento, então o número medido em série **é** o do quadro paralelo.
#[doc(hidden)]
pub static TILE_MAX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// ⚠️ Só para a sonda: gravar o custo de **cada** ladrilho, e não só o do pior.
///
/// ⭐ Ele existe para uma pergunta que só o **oráculo** responde: *se os ladrilhos fossem
/// escalonados pelo custo VERDADEIRO, o `1,47×` da §89 desaparecia?* Provar a direcção com o custo
/// medido é mais barato — e mais honesto — do que construir um estimador e descobrir depois que a
/// ordem não era o mecanismo. *Simule antes de construir.*
///
/// ⚠️ **Lê-se num traçado SERIAL**, como o [`TILE_MAX`]: a diferença dos contadores globais em torno
/// de um ladrilho só é a conta dele quando ninguém mais escreve entretanto.
#[doc(hidden)]
pub static RECORD_TILE_COSTS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// ⚠️ Só para a sonda — ver [`RECORD_TILE_COSTS`]. Cada entrada é `(índice do ladrilho, amostras)`.
#[doc(hidden)]
pub static TILE_COSTS: std::sync::Mutex<Vec<(usize, u64)>> = std::sync::Mutex::new(Vec::new());

/// ⭐⭐⭐ **Quantas árvores cada FATIA DE PROFUNDIDADE especializou** (W81) — ver
/// [`crate::SLAB_SAMPLES`], que é o outro lado da mesma pergunta.
///
/// ⚠️ **A montagem é `20 %` do quadro e paga-se por FATIA CONSTRUÍDA**, não por amostra. Uma fatia
/// que compila uma fita para servir vinte raios custa o mesmo que uma que serve vinte mil — e o
/// [`SPECIALISED`] soma-as todas num número só, que não distingue as duas.
#[doc(hidden)]
pub static SLAB_SPEC: [std::sync::atomic::AtomicU64; crate::SLABS_COUNTED] =
    [const { std::sync::atomic::AtomicU64::new(0) }; crate::SLABS_COUNTED];
