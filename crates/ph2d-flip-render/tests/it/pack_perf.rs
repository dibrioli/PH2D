//! Teto de custo do empacotamento — guard de ORDEM, não microbenchmark.
//!
//! O `pack` roda **a cada frame** para o traço em curso (o preview ao vivo), então o
//! broadphase de vizinhos geométricos (`neighbors.rs`) não pode escalar mal. Os
//! tetos abaixo são folgados o bastante para não serem flaky num runner carregado, e
//! apertados o bastante para pegar uma regressão de ORDEM (o par-a-par `O(n²)`
//! voltando, ou o grid perdendo eficácia).
//!
//! Números medidos em `--release` na workstation (em debug não dizem nada).

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_render::pack_drawing;
use std::time::Instant;

/// O caso COMUM: um traço longo e ondulado que NÃO volta sobre si mesmo (um
/// contorno, uma curva, um fio de cabelo).
fn realistic_long_stroke(n: usize) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for k in 0..n {
        let t = k as f32 * 2.5; // amostragem de tablet: ~2.5 px entre pontos
        s.push_point(Point {
            pos: Vec2::new(20.0 + t, 500.0 + 180.0 * (t * 0.01).sin()),
            width: 16.0,
            opacity: 1.0,
            color: Rgba::new(0.1, 0.1, 0.1, 1.0),
        });
    }
    s.hardness = 0.7;
    d.strokes.push(s);
    d
}

/// O caso PATOLÓGICO: um rabisco browniano num palmo de tela — cada segmento tem
/// centenas de vizinhos REAIS. É o que o `PAIR_BUDGET` do broadphase protege.
fn dense_scribble(n: usize) -> FlipDrawing {
    let mut state: u32 = 0xC0FF_EE01;
    let mut rnd = || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f32::from((state >> 16) as u16) / f32::from(u16::MAX)
    };
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    let (mut x, mut y) = (200.0f32, 200.0f32);
    for _ in 0..n {
        x = (x + (rnd() - 0.5) * 24.0).clamp(0.0, 400.0);
        y = (y + (rnd() - 0.5) * 24.0).clamp(0.0, 400.0);
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: 16.0,
            opacity: 1.0,
            color: Rgba::new(0.1, 0.1, 0.1, 1.0),
        });
    }
    s.hardness = 0.7;
    d.strokes.push(s);
    d
}

fn pack_ms(d: &FlipDrawing) -> (f32, usize) {
    let t = Instant::now();
    let g = pack_drawing(d);
    (t.elapsed().as_secs_f32() * 1000.0, g.seg_extras.len())
}

#[test]
fn packing_a_realistic_long_stroke_is_cheap() {
    let (ms, extras) = pack_ms(&realistic_long_stroke(4000));
    eprintln!("traço real de 4000 pontos: {ms:.2} ms · {extras} vizinhos");
    // Medido ~1,7 ms em release. O par-a-par ingênuo levaria ~1 s aqui.
    //
    // ⚠️ **A CONTAGEM de vizinhos caiu 4× com as cápsulas fundidas** (2026-07-28): 47.546 →
    // 11.954 no mesmo traço, com o tempo do pack INALTERADO — e o que a contagem paga não é o
    // pack, é o LAÇO DO FRAGMENT, que roda por pixel a cada frame.
    //
    // **O teto é por PERFIL** — pelo MESMO motivo do gate irmão abaixo (leia o comentário
    // dele): o `nextest --workspace` roda em DEBUG e em paralelo, então um teto calibrado
    // em release fica verde isolado e vermelho na suíte cheia — um flaky que não denuncia
    // regressão nenhuma, só carga de máquina. A folga relativa (~18×) é a mesma nos dois
    // perfis; a propriedade guardada é a mesma: o pack de um traço normal não voltou a ser
    // O(n²). (Este era o **segundo** assert do arquivo, e ficou para trás quando a linha
    // `line/Vector` consertou o primeiro — a mesma mina, meio desarmada.)
    let ceiling = if cfg!(debug_assertions) { 200.0 } else { 30.0 };
    assert!(
        ms < ceiling,
        "o pack de um traço longo NORMAL regrediu de ordem: {ms:.1} ms \
         (teto {ceiling} ms neste perfil)"
    );
}

#[test]
fn packing_a_dense_scribble_is_bounded() {
    let (ms, extras) = pack_ms(&dense_scribble(4000));
    eprintln!("rabisco denso de 4000 pontos: {ms:.2} ms · {extras} vizinhos");
    // ⚠️ **O rabisco PATOLÓGICO ficou ~30 % mais caro no pack** (15,9 → 20,1 ms em release,
    // 2026-07-28): a fusão colhe TODOS os candidatos antes de agrupá-los em runs, em vez de
    // rejeitar em O(1) pelos 16 mais próximos. É o preço de o teto ser de CÁPSULAS; o caso
    // NORMAL não pagou nada (1,6 ms, igual) e o fragment ficou 4× mais barato. Teto 120 ms.
    //
    // O `PAIR_BUDGET` corta o trabalho: medido ~14 ms em release (sem ele, ~27 ms e
    // crescendo). O teto existe para o frame do preview não desabar.
    //
    // **O teto é por PERFIL, e não por acaso.** O doc-comment desta suíte já diz que "em
    // debug os números não dizem nada" — e o `cargo nextest run --workspace` roda em DEBUG.
    // Medido nesta workstation: **13,7 ms em release · 78 ms em debug ocioso**, e o nextest
    // roda dezenas de testes em paralelo, o que empurra o debug para além de 130 ms. Com o
    // teto único de 120 ms o gate ficava vermelho na suíte cheia e VERDE isolado — um flaky
    // que não denuncia regressão nenhuma, só carga de máquina. (Encontrado pela linha
    // `line/Vector` quando ele derrubou o gate de fechamento dela; o gate em si é do Flip.)
    //
    // Os dois tetos guardam a MESMA propriedade — que o broadphase não voltou a ser O(n²) —
    // com a mesma folga relativa (~9×) no perfil em que cada um roda.
    let ceiling = if cfg!(debug_assertions) { 700.0 } else { 120.0 };
    assert!(
        ms < ceiling,
        "o teto de trabalho do broadphase (PAIR_BUDGET) não está segurando: \
         {ms:.1} ms (teto {ceiling} ms neste perfil)"
    );
}
