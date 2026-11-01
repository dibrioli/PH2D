//! F4-2 E6a: o depósito por somas. Doc: `docs/Painter/Arquitetura/E6a_deposito.md`.

use crate::grid::Grid;
use crate::grid::tinta::{M_UM, garante, soma_da_massa};
use crate::opacity::alpha_of_mass;
use crate::painter::Engine;
use crate::sim::LeiDaTinta;
use ph2d_paint_tinta::{Bandas, Soma, Tinta, ecra_sobre_branco, optica_celula};

const W: usize = 96;
const H: usize = 64;

fn traco(e: &mut Engine, x0: f64, x1: f64, y: f64) {
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 6.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

fn motor(lei: LeiDaTinta, cor: [f32; 3]) -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.lei = crate::assenta::Lei::Cobre;
    e.sliders.water = 1.0;
    e.sim.set_tinta(lei);
    e.color = cor.map(f64::from);
    e
}

/// A cobertura sobre branco da fatia (o α do ecrã «exacto sobre branco», F4_2 §3.2), na água (`m = 0`).
fn cobre(filme: &Soma, aberto: &Soma) -> f32 {
    let o = optica_celula(filme, aberto, 0.0, true);
    ecra_sobre_branco(&o, &Bandas([1.0; 3])).1
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / a.abs().max(b.abs()).max(1e-12)
}

/// Σ filme.v·M_UM × Σ susp e Σ aberto.v·M_UM × Σ sett.
fn somas(g: &Grid) -> (f64, f64, f64, f64, Soma) {
    let (mut f, mut s, mut a, mut t) = (0.0, 0.0, 0.0, 0.0);
    let mut toda = Soma::default();
    for i in 0..g.cells {
        let c = g.tinta[i];
        f += f64::from(c.filme.v) * f64::from(M_UM);
        a += f64::from(c.aberto.v) * f64::from(M_UM);
        s += f64::from(g.susp[i]);
        t += f64::from(g.sett[i]);
        toda = toda.junta(c.filme).junta(c.aberto);
    }
    (f, s, a, t, toda)
}

#[test]
fn calibracao_de_m_um() {
    let alvo = alpha_of_mass(600.0) as f32;
    let cores: [(&str, [f32; 3]); 4] = [
        ("azul", [20.0, 40.0, 200.0]),
        ("amarelo", [240.0, 210.0, 20.0]),
        ("vermelho", [200.0, 30.0, 30.0]),
        ("branco", [255.0, 255.0, 255.0]),
    ];
    println!("[cal] alpha_of_mass(600) = {alvo}");
    for t in [0.0f32, 1.0] {
        for (nome, rgb) in cores {
            // A massa (em unidades de M_UM = 1) que cobre `alvo`: bissecção em log.
            let c = |m: f32| {
                cobre(
                    &Soma::de(Tinta::de_srgb(rgb.map(|x| x / 255.0), t), m),
                    &Soma::default(),
                )
            };
            let (mut lo, mut hi) = (1e-6f32, 1e6f32);
            for _ in 0..200 {
                let mid = (lo * hi).sqrt();
                if c(mid) < alvo { lo = mid } else { hi = mid }
            }
            let v_alfa = hi;
            // A cobertura DERIVADA, eq. (3) (média das bandas): o critério do plano §2.3.
            let h = |m: f32| {
                let o = ph2d_paint_tinta::optica(
                    &Soma::de(Tinta::de_srgb(rgb.map(|x| x / 255.0), t), m),
                    0.0,
                    true,
                );
                ph2d_paint_tinta::cobertura(&o).0.iter().sum::<f32>() / 3.0
            };
            let (mut lo, mut hi) = (1e-6f32, 1e6f32);
            for _ in 0..200 {
                let mid = (lo * hi).sqrt();
                if h(mid) < alvo { lo = mid } else { hi = mid }
            }
            let com_m_um = cobre(&soma_da_massa(rgb, t, 0.0, 600.0), &Soma::default());
            println!(
                "[cal] t={t} {nome:9} α: v={v_alfa:.4} M_UM={:.1} · C(3): v={hi:.4} M_UM={:.1} · com M_UM={M_UM}: α(600)={com_m_um:.4}",
                600.0 / v_alfa,
                600.0 / hi
            );
            // O critério (a tabela no doc): a tinta OPACA que menos esconde (o branco, onde α e C coincidem) cobre
            // o que `alpha_of_mass(600)` cobria; nenhuma opaca cobre menos.
            if t == 0.0 {
                let o = ph2d_paint_tinta::optica(&soma_da_massa(rgb, t, 0.0, 600.0), 0.0, true);
                let c = ph2d_paint_tinta::cobertura(&o).0.iter().sum::<f32>() / 3.0;
                assert!(c >= alvo - 1e-3, "{nome}: C(600) = {c} < {alvo}");
                if nome == "branco" {
                    assert!((c - alvo).abs() < 1e-3, "branco: C(600) = {c} ≠ {alvo}");
                }
            }
        }
    }
}

