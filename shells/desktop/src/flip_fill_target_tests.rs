//! Gates do **BUGS #19** — a forma que se CRUZA e a região fechada por vários traços
//! (smoke do Enio, 2026-07-18). Módulo irmão: o `flip_fill_tests.rs` bateu no teto de
//! LOC do shell (600), e estes gates são de UM assunto só — o critério do
//! `flip_fill_target::filled_shape_target`.

use crate::flip_fill::tests::{boxed_drawing, style};
use crate::flip_fill::{boundaries, fill_click, ring_area, ring_contains};
use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_tool_flip::FillMode as ToolFillMode;
use ph2d_vec_scene::Xform;

// ─────────────────────────────────────────────────────────────────────────────
// BUGS #19 — a forma que se CRUZA (smoke do Enio, 2026-07-18)
// ─────────────────────────────────────────────────────────────────────────────

/// A gota do screenshot: o traço desce pela esquerda, contorna o fundo, sobe pela
/// direita e **cruza a própria descida**, deixando duas pontas para fora.
///
/// (Verificado à mão: o segmento P4→P5 cruza o P0→P1 em t≈0,059 / s≈0,706.)
fn self_crossing_teardrop() -> FlipDrawing {
    let mut d = FlipDrawing::new();
    let mut s = FlipStroke::new();
    for p in [
        Vec2::new(2.0, 5.0),  // P0 — ponta de cima, começa a descer
        Vec2::new(0.0, 2.0),  // P1
        Vec2::new(1.0, -1.0), // P2 — fundo
        Vec2::new(3.0, -1.0), // P3
        Vec2::new(4.0, 2.0),  // P4 — sobe pela direita
        Vec2::new(1.0, 6.0),  // P5 — CRUZA o P0→P1 e sai para fora
    ] {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    d.strokes.push(s);
    d
}

/// **REPRO do smoke: o fill de uma forma que se cruza sai fora da linha.**
///
/// O `filled_shape_target` roda DEPOIS do solver e, quando dispara, joga fora o
/// contorno traçado e pinta o polígono do PRÓPRIO traço. Num traço que se cruza esse
/// polígono não é a região que o usuário vê: o even-odd o lê como o lobo grande **mais
/// a cunha** entre as duas pontas — que é literalmente o triângulo do screenshot.
///
/// E é por isso que **Gap e Trap não ajudam**: os dois mudam o contorno TRAÇADO, e o
/// contorno traçado é justamente o que este caminho descarta.
#[test]
fn a_self_crossing_shape_is_not_filled_by_its_own_polygon() {
    let mut d = self_crossing_teardrop();
    let click = Vec2::new(2.0, 1.0); // bem dentro do lobo grande

    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        click,
        10.0 / 1080.0,
        &Xform::IDENTITY,
    )
    .expect("o lobo da gota e uma regiao fechada — tem de preencher");

    // O traço ORIGINAL não pode ter ganhado `fill`: um traço que se cruza não tem um
    // interior só, então "a forma pinta a si mesma" não se aplica a ele.
    assert!(
        d.strokes.iter().all(|s| s.hide_stroke || s.fill.is_none()),
        "o traco que se CRUZA pintou o proprio poligono — e o even-odd dele inclui a \
         cunha entre as duas pontas (o triangulo do screenshot). A regiao tem de vir \
         do contorno TRACADO, que e o unico que sabe onde o usuario clicou."
    );

    // E o preenchimento existe (o irmão de PRESENÇA — "não pintou o polígono" ficaria
    // verde com fill nenhum).
    let region = d
        .strokes
        .iter()
        .find(|s| s.hide_stroke && s.fill.is_some())
        .expect("tem de haver uma regiao preenchida");

    // A cor não pode alcançar a cunha: um ponto claramente ACIMA do cruzamento, entre
    // as duas pontas, é fundo — e era ele que aparecia laranja no screenshot.
    assert!(
        !ring_contains(region.positions(), Vec2::new(1.9, 5.4)),
        "a cor vazou para a cunha entre as duas pontas do traco"
    );
}

