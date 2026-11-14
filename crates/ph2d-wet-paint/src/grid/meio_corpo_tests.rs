//! E10 A2 — o MEIO e o CORPO viajam na célula do óleo (doc `docs/Painter/Arquitetura/E10a2_meio_e_corpo.md`).

use super::{OleoCelula, da_celula};
use crate::painter::Engine;
use crate::sim::LeiDaTinta;
use crate::solver::ladeira_do_oleo::{tensao, tensao_com_corpo};
use crate::tuning::Knob;

const L: usize = 96;

fn traco(e: &mut Engine, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
    e.pointer_down(x0, y0, None);
    for i in 1..=12 {
        let t = f64::from(i) / 12.0;
        e.pointer_frame(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
    }
    e.pointer_up();
    while e.release_frame(x1, y1) {}
}

fn passos(e: &mut Engine, n: usize) {
    for _ in 0..n {
        e.step_simulation();
    }
}

/// ⭐ Medium 0 / 0,5 / 1 → o `meio` da célula e, na lei Pigmento, `Σv·m ÷ Σv` da tinta = o Medium que o motor GUARDOU
/// ± 1e-6 — o da lei de escrita do knob (B35.6: o degrau Water · Oil, `0,5` = Water), não o número pedido.
#[test]
fn o_meio_viaja_do_controlo_ao_deposito() {
    for pedido in [0.0f32, 0.5, 1.0] {
        let mut e = Engine::new(L, L);
        e.sim.set_tinta(LeiDaTinta::Pigmento);
        e.set_knob(Knob::Medium, f64::from(pedido));
        let medium = e.tuning.get(Knob::Medium) as f32;
        assert_eq!(
            medium,
            if pedido > 0.5 { 1.0 } else { 0.0 },
            "Medium {pedido}: o degrau Water · Oil"
        );
        e.sliders.water = 0.3;
        e.sliders.pigmento = 1.0;
        e.sliders.pressure = 1.0;
        e.sliders.size = 0.3;
        traco(&mut e, (20.0, 48.0), (40.0, 48.0));
        let g = e.active_grid();
        let pintadas: Vec<usize> = (0..g.cells)
            .filter(|&i| g.susp[i] + g.sett[i] > 0.0)
            .collect();
        assert!(!pintadas.is_empty(), "Medium {medium}: o dab não pousou");
        for &i in &pintadas {
            let m = da_celula(&g.oleo, i).meio;
            assert!(
                (m - medium).abs() <= 1e-6,
                "Medium {medium}: célula {i} com meio {m}"
            );
        }
        let (mut sv, mut svm) = (0.0f64, 0.0f64);
        for c in &g.tinta {
            let v = f64::from(c.filme.v) + f64::from(c.aberto.v);
            sv += v;
            svm += v * f64::from(c.meio());
        }
        assert!(sv > 0.0, "a lei Pigmento não pousou volume");
        let meio = svm / sv;
        assert!(
            (meio - f64::from(medium)).abs() <= 1e-6,
            "Medium {medium}: Σv·m÷Σv = {meio}"
        );
        if medium == 0.0 {
            assert!(g.oleo.is_empty(), "Medium 0 sem Real Oil alocou o campo");
        }
    }
}

/// O que o 1.º traço (de água) perde para FORA das duas faixas quando o 2.º o cruza molhado, ÷ a massa dele; e o p99
/// da massa na borda do 1.º (o degrau que a parede binária empilhava).
fn fuga_e_borda(body: f64) -> (f64, f64) {
    let corre = |dois: bool| {
        let mut e = Engine::new(L, L);
        e.set_knob(Knob::Body, body);
        e.sliders.water = 0.5;
        e.sliders.size = 0.3;
        traco(&mut e, (10.0, 48.0), (86.0, 48.0));
        passos(&mut e, 10);
        if dois {
            traco(&mut e, (48.0, 6.0), (48.0, 90.0));
        }
        passos(&mut e, 300);
        let g = e.active_grid();
        (
            g.s,
            (0..g.cells)
                .map(|i| f64::from(g.susp[i]) + f64::from(g.sett[i]))
                .collect::<Vec<_>>(),
        )
    };
    let (s, so1) = corre(false);
    let (_, ab) = corre(true);
    let r = 2.0 + 0.3 * 33.0;
    let m1: f64 = so1.iter().sum();
    let (mut fora, mut borda) = (0.0, vec![]);
    for (i, &m) in ab.iter().enumerate() {
        let (dx, dy) = (((i % s) as f64 - 48.0).abs(), ((i / s) as f64 - 48.0).abs());
        if dx > r + 2.0 && dy > r + 2.0 {
            fora += m;
        }
        if dx > r + 2.0 && dy > r - 3.0 && dy <= r {
            borda.push(m);
        }
    }
    borda.sort_by(f64::total_cmp);
    let p99 = borda.get(borda.len() * 99 / 100).copied().unwrap_or(0.0);
    (fora / m1.max(1e-9), p99)
}

/// ⭐ O corpo segura a tinta de água ao lado de um traço molhado SEM o degrau da parede: a fuga com o B₁ é ≲ 1 % e a
/// borda não sobe acima da de Body 0 (a parede binária empilhava 12–39 µm ali, Bug #56).
#[test]
fn o_corpo_viaja_na_celula_e_segura_a_tinta() {
    let (f0, b0) = fuga_e_borda(0.0);
    let (f1, b1) = fuga_e_borda(B1);
    assert!(
        f1 <= 0.01 || f1 <= 0.1 * f0,
        "Body {B1}: fuga {f1:.4} (Body 0: {f0:.4})"
    );
    assert!(
        b1 <= b0 * 1.5 + 1.0,
        "Body {B1}: a borda empilha — p99 {b1:.1} × {b0:.1}"
    );
}

/// O B₁ medido (`E10a2_meio_e_corpo.md`): o Body que substitui o `levelingOwnStroke = 1`.
const B1: f64 = 0.16;

/// ⭐ Sem meio e sem corpo é o de sempre: a célula do pincel é a NEUTRA (o campo não aloca) e a tensão com corpo 0 é
/// a do meio AO BIT em toda a grelha de entradas. (O motor inteiro: os `advect_jacobi_*` de hash fixo.)
#[test]
fn sem_meio_sem_corpo_e_o_de_sempre() {
    let mut e = Engine::new(L, L);
    traco(&mut e, (20.0, 48.0), (60.0, 48.0));
    assert!(
        e.active_grid().oleo.is_empty(),
        "Medium 0 · Body 0 · Real Oil off alocou o campo"
    );
    assert_eq!(da_celula(&super::sem_campo(), 3), OleoCelula::NEUTRA);
    for t0 in [0.0, 3.0] {
        for m in [0.0, 0.5, 1.0] {
            for massa in [0.0, 1.0, 600.0] {
                for f in [0.0, 0.3, 5.0] {
                    let a = tensao(t0, m, massa, f);
                    let b = tensao_com_corpo(t0, m, 0.0, (massa, massa), f);
                    assert_eq!(a.to_bits(), b.to_bits());
                }
            }
        }
    }
}
