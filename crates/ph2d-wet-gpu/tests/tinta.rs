//! E11 (F4_2 §6): a lei `Pigmento` na placa, meio e pasta incluídos. A régua é POR PLACA
//! (`project_same_bit_law_is_per_device_2026_10_24`): a placa dá o mesmo bit entre corridas e a resposta casa com a CPU
//! canónica dentro das tolerâncias medidas abaixo; bit cruzado entre fabricantes não se exige.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-wet-gpu --test tinta -- --ignored --nocapture
//! ```

use ph2d_gpu::GpuContext;
use ph2d_wet_gpu::passo::PassoPlaca;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::grid::tinta::pigmento::optica_celula;
use ph2d_wet_paint::grid::tinta::{CelulaTinta, M_UM, em, pilha};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::{LeiDaTinta, Sim, sim_step};
use ph2d_wet_paint::tuning::Knob;

/// A cor sobre branco, níveis 0–255. O p99.9 é o do degrau da lei velha (`tests/passo.rs`) e vale inteiro.
const COR_P999: f64 = 0.5;
/// O máximo e a fracção de células acima do [`COR_P999`] são da FRANJA: a máscara `active` pergunta `film > 0`, e um resíduo de
/// film que desaparece muda o escoamento de poucas células. A própria CPU move-se assim (só zerar abaixo de 1e-38: 13 níveis em 3
/// células da pasta; abaixo de 1e-20: 112 em 119). Tabela: `docs/Painter/49_realshader/PLACA_E11_tinta.md`.
const COR_MAX: f64 = 32.0;
const FRACAO_NA_FRANJA: f64 = 1e-3;
/// Quanto a soma do filme e do aberto (massa inteira da grade) pode afastar-se da CPU, relativo.
const MASSA_REL: f64 = 1e-3;

struct Cena {
    nome: &'static str,
    medium: f64,
    /// `None` = a Dilution de fábrica.
    dilution: Option<f64>,
    body: f64,
    /// A abertura de partida de toda a tinta (`1` = fresca): abaixo de 1 o re-molhar e o reabrir pelo volume mordem.
    abertura: f32,
    /// A fracção que desce mais que a água, com a gravidade da cena (`0` = sem `sedimenta`).
    sedimenta: f64,
}

const CENAS: [Cena; 6] = [
    Cena {
        nome: "água",
        medium: 0.0,
        dilution: None,
        body: 0.0,
        abertura: 1.0,
        sedimenta: 0.0,
    },
    Cena {
        nome: "óleo diluído",
        medium: 1.0,
        dilution: Some(0.8),
        body: 0.0,
        abertura: 1.0,
        sedimenta: 0.0,
    },
    Cena {
        // φ = 0,8: Bingham a morder (acima de φ_c) e um resto de líquido livre; abaixo de `Dilution` 0,1 não há líquido
        // nenhum e o passo não tem o que andar (a cena seria muda).
        nome: "pasta alta",
        medium: 1.0,
        dilution: Some(0.2),
        body: 0.5,
        abertura: 1.0,
        sedimenta: 0.0,
    },
    Cena {
        nome: "óleo meio curado",
        medium: 1.0,
        dilution: Some(0.8),
        body: 0.0,
        abertura: 0.5,
        sedimenta: 0.0,
    },
    Cena {
        // Quase fechada e com o re-molhar no máximo: o que levanta é a abertura que decide.
        nome: "óleo quase fechado",
        medium: 1.0,
        dilution: Some(0.8),
        body: 0.0,
        abertura: 0.02,
        sedimenta: 0.0,
    },
    Cena {
        nome: "água a sedimentar",
        medium: 0.0,
        dilution: None,
        body: 0.0,
        abertura: 1.0,
        sedimenta: 0.3,
    },
];

/// O que a CPU e a placa podem discordar por campo, no p99.9: ~20× o medido nas duas placas (05/11). A cor não vê uma lei
/// que ainda não mudou a tinta; o campo vê.
const FILM_P999: f64 = 1e-3;
const MASSA_P999: f64 = 0.5;
const SOMA_P999: f64 = 1e-3;
const ABERTURA_P999: f64 = 1e-4;
/// A placa pode discordar da CPU tanto quanto a CPU discorda de si mesma a 1 ulp (o controlo), com esta folga.
const FOLGA_DO_CONTROLO: f64 = 1.5;
const MEIO_P999: f64 = 1e-4;
const ESPELHO_P999: f64 = 1.0;

