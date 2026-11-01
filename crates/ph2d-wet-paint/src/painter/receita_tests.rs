//! E4/E10 — os portões da regra do dono, medidos no DEPÓSITO real (a porta da faixa, a que o pincel usa), e a
//! migração exacta da receita da E4 para os controlos da E10 (Dilution, Volume).

use super::*;
use crate::grid::tinta::M_UM;
use crate::painter::Engine;

/// Um dab pela porta do produto; devolve (Σ pigmento, Σ água) na janela do rasto.
fn um_dab(diluicao: f64, volume_um: f64, pressao: f64, b: f64) -> (f64, f64) {
    let mut e = Engine::new(64, 64);
    e.set_knob(Knob::Dilution, diluicao);
    e.set_knob(Knob::Volume, volume_um);
    e.sliders.pressure = pressao;
    e.color = [220.0, 45.0, 40.0];
    e.begin_direct_stroke(0, 32.0, 32.0);
    e.direct_segment(0, 4.0);
    e.dispatch_pressure_dab_lane(0, 32.0, 32.0, b, 10.0, None, None, None);
    let (pig, water) = e.lane_trails[0].planes_for_measure();
    let sp: f64 = pig.iter().map(|&v| f64::from(v)).sum();
    let sw: f64 = water.iter().map(|&v| f64::from(v)).sum();
    assert!(sp + sw > 0.0, "o dab não pousou nada");
    (sp, sw)
}

fn volume((p, w): (f64, f64)) -> f64 {
    p + K_W * w
}

fn phi((p, w): (f64, f64)) -> f64 {
    p / volume((p, w))
}

fn espalhamento(v: &[f64]) -> f64 {
    let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    hi / lo - 1.0
}

/// Tolerância do DEPÓSITO: a soma em `f32` de ~300 células.
const TOL: f64 = 1e-4;
/// Tolerância da RECEITA (f64, sem grade).
const TOL_RECEITA: f64 = 1e-7;

/// A receita da E4 (antes da E10), verbatim: a referência da migração.
fn receita_e4(pigmento: f64, agua: f64, agua_por_dab: f64, volume: f64, carga: f64) -> Receita {
    let p = pigmento.max(0.0);
    let v = K_W * veiculo(agua, agua_por_dab).max(0.0);
    let phi = if p + v > 0.0 { p / (p + v) } else { 0.0 };
    Receita {
        phi,
        m: 0.0,
        h: volume.max(0.0) * carga.max(0.0) * H_REF,
    }
}

fn rel(a: f64, b: f64) -> f64 {
    if a == b {
        0.0
    } else {
        (a - b).abs() / a.abs().max(b.abs())
    }
}

#[test]
fn migra_para_controlos_da_a_receita_da_e4() {
    let mut pior = (0.0f64, 0.0f64);
    let mut n = 0;
    for pig in [0.0, 1.0, 50.0, 78.0, 300.0, 600.0, 1500.0, 2000.0] {
        for wpd in [0.0, 0.25, 1.0, 4.0] {
            for agua in [0.0, 0.2, 0.6, 1.0] {
                for opacidade in [0.05, 0.4, 1.0] {
                    for carga in [0.0, 0.4505, 1.0, 4.0] {
                        // o PPD do preset já leva a Opacity dobrada (E4 `migra_da_lei_velha`)
                        let e4 = receita_e4(pig, agua, wpd, opacidade, carga);
                        let (d, v) = migra_para_controlos(pig, wpd, agua, carga, opacidade);
                        let r = receita(&Controlos {
                            diluicao: d,
                            volume_um: v,
                            meio: 0.0,
                        });
                        // 0 pigmento e 0 veículo: a E4 dava φ 0 — a Dilution 1 dá o mesmo
                        pior.0 = pior.0.max((r.phi - e4.phi).abs());
                        pior.1 = pior.1.max(rel(r.h, e4.h));
                        n += 1;
                    }
                }
            }
        }
    }
    eprintln!(
        "{n} receitas: |Δφ| máx {:.3e}, Δh relativo máx {:.3e}",
        pior.0, pior.1
    );
    assert!(pior.0 < 1e-12, "φ migrado difere da E4 em {:.3e}", pior.0);
    assert!(pior.1 < 1e-12, "h migrado difere da E4 em {:.3e}", pior.1);
}

