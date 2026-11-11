//! B13.A1 (AU-12-1): a ladeira do óleo só visita a faixa viva, e o que o passe de fluxo LÊ dela é o mesmo bit da
//! varredura da folha inteira de antes (o oráculo [`de_antigo`], verbatim).

use super::{LadeiraDoOleo, altura, fraccao_assente, rota};
use crate::grid::Grid;
use crate::painter::Engine;
use std::cell::RefCell;

/// O que o gancho de [`super::do_passe`] viu neste fio.
#[derive(Default)]
pub(super) struct Prova {
    pub chamadas: usize,
    pub visitadas: usize,
    pub lidas: usize,
    pub folha: usize,
    pub diferentes: usize,
    /// Leituras da coluna `>= LONGE` (a 2.ª mancha nas mesmas linhas).
    pub longe: usize,
}

thread_local! {
    pub(super) static PROVA: RefCell<Option<Prova>> = const { RefCell::new(None) };
}

/// O `de` de antes de B13.A1, verbatim: a folha inteira (`0..s`) das linhas `r0..=r1`, em série.
fn de_antigo(g: &Grid, r0: usize, r1: usize, antes: bool) -> Option<(usize, Vec<f32>, Vec<f32>)> {
    if g.oleo.is_empty() || r1 < r0 || (antes && g.oleo_antes.len() != g.cells) {
        return None;
    }
    let s = g.s;
    let r1 = r1.min(g.cells / s - 1);
    let mut h = vec![0f32; (r1 + 1 - r0) * s];
    let mut assente = vec![1f32; (r1 + 1 - r0) * s];
    let mut alguma = false;
    for y in r0..=r1 {
        for x in 0..s {
            let i = y * s + x;
            let k = (y - r0) * s + x;
            match rota() {
                4 => assente[k] = fraccao_assente(g, i),
                5..=7 if antes => {
                    assente[k] = g.oleo_antes_assente.get(i).copied().unwrap_or(1.0);
                }
                _ => {}
            }
            let v = if antes {
                g.oleo_antes[i]
            } else {
                altura(g, i, false)
            };
            if v > 0.0 {
                h[(y - r0) * s + x] = v;
                alguma = true;
            }
        }
    }
    alguma.then_some((r0, h, assente))
}

/// As células finas que o passe de fluxo LÊ da ladeira (`em` e `assente`), enumeradas como o laço dele as lê.
fn lidas(g: &Grid) -> Vec<usize> {
    let (gw, gh, rf, fs) = (g.w as i32, g.h as i32, g.flow.rf, g.s);
    let (gbx0, by0, gbx1, by1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), rf);
    let p = |c, w| crate::flow::flow_probe(c, w, rf) as usize;
    let mut v = Vec::new();
    for y in by0..=by1 {
        let (bx0, bx1) =
            crate::grid::span_x_of(&g.frow_lo, &g.frow_hi, g.spans_enabled, gbx0, gbx1, y);
        let runs: Vec<(i32, i32)> =
            crate::grid::faixa_tiles::span_runs(g.ftiles_row(y), bx0, bx1).collect();
        for x in runs.into_iter().flat_map(|(a, b)| a..=b) {
            let (m, px) = (p(y, gh), p(x, gw));
            if g.active[m * fs + px] == 0 {
                continue;
            }
            v.extend([
                m * fs + p(x - 1, gw),
                m * fs + p(x + 1, gw),
                m * fs + px,
                p(y - 1, gh) * fs + px,
                p(y + 1, gh) * fs + px,
            ]);
        }
    }
    v
}

/// O gancho de [`super::do_passe`]: conta e compara, célula LIDA a célula lida, com o oráculo.
pub(super) fn observa(g: &Grid, r0: usize, r1: usize, antes: bool, l: &LadeiraDoOleo) {
    let nova = Some(l).filter(|l| l.alguma);
    PROVA.with_borrow_mut(|p| {
        let Some(p) = p.as_mut() else { return };
        p.chamadas += 1;
        p.folha += (r1.min(g.cells / g.s - 1) + 1).saturating_sub(r0) * g.s;
        p.visitadas += l.visitadas;
        let velha = de_antigo(g, r0, r1, antes);
        let em_v = |fi: usize| {
            velha.as_ref().map_or((0f32, 1f32), |(r0, h, a)| {
                let k = fi - r0 * g.s;
                (h[k], a[k])
            })
        };
        let em_n = |fi: usize| nova.map_or((0f64, 1f64), |l| (l.em(fi), l.assente(fi)));
        for fi in lidas(g) {
            p.lidas += 1;
            p.longe += usize::from(fi % g.s >= LONGE);
            let ((hv, av), (hn, an)) = (em_v(fi), em_n(fi));
            if f64::from(hv).to_bits() != hn.to_bits() || f64::from(av).to_bits() != an.to_bits() {
                p.diferentes += 1;
            }
        }
    });
}

const LONGE: usize = 3000;

