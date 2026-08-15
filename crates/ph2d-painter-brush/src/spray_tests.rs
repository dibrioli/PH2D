//! Os gates do **SPRAY** — `n` marcas por ponto do caminho (plano 38 W5).
//!
//! O que a wave promete, e o que cada gate pergunta:
//!
//! 1. **`count = 1` é o mundo de sempre**, e sem jitter nenhum `count = n` é `n` cópias EXATAS.
//! 2. **`n` multiplica**, e multiplica exatamente (é aqui que um `0..n` no lugar de `1..n` morre).
//! 3. **Cada marca sorteia o SEU jitter** — a nuvem espalha em vez de empilhar.
//! 4. **A nuvem é centrada no ponto do CAMINHO**, nunca na primeira marca dele (é o gate que mata o
//!    desenho *"derive as cópias do dab base por um deslocamento extra"*, cujo raio sairia 2×).
//! 5. **Um carimbo deliberado não espalha** (Drag Dot / Anchored / Grid Stamp).
//! 6. **A Symmetry multiplica a nuvem inteira**, porque toda marca sai pela porta dela.
//! 7. **A memória dos fios recebe UM ponto por ponto do caminho**, não `n` — o Sketchy costura o
//!    traço, não o próprio spray.

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke::spray::SPRAY_COUNT_MAX;
use crate::stroke::{Dab, Stroke, StrokePoint};
use crate::stroke_method::StrokeMethod;
use crate::symmetry::{MirrorAxis, SymmetrySettings};

fn spec(count: u32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.5,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        spray_count: count,
        ..Default::default()
    }
}

fn plain_dynamics() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// Um traço RETO — a fixture mais simples que existe, porque a pergunta desta wave é sobre a
/// CONTAGEM e não sobre a forma do caminho.
///
/// ⚠️ **Ela ACUMULA, e tem de acumular:** o `Stroke::extend` começa por `out.clear()`, então ler o
/// buffer no fim de um traço devolve só as marcas do ÚLTIMO evento. Foi assim que a primeira versão
/// deste gate mediu **duas** marcas num traço de trinta e três.
fn straight(sp: BrushSpec, len: f32) -> Vec<Dab> {
    let mut s = Stroke::new(sp, plain_dynamics(), 7);
    let mut out = Vec::new();
    let mut all = Vec::new();
    s.begin(
        StrokePoint {
            pos: [200.0, 200.0],
            pressure: 1.0,
        },
        &mut out,
    );
    all.append(&mut out);
    for k in 1..=20 {
        #[allow(clippy::cast_precision_loss)]
        let t = k as f32 / 20.0;
        s.extend(
            StrokePoint {
                pos: [200.0 + len * t, 200.0],
                pressure: 1.0,
            },
            &mut out,
        );
        all.append(&mut out);
    }
    all
}

/// **1. O NEUTRO, ao bit.** Sem jitter nenhum, uma nuvem de `n` é `n` cópias exatas da marca única —
/// mesma posição, mesmo raio, mesma cor. É a prova de que o spray não mexe em NADA do caminho: nem
/// no espaçamento, nem no arco, nem no fluxo de sorteio (com o jitter desarmado, ninguém sorteia).
#[test]
fn a_spray_without_jitter_is_the_same_mark_repeated() {
    let one = straight(spec(1), 400.0);
    let four = straight(spec(4), 400.0);
    assert!(
        one.len() > 8,
        "controle: o traço tem de emitir dabs ({})",
        one.len()
    );
    assert_eq!(
        four.len(),
        one.len() * 4,
        "count 4 tem de emitir 4x as marcas de count 1"
    );
    for (i, d) in four.iter().enumerate() {
        let base = &one[i / 4];
        assert_eq!(
            d.center, base.center,
            "marca {i}: a posição tem de ser a mesma"
        );
        assert_eq!(
            d.radius_px, base.radius_px,
            "marca {i}: o raio tem de ser o mesmo"
        );
        assert_eq!(d.color, base.color, "marca {i}: a cor tem de ser a mesma");
        assert_eq!(
            d.arc_len, base.arc_len,
            "marca {i}: o arco é do CAMINHO, não da marca"
        );
    }
}

