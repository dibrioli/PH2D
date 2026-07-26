//! **T0 — o retângulo sujo SUPER-REIVINDICA? E vale atacar isso?**
//!
//! A medição que ABRIU a frente T do [plano 26](../../../../../docs/Painter/26_plano_performance_procreate.md)
//! — **e que a FECHOU.** Irmã de `measure_window_premise.rs` (*a reivindicação é verdadeira?*) e de
//! `measure_gpu_frontier.rs` (*quanto custa o fold?*); esta pergunta a terceira coisa: **ela é
//! APERTADA, e apertá-la paga?**
//!
//! ## ⛔ O VEREDITO, para ninguém reconstruir a frente sem saber o que já foi medido
//!
//! A frente T (*substituir o bbox único por um conjunto de TILES*) foi **construída inteira** — o tipo
//! `TileSet` com bitset + `bounds()` byte-idêntico como ponte, a migração do campo, o composite parcial
//! percorrendo os retângulos, 13 gates e 6 mutações — **e revertida na medição de fechamento**, que é o
//! critério de parada que o próprio plano escreveu. Três números, nesta ordem:
//!
//! | pergunta | resposta |
//! |---|---|
//! | o bbox mente? | **sim, 1,66× a 916×** (tabela abaixo) |
//! | uma grade de tiles pega essa mentira? | **não** — a reivindicação REAL cai só ~1,4× |
//! | e no relógio? | **+12-14%** em dois gestos, **−75%** no mais comum |
//!
//! ⚠️ **A causa, e ela é a coisa que se leva desta frente:** a grade **não pode ser mais apertada do
//! que aquilo que lhe contam**. O `mark_dirty` recebe o bbox de cada *SEGMENTO* do traço — medido,
//! **90×54 texels para um pincel de 24 px** — então o piso que esta sonda calcula (marcar os tiles que
//! os texels MUDADOS cruzam) é inalcançável: entre 45.568 (piso) e 145.856 (bbox) a marcação real
//! entrega 104.000. O over-claim mora nos **CHAMADORES** do `mark_dirty`, não na união deles.
//!
//! ⚠️ **E a sonda de fechamento reprovou a si mesma antes de reprovar a frente:** a 1ª versão media o
//! frame do **pen-up** — o commit, que re-suja o envelope inteiro — onde a reivindicação por tiles é
//! igual ou MAIOR que o bbox (414.208 contra 419.904). O frame do commit acontece uma vez por traço; os
//! outros sessenta são os de traço ABERTO, e são esses que a sonda mede hoje.
//!
//! ⚠️ **O que a frente ACHOU e vale mais que ela:** a drenagem parcial é dominada pela **LUZ do
//! impasto** (7,9 ms de ~10 ms a 1024²), que sozinha custa **3× um composite de tela inteira** (2,8 ms).
//! Cortar a área da reivindicação em 5× moveu o relógio em 5%. O alvo estava errado.
//!
//! ## O que cada coluna é
//!
//! - **tocados** — texels em que `(RGBA, relevo, cobertura)` de fato MUDOU. O trabalho irredutível.
//! - **bbox** — a área do `dirty_rect`, que é o que os consumidores percorrem hoje.
//! - **REAL** — o que uma grade de 16 px marcaria a partir dos rects que o `mark_dirty` **recebeu**.
//!   É a previsão honesta, e é ela que fecha a frente.
//! - **piso** — o que a grade marcaria se lhe contassem exatamente os texels mudados. **Inalcançável**
//!   sem apertar os chamadores; fica aqui porque é o teto do que a frente poderia render.
//!
//! Rodar:
//! `cargo test -p ph2d-tool-painter --release measure_dirty_overclaim -- --ignored --nocapture`

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff, MirrorAxis, StrokeMethod};