fn contexto() -> Option<GpuContext> {
    match GpuContext::new(GpuContext::default_instance(), None) {
        Ok(g) => {
            let i = g.adapter.get_info();
            eprintln!("placa: {} · {:?} · {}", i.name, i.backend, i.driver_info);
            Some(g)
        }
        Err(e) => {
            eprintln!("⛔ sem placa ({e}) — nada medido");
            None
        }
    }
}

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

fn pinta(c: &Cena, w: usize, h: usize) -> Engine {
    let mut e = Engine::new(w, h);
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.pickup_assentado = 0.0;
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.set_knob(Knob::Medium, c.medium);
    e.set_knob(Knob::Body, c.body);
    if let Some(d) = c.dilution {
        e.set_knob(Knob::Dilution, d);
    }
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let (wf, hf) = (w as f64, h as f64);
    // O primeiro traço entra pela borda da folha: o dreno do anel tem de levar a soma com a massa.
    traco(&mut e, 2.0, 0.9 * wf, 0.3 * hf);
    e.color = [0.8, 0.2, 0.1];
    traco(&mut e, 0.2 * wf, 0.8 * wf, 0.36 * hf);
    e.color = [0.1, 0.3, 0.9];
    traco(&mut e, 0.15 * wf, 0.7 * wf, 0.62 * hf);
    if c.sedimenta > 0.0 {
        e.sim.sedimenta = c.sedimenta;
        e.sim.gravity_override = Some([0.0, 0.08]);
    }
    if c.abertura < 0.1 {
        // Muito assente debaixo da água: sem isto o que levanta fica abaixo da régua e a abertura não decide nada à vista.
        e.set_knob(Knob::Rewet, 8.0);
        let idx = e.active_layer;
        let g = &mut e.layers[idx].grid;
        for i in 0..g.cells {
            if g.film[i] > 0.0 {
                g.sett[i] = 2000.0;
                g.tinta[i].aberto = ph2d_wet_paint::grid::tinta::soma_da_massa(
                    [200.0, 60.0, 30.0],
                    0.2,
                    1.0,
                    2000.0,
                );
            }
        }
    }
    if c.abertura < 1.0 {
        let idx = e.active_layer;
        for t in &mut e.layers[idx].grid.tinta {
            t.o = c.abertura;
        }
    }
    e
}

fn da_placa(gpu: &GpuContext, e: &Engine, passos: usize) -> (Sim, Grid) {
    let idx = e.active_layer;
    let (mut s, mut g) = (e.sim.clone(), e.layers[idx].grid.clone());
    let mut p = PassoPlaca::new(gpu, &g, &s, &e.tuning);
    for _ in 0..passos {
        p.passo(gpu, &mut s, &mut g, &e.tuning);
    }
    p.desce(gpu, &mut g);
    (s, g)
}

fn da_cpu(e: &Engine, passos: usize) -> (Sim, Grid) {
    let idx = e.active_layer;
    let (mut s, mut g) = (e.sim.clone(), e.layers[idx].grid.clone());
    for _ in 0..passos {
        sim_step(&mut s, &mut g, &e.tuning);
    }
    (s, g)
}

/// A cor sobre branco da lei nova, sRGB 0–255: filme e aberto pela óptica da célula sobre o fundo curado.
fn cor(g: &Grid, i: usize) -> [f64; 3] {
    let c = em(g, i);
    let molhado = g.film[i] > 0.0;
    let o = optica_celula(&c.filme, &c.aberto, None, c.meio(), molhado);
    pilha(&o, &c)
        .0
        .0
        .map(|r| f64::from(ph2d_paint_tinta::oklab::linear_para_srgb(r)) * 255.0)
}