fn traco(e: &mut Engine, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
    e.pointer_down(x0, y0, None);
    for i in 1..=8 {
        let t = f64::from(i) / 8.0;
        e.pointer_frame(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
    }
    e.pointer_up();
    while e.release_frame(x1, y1) {}
}

/// Folha da largura do 4K: um traço de Real Oil à direita e outro à esquerda (o óleo debaixo da água) e uma poça de água na
/// esquerda, que é o que corre. Antes: cada passe de fluxo varria `linhas × s` (a largura da folha inteira).
#[test]
fn a_ladeira_so_visita_a_faixa_viva() {
    const W: usize = 3840;
    const H: usize = 120;
    let mut e = Engine::new(W, H);
    e.sim.oleo = true;
    e.set_knob(crate::tuning::Knob::PigmentPerDab, 3000.0);
    e.sliders.size = 9.0;
    traco(&mut e, (3600.0, 30.0), (3700.0, 90.0));
    traco(&mut e, (40.0, 50.0), (70.0, 60.0));
    for _ in 0..60 {
        e.step_simulation();
    }
    // O produto só passa a ladeira ao nivelamento com o Real Oil na mão (sem ele o chão leva-a como força).
    e.sliders.water = 1.0;
    // A fotografia do óleo de antes (a porta `begin_direct_stroke` do traço novo; o `pointer_down` não passa por ela).
    super::fotografa(e.active_grid_mut(), true);
    e.pointer_down(30.0, 40.0, None);
    for i in 1..=8 {
        e.pointer_frame(30.0 + 4.0 * f64::from(i), 40.0 + 4.0 * f64::from(i));
    }
    e.pointer_up();
    PROVA.set(Some(Prova::default()));
    for _ in 0..24 {
        e.step_simulation();
    }
    let p = PROVA.take().expect("a prova");
    assert!(
        p.chamadas > 0 && p.lidas > 0,
        "o passe de fluxo não leu a ladeira ({} chamadas)",
        p.chamadas
    );
    assert_eq!(
        p.diferentes, 0,
        "{} de {} leituras mudaram de bit",
        p.diferentes, p.lidas
    );
    assert!(
        p.visitadas * 8 <= p.folha,
        "a ladeira visitou {} células de {} da folha (lidas {})",
        p.visitadas,
        p.folha,
        p.lidas
    );
}

/// B13.2: duas manchas de óleo NAS MESMAS LINHAS, água nas duas, ~55 tiles secos entre elas. A janela da ladeira
/// era um intervalo por linha fina (o vão inteiro); com os tiles da faixa de fluxo anda pelos troços. As leituras
/// do passe são o mesmo bit do oráculo, com e sem tiles; a 2.ª mancha é lida (não-vacuidade: `.take(1)` ⇒ NaN).
fn duas_manchas(tiles: bool) -> Prova {
    const W: usize = 3840;
    const H: usize = 120;
    let mut e = Engine::new(W, H);
    e.active_grid_mut().tiles_enabled = tiles;
    e.sim.oleo = true;
    e.set_knob(crate::tuning::Knob::PigmentPerDab, 3000.0);
    e.sliders.size = 9.0;
    traco(&mut e, (3600.0, 40.0), (3700.0, 70.0));
    traco(&mut e, (40.0, 40.0), (140.0, 70.0));
    for _ in 0..60 {
        e.step_simulation();
    }
    e.sliders.water = 1.0;
    super::fotografa(e.active_grid_mut(), true);
    for x0 in [50.0, 3610.0] {
        e.pointer_down(x0, 45.0, None);
        for i in 1..=8 {
            e.pointer_frame(x0 + 8.0 * f64::from(i), 45.0 + 2.0 * f64::from(i));
        }
        e.pointer_up();
    }
    PROVA.set(Some(Prova::default()));
    for _ in 0..24 {
        e.step_simulation();
    }
    PROVA.take().expect("a prova")
}

#[test]
fn a_ladeira_anda_pelos_tiles_da_faixa_ao_bit() {
    let (on, off) = (duas_manchas(true), duas_manchas(false));
    for (nome, p) in [("tiles", &on), ("intervalo", &off)] {
        assert!(
            p.chamadas > 0 && p.longe > 0,
            "{nome}: a 2.ª mancha não foi lida ({} leituras)",
            p.longe
        );
        assert_eq!(
            p.diferentes, 0,
            "{nome}: {} de {} leituras mudaram de bit",
            p.diferentes, p.lidas
        );
    }
    assert_eq!(on.lidas, off.lidas, "o passe leu outras células");
    assert!(
        on.visitadas * 3 <= off.visitadas,
        "a ladeira visitou {} células com tiles × {} sem",
        on.visitadas,
        off.visitadas
    );
}

/// Medição (B13.2): o `do_passe` da ladeira na cena de [`duas_manchas`] a 4K, tiles on × off intercalados no MESMO
/// processo, o mínimo de 12. `cargo test --profile smoke -p ph2d-wet-paint --lib mede_a_ladeira -- --ignored --nocapture`.
#[test]
#[ignore = "medição"]
fn mede_a_ladeira_tiles_on_off() {
    let mut e = Engine::new(3840, 120);
    e.sim.oleo = true;
    e.set_knob(crate::tuning::Knob::PigmentPerDab, 3000.0);
    e.sliders.size = 9.0;
    traco(&mut e, (3600.0, 40.0), (3700.0, 70.0));
    traco(&mut e, (40.0, 40.0), (140.0, 70.0));
    for _ in 0..60 {
        e.step_simulation();
    }
    super::fotografa(e.active_grid_mut(), true);
    for x0 in [50.0, 3610.0] {
        e.pointer_down(x0, 45.0, None);
        for i in 1..=8 {
            e.pointer_frame(x0 + 8.0 * f64::from(i), 45.0 + 2.0 * f64::from(i));
        }
        e.pointer_up();
    }
    let g = e.active_grid_mut();
    g.refresh_flow_spans();
    let bb = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), g.flow.rf);
    let mut min = [f64::MAX; 2];
    for k in 0..24 {
        let v = (k + k / 2) % 2;
        g.tiles_enabled = v == 0;
        let t = std::time::Instant::now();
        for _ in 0..10 {
            if let Some(l) = super::do_passe(g, bb, true, crate::par::Rows::Serial) {
                l.devolve(g);
            }
        }
        min[v] = min[v].min(t.elapsed().as_secs_f64() * 100.0);
    }
    println!(
        "ladeira do_passe ms/passe (mín. de 12): tiles on {:.3} · off {:.3}",
        min[0], min[1]
    );
}