/// **2. A CONTAGEM multiplica, e multiplica exatamente.** É este gate que mata um `0..n` no lugar de
/// `1..n` — ali `count = 1` já emitiria duas marcas, e a razão entre 2 e 4 sairia 1,5 em vez de 2.
#[test]
fn the_count_multiplies_the_marks_exactly() {
    let n1 = straight(spec(1), 400.0).len();
    for n in [2u32, 3, 5, 8] {
        let got = straight(spec(n), 400.0).len();
        assert_eq!(
            got,
            n1 * n as usize,
            "count {n}: esperava {} marcas, veio {got}",
            n1 * n as usize
        );
    }
}

/// **3. Cada marca sorteia o SEU jitter** — com o scatter armado a nuvem ESPALHA, e é isso que a
/// separa de `n` carimbos empilhados.
#[test]
fn every_mark_of_a_cloud_draws_its_own_jitter() {
    let mut sp = spec(8);
    sp.jitter = 0.5;
    sp.jitter_scale = 0.5;
    let out = straight(sp, 400.0);
    assert!(
        out.len() >= 16,
        "controle: a fixture tem de ter mais de uma nuvem"
    );
    // As oito primeiras marcas são a nuvem do PRIMEIRO ponto do caminho: nenhuma pode repetir a
    // outra em posição E raio.
    let cloud = &out[..8];
    let mut distinct = 0;
    for i in 1..cloud.len() {
        if cloud[i].center != cloud[0].center || cloud[i].radius_px != cloud[0].radius_px {
            distinct += 1;
        }
    }
    assert!(
        distinct >= 6,
        "a nuvem empilhou: só {distinct} de 7 marcas diferem da primeira"
    );
}

/// **4. A nuvem é centrada no PONTO DO CAMINHO, nunca na primeira marca.**
///
/// ⚠️ É o gate que mata o desenho barato — *"derive cada cópia do dab base por mais um deslocamento"*
/// —, que espalharia a nuvem por **duas** vezes o raio do jitter e a deslocaria do caminho. A lei é
/// a que o slider promete: toda marca cai dentro de `jitter × diâmetro` do ponto onde a tinta caiu.
#[test]
fn the_cloud_is_centred_on_the_path_point_not_on_its_first_mark() {
    let radius = 12.0f32;
    let jitter = 0.5f32;
    let mut sp = spec(12);
    sp.radius_px = radius;
    sp.jitter = jitter;
    let out = straight(sp, 400.0);
    assert!(out.len() >= 24, "controle: precisa de mais de uma nuvem");
    // O caminho é RETO em y = 200: o ponto do caminho de cada nuvem tem `y = 200` exato, então o
    // desvio vertical de cada marca é o jitter puro — medi-lo não precisa saber onde o caminhador
    // pôs cada dab em x.
    let reach = jitter * 2.0 * radius;
    let mut worst = 0.0f32;
    for d in &out {
        worst = worst.max((d.center[1] - 200.0).abs());
    }
    assert!(
        worst <= reach,
        "uma marca caiu a {worst:.2} px do caminho, e o alcance autorado é {reach:.2}"
    );
    // …e o CONTROLE: ela de facto usa o alcance (senão o gate passaria sobre um jitter morto).
    assert!(
        worst > reach * 0.5,
        "controle: a nuvem mal saiu do eixo ({worst:.2} de {reach:.2}) — o jitter não está armado"
    );
}