fn d_cor(a: &Grid, b: &Grid, i: usize) -> f64 {
    let (x, y) = (cor(a, i), cor(b, i));
    (x[0] - y[0])
        .abs()
        .max((x[1] - y[1]).abs())
        .max((x[2] - y[2]).abs())
}

/// `(máx, p99.9, células acima do` [`COR_P999`]`)` da cor no interior.
fn desvio_cor(a: &Grid, b: &Grid) -> (f64, f64, usize) {
    let (mx, p) = desvio(a, |i| d_cor(a, b, i));
    let mut n = 0;
    for y in 1..=a.h {
        for x in 1..=a.w {
            n += usize::from(d_cor(a, b, x + y * a.s) > COR_P999);
        }
    }
    (mx, p, n)
}

/// `(máx, p99.9)` de `|a − b|` no interior da folha.
fn desvio(g: &Grid, f: impl Fn(usize) -> f64) -> (f64, f64) {
    let mut d = Vec::with_capacity(g.w * g.h);
    for y in 1..=g.h {
        for x in 1..=g.w {
            d.push(f(x + y * g.s).abs());
        }
    }
    d.sort_by(f64::total_cmp);
    let n = d.len();
    (d[n - 1], d[(n as f64 * 0.999) as usize])
}

fn campos(g: &Grid, i: usize) -> [f32; 16] {
    let c: CelulaTinta = em(g, i);
    let m = g.oleo.get(i).map_or(0.0, |o| o.meio);
    [
        g.film[i],
        g.susp[i],
        g.sett[i],
        c.filme.v,
        c.filme.lab[0],
        c.filme.lab[1],
        c.filme.lab[2],
        c.filme.t,
        c.filme.m,
        c.aberto.v,
        c.aberto.lab[0],
        c.aberto.lab[1],
        c.aberto.lab[2],
        c.aberto.t,
        c.aberto.m,
        m,
    ]
}

fn total(g: &Grid, f: impl Fn(&CelulaTinta) -> f32) -> f64 {
    (0..g.cells).map(|i| f64::from(f(&em(g, i)))).sum()
}

