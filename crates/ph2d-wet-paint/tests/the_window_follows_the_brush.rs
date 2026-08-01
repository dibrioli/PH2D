//! **A JANELA DO TRAIL SEGUE O PINCEL** — o cap que o modelo de referência
//! impunha ao produto.
//!
//! ⚠️ **Report do Enio (2026-08-01):** *"todos esses testes tenho feito com raio
//! 300, mas na prática o app limita o tamanho para aproximadamente 200"*. Medido
//! pela porta do artista antes de tocar em código: o slider promete
//! `BRUSH_SIZE_MAX_PX = 512` e o traço **saturava em 119 px de largura** a
//! partir do raio 100 — raio efetivo **59,5**, **0,15×** do pedido em 400, e em
//! SILÊNCIO.
//!
//! A causa é o `TRAIL_HALF = 61 // ceil(35 + 4*6) + 2`: **35 é o teto de raio do
//! modelo JS de referência**, e ele era o teto deste produto. É a forma exata
//! que o CLAUDE.md §0 nomeia — *nunca deixe o fallback definir o produto* — e o
//! §0 também diz o que fazer: **medir**, e escrever o número que a medição deu.

use ph2d_wet_paint::brush::BrushShape;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Dab, TRAIL_HALF, TRAIL_SIZE, Trail, TrailMode};

fn dab(x: f64, r: f64) -> Dab {
    Dab {
        x,
        y: 200.0,
        r,
        hardness: 0.5,
        intensity: 1.0,
        water_amount: 0.5,
        dry_gate: 0.0,
        shape: BrushShape::Round,
        dir_x: 1.0,
        dir_y: 0.0,
    }
}

/// **A janela cresce com o raio** — e o oráculo é o RETÂNGULO TOCADO, nunca uma
/// constante (o gate não pode espelhar a regra que ele julga).
///
/// Mutação que sangra: devolver cedo de `Trail::fit_to` (o cap de volta) — a
/// largura tocada satura e as duas asserções de crescimento morrem.
#[test]
fn the_touched_rect_grows_with_the_brush_instead_of_saturating() {
    let mut e = Engine::new(1400, 400);
    let p = e.sim.gather_params(&e.tuning);
    let tex: Vec<f32> = e.bristle_texture_for_measure();
    let mut widths = Vec::new();
    for r in [30.0f64, 120.0, 300.0] {
        let mut t = Trail::default();
        t.start_stroke(700.0, 200.0, [0.2, 0.3, 0.4], TrailMode::Paint);
        t.on_segment(4.0, 0.05 * 2.0 * r);
        let mut ge = Engine::new(1400, 400);
        let g = ge.active_grid_mut();
        let _ = t.accumulate_paint(g, &p, &tex, &dab(700.0, r), false);
        let rect = t.touched_extent_for_measure();
        widths.push(rect.map_or(0, |(x0, _, x1, _)| (x1 - x0 + 1).max(0)));
    }
    assert!(
        widths[1] > widths[0] * 2,
        "raio 120 tocou {} contra {} do raio 30: a janela parou de crescer com o \
         pincel, e o teto do MODELO de referencia voltou a ser o teto do PRODUTO",
        widths[1],
        widths[0]
    );
    assert!(
        widths[2] > widths[1] * 2,
        "raio 300 tocou {} contra {} do raio 120: idem — e e no raio grande que o \
         artista bate no cap (medido: o traco saturava em 119 px de largura)",
        widths[2],
        widths[1]
    );
}

/// **E o PISO fica**, que é o que mantém o fingerprint byte-idêntico.
///
/// ⚠️ Um pincel dentro do teto do modelo JS tem de produzir a janela EXATA de
/// antes — é isto que torna a wave uma extensão e não uma mudança de modelo, e
/// é por isso que o `TRAIL_HALF` continua existindo como PISO.
#[test]
fn a_brush_within_the_reference_models_ceiling_keeps_the_old_window() {
    let mut e = Engine::new(400, 400);
    let p = e.sim.gather_params(&e.tuning);
    let tex: Vec<f32> = e.bristle_texture_for_measure();
    let mut t = Trail::default();
    t.start_stroke(200.0, 200.0, [0.2, 0.3, 0.4], TrailMode::Paint);
    t.on_segment(4.0, 6.0);
    let g = e.active_grid_mut();
    let _ = t.accumulate_paint(g, &p, &tex, &dab(200.0, 20.0), false);
    assert_eq!(
        t.window_half_for_measure(),
        TRAIL_HALF,
        "um pincel dentro do teto do modelo (raio 20 <= 35) moveu a janela: o \
         fingerprint do ADR-0134 deixa de ser byte-identico POR CONSTRUCAO"
    );
    assert_eq!(TRAIL_SIZE, TRAIL_HALF * 2 + 1);
}
