//! **A JANELA DO RASTO CABE O DAB INTEIRO** (smoke do dono, 2026-10-16: «parecem dois dabs de cada vez,
//! mas o segundo cortado»).
//!
//! A janela ancora no 1.º dab e só crescia ali, com a margem `4 × spacing` (o knob escondido do motor, 2
//! células) — a deriva de uma janela no caminho do PRÓPRIO motor, onde os dabs estão a um `spacing`. Na
//! porta da lane (o pincel do produto) cada dab chega com a sua CORDA (o Spacing do pincel × o diâmetro, o
//! sub-passo da cápsula, o jitter), e `window_size = ⌊corda ÷ spacing⌋ ≤ 2` junta até três dabs a
//! `2 × corda` da âncora: o que passava de `âncora ± half` molhava a célula (`wrote`) e perdia o pigmento
//! numa recta vertical. Medido no hospedeiro (Spacing 0,3, raio 49): a borda vertical era `0,79` da altura
//! do traço e a tinta `0,931` da do mesmo traço numa janela grande.
//!
//! O oráculo é o MESMO traço numa janela crescida de antemão (o `fit_to` só cresce e o `start_stroke`
//! guarda a meia-largura): uma janela maior não muda a lei de nenhuma célula, só deixa de cortar.

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};

const Y: f64 = 150.0;

fn dab(x: f64, r: f64) -> Dab {
    Dab {
        x,
        y: Y,
        r,
        hardness: 0.5,
        intensity: 1.0,
        water_amount: 0.5,
        dry_gate: 0.0,
        reach: 1.0,
        window_reach: 1.0,
    }
}

/// Uma janela crescida para o raio `r` (um dab longe da grade: nada pousa, a janela cresce).
fn janela_crescida(e: &mut Engine, r: f64) -> Trail {
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let mut t = Trail::default();
    t.start_stroke(-5000.0, -5000.0, [0.0; 3], TrailMode::Paint);
    let mut longe = dab(-5000.0, r);
    longe.y = -5000.0;
    let _ = t.accumulate_paint(e.active_grid_mut(), &p, &tex, &longe, false);
    t.drop_remainder();
    t
}

/// O que um traço pela porta da lane deixa: `(dabs cujo carimbo saiu da janela, pigmento da janela antes de
/// cada pouso, a grade final susp + sett)`.
type Traco = (Vec<String>, Vec<f64>, Vec<f32>);

/// O traço da porta da lane: `n` dabs de raio `r` a `corda` células, cada um com o seu `on_segment` (o
/// primeiro nasce a lane e pousa sozinho, como no hospedeiro), e os raios `raios(k)`.
fn traco_da_lane(crescida: bool, corda: f64, raios: &dyn Fn(usize) -> f64, n: usize) -> Traco {
    let mut e = Engine::new(900, 300);
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let mut t = if crescida {
        janela_crescida(&mut e, 600.0)
    } else {
        Trail::default()
    };
    let x0 = 150.0;
    t.start_stroke(x0, Y, [40.0, 80.0, 200.0], TrailMode::Paint);
    let (mut fora, mut na_janela) = (Vec::new(), Vec::new());
    #[allow(clippy::cast_precision_loss)]
    for k in 0..n {
        let x = x0 + corda * k as f64;
        if k > 0 {
            t.on_segment(corda, 2.0);
        }
        let g = e.active_grid_mut();
        let acc = t.accumulate_paint(g, &p, &tex, &dab(x, raios(k)), false);
        let (ax, ay) = t.anchor_for_measure();
        let h = t.window_half_for_measure();
        if let Some(w) = acc.wrote
            && (w.x0 < ax - h || w.x1 > ax + h || w.y0 < ay - h || w.y1 > ay + h)
        {
            fora.push(format!(
                "dab {k} em x {x:.0}: carimbo x {}..{} contra a janela {}..{}",
                w.x0,
                w.x1,
                ax - h,
                ax + h
            ));
        }
        if acc.window_full {
            na_janela.push(t.planes_for_measure().0.iter().map(|&v| f64::from(v)).sum());
            let _ = t.transfer_paint(g, &p);
        }
    }
    let g = e.active_grid();
    let m = g.susp.iter().zip(&g.sett).map(|(a, b)| a + b).collect();
    (fora, na_janela, m)
}

