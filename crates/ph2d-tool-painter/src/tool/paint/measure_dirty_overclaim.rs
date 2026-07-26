//! **T0 — o retângulo sujo SUPER-REIVINDICA? Por quanto?**
//!
//! A medição que ABRE a frente T do [plano 26](../../../../../docs/Painter/26_plano_performance_procreate.md)
//! — e que pode CANCELÁ-LA. Irmã de `measure_window_premise.rs` (*a reivindicação é verdadeira?*) e de
//! `measure_gpu_frontier.rs` (*quanto custa o fold?*); esta pergunta a terceira coisa: **a reivindicação
//! é APERTADA?**
//!
//! O `dirty_rect` é UM retângulo e ele **unifica** (`stamp_preview.rs`: `union_region(acc, rect)`), então
//! dois dabs distantes reivindicam a caixa que os contém *e tudo entre eles*. Todo consumidor a jusante
//! — o fold do impasto, o upload por-camada, o composite parcial — paga a ÁREA dessa caixa. Se a caixa é
//! justa, tiles não têm o que economizar e a frente morre aqui.
//!
//! ⚠️ **Ela não afirma nada; IMPRIME.** O número que ela produz é uma decisão de arquitetura, não um
//! contrato — um `assert` aqui seria pinar a conclusão antes de tê-la.
//!
//! ## O que cada coluna é
//!
//! - **tocados** — texels em que `(RGBA, relevo, cobertura)` de fato MUDOU. É o trabalho irredutível.
//! - **bbox** — a área do `dirty_rect` de hoje, que é o que os consumidores percorrem.
//! - **razão** — `bbox / tocados`. **1,0 = a caixa é justa e a frente T está cancelada.**
//! - **t64 / t128 / t256** — a razão que um esquema por TILES daria, marcando os tiles que os texels
//!   tocados cruzam. ⚠️ É um **piso**: o `TileSet` real marcará a partir dos MESMOS rects que o
//!   `mark_dirty` recebe hoje (as pegadas dos dabs), que são apertadas mas não exatas. A distância
//!   entre este piso e o bbox é o que a frente pode recuperar; nada além dela.
//!
//! Rodar:
//! `cargo test -p ph2d-tool-painter --release the_dirty_rect_overclaim -- --ignored --nocapture`

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

/// Uma cena: monta, drena a reivindicação anterior, roda o gesto de UM FRAME, drena e mede.
///
/// ⚠️ A unidade é o **frame**, não o traço: o consumidor drena uma vez por frame, então a super-
/// reivindicação que importa é a de tudo que o `union_region` acumulou entre dois drains.
fn probe(name: &str, build: &dyn Fn() -> PainterTool, act: &dyn Fn(&mut PainterTool)) {
    let mut t = build();
    let _ = t.take_preview_dirty();
    let before = snap(&t);
    act(&mut t);
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
    // O que a frente T de fato POUPA neste gesto, em texels que ninguém mais percorre. É ABSOLUTO de
    // propósito: uma razão ruim sobre 800 texels não custa nada, e é exatamente o caso do drag dot.
    let best = claims.iter().map(|c| c.0).min().unwrap_or(bbox);
    let saved = bbox as i64 - best as i64;
    println!(
        "[overclaim] {name:<26} {hit:>7} {bbox:>8} {:>6.2}x  {:>8} {:>5.2}x {:>4}r {:>8} {:>5.2}x {:>4}r {:>8} {:>5.2}x {:>4}r  {saved:>+9}",
        ratio(bbox),
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

/// **A medição que abre — ou fecha — a frente dos TILES.**
///
/// O critério foi escrito ANTES de rodar (senão é escolher a régua depois do tiro):
///
/// | razão bbox/tocados | veredito |
/// |---|---|
/// | ≈ 1 no gesto do produto | **frente T CANCELADA** — a caixa já é justa |
/// | ≥ 2 em gesto comum | a frente se paga; o tile é escolhido pela varredura |
///
/// ## Medido (2026-07-25, 1024², pincel r=12, impasto ligado)
///
/// | cena | tocados | bbox | razão | t=16 | poupa |
/// |---|---|---|---|---|---|
/// | reto curto (controle) | 2.362 | 3.927 | **1,66×** | 4.096 / 2 rects | −169 |
/// | reto diagonal | 6.769 | 44.944 | 6,64× | 14.848 / 13r | +30.096 |
/// | diagonal, mão rápida | 21.020 | 357.604 | **17,01×** | 45.568 / 37r | +312.036 |
/// | traço em L | 23.322 | 253.340 | 10,86× | 40.960 / 31r | +212.380 |
/// | **Line editor (re-stamp)** | 447 | 409.600 | **916,33×** | 2.560 / 5r | +407.040 |
/// | simetria radial 6 | 14.136 | 301.131 | 21,30× | 27.648 / 32r | +273.483 |
/// | espelho X | 4.724 | 21.483 | 4,55× | 7.168 / 4r | +14.315 |
/// | drag dot (controle 2) | 377 | 837 | 2,22× | 1.536 / 3r | −699 |
///
/// **VEREDITO: a frente T não está cancelada.** A caixa é justa exatamente onde o controle previu (traço
/// curto, 1,66×) e mente por **uma a três ordens de grandeza** em cinco gestos que o artista faz o tempo
/// todo. O pior é o **editor de forma**: ele re-carimba a figura inteira a cada frame, então muda **447
/// texels** e reivindica **409.600** — 916×, *por frame de arrasto*.
///
/// ⚠️ **DUAS correções que a medição fez em mim, e as duas invertem uma escolha do plano 26:**
///
/// 1. **`64/128/256` — os números que o plano citou da literatura — PERDEM para o bbox** no gesto comum
///    (reto curto 10,40× contra 1,66×; drag dot 43× contra 2,22×). Um tile de 64 são 4.096 texels e a
///    faixa de um pincel r=12 tem 24 px: o arredondamento come o ganho inteiro. **O tile útil é da ordem
///    do diâmetro do pincel, não da tela.** A 16 o pior caso do controle custa **169 texels** — que é
///    ruído — e o melhor poupa **407 mil**.
/// 2. **A razão sozinha decide errado.** No drag dot o tile de 16 é 4,07× *pior* em razão e **699 texels**
///    pior em absoluto: nada. A grandeza que decide é a **absoluta** (a coluna `poupa`), porque é ela que
///    o fold e o upload percorrem.
///
/// ⚠️ **As colunas `t=` são um PISO, não a previsão.** Elas marcam os tiles que os texels **mudados**
/// cruzam; o `TileSet` real marcará a partir dos mesmos rects que o `mark_dirty` recebe hoje (as pegadas
/// dos dabs), que são apertadas mas não exatas — no re-stamp do Line a diferença é grande (a faixa
/// inteira da figura é re-carimbada, ainda que quase nada MUDE). O número que escolhe o tamanho do tile
/// é o da marcação REAL, e ele só existe depois do T1 — por isso o plano põe o T3 depois do T1, e não
/// aqui.
#[allow(clippy::too_many_lines)] // uma cena por gesto: a lista É a medição
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_dirty_rect_overclaim_is_measured_not_assumed() {
    println!(
        "[overclaim] cena                       tocados     bbox  razao        t=16                t=32                t=64            poupa"
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