#[test]
fn o_deposito_conserva_volume_e_somas() {
    let cor = [25.0f32, 76.0, 204.0];
    let mut e = motor(LeiDaTinta::Pigmento, cor);
    traco(&mut e, 10.0, 80.0, 30.0);
    let g = &e.layers[0].grid;
    let (f, s, a, t, toda) = somas(g);
    println!("[e6a] Σfilme·M={f} Σsusp={s} Σaberto·M={a} Σsett={t}");
    assert!(s > 0.0, "o traço pousou");
    assert!(rel(f, s) < 1e-5, "filme {f} × susp {s}");
    assert!(rel(a, t) < 1e-5 || (a + t) < 1e-3, "aberto {a} × sett {t}");
    let media = toda.cor().expect("tinta").lab;
    let pincel = Tinta::de_srgb(cor.map(|x| x / 255.0), 0.0).lab;
    let d = (0..3)
        .map(|k| (media[k] - pincel[k]).abs())
        .fold(0.0f32, f32::max);
    println!("[e6a] cor média {media:?} × pincel {pincel:?} (Δ {d})");
    assert!(d < 1e-4, "a cor média do filme é a do pincel");
}

#[test]
fn n_passadas_monotonas_no_motor() {
    let mut e = motor(LeiDaTinta::Pigmento, [204.0, 51.0, 25.0]);
    let mut antes = 0.0f32;
    for n in 1..=6 {
        traco(&mut e, 10.0, 80.0, 30.0);
        let g = &e.layers[0].grid;
        let row = 30 * g.s;
        let c: f32 = (30..60)
            .map(|x| cobre(&g.tinta[row + x].filme, &g.tinta[row + x].aberto))
            .sum::<f32>()
            / 30.0;
        println!("[e6a] passada {n}: cobertura média {c:.5}");
        assert!(c >= antes - 1e-6, "passada {n}: {c} < {antes}");
        antes = c;
    }
    assert!(antes > 0.0);
}

#[test]
fn trocar_de_lei_a_meio_nao_apaga_a_tela() {
    let mut e = motor(LeiDaTinta::Velha, [25.0, 153.0, 51.0]);
    traco(&mut e, 10.0, 80.0, 20.0);
    e.step_simulation();
    traco(&mut e, 10.0, 80.0, 40.0);
    let g = &mut e.layers[0].grid;
    assert!(g.tinta.is_empty());
    let (susp, sett) = (g.susp.clone(), g.sett.clone());
    let rgb = g.susp_rgb.clone();
    garante(g);
    let mut n = 0;
    for i in 0..g.cells {
        let c = g.tinta[i];
        assert!(
            rel(f64::from(c.filme.v * M_UM), f64::from(susp[i])) < 1e-6,
            "filme {i}"
        );
        assert!(
            rel(f64::from(c.aberto.v * M_UM), f64::from(sett[i])) < 1e-6,
            "aberto {i}"
        );
        if let Some(t) = c.filme.cor() {
            let esperado = Tinta::de_srgb(rgb[i].map(|x| x / 255.0), 0.0).lab;
            assert!(
                (0..3).all(|k| (t.lab[k] - esperado[k]).abs() < 1e-5),
                "cor {i}"
            );
            n += 1;
        }
    }
    assert!(n > 100, "a tela tinha tinta ({n})");
}