/// Lado do canvas. 1024² é o regime do produto (2048–4096 é o comum) e mantém a sonda em segundos.
const N: u32 = 1024;
/// Os tamanhos de tile que o plano manda varrer — o trade é público: menor desperdiça menos e
/// gerencia mais, então o número sai da MEDIÇÃO e não do gosto.
///
/// ⚠️ A 1ª varredura usou `64/128/256` (os números que o plano citou da literatura) e eles **perderam
/// para o bbox** no gesto comum: um tile de 64 são 4096 texels e a faixa de um pincel de r=12 tem 24 px
/// de largura, então o arredondamento come o ganho. O tile útil é da ordem do **diâmetro do pincel**,
/// não da tela — daí a faixa desceu.
const TILES: [u32; 3] = [16, 32, 64];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um pincel duro e opaco com **impasto ligado** — os três planos do relevo existem, então a sonda mede
/// a mesma superfície que o fold percorre, e não só o RGBA.
fn brush(method: StrokeMethod, r: f32) -> BrushSpec {
    BrushSpec {
        radius_px: r,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        impasto: true,
        impasto_depth: 0.5,
        impasto_smoothing: 0.0,
        impasto_body: 1.0,
        stroke_method: method,
        ..Default::default()
    }
}

/// Arma o pincel em TODOS os slots — cada `PaintMode` tem `BrushSpec` próprio, e armar só o vivo é como
/// um default some ao trocar de ferramenta (a lição do `toggle_brush_impasto`).
fn arm(t: &mut PainterTool, b: BrushSpec) {
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
}

fn fresh(method: StrokeMethod, r: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    arm(&mut t, brush(method, r));
    t
}

/// O estado que um consumidor a jusante lê: a cor E os dois planos que a luz integra.
struct Snap {
    rgba: Vec<u8>,
    relief: Vec<f32>,
    cover: Vec<u8>,
}

fn snap(t: &PainterTool) -> Snap {
    let (relief, cover) = t.impasto_gpu_planes().map_or_else(
        || (vec![0.0; (N * N) as usize], vec![0u8; (N * N) as usize]),
        |p| (p.relief, p.cover),
    );
    Snap {
        rgba: t.canvas_rgba.as_ref().clone(),
        relief,
        cover,
    }
}

/// Máscara dos texels que MUDARAM entre dois snapshots.
fn touched(a: &Snap, b: &Snap) -> Vec<bool> {
    (0..(N * N) as usize)
        .map(|i| {
            a.rgba[i * 4..i * 4 + 4] != b.rgba[i * 4..i * 4 + 4]
                || a.relief[i] != b.relief[i]
                || a.cover[i] != b.cover[i]
        })
        .collect()
}

/// O que um esquema de tiles de lado `t` reivindicaria para cobrir `mask`: `(área, nº de retângulos
/// coalescidos por linha)`.
///
/// ⚠️ **Os dois números decidem coisas diferentes e nenhum basta sozinho.** A área é o que o fold e o
/// composite percorrem; a CONTAGEM é o que o upload paga por-chamada. Um esquema que ganha 8× em área e
/// entrega 300 retângulos pode perder no `write_texture`, e é exatamente o risco 🟡 que o plano nomeou.
fn tile_claim(mask: &[bool], t: u32) -> (u64, usize) {
    let cols = N.div_ceil(t);
    let rows = N.div_ceil(t);
    let mut marked = vec![false; (cols * rows) as usize];
    for y in 0..N {
        for x in 0..N {
            if mask[(y * N + x) as usize] {
                marked[((y / t) * cols + (x / t)) as usize] = true;
            }
        }
    }
    let n = marked.iter().filter(|m| **m).count() as u64;
    // Corridas contíguas por LINHA de tiles — o `iter_rects` que o `TileSet` vai expor.
    let mut rects = 0usize;
    for row in 0..rows {
        let mut run = false;
        for col in 0..cols {
            let on = marked[(row * cols + col) as usize];
            if on && !run {
                rects += 1;
            }
            run = on;
        }
    }
    (n * u64::from(t) * u64::from(t), rects)
}