#[test]
fn o_arranque_do_motor_e_a_receita_da_e4() {
    let e = Engine::new(8, 8);
    let r = e.receita_do_dab(&e.sim.gather_params(&e.tuning));
    let e4 = receita_e4(PIGMENTO_DO_MOTOR, 1.0, 1.0, 1.0, 1.0);
    eprintln!(
        "arranque: φ {:.17} (E4 {:.17}, Δ {:.3e}) · h {:.17} (E4 {:.17}, Δ {:.3e})",
        r.phi,
        e4.phi,
        r.phi - e4.phi,
        r.h,
        e4.h,
        r.h - e4.h
    );
    assert!(rel(r.phi, e4.phi) < 1e-12 && rel(r.h, e4.h) < 1e-12);
}

#[test]
fn volume_fixo_varrer_diluicao_nao_muda_o_volume_depositado() {
    let vol = H_REF / f64::from(M_UM);
    let dil = [0.05, 0.2, 0.37, 0.6, 0.9, 0.99];
    let q: Vec<f64> = dil
        .iter()
        .map(|&d| {
            receita(&Controlos {
                diluicao: d,
                volume_um: vol,
                meio: 0.0,
            })
            .h
        })
        .collect();
    let er = espalhamento(&q);
    let v: Vec<f64> = dil
        .iter()
        .map(|&d| volume(um_dab(d, vol, PRESSAO_DE_REFERENCIA, 5.0)))
        .collect();
    let e = espalhamento(&v);
    eprintln!("volume: receita {er:.3e}; depósito {e:.6} sobre {v:?}");
    assert!(
        er < TOL_RECEITA,
        "a Dilution mudou o h da receita em {er:.3e}"
    );
    assert!(e < TOL, "a Dilution mudou o volume depositado em {e:.4}");
}

#[test]
fn diluicao_fixa_varrer_volume_nao_muda_a_concentracao() {
    let vols = [0.1, 0.35, 0.69, 5.0, 100.0];
    let f: Vec<f64> = vols
        .iter()
        .map(|&v| {
            receita(&Controlos {
                diluicao: 0.4,
                volume_um: v,
                meio: 0.0,
            })
            .phi
        })
        .collect();
    let er = espalhamento(&f);
    let d: Vec<_> = vols[..3]
        .iter()
        .map(|&v| um_dab(0.4, v, PRESSAO_DE_REFERENCIA, 5.0))
        .collect();
    let fd: Vec<f64> = d.iter().map(|&x| phi(x)).collect();
    let q: Vec<f64> = d.iter().map(|&x| volume(x)).collect();
    let e = espalhamento(&fd);
    eprintln!("φ: receita {er:.3e}; depósito {e:.6} sobre {fd:?}; volume {q:?}");
    assert!(
        er < TOL_RECEITA,
        "o Volume mudou a φ da receita em {er:.3e}"
    );
    assert!(
        e < TOL,
        "o Volume mudou a concentração depositada em {e:.4}"
    );
    assert!(
        q[0] < q[1] && q[1] < q[2],
        "o Volume tem de mexer na quantidade: {q:?}"
    );
}