/// **5. Um carimbo DELIBERADO não espalha.** Drag Dot, Anchored e Grid Stamp põem UMA marca no lugar
/// que o artista apontou — espalhá-los contradiria a razão de existir de cada um, e é por isso que
/// eles já não sorteiam jitter nenhum.
#[test]
fn a_deliberate_stamp_does_not_spray() {
    for method in [
        StrokeMethod::DragDot,
        StrokeMethod::Anchored,
        StrokeMethod::GridStamp,
    ] {
        let mut one = spec(1);
        one.stroke_method = method;
        let mut many = spec(8);
        many.stroke_method = method;
        let a = straight(one, 400.0).len();
        let b = straight(many, 400.0).len();
        assert!(a > 0, "controle: {method:?} tem de carimbar alguma coisa");
        assert_eq!(
            b, a,
            "{method:?} espalhou: {a} marcas com count 1, {b} com count 8"
        );
    }
}

/// **6. A Symmetry multiplica a nuvem INTEIRA** — toda marca sai pela porta do espelho, como o dab
/// base. Se uma cópia a contornasse, um traço espelhado sprayado pintaria metade da nuvem só de um
/// lado.
#[test]
fn symmetry_mirrors_every_mark_of_the_cloud() {
    let mut sp = spec(4);
    sp.symmetry = SymmetrySettings {
        enabled: true,
        circular: false,
        axis: MirrorAxis::X,
        center: [400.0, 400.0],
        ..Default::default()
    };
    let mirrored = straight(sp, 400.0).len();
    let plain = straight(spec(4), 400.0).len();
    assert_eq!(
        mirrored,
        plain * 2,
        "o espelho tem de dobrar a nuvem inteira: {plain} → {mirrored}"
    );
}

/// **7. A memória dos fios recebe UM ponto por ponto do CAMINHO**, não `n`.
///
/// ⚠️ O Sketchy costura o traço a si mesmo. Se cada cópia do spray entrasse na memória, o traço
/// passaria a costurar-se à própria nuvem — e a contagem de fios explodiria com um slider que não
/// fala de fios. As cópias não passam pelo [`Stroke::emit`] exatamente por isso.
#[test]
fn the_spray_does_not_feed_the_thread_memory() {
    let threads_for = |count: u32| {
        let mut sp = spec(count);
        sp.line_kind = LineKind::Sketchy;
        sp.sketchy_reach = 2.0;
        sp.sketchy_density = 0.4;
        let mut s = Stroke::new(sp, plain_dynamics(), 7);
        let mut out = Vec::new();
        let mut threads = Vec::new();
        let mut total = 0usize;
        s.begin(
            StrokePoint {
                pos: [200.0, 200.0],
                pressure: 1.0,
            },
            &mut out,
        );
        s.take_threads(&mut threads);
        total += threads.len();
        // Um zigue-zague apertado: é onde o Sketchy tem vizinhos legítimos a costurar.
        for k in 1..=24 {
            #[allow(clippy::cast_precision_loss)]
            let x = 200.0 + (k as f32) * 3.0;
            let y = if k % 2 == 0 { 200.0 } else { 212.0 };
            s.extend(
                StrokePoint {
                    pos: [x, y],
                    pressure: 1.0,
                },
                &mut out,
            );
            threads.clear();
            s.take_threads(&mut threads);
            total += threads.len();
        }
        total
    };
    let one = threads_for(1);
    assert!(
        one > 0,
        "controle: o Sketchy tem de costurar alguma coisa ({one})"
    );
    assert_eq!(
        threads_for(8),
        one,
        "o spray alimentou a memória dos fios — a contagem mudou com um slider que não fala de fios"
    );
}

/// **O teto é honrado**, e um `count` degenerado cai no neutro em vez de emitir nada.
#[test]
fn the_count_is_clamped_to_the_measured_ceiling() {
    let one = straight(spec(1), 400.0).len();
    assert_eq!(
        straight(spec(0), 400.0).len(),
        one,
        "count 0 tem de valer 1"
    );
    assert_eq!(
        straight(spec(SPRAY_COUNT_MAX + 7), 400.0).len(),
        one * SPRAY_COUNT_MAX as usize,
        "acima do teto tem de ser clampado NO teto"
    );
}