/// A reivindicação REAL que uma grade de tiles produziria — marcada a partir dos rects que o
/// `mark_dirty` de fato recebeu (`PainterTool::marks`), não dos texels que mudaram.
///
/// ⚠️ **Esta é a previsão; `tile_claim` é só o PISO**, e a distância entre os dois é o que MATOU a
/// frente T: o piso marca os tiles que os texels *mudados* cruzam, e os chamadores marcam o bbox de
/// cada SEGMENTO do traço — 90×54 para um pincel de 24 px. A grade não pode ser mais apertada do que
/// aquilo que lhe contam.
fn real_claim(t: &PainterTool, tile: u32) -> (u64, usize) {
    let cols = N.div_ceil(tile);
    let rows = N.div_ceil(tile);
    let mut marked = vec![false; (cols * rows) as usize];
    for r in &t.marks {
        let x1 = r.x.saturating_add(r.w).min(N);
        let y1 = r.y.saturating_add(r.h).min(N);
        if r.x >= x1 || r.y >= y1 {
            continue;
        }
        for row in (r.y / tile)..=((y1 - 1) / tile) {
            for col in (r.x / tile)..=((x1 - 1) / tile) {
                marked[(row * cols + col) as usize] = true;
            }
        }
    }
    let n = marked.iter().filter(|m| **m).count() as u64;
    let mut rects = 0usize;
    for row in 0..rows {
        let mut run = false;
        for col in 0..cols {
            let on = marked[(row * cols + col) as usize];
            if on && !run {
                rects += 1;
            }
            run = on;
        }
    }
    (n * u64::from(tile) * u64::from(tile), rects)
}

/// Uma cena: monta, drena a reivindicação anterior, roda o gesto de UM FRAME, drena e mede.
///
/// ⚠️ A unidade é o **frame**, não o traço: o consumidor drena uma vez por frame, então a super-
/// reivindicação que importa é a de tudo que o `union_region` acumulou entre dois drains.
fn probe(name: &str, build: &dyn Fn() -> PainterTool, act: &dyn Fn(&mut PainterTool)) {
    let mut t = build();
    let _ = t.take_preview_dirty();
    t.marks.clear(); // só as marcas DESTE frame
    let before = snap(&t);
    act(&mut t);
    let real = real_claim(&t, 16);
    let _ = t.take_preview_dirty();
    let claim = t.preview_gpu_region();
    let after = snap(&t);

    let mask = touched(&before, &after);
    let hit = mask.iter().filter(|m| **m).count() as u64;
    if hit == 0 {
        println!("[overclaim] {name:<26} (nada mudou nesta sonda; claim={claim:?})");
        return;
    }
    let bbox = claim.map_or(u64::from(N) * u64::from(N), |(_, _, w, h)| {
        u64::from(w) * u64::from(h)
    });
    #[allow(clippy::cast_precision_loss)] // razões de área; o erro de f64 aqui é irrelevante
    let ratio = |a: u64| a as f64 / hit as f64;
    let claims: Vec<(u64, usize)> = TILES.iter().map(|t| tile_claim(&mask, *t)).collect();
    // ⚠️ O que o PISO pouparia — não o que a frente entrega. A coluna REAL é a previsão; esta é o
    // teto. É ABSOLUTA de propósito: uma razão ruim sobre 800 texels não custa nada (o drag dot).
    let best = claims.iter().map(|c| c.0).min().unwrap_or(bbox);
    let saved = bbox as i64 - best as i64;
    println!(
        "[overclaim] {name:<26} {hit:>7} {bbox:>8} {:>6.2}x | REAL {:>8} {:>5.2}x {:>4}r | piso {:>8} {:>5.2}x {:>4}r {:>8} {:>5.2}x {:>4}r {:>8} {:>5.2}x {:>4}r  piso{saved:>+9}",
        ratio(bbox),
        real.0,
        ratio(real.0),
        real.1,
        claims[0].0,
        ratio(claims[0].0),
        claims[0].1,
        claims[1].0,
        ratio(claims[1].0),
        claims[1].1,
        claims[2].0,
        ratio(claims[2].0),
        claims[2].1,
    );
}