/// ⭐ O portão da E11: meio e pasta, placa × CPU por campo e na cor, e a mesma placa consigo mesma ao bit.
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn paridade_da_tinta_por_placa() {
    let Some(gpu) = contexto() else { return };
    let passos = 120usize;
    let mut falhas = Vec::new();
    for c in &CENAS {
        let e = pinta(c, 320, 200);
        let idx = e.active_layer;
        let g0 = &e.layers[idx].grid;
        assert!(g0.has_fluid, "{}: o traço deixou água", c.nome);
        assert_eq!(
            g0.oleo.iter().any(|o| o.meio > 0.0),
            c.medium > 0.0,
            "{}: o meio pousou",
            c.nome
        );
        assert_eq!(
            g0.oleo.iter().any(|o| o.corpo > 0.0),
            c.body > 0.0,
            "{}: o corpo pousou",
            c.nome
        );
        let (sc, gc) = da_cpu(&e, passos);
        let (sg, gg) = da_placa(&gpu, &e, passos);
        let (_, gg2) = da_placa(&gpu, &e, passos);
        assert_eq!(sg.frame, sc.frame, "{}: o relógio", c.nome);
        assert!(
            sc.frame - e.sim.frame >= passos as u64 / 2,
            "{}: a cena secou cedo e o passo não andou (muda)",
            c.nome
        );

        // A mesma placa consigo mesma: o bit, célula a célula, entre duas corridas.
        let dif_bit = (0..gg.cells)
            .filter(|&i| campos(&gg, i).map(f32::to_bits) != campos(&gg2, i).map(f32::to_bits))
            .count();

        let massa = |g: &Grid| -> f64 { g.susp.iter().chain(&g.sett).map(|&v| f64::from(v)).sum() };
        let (mc, mg) = (massa(&gc), massa(&gg));
        let rel = (mc - mg).abs() / mc.max(1e-9);
        let vm = |g: &Grid| (total(g, |t| t.filme.v) + total(g, |t| t.aberto.v)) * f64::from(M_UM);
        let (vc, vg) = (vm(&gc), vm(&gg));
        let rel_v = (vc - vg).abs() / vc.max(1e-9);
        let (cm, cp, acima) = desvio_cor(&gc, &gg);
        // O controlo da franja: a CPU contra ela própria com film, susp e sett de partida a ±1 ulp, célula sim, célula não, e a
        // evaporação a 1 ulp de f32 (a conta que a placa faz em f32 e a CPU em f64, à frente do limiar `film < 1e-4`). É o tamanho
        // do que a LEI faz a uma perturbação de arredondamento, sem placa no meio; a régua nunca lhe pede menos que isso.
        let g1 = {
            let (mut s1, mut g1) = (e.sim.clone(), e.layers[idx].grid.clone());
            for plano in [&mut g1.film, &mut g1.susp, &mut g1.sett] {
                for (i, f) in plano.iter_mut().enumerate().filter(|(_, f)| **f > 0.0) {
                    *f = f32::from_bits(if i % 2 == 0 {
                        f.to_bits() + 1
                    } else {
                        f.to_bits() - 1
                    });
                }
            }
            let mut t1 = e.tuning.clone();
            let ev = t1.get(Knob::Evaporation) as f32;
            t1.set(
                Knob::Evaporation,
                f64::from(f32::from_bits(ev.to_bits() + 1)),
            );
            for _ in 0..passos {
                sim_step(&mut s1, &mut g1, &t1);
            }
            g1
        };
        let (cu_m, cu_p, cu_n) = desvio_cor(&gc, &g1);
        eprintln!(
            "   controlo CPU × CPU a 1 ulp: cor máx {cu_m:.4} p99.9 {cu_p:.4} · > {COR_P999}: {cu_n}"
        );
        let mut pior: Vec<(f64, usize)> = (0..gc.cells).map(|i| (d_cor(&gc, &gg, i), i)).collect();
        pior.sort_by(|a, b| b.0.total_cmp(&a.0));
        let limiar = pior
            .iter()
            .filter(|p| p.0 > COR_P999 && (gc.film[p.1] > 0.0) != (gg.film[p.1] > 0.0))
            .count();
        eprintln!("   cor > {COR_P999}: {acima} células, {limiar} com o film a zero só de um lado");
        for &(d, i) in pior.iter().take(3) {
            eprintln!(
                "   Δ{d:.3} ({},{}) film {:.5}/{:.5} susp {:.3}/{:.3} sett {:.3}/{:.3} act {}/{}",
                i % gc.s,
                i / gc.s,
                gc.film[i],
                gg.film[i],
                gc.susp[i],
                gg.susp[i],
                gc.sett[i],
                gg.sett[i],
                gc.active[i],
                gg.active[i],
            );
        }
        let linha = |nome: &str, k: usize| {
            // O meio de uma célula sem tinta não se vê nem conta: a água que lá chega é a que manda.
            let (mx, p) = desvio(&gc, |i| {
                if k == 15 && (gc.susp[i] + gc.sett[i] <= 0.0 || gg.susp[i] + gg.sett[i] <= 0.0) {
                    return 0.0;
                }
                f64::from(campos(&gc, i)[k]) - f64::from(campos(&gg, i)[k])
            });
            eprintln!("   {nome:<10} máx {mx:.5e} · p99.9 {p:.5e}");
        };
        eprintln!(
            "{}: {passos} passos · bit entre corridas ≠ {dif_bit} · cor máx {cm:.4} p99.9 {cp:.4} · \
             massa CPU {mc:.2} placa {mg:.2} (rel {rel:.2e}) · V·M_UM rel {rel_v:.2e}",
            c.nome
        );
        for (nome, k) in [
            ("film", 0),
            ("susp", 1),
            ("sett", 2),
            ("filme.v", 3),
            ("filme.L", 4),
            ("filme.m", 8),
            ("aberto.v", 9),
            ("aberto.L", 10),
            ("aberto.m", 14),
            ("meio", 15),
        ] {
            linha(nome, k);
        }
        // A devolução: a CPU continua de onde a placa parou (a soma tem de ter descido com a massa).
        let (mut sa, mut ga, mut sb, mut gb) = (sc.clone(), gc.clone(), sg.clone(), gg.clone());
        for _ in 0..12 {
            sim_step(&mut sa, &mut ga, &e.tuning);
            sim_step(&mut sb, &mut gb, &e.tuning);
        }
        let (dm, dp, dn) = desvio_cor(&ga, &gb);
        eprintln!(
            "   devolvido +12 passos na CPU: cor máx {dm:.4} p99.9 {dp:.4} · > {COR_P999}: {dn}"
        );

        // Os campos, um a um; o meio só onde há tinta.
        let tem = |i: usize| gc.susp[i] + gc.sett[i] > 0.0 && gg.susp[i] + gg.sett[i] > 0.0;
        let p999 = |k: usize, g: &Grid| {
            desvio(&gc, |i| {
                if k == 15 && !tem(i) {
                    return 0.0;
                }
                f64::from(campos(&gc, i)[k]) - f64::from(campos(g, i)[k])
            })
            .1
        };
        for (nome, k, teto) in [
            ("film", 0, FILM_P999),
            ("susp", 1, MASSA_P999),
            ("sett", 2, MASSA_P999),
            ("filme.v", 3, SOMA_P999),
            ("filme.L", 4, SOMA_P999),
            ("filme.a", 5, SOMA_P999),
            ("filme.b", 6, SOMA_P999),
            ("filme.t", 7, SOMA_P999),
            ("filme.m", 8, SOMA_P999),
            ("aberto.v", 9, SOMA_P999),
            ("aberto.L", 10, SOMA_P999),
            ("aberto.a", 11, SOMA_P999),
            ("aberto.b", 12, SOMA_P999),
            ("aberto.t", 13, SOMA_P999),
            ("aberto.m", 14, SOMA_P999),
            ("meio", 15, MEIO_P999),
        ] {
            let d = p999(k, &gg);
            if d > teto.max(FOLGA_DO_CONTROLO * p999(k, &g1)) {
                falhas.push(format!("{}: {nome} p99.9 {d:.3e} > {teto:.1e}", c.nome));
            }
        }
        let d_o = desvio(&gc, |i| f64::from(em(&gc, i).o) - f64::from(em(&gg, i).o)).1;
        eprintln!("   abertura   p99.9 {d_o:.5e}");
        if d_o > ABERTURA_P999 {
            falhas.push(format!("{}: abertura p99.9 {d_o:.3e}", c.nome));
        }
        // O espelho sRGB, onde a lei nova o escreve (a fatia tem tinta dos dois lados).
        let d_e = desvio(&gc, |i| {
            if em(&gc, i).aberto.v <= 0.0 || em(&gg, i).aberto.v <= 0.0 {
                return 0.0;
            }
            (0..3)
                .map(|k| f64::from((gc.sett_rgb[i][k] - gg.sett_rgb[i][k]).abs()))
                .fold(0.0, f64::max)
        })
        .1;
        eprintln!("   espelho    p99.9 {d_e:.5e}");
        if d_e > ESPELHO_P999 {
            falhas.push(format!("{}: espelho p99.9 {d_e:.3e}", c.nome));
        }
        if dif_bit != 0 {
            falhas.push(format!(
                "{}: {dif_bit} células mudam entre corridas",
                c.nome
            ));
        }
        let teto = (gc.w * gc.h) as f64 * FRACAO_NA_FRANJA;
        let f = FOLGA_DO_CONTROLO;
        if cm > COR_MAX.max(f * cu_m)
            || cp > COR_P999.max(f * cu_p)
            || acima as f64 > teto.max(f * cu_n as f64)
        {
            falhas.push(format!(
                "{}: cor máx {cm} p99.9 {cp}, {acima} células",
                c.nome
            ));
        }
        if rel > MASSA_REL || rel_v > MASSA_REL {
            falhas.push(format!(
                "{}: massa rel {rel:.2e}, V rel {rel_v:.2e}",
                c.nome
            ));
        }
        if dm > COR_MAX.max(f * cu_m)
            || dp > COR_P999.max(f * cu_p)
            || dn as f64 > teto.max(f * cu_n as f64)
        {
            falhas.push(format!(
                "{}: devolvido, cor máx {dm} p99.9 {dp}, {dn} células",
                c.nome
            ));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// O Smooth Edges é da célula e a composição é da CPU: a placa não o sobe nem o desce, e o valor não se perde na ida e volta
/// nem quando a CPU pousa um traço novo com a placa a meio (o espelho por hash).
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn o_suave_da_celula_sobrevive_a_placa() {
    let Some(gpu) = contexto() else { return };
    let c = &CENAS[0];
    let pintada = || {
        let mut e = Engine::new(320, 200);
        e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
        e.sim.pickup_assentado = 0.0;
        e.sim.set_tinta(LeiDaTinta::Pigmento);
        e.set_knob(Knob::Medium, c.medium);
        e.sliders.water = 1.0;
        e.sliders.size = 1.0;
        e.sim.suave = 0.6;
        traco(&mut e, 32.0, 288.0, 60.0);
        e
    };
    let mut ec = pintada();
    let mut eg = pintada();
    let idx = eg.active_layer;
    let antes = eg.layers[idx].grid.aparencia.clone();
    assert!(
        antes.iter().any(|a| a.suave > 0.5),
        "a cena pousou o suave (senão a prova é muda)"
    );
    let teto = eg.layers[idx].grid.suave_teto;
    assert_eq!(teto, 0.6, "o tecto do suave");

    let mut p = {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = &eg;
        PassoPlaca::new(&gpu, &layers[idx].grid, sim, tuning)
    };
    let anda = |p: &mut PassoPlaca, e: &mut Engine, n: usize| {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = e;
        for _ in 0..n {
            p.passo(&gpu, sim, &mut layers[idx].grid, tuning);
        }
    };
    let anda_cpu = |e: &mut Engine, n: usize| {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = e;
        for _ in 0..n {
            sim_step(sim, &mut layers[idx].grid, tuning);
        }
    };
    anda(&mut p, &mut eg, 30);
    anda_cpu(&mut ec, 30);
    p.desce_suja(&gpu, &mut eg.layers[idx].grid);
    assert_eq!(
        eg.layers[idx].grid.aparencia, antes,
        "a descida suja mexeu na aparência"
    );
    assert_eq!(eg.layers[idx].grid.suave_teto, teto);

    // Um traço novo, com outro Smooth Edges, pousado na CPU com a placa a meio: o espelho sobe-o e a placa continua.
    for e in [&mut ec, &mut eg] {
        e.sim.suave = 0.2;
        e.color = [0.1, 0.3, 0.9];
        traco(e, 32.0, 288.0, 130.0);
    }
    let gg = &eg.layers[idx].grid;
    assert!(p.sobe_sujo(&gpu, gg).is_some(), "o traço novo sobe");
    anda(&mut p, &mut eg, 30);
    anda_cpu(&mut ec, 30);
    p.desce(&gpu, &mut eg.layers[idx].grid);

    let (gc, gg) = (&ec.layers[idx].grid, &eg.layers[idx].grid);
    assert_eq!(gg.suave_teto, gc.suave_teto, "o tecto do suave");
    let (mx, _) = desvio(gg, |i| {
        f64::from(gg.aparencia[i].suave) - f64::from(gc.aparencia[i].suave)
    });
    let n02 = gg
        .aparencia
        .iter()
        .filter(|a| (a.suave - 0.2).abs() < 0.05)
        .count();
    let n06 = gg
        .aparencia
        .iter()
        .filter(|a| (a.suave - 0.6).abs() < 0.05)
        .count();
    let (cm, _) = desvio(gg, |i| {
        let (a, b) = (cor(gc, i), cor(gg, i));
        (a[0] - b[0])
            .abs()
            .max((a[1] - b[1]).abs())
            .max((a[2] - b[2]).abs())
    });
    eprintln!(
        "suave: Δ máx placa × CPU {mx:.2e} · células 0,6 {n06} · 0,2 {n02} · cor máx {cm:.4}"
    );
    assert!(n06 > 0 && n02 > 0, "os dois valores continuam na tela");
    assert!(mx < 0.02, "o suave da célula afastou-se da CPU: {mx}");
    assert!(cm <= COR_MAX, "a cor depois do traço novo: {cm}");
}

/// A cura anda na placa e o fecho é o da CPU: as mesmas células fecham, com o mesmo fundo, e o `sett` sai da grade.
#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn a_cura_fecha_na_placa_como_na_cpu() {
    let Some(gpu) = contexto() else { return };
    let mut e = pinta(&CENAS[1], 320, 200);
    let idx = e.active_layer;
    {
        let g = &mut e.layers[idx].grid;
        for y in 170..190 {
            for x in 200..240 {
                let i = x + y * g.s;
                g.film[i] = 0.0;
                g.susp[i] = 0.0;
                g.sett[i] = 500.0;
                g.tinta[i].filme = Default::default();
                g.tinta[i].aberto = ph2d_wet_paint::grid::tinta::soma_da_massa(
                    [200.0, 60.0, 30.0],
                    0.2,
                    1.0,
                    500.0,
                );
                g.tinta[i].o = 0.010_002;
            }
        }
    }
    let (_, gc) = da_cpu(&e, 80);
    let (_, gg) = da_placa(&gpu, &e, 80);
    let fechadas = |g: &Grid| g.tinta.iter().filter(|c| c.altura_fechada > 0.0).count();
    let (nc, ng) = (fechadas(&gc), fechadas(&gg));
    let (dr, _) = desvio(&gc, |i| {
        f64::from(gc.tinta[i].r_g[0]) - f64::from(gg.tinta[i].r_g[0])
    });
    let (dh, _) = desvio(&gc, |i| {
        f64::from(gc.tinta[i].altura_fechada) - f64::from(gg.tinta[i].altura_fechada)
    });
    eprintln!("cura: fechadas CPU {nc} placa {ng} · r_g Δ máx {dr:.2e} · altura Δ máx {dh:.2e}");
    assert!(
        nc >= 800,
        "a cena tem de fechar (senão a prova é muda): {nc}"
    );
    assert_eq!(nc, ng, "as mesmas células fecham");
    assert!(
        dr < 1e-4 && dh < 1e-4,
        "o fundo fechado: r_g {dr:.2e}, altura {dh:.2e}"
    );
}

/// O custo do passo com a lei nova contra a velha, na mesma placa e no mesmo processo, em blocos intercalados; conta o MÍNIMO.
#[test]
#[ignore = "placa — medida: PH2D_GPU=1, corra com --ignored --nocapture"]
fn mede_o_passo_da_tinta_a_4k() {
    let Some(gpu) = contexto() else { return };
    let (w, h) = (4096, 2560);
    let mut velha = {
        let mut e = pinta(&CENAS[0], w, h);
        e.sim.set_tinta(LeiDaTinta::Velha);
        let idx = e.active_layer;
        e.layers[idx].grid.tinta = Vec::new();
        e
    };
    let mut nova = pinta(&CENAS[0], w, h);
    let idx = nova.active_layer;
    let mut pv = PassoPlaca::new(&gpu, &velha.layers[idx].grid, &velha.sim, &velha.tuning);
    let mut pn = PassoPlaca::new(&gpu, &nova.layers[idx].grid, &nova.sim, &nova.tuning);
    let bloco = |p: &mut PassoPlaca, e: &mut Engine| {
        let Engine {
            sim,
            layers,
            tuning,
            ..
        } = e;
        let t = std::time::Instant::now();
        for _ in 0..6 {
            p.passo(&gpu, sim, &mut layers[idx].grid, tuning);
        }
        t.elapsed().as_secs_f64() * 1000.0 / 6.0
    };
    let (mut mv, mut mn) = (f64::MAX, f64::MAX);
    for r in 0..8 {
        if r % 2 == 0 {
            mv = mv.min(bloco(&mut pv, &mut velha));
            mn = mn.min(bloco(&mut pn, &mut nova));
        } else {
            mn = mn.min(bloco(&mut pn, &mut nova));
            mv = mv.min(bloco(&mut pv, &mut velha));
        }
    }
    eprintln!(
        "4096×2560: lei velha {mv:.2} ms/passo · Pigmento {mn:.2} ms/passo (mínimo de 8 blocos de 6)"
    );
}