#[test]
fn a_pressao_so_toca_a_quantidade() {
    let vol = H_REF / f64::from(M_UM);
    let mut f = Vec::new();
    let mut q = Vec::new();
    for pressao in [0.1, 0.7, 1.0] {
        for b in [2.0, 5.0, 8.0] {
            let d = um_dab(0.4, vol, pressao, b);
            f.push(phi(d));
            q.push(volume(d));
        }
    }
    let e = espalhamento(&f);
    eprintln!("φ por pressão: espalhamento {e:.6} sobre {f:?}; volume {q:?}");
    assert!(e < TOL, "a pressão mudou a receita em {e:.4}");
    assert!(q[0] < q[1], "a pressão tem de mexer na quantidade: {q:?}");
}

/// O pigmento que o motor guarda: a grade activa (suspenso + assente) e as janelas dos rastos.
fn pigmento_no_motor(e: &Engine) -> f64 {
    let g = e.active_grid();
    let soma = |v: &[f32]| v.iter().map(|&x| f64::from(x)).sum::<f64>();
    let rastos: f64 = std::iter::once(&e.trail)
        .chain(e.lane_trails.iter())
        .map(|t| soma(t.planes_for_measure().0))
        .sum();
    soma(&g.susp) + soma(&g.sett) + rastos
}

/// ⭐ **Toda porta que POUSA tinta lê a receita** (E10-C): o produto já não escreve o `PigmentPerDab`, e o
/// depósito só cai no cru ([`crate::sim::Params::pigmento_por_deposito`]) com `pigmento_do_dab = None`. Cada porta
/// pinta com o PPD a 600 (o arranque) e a 5: o pigmento guardado tem de ser o MESMO, ao bit.
#[test]
fn toda_porta_que_pousa_tinta_le_a_receita() {
    type Porta = fn(&mut Engine);
    let portas: [(&str, Porta); 5] = [
        ("dispatch_pressure_dab", |e| {
            e.pointer_down(20.0, 32.0, None);
            for x in [24.0, 30.0, 36.0] {
                e.dispatch_pressure_dab(x, 32.0, 6.0, 10.0);
            }
        }),
        ("dispatch_pressure_dab_lane_cor", |e| {
            e.begin_direct_stroke(0, 32.0, 32.0);
            e.direct_segment(0, 4.0);
            e.dispatch_pressure_dab_lane(0, 32.0, 32.0, 5.0, 10.0, None, None, None);
        }),
        ("land_direct_remainders", |e| {
            e.begin_direct_stroke(0, 32.0, 32.0);
            e.direct_segment(0, 4.0);
            e.dispatch_pressure_dab_lane(0, 32.0, 32.0, 5.0, 10.0, None, None, None);
            e.land_direct_remainders(&[0]);
            e.end_direct_stroke();
        }),
        ("deposit_mask", |e| {
            let sil = |_: i32, _: i32| 1.0;
            e.deposit_mask(32.0, 32.0, 8.0, 5.0, 1.0, [220.0, 45.0, 40.0], &sil, None);
        }),
        ("pousa_grao_solto", |e| {
            let sil = |_: i32, _: i32| 1.0;
            let mut t = crate::trail::Trail::default();
            e.pousa_grao_solto(&mut t, [220.0, 45.0, 40.0], 32.0, 32.0, 5.0, 3.0, 1.0, &sil);
        }),
    ];
    for (nome, porta) in portas {
        let medida = |ppd: f64| {
            let mut e = Engine::new(64, 64);
            e.set_knob(Knob::Dilution, 0.4);
            e.set_knob(Knob::Volume, H_REF / f64::from(M_UM));
            e.set_knob(Knob::PigmentPerDab, ppd);
            e.color = [220.0, 45.0, 40.0];
            porta(&mut e);
            pigmento_no_motor(&e)
        };
        let (arranque, cru) = (medida(600.0), medida(5.0));
        eprintln!("{nome}: Σ pigmento PPD 600 = {arranque} · PPD 5 = {cru}");
        assert!(arranque > 0.0, "{nome}: a porta não pousou nada");
        assert_eq!(
            arranque.to_bits(),
            cru.to_bits(),
            "{nome}: o PigmentPerDab cru chegou ao depósito"
        );
    }
}