/// **A medição que abriu — e fechou — a frente dos TILES.**
///
/// O critério foi escrito ANTES de rodar (senão é escolher a régua depois do tiro):
///
/// | razão bbox/tocados | veredito |
/// |---|---|
/// | ≈ 1 no gesto do produto | frente T CANCELADA — a caixa já é justa |
/// | ≥ 2 em gesto comum | a frente se paga; o tile é escolhido pela varredura |
///
/// A régua disse *"se paga"* e a **construção provou que não** — porque a régua perguntava do bbox e o
/// que decide é a MARCAÇÃO (ver o doc do módulo). Fica aqui inteira, com o critério original e o
/// resultado que o contradiz, porque uma régua que só aparece depois do resultado não é régua.
///
/// ## Medido (2026-07-25, 1024², pincel r=12, impasto ligado)
///
/// | cena | tocados | bbox | razão | **REAL (grade 16)** | piso |
/// |---|---|---|---|---|---|
/// | reto curto (controle) | 2.362 | 3.927 | 1,66× | **6.912 / 3r** | 4.096 |
/// | reto diagonal | 6.769 | 44.944 | 6,64× | **23.808 / 15r** | 14.848 |
/// | diagonal, mão rápida | 21.020 | 357.604 | 17,01× | **72.192 / 38r** | 45.568 |
/// | traço em L | 23.322 | 253.340 | 10,86× | **49.408 / 31r** | 40.960 |
/// | Line editor (re-stamp) | 447 | 409.600 | **916,33×** | **430.336 / 41r** | 2.560 |
/// | simetria radial 6 | 14.136 | 301.131 | 21,30× | **321.024 / 33r** | 27.648 |
/// | espelho X | 4.724 | 21.483 | 4,55× | **32.256 / 3r** | 7.168 |
/// | drag dot (controle 2) | 377 | 837 | 2,22× | **1.536 / 3r** | 1.536 |
///
/// **Leia a coluna REAL contra a coluna bbox, e a frente morre ali:** ela ganha em dois gestos
/// (diagonal 5×, L 5×) e **PERDE nos outros quatro** — no re-stamp de forma e na simetria a grade
/// reivindica MAIS que o bbox, porque aqueles caminhos marcam a figura inteira e o arredondamento só
/// acrescenta. A razão contra os *tocados* (916×) descreve uma mentira verdadeira que **nenhuma grade
/// alcança**, porque a grade não vê os texels mudados: vê os rects que lhe contam.
///
/// ⚠️ **DUAS correções que a medição fez em mim, e as duas invertem uma escolha do plano 26:**
///
/// 1. **`64/128/256` — os números que o plano citou da literatura — PERDEM para o bbox** no gesto
///    comum (reto curto 10,40× contra 1,66×). Um tile de 64 são 4.096 texels e a faixa de um pincel
///    r=12 tem 24 px: o arredondamento come o ganho. **O tile útil é da ordem do diâmetro do pincel.**
/// 2. **A razão sozinha decide errado.** No drag dot a grade de 16 é 4,07× *pior* em razão e **699
///    texels** pior em absoluto: nada. A grandeza que decide é a **absoluta**.
#[allow(clippy::too_many_lines)] // uma cena por gesto: a lista É a medição
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_dirty_rect_overclaim_is_measured_not_assumed() {
    println!(
        "[overclaim] cena                       tocados     bbox  razao |      REAL (o TileSet do produto) |      piso t=16              t=32                t=64        piso poupa"
    );

    // ── CONTROLE: um traço reto curto, do tamanho de um frame de mão calma. ───────────────────────
    // Se ATÉ aqui a razão for grande, a causa é o pincel, não o caminho.
    probe(
        "reto curto (controle)",
        &|| {
            let mut t = fresh(StrokeMethod::Space, 12.0);
            t.on_canvas_pointer(cp([300.0, 512.0], PointerPhase::Down));
            t
        },
        &|t| {
            for i in 1u8..=16 {
                let x = 300.0 + f32::from(i) * 6.0;
                t.on_canvas_pointer(cp([x, 512.0], PointerPhase::Move));
            }
        },
    );

    // ── DIAGONAL: mesma tinta, caixa QUADRADA. É o caso mais simples em que a caixa mente. ────────
    probe(
        "reto diagonal",
        &|| {
            let mut t = fresh(StrokeMethod::Space, 12.0);
            t.on_canvas_pointer(cp([300.0, 300.0], PointerPhase::Down));
            t
        },
        &|t| {
            for i in 1u8..=16 {
                let d = 300.0 + f32::from(i) * 12.0;
                t.on_canvas_pointer(cp([d, d], PointerPhase::Move));
            }
        },
    );

    // ── MÃO RÁPIDA: o mesmo frame, mas a mão andou 600 px. A caixa cresce com a VELOCIDADE. ───────
    probe(
        "diagonal, mao rapida",
        &|| {
            let mut t = fresh(StrokeMethod::Space, 12.0);
            t.on_canvas_pointer(cp([200.0, 200.0], PointerPhase::Down));
            t
        },
        &|t| {
            for i in 1u8..=16 {
                let d = 200.0 + f32::from(i) * 37.5;
                t.on_canvas_pointer(cp([d, d], PointerPhase::Move));
            }
        },
    );

    // ── O L: a hipótese do plano. Duas pernas ortogonais num frame reivindicam o QUADRADO inteiro,
    //    inclusive o quadrante que a tinta nunca visitou.
    probe(
        "traco em L",
        &|| {
            let mut t = fresh(StrokeMethod::Space, 12.0);
            t.on_canvas_pointer(cp([250.0, 250.0], PointerPhase::Down));
            t
        },
        &|t| {
            for i in 1u8..=8 {
                t.on_canvas_pointer(cp([250.0 + f32::from(i) * 62.5, 250.0], PointerPhase::Move));
            }
            for i in 1u8..=8 {
                t.on_canvas_pointer(cp([750.0, 250.0 + f32::from(i) * 62.5], PointerPhase::Move));
            }
        },
    );

    // ── O EDITOR DE FORMA: `Line` RE-CARIMBA o traço inteiro a cada frame, então esta razão é
    //    paga em TODO frame do arrasto, não uma vez.
    //    ⚠️ O `Line` é um editor de POLILINHA: cada `Down` CRIA um ponto e o `Move` o arrasta, então um
    //    segmento exige DOIS pontos. A 1ª versão desta cena fazia Down+Move e mediu *"nada mudou"* —
    //    fixture que não continha o fenômeno, não produto quieto.
    probe(
        "Line editor (re-stamp)",
        &|| {
            let mut t = fresh(StrokeMethod::Line, 12.0);
            t.on_canvas_pointer(cp([200.0, 200.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([200.0, 200.0], PointerPhase::Up));
            t.on_canvas_pointer(cp([800.0, 800.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([800.0, 800.0], PointerPhase::Move));
            t
        },
        &|t| {
            t.on_canvas_pointer(cp([810.0, 810.0], PointerPhase::Move));
        },
    );

    // ── SIMETRIA RADIAL: 6 cópias espalhadas pela tela — a caixa que as contém é a tela. ──────────
    probe(
        "simetria radial 6",
        &|| {
            let mut t = fresh(StrokeMethod::Space, 12.0);
            t.toggle_symmetry_enabled();
            t.toggle_symmetry_circular();
            t.set_symmetry_segments(6);
            t.on_canvas_pointer(cp([700.0, 512.0], PointerPhase::Down));
            t
        },
        &|t| {
            for i in 1u8..=16 {
                t.on_canvas_pointer(cp([700.0 + f32::from(i) * 6.0, 512.0], PointerPhase::Move));
            }
        },
    );

    // ── ESPELHO: duas cópias, uma em cada metade. A caixa é a largura da tela. ────────────────────
    probe(
        "espelho X",
        &|| {
            let mut t = fresh(StrokeMethod::Space, 12.0);
            t.toggle_symmetry_enabled();
            t.set_symmetry_axis(MirrorAxis::X);
            t.on_canvas_pointer(cp([200.0, 512.0], PointerPhase::Down));
            t
        },
        &|t| {
            for i in 1u8..=16 {
                t.on_canvas_pointer(cp([200.0 + f32::from(i) * 6.0, 512.0], PointerPhase::Move));
            }
        },
    );

    // ── DRAG DOT: a pegada é UM disco. Aqui a caixa TEM de ser justa — é o controle pelo outro lado.
    probe(
        "drag dot (controle 2)",
        &|| {
            let mut t = fresh(StrokeMethod::DragDot, 12.0);
            t.on_canvas_pointer(cp([512.0, 512.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([512.0, 460.0], PointerPhase::Move));
            t
        },
        &|t| {
            t.on_canvas_pointer(cp([512.0, 455.0], PointerPhase::Move));
        },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// T2 — O QUE O 1º CONSUMIDOR GANHOU (o composite parcial da pista CPU)
// ─────────────────────────────────────────────────────────────────────────────

/// **Quanto custa uma drenagem, agora que ela percorre TILES.**
///
/// A sonda do over-claim mede a REIVINDICAÇÃO; esta mede o RELÓGIO do consumidor que passou a honrá-la.
/// Ela não afirma nada — o antes/depois se obtém rodando-a com a mutação *"um passe sobre o bbox"*
/// aplicada ao `take_preview_arc`, que é como o número desta wave foi levantado.
///
/// ## Medido (2026-07-25, 1024², pincel r=12, impasto ligado)
///
/// | cena | bbox (antes) | tiles (depois) |
/// |---|---|---|
/// | traço reto curto | *ver output* | *ver output* |
/// | diagonal, mão rápida | | |
/// | traço em L | | |
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_partial_drain_cost -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_partial_drain_cost_is_measured_not_assumed() {
    /// Mede a MEDIANA de `take_preview_arc` sobre um gesto repetido — mediana, não média, porque um
    /// outlier de agendador não descreve o produto.
    ///
    /// ⚠️ **O gesto é UM TRAÇO ABERTO, drenado entre lotes de movimento** — que é o que o produto faz
    /// 60 vezes por segundo. A 1ª versão desta sonda fechava o traço (`PointerPhase::Up`) a cada
    /// rodada e media o frame do **COMMIT**, onde o envelope inteiro é re-sujado: ali a reivindicação
    /// por tiles é igual ou MAIOR que o bbox (medido, 414.208 contra 419.904 texels) e a frente parece
    /// inútil. O frame do commit acontece uma vez por traço; os outros sessenta são estes.
    fn drain_ms(name: &str, act: &dyn Fn(&mut PainterTool, u8)) {
        let mut t = fresh(StrokeMethod::Space, 12.0);
        // A pilha tem de ser NÃO-trivial para a drenagem COMPOR (senão ela devolve o `canvas_rgba` cru
        // e esta sonda mediria um `Arc::clone`). Um traço de impasto basta.
        t.on_canvas_pointer(cp([500.0, 500.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([510.0, 500.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([510.0, 500.0], PointerPhase::Up));
        let _ = t.take_preview_arc();

        let mut samples = Vec::new();
        for round in 0..12u8 {
            act(&mut t, round);
            let t0 = std::time::Instant::now();
            let got = t.take_preview_arc();
            samples.push(t0.elapsed().as_secs_f64() * 1e3);
            assert!(got.is_some(), "a drenagem nao produziu nada");
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "[drain] {name:<26} p50 {:>7.3} ms  braco {:?}",
            samples[samples.len() / 2],
            t.last_drain_branch
        );
    }

    println!("[drain] cena                        mediana");
    // Cada rodada é UM FRAME de um traço que continua aberto: o lote de movimento, e depois a drenagem.
    drain_ms("reto, mao calma", &|t, k| {
        if k == 0 {
            t.on_canvas_pointer(cp([60.0, 512.0], PointerPhase::Down));
        }
        let x0 = 60.0 + f32::from(k) * 60.0;
        for i in 1u8..=16 {
            t.on_canvas_pointer(cp([x0 + f32::from(i) * 3.75, 512.0], PointerPhase::Move));
        }
    });
    drain_ms("diagonal, mao rapida", &|t, k| {
        if k == 0 {
            t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
        }
        let o = 40.0 + f32::from(k) * 80.0;
        for i in 1u8..=16 {
            let d = o + f32::from(i) * 5.0;
            t.on_canvas_pointer(cp([d, d], PointerPhase::Move));
        }
    });
    // O L num único frame: as duas pernas no MESMO lote, que é o gesto que o bbox mais mente.
    drain_ms("cotovelo num frame", &|t, k| {
        if k == 0 {
            t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Down));
        }
        let o = 100.0 + f32::from(k) * 20.0;
        for i in 1u8..=8 {
            t.on_canvas_pointer(cp([o + f32::from(i) * 50.0, o], PointerPhase::Move));
        }
        for i in 1u8..=8 {
            t.on_canvas_pointer(cp([o + 400.0, o + f32::from(i) * 50.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([o + 20.0, o + 400.0], PointerPhase::Move));
    });
}

/// **ONDE vão os 14 ms de uma drenagem parcial.**
///
/// A medição do T2 mostrou que cortar a área do composite em **5×** move o relógio da drenagem em
/// **5%** — o que só pode significar que o composite não é o custo dominante. Esta sonda parte o
/// braço parcial em estágios e mede cada um sobre a MESMA região, para o próximo passo da frente
/// atacar o que de fato custa em vez do que a intuição diz.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_partial_drain_stages -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_partial_drain_stages_are_measured_not_assumed() {
    use crate::compositor::{Region, composite_region};

    let mut t = fresh(StrokeMethod::Space, 12.0);
    // Pilha não-trivial + relevo de verdade (senão a luz não roda e a sonda mede outra coisa).
    t.on_canvas_pointer(cp([500.0, 500.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([510.0, 500.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([510.0, 500.0], PointerPhase::Up));
    let _ = t.take_preview_arc();
    // A diagonal de mão rápida: a cena cujo bbox mente 17×.
    t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Down));
    for i in 1u8..=16 {
        let d = 100.0 + f32::from(i) * 37.5;
        t.on_canvas_pointer(cp([d, d], PointerPhase::Move));
    }
    let bbox = t.dirty_rect.expect("o gesto reivindicou alguma coisa");
    println!(
        "[stages] bbox {}x{} = {} texels",
        bbox.w,
        bbox.h,
        u64::from(bbox.w) * u64::from(bbox.h)
    );

    let ms = |f: &mut dyn FnMut()| {
        const N: u32 = 8;
        let t0 = std::time::Instant::now();
        for _ in 0..N {
            f();
        }
        t0.elapsed().as_secs_f64() * 1e3 / f64::from(N)
    };

    let active = t.layers.active().unwrap_or(crate::layers::LayerId(0));
    let mut buf = Vec::new();
    println!(
        "[stages] composite_region(bbox)     {:>7.3} ms",
        ms(&mut || {
            let src = crate::tool::internal::ToolPixelSource {
                active_id: active,
                active_rgba: &t.canvas_rgba,
                images: &t.images,
            };
            buf = composite_region(&t.layers, &src, N, N, bbox);
        })
    );
    println!(
        "[stages] apply_impasto_light(bbox)  {:>7.3} ms",
        ms(&mut || t.apply_impasto_light(&mut buf, bbox))
    );
    println!(
        "[stages] impasto_fields() sozinho   {:>7.3} ms",
        ms(&mut || {
            std::hint::black_box(t.impasto_fields().is_some());
        })
    );
    let full = Region {
        x: 0,
        y: 0,
        w: N,
        h: N,
    };
    println!(
        "[stages] composite_region(TELA)     {:>7.3} ms",
        ms(&mut || {
            let src = crate::tool::internal::ToolPixelSource {
                active_id: active,
                active_rgba: &t.canvas_rgba,
                images: &t.images,
            };
            std::hint::black_box(composite_region(&t.layers, &src, N, N, full).len());
        })
    );
}
