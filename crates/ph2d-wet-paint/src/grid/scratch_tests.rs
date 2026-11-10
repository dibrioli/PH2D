//! B12.4 (missão 4K): o 1.º [`SolverScratch::ensure`] não ESCREVE a tela — todo plano nasce a zeros, a página
//! que o sistema dá preguiçosa (`alloc_zeroed`). O `bt` a [`SolverScratch::BT_NADA`] eram `n` bytes escritos
//! (8,3 MB a 2880²) que nenhum passe lê: o `prepare_rows` escreve a faixa viva que o `outflow_rows` lê. A
//! paridade ao bit do passo é a de `advect_jacobi_bits_are_the_recorded_ones` (constante gravada antes).
use super::SolverScratch;

/// Os elementos que o `ensure` deixou diferentes de zero (bit a bit) — o que ele teve de ESCREVER.
fn escritos(s: &SolverScratch) -> usize {
    let f = |v: &[f32]| v.iter().filter(|x| x.to_bits() != 0).count();
    f(s.uv.as_flattened())
        + f(&s.outflow)
        + s.bt.iter().filter(|&&x| x != 0).count()
        + f(&s.dst_film)
        + f(&s.dst_susp)
        + f(s.dst_rgb.as_flattened())
        + s.edge.iter().filter(|&&x| x != 0).count()
}

#[test]
fn o_primeiro_ensure_nao_escreve_o_rascunho() {
    let n = 4096;
    let mut s = SolverScratch::default();
    s.ensure(n);
    assert_eq!(s.bt.len(), n);
    assert_eq!(
        escritos(&s),
        0,
        "o ensure escreveu planos que nenhum passe lê"
    );
}

/// Os planos que o passo do motor escreve na grade, bit a bit.
fn estado(g: &crate::grid::Grid) -> Vec<u32> {
    let mut v = Vec::new();
    for p in [&g.film, &g.susp, &g.sett, &g.vel_x, &g.vel_y, &g.relevo] {
        v.extend(p.iter().map(|x| x.to_bits()));
    }
    for p in [&g.susp_rgb, &g.sett_rgb] {
        v.extend(p.as_flattened().iter().map(|x| x.to_bits()));
    }
    v.extend(g.active.iter().map(|&x| u32::from(x)));
    v.extend(g.bloom.iter().map(|&x| u32::from(x)));
    v.extend(g.wet.iter().map(|&x| u32::from(x)));
    v
}

/// Quatro traços molhados e a água a correr; `veneno` = o rascunho nasce a lixo.
fn pinta(rf: usize, backrun: f64, macia: f64, veneno: bool) -> Vec<u32> {
    use crate::painter::Engine;
    use crate::tuning::Knob;
    super::veneno::LIGADO.set(veneno);
    let mut e = Engine::with_flow_ratio(300, 220, rf);
    e.sliders.water = 1.0;
    e.tuning.set(Knob::ExtBackrun, backrun);
    e.tuning.set(Knob::EdgeSoftMass, macia);
    for (k, (x, y)) in [(40.0, 40.0), (200.0, 60.0), (60.0, 150.0), (180.0, 170.0)]
        .into_iter()
        .enumerate()
    {
        e.pointer_down(x, y, None);
        for i in 0..6 + k {
            e.pointer_frame(x + 6.0 * i as f64, y + 2.0 * i as f64);
            e.step_simulation();
        }
        e.pointer_up();
        for _ in 0..20 {
            e.step_simulation();
        }
    }
    super::veneno::LIGADO.set(false);
    estado(e.active_grid())
}

/// ⭐ B12.4: **nenhum passe lê o rascunho que não escreveu** — o que torna seguro nascer a zeros sem escrever
/// (`alloc_zeroed`). Com cada plano novo envenenado, a grade de cada caso (grade de fluxo fina e grossa, backrun,
/// borda macia) sai ao bit a do rascunho limpo; o contador prova que o veneno chegou a cada passe.
#[test]
fn o_rascunho_envenenado_da_a_mesma_grade() {
    use std::sync::atomic::Ordering::SeqCst;
    let casos = [(1, 0.0, 0.0), (2, 0.0, 0.0), (1, 1.0, 0.0), (1, 0.0, 2.0)];
    // A rota fundida e a de fases (esta lê `uv`/`bt`/`outflow` do rascunho; a escolha não muda bits).
    for fases in [false, true] {
        if fases {
            crate::solver::ADVECT_FASES.fetch_add(1, SeqCst);
        }
        for (rf, backrun, macia) in casos {
            confere_veneno(rf, backrun, macia);
        }
        if fases {
            crate::solver::ADVECT_FASES.fetch_sub(1, SeqCst);
        }
    }
}

fn confere_veneno(rf: usize, backrun: f64, macia: f64) {
    let limpo = pinta(rf, backrun, macia, false);
    let antes = super::veneno::APLICADOS.get();
    let envenenado = pinta(rf, backrun, macia, true);
    let aplicados = super::veneno::APLICADOS.get() - antes;
    let minimo = if backrun > 0.0 { 2 } else { 1 };
    assert!(
        aplicados >= minimo,
        "rf {rf} backrun {backrun}: o veneno não chegou ao passo ({aplicados})"
    );
    let n = limpo.len() / 11;
    assert!(
        limpo[3 * n..5 * n].iter().any(|&v| v != 0),
        "rf {rf}: a água não correu (velocidade nula)"
    );
    assert_eq!(limpo.len(), envenenado.len());
    let difs = limpo
        .iter()
        .zip(&envenenado)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        difs, 0,
        "rf {rf} backrun {backrun} macia {macia}: valores mudaram com o rascunho envenenado"
    );
}