fn confere(nome: &str, corda: f64, raios: &dyn Fn(usize) -> f64, n: usize) {
    let (fora, janela, grade) = traco_da_lane(false, corda, raios, n);
    let (_, janela_o, grade_o) = traco_da_lane(true, corda, raios, n);
    assert!(
        fora.is_empty(),
        "{nome}: {} dabs carimbaram fora da janela — {:?}",
        fora.len(),
        &fora[..fora.len().min(3)]
    );
    assert_eq!(janela.len(), janela_o.len(), "{nome}: os pousos");
    for (k, (a, o)) in janela.iter().zip(&janela_o).enumerate() {
        assert!(
            (a - o).abs() <= 1e-9 * o.abs().max(1.0),
            "{nome}: o pigmento da janela no pouso {k}: {a:.3} contra {o:.3} da janela grande"
        );
    }
    let diferentes = grade.iter().zip(&grade_o).filter(|(a, b)| a != b).count();
    assert_eq!(
        diferentes, 0,
        "{nome}: células da grade diferentes da janela grande"
    );
}

/// ⭐ **Dabs a 30 células, raio 49** — o 3.º dab de cada janela estava a 60 células da âncora, a janela
/// tinha meia-largura 61: o carimbo ia até `âncora + 109` e o pigmento parava em `âncora + 61`.
///
/// **Mutação que sangra:** tirar a corda da margem do `fit_to` E a rede (`ensure_fits`).
#[test]
fn every_dab_of_a_spaced_lane_stroke_fits_the_window() {
    confere("raio 49 · corda 30", 30.0, &|_| 49.0, 12);
}

/// ⭐ **O raio que CRESCE dentro de uma janela** (a caneta a pousar, o sub-dab da cápsula que interpola o
/// raio) — a janela foi medida pelo 1.º dab da janela; os seguintes são maiores dos DOIS lados. A corda
/// é pequena (a margem da corda não chega): só a rede que cresce a janela no lugar o cobre.
///
/// **Mutação que sangra:** tirar a rede (`ensure_fits`) — a corda sozinha não chega.
#[test]
fn a_radius_that_grows_inside_a_window_still_fits() {
    #[allow(clippy::cast_precision_loss)]
    confere(
        "raio a crescer 40 → 300",
        2.0,
        &|k| 40.0 + 30.0 * k as f64,
        10,
    );
}

/// **O Blend também cabe** — a janela do Blend nunca era medida (`accumulate_blend` não chamava o
/// `fit_to`): um pincel de raio 100 esfregava só `± 61` células à volta da âncora.
///
/// **Mutação que sangra:** tirar o `anchor_first_dab` do `accumulate_blend`.
#[test]
fn a_blend_dab_fits_the_window() {
    let largura = |crescida: bool| {
        let mut e = Engine::new(900, 400);
        let p = e.sim.gather_params(&e.tuning);
        let tex = e.bristle_texture_for_measure();
        let mut t = if crescida {
            janela_crescida(&mut e, 600.0)
        } else {
            Trail::default()
        };
        t.start_stroke(450.0, Y, [40.0, 80.0, 200.0], TrailMode::Blend);
        let mut d = dab(450.0, 100.0);
        d.y = 200.0;
        let _ = t.accumulate_blend(e.active_grid(), &p, &tex, &d);
        t.touched_extent_for_measure()
            .map_or(0, |(x0, _, x1, _)| x1 - x0 + 1)
    };
    let (a, o) = (largura(false), largura(true));
    assert_eq!(a, o, "a largura esfregada: {a} contra {o} da janela grande");
}

/// **A corda mede a janela NA ÂNCORA** — a rede que cresce a meio da janela é o cinto (o raio que cresce, o
/// jitter); a deriva de um traço espaçado conhece-se no 1.º dab da janela, e é lá que a janela cresce (uma
/// realocação por traço, não uma por janela).
///
/// **Mutação que sangra:** a margem do `fit_to` sem a corda (`4 × spacing`) — a rede cresce a janela no
/// 3.º dab.
#[test]
fn the_chord_sizes_the_window_at_its_anchor() {
    let mut e = Engine::new(900, 300);
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let mut t = Trail::default();
    t.start_stroke(150.0, Y, [40.0, 80.0, 200.0], TrailMode::Paint);
    let mut a_meio = Vec::new();
    let mut no_inicio = true;
    #[allow(clippy::cast_precision_loss)]
    for k in 0..12 {
        if k > 0 {
            t.on_segment(30.0, 2.0);
        }
        let antes = t.window_half_for_measure();
        let g = e.active_grid_mut();
        let acc = t.accumulate_paint(g, &p, &tex, &dab(150.0 + 30.0 * k as f64, 49.0), false);
        let depois = t.window_half_for_measure();
        if !no_inicio && depois != antes {
            a_meio.push(format!("dab {k}: {antes} → {depois}"));
        }
        no_inicio = acc.window_full;
        if acc.window_full {
            let _ = t.transfer_paint(g, &p);
        }
    }
    assert!(a_meio.is_empty(), "a janela cresceu a meio: {a_meio:?}");
}