/// **A régua que escolhe o critério** (`--ignored --nocapture`).
///
/// Mede, para uma forma LEGÍTIMA (o quadrado que deve auto-preencher) e para a gota que
/// se CRUZA, duas grandezas: o erro de ÁREA (o critério atual) e a distância máxima de
/// um ponto do traço ao contorno traçado. É a tabela que decide, em vez do olho.
#[test]
#[ignore = "régua — rode com --ignored --nocapture"]
fn measure_which_criterion_separates_the_two_cases() {
    fn max_dist_to_ring(pts: &[Vec2], ring: &[Vec2]) -> f32 {
        pts.iter()
            .map(|p| {
                let n = ring.len();
                (0..n)
                    .map(|i| {
                        let (a, b) = (ring[i], ring[(i + 1) % n]);
                        let ab = Vec2::new(b.x - a.x, b.y - a.y);
                        let l2 = ab.x * ab.x + ab.y * ab.y;
                        let t = if l2 <= 0.0 {
                            0.0
                        } else {
                            (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
                        };
                        let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
                        (dx * dx + dy * dy).sqrt()
                    })
                    .fold(f32::INFINITY, f32::min)
            })
            .fold(0.0f32, f32::max)
    }

    println!("\n=== o que separa 'a forma pinta a si mesma' de 'nao pinta' ===");
    println!(
        "{:>22} {:>12} {:>14} {:>14}",
        "caso", "erro area", "dist max", "dist/eps_rdp"
    );

    let px_to_world = 10.0f32 / 1080.0;
    let precision_buf = 1.6f32 / px_to_world; // px de buffer por unidade de doc
    let eps = ph2d_flip_fill::RDP_EPSILON_PX / precision_buf;

    // Um circulo de N lados (racional, HR-5): a forma legitima com MUITOS pontos.
    fn polygon(n: usize, r: f32) -> FlipDrawing {
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        for i in 0..n {
            // `sin`/`cos` aqui e DELIBERADO: isto e uma regua de medicao, nao codigo de
            // produto, e um circulo de verdade importa mais que a pureza de HR-5. (A
            // parametrizacao racional "esperta" ja custou um falso vermelho nesta linha:
            // BUGS #13, o helper que descrevia um SEMIcirculo.)
            let a = (i as f32 / n as f32) * std::f32::consts::TAU;
            let (x, y) = (a.cos(), a.sin());
            s.push_point(Point {
                pos: Vec2::new(x * r, y * r),
                width: 6.0,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        s.closed = true;
        d.strokes.push(s);
        d
    }
    // Uma forma legitima TREMIDA (a mao humana nao desenha reto).
    fn wobbly() -> FlipDrawing {
        let base = polygon(64, 3.0);
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        let mut seed = 0x1234_5678u32;
        for p in base.strokes[0].positions() {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let j = ((seed >> 16) & 0xFF) as f32 / 255.0 - 0.5;
            s.push_point(Point {
                pos: Vec2::new(p.x + j * 0.08, p.y - j * 0.08),
                width: 6.0,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        s.closed = true;
        d.strokes.push(s);
        d
    }

    for (name, d, click) in [
        ("quadrado (legitimo)", boxed_drawing(), Vec2::new(0.5, 0.5)),
        (
            "poligono 64 (legitimo)",
            polygon(64, 3.0),
            Vec2::new(0.0, 0.0),
        ),
        (
            "poligono 200 (legitimo)",
            polygon(200, 3.0),
            Vec2::new(0.0, 0.0),
        ),
        ("tremido (legitimo)", wobbly(), Vec2::new(0.0, 0.0)),
        (
            "gota que se cruza",
            self_crossing_teardrop(),
            Vec2::new(2.0, 1.0),
        ),
    ] {
        let strokes = boundaries(&d);
        let Ok(r) = ph2d_flip_fill::fill_at(
            &strokes,
            click,
            ph2d_flip_fill::FillParams {
                precision: precision_buf,
                ..Default::default()
            },
        ) else {
            let e = ph2d_flip_fill::fill_at(
                &strokes,
                click,
                ph2d_flip_fill::FillParams {
                    precision: precision_buf,
                    ..Default::default()
                },
            )
            .unwrap_err();
            println!("{name:>22}   (o solver recusou: {e:?})");
            continue;
        };
        let s = &d.strokes[0];
        let a_stroke = ring_area(s.positions()).abs();
        let a_traced = ring_area(&r.outer).abs();
        let area_err = (a_stroke - a_traced).abs() / a_stroke.max(a_traced);
        let dist = max_dist_to_ring(s.positions(), &r.outer);
        println!(
            "{name:>22} {:>11.1}% {:>14.4} {:>14.2}",
            area_err * 100.0,
            dist,
            dist / eps
        );
    }
    println!("\n(o criterio atual e AREA <= 15%; eps_rdp = {eps:.4} unidades de doc)\n");
}

/// **A 3ª foto do smoke: a região é fechada por VÁRIOS traços, e a cor não pode sair
/// com a forma de um deles.**
///
/// Um "C" (esquerda, fundo, direita de uma caixa) + um arco que fecha o topo **por
/// fora**. A região que o balde traça inclui a barriga do arco; o polígono do "C",
/// porém, fecha com uma CORDA RETA — então pintar "a forma do C" corta exatamente onde
/// a barriga está, e a cor aparece fora da linha.
///
/// ⚠️ **O pré-filtro de área NÃO pega isto** (a barriga é pequena), e a direção
/// *traço→contorno* também não (todo ponto do C está sobre a fronteira). Quem pega é a
/// direção *contorno→traço*: os pontos da barriga estão longe do C. É o gate dessa
/// camada — sem ele a mutação que remove a 2ª direção sobrevive
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn a_region_closed_by_another_stroke_is_not_filled_by_the_first_ones_polygon() {
    let mut d = FlipDrawing::new();

    // O "C": sobe pela esquerda, atravessa o fundo, sobe pela direita.
    let mut c = FlipStroke::new();
    for p in [
        Vec2::new(-2.0, 2.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, -2.0),
        Vec2::new(2.0, 2.0),
    ] {
        c.push_point(Point {
            pos: p,
            width: 0.1,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    d.strokes.push(c);

    // O arco que fecha o topo, com uma barriga para FORA (o pico em y = 3).
    let mut arc = FlipStroke::new();
    for p in [
        Vec2::new(2.0, 2.0),
        Vec2::new(1.0, 2.7),
        Vec2::new(0.0, 3.0),
        Vec2::new(-1.0, 2.7),
        Vec2::new(-2.0, 2.0),
    ] {
        arc.push_point(Point {
            pos: p,
            width: 0.1,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    d.strokes.push(arc);

    fill_click(
        &mut d,
        &style(ToolFillMode::Paint),
        Vec2::new(0.0, 0.0),
        10.0 / 1080.0,
        &Xform::IDENTITY,
    )
    .expect("a regiao esta fechada — tem de preencher");

    assert!(
        d.strokes.iter().all(|s| s.hide_stroke || s.fill.is_none()),
        "um dos traços pintou o PRÓPRIO polígono; a região é fechada por dois, e a \
         corda reta do 'C' corta a barriga do arco — a cor sai fora da linha"
    );

    // Presença: a região existe, e ela ALCANÇA a barriga (que a corda reta cortaria).
    let region = d
        .strokes
        .iter()
        .find(|s| s.hide_stroke && s.fill.is_some())
        .expect("tem de haver uma regiao preenchida");
    assert!(
        ring_contains(region.positions(), Vec2::new(0.0, 2.5)),
        "a cor nao alcancou a barriga do arco — o preenchimento parou na corda reta"
    );
}
