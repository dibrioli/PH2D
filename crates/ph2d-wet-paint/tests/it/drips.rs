//! **As gotas** (W1b, [`ph2d_wet_paint::drips`]) — as leis do motor, e a medição do custo a 4096².
//!
//! A forma das gotas contra o Rebelle é gateada no crate do tool (`gotas_pela_lei_do_rebelle`, a bancada
//! do doc 06); aqui fica o que só o motor responde: inerte sem gravidade, a linha de contacto conserva a
//! massa, determinismo ao bit, e o custo.

use ph2d_wet_paint::drips::DripPaper;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::tuning::Knob;
use std::time::Instant;

/// O dial do painter no máximo (anel 8 = escala 2 sobre a gravidade da tela, `0,01`), para baixo.
fn inclina(e: &mut Engine, ligada: bool) {
    e.set_knob(Knob::Gravity, 0.01);
    e.sim.tilt_on = ligada;
    e.sim.tilt_dir_x = 0.0;
    e.sim.tilt_dir_y = 1.0;
    e.sim.tilt_scale = 2.0;
}

/// Uma cena pequena: uma POÇA posta à mão (uma faixa de 200 × 30 células com água funda e pigmento) —
/// as leis do motor não dependem de como a água chegou ali, e a poça passa o que o papel segura.
fn cena(gotas: Option<DripPaper>, inclinada: bool) -> Engine {
    let mut e = Engine::new(320, 420);
    e.sim.gotas = gotas;
    inclina(&mut e, inclinada);
    let idx = e.active_layer;
    let g = &mut e.layers[idx].grid;
    for y in 60..90 {
        for x in 60..260 {
            let i = x + y * g.s;
            g.film[i] = 3.0;
            g.susp[i] = 2000.0;
            g.susp_rgb[i] = [23.0, 101.0, 179.0];
        }
    }
    g.expand_bbox(60, 60, 259, 89);
    g.has_fluid = true;
    e
}

fn passos(e: &mut Engine, n: usize) {
    for _ in 0..n {
        e.step_simulation();
    }
}

fn bits(e: &Engine) -> Vec<u32> {
    let g = &e.layers[e.active_layer].grid;
    g.film
        .iter()
        .chain(&g.susp)
        .chain(&g.sett)
        .map(|v| v.to_bits())
        .collect()
}

/// A linha mais baixa com pigmento assente visível.
fn fundo(e: &Engine) -> usize {
    let g = &e.layers[e.active_layer].grid;
    (1..=g.h)
        .rev()
        .find(|&y| (1..=g.w).any(|x| g.sett[x + y * g.s] > 50.0))
        .unwrap_or(0)
}

/// Tela direita (sem gravidade): a lei das gotas ligada ou desligada dá o MESMO grid, ao bit — a linha de
/// contacto e as gotas só existem com a tela inclinada. Mutação que sangra: o `pin_downhill` ou o
/// `drip_pass` sem a guarda da gravidade nula.
#[test]
fn sem_gravidade_a_lei_das_gotas_e_inerte_ao_bit() {
    let mut com = cena(Some(DripPaper::DEFAULT), false);
    let mut sem = cena(None, false);
    passos(&mut com, 400);
    passos(&mut sem, 400);
    assert!(
        bits(&com) == bits(&sem),
        "a lei das gotas mexeu na tela direita"
    );
}

/// Com a tela inclinada, a linha de contacto muda o traço (o leitor decide), segura a tinta na orla sem
/// Create Drips, e com ele a gota desce.
#[test]
fn com_a_tela_inclinada_a_linha_de_contacto_segura_e_a_gota_desce() {
    let mut sem_lei = cena(None, true);
    let mut sem_fios = cena(
        Some(DripPaper {
            create: false,
            ..DripPaper::DEFAULT
        }),
        true,
    );
    let mut com_fios = cena(Some(DripPaper::DEFAULT), true);
    for e in [&mut sem_lei, &mut sem_fios, &mut com_fios] {
        passos(e, 1200);
    }
    assert!(
        bits(&sem_lei) != bits(&sem_fios),
        "a linha de contacto não mudou nada"
    );
    assert!(
        fundo(&sem_fios) < 110,
        "sem Create Drips a tinta desceu a {}",
        fundo(&sem_fios)
    );
    assert!(
        fundo(&com_fios) > 250,
        "com Create Drips a gota parou em {} — gotas {:?}",
        fundo(&com_fios),
        com_fios.layers[0].grid.drips
    );
}

/// A linha de contacto não cria nem destrói água nem pigmento: o `uv` que ela corta é o MESMO que a saída
/// e o transporte leem, logo o gather continua a conservar.
#[test]
fn a_linha_de_contacto_conserva_a_massa() {
    let mut e = cena(Some(DripPaper::DEFAULT), true);
    passos(&mut e, 80);
    let p = e.sim.gather_params(&e.tuning);
    let idx = e.active_layer;
    let g = &mut e.layers[idx].grid;
    let soma = |g: &ph2d_wet_paint::grid::Grid| -> (f64, f64) {
        (
            g.film.iter().map(|&v| f64::from(v)).sum(),
            g.susp.iter().map(|&v| f64::from(v)).sum(),
        )
    };
    let antes = soma(g);
    ph2d_wet_paint::solver::advect_jacobi_pinned(g, &p, 0.0, 0.02);
    let depois = soma(g);
    assert!(
        (antes.0 - depois.0).abs() <= 1e-4 * antes.0.max(1.0),
        "água {antes:?} → {depois:?}"
    );
    assert!(
        (antes.1 - depois.1).abs() <= 1e-4 * antes.1.max(1.0),
        "pigmento {antes:?} → {depois:?}"
    );
}

/// Determinismo (ADR-0134/0147): duas corridas iguais com gotas dão o mesmo grid ao bit.
#[test]
fn as_gotas_sao_deterministas_ao_bit() {
    let mut a = cena(Some(DripPaper::DEFAULT), true);
    let mut b = cena(Some(DripPaper::DEFAULT), true);
    passos(&mut a, 600);
    passos(&mut b, 600);
    assert!(bits(&a) == bits(&b));
    assert_eq!(
        a.layers[a.active_layer].grid.drips,
        b.layers[b.active_layer].grid.drips
    );
    assert!(fundo(&a) > 150, "o controlo: houve gota ({})", fundo(&a));
}

/// **O custo das gotas a 4096²** (ADR-0134: pior tick ≤ 2 ms). Duas variantes no MESMO processo,
/// intercaladas passo a passo (`docs/DevOps/MEDIR_VELOCIDADE.md`): a lei das gotas ligada (linha de
/// contacto + gotas, Size/Length 10/10 — o pior caso) e desligada, sobre a mesma cena inclinada ao máximo.
/// Duas cenas: três POÇAS largas (1500 × 200 células, as gotas nascem às dezenas) e a tela INTEIRA molhada
/// (o pior caso da varredura das orlas). Por passo: o estágio final (onde as gotas moram, ver
/// `sim_step_stage`), o advect (onde mora a linha de contacto) e o passo inteiro. Corre com
/// `--profile smoke --test it measure_drip_pass_cost -- --ignored --nocapture`.
#[test]
#[ignore = "medição: o custo das gotas a 4096²"]
fn measure_drip_pass_cost() {
    const SIDE: usize = 4096;
    const PASSOS: usize = 200;
    let pocas = |e: &mut Engine| {
        let idx = e.active_layer;
        let g = &mut e.layers[idx].grid;
        for k in 0..3 {
            let y0 = 800 + 1000 * k;
            for y in y0..y0 + 200 {
                for x in 1200..2700 {
                    let i = x + y * g.s;
                    g.film[i] = 3.0;
                    g.susp[i] = 2000.0;
                    g.susp_rgb[i] = [23.0, 101.0, 179.0];
                }
            }
        }
        g.expand_bbox(1200, 800, 2699, 2999);
        g.has_fluid = true;
    };
    // A tela INTEIRA com água rasa (abaixo do que o papel segura: nenhuma gota) — o pior caso da varredura
    // das orlas e da linha de contacto, que pagam pela área molhada.
    let tela_molhada = |e: &mut Engine| {
        let idx = e.active_layer;
        let g = &mut e.layers[idx].grid;
        for y in 1..=SIDE {
            for x in 1..=SIDE {
                let i = x + y * g.s;
                g.film[i] = 0.5;
                g.susp[i] = 300.0;
                g.susp_rgb[i] = [23.0, 101.0, 179.0];
            }
        }
        g.expand_bbox(1, 1, SIDE as i32, SIDE as i32);
        g.has_fluid = true;
    };
    type Cena<'a> = (&'a str, &'a dyn Fn(&mut Engine));
    let cenas: [Cena<'_>; 2] = [("três poças", &pocas), ("tela molhada", &tela_molhada)];
    println!(
        "[gotas] load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (nome, prepara) in cenas {
        let monta = |gotas: Option<DripPaper>| {
            let mut e = Engine::new(SIDE, SIDE);
            e.sim.gotas = gotas;
            inclina(&mut e, true);
            prepara(&mut e);
            e
        };
        let mut com = monta(Some(DripPaper {
            create: true,
            size: 10,
            length: 10,
            easy: false,
        }));
        let mut sem = monta(None);
        // [lado] → tempos por passo: (estágio final, advect, passo inteiro)
        let mut t: [Vec<[f64; 3]>; 2] = [Vec::new(), Vec::new()];
        let mut gotas_max = 0usize;
        for _ in 0..PASSOS {
            for (lado, e) in [(0usize, &mut com), (1, &mut sem)] {
                let t0 = Instant::now();
                let (mut estagio, mut adv) = (0, 0.0);
                let fim = loop {
                    let t1 = Instant::now();
                    let r = e.step_stage();
                    let ms = t1.elapsed().as_secs_f64() * 1e3;
                    if estagio == 3 {
                        adv = ms;
                    }
                    estagio += 1;
                    if r.is_some() {
                        break ms;
                    }
                };
                t[lado].push([fim, adv, t0.elapsed().as_secs_f64() * 1e3]);
            }
            gotas_max = gotas_max.max(com.layers[com.active_layer].grid.drips.len());
        }
        for (lado, quem) in [(0, "com a lei"), (1, "sem a lei")] {
            let col = |k: usize| {
                let mut v: Vec<f64> = t[lado].iter().map(|r| r[k]).collect();
                v.sort_by(f64::total_cmp);
                (v[v.len() / 2], v[v.len() * 99 / 100], v[v.len() - 1])
            };
            let (f50, f99, fmax) = col(0);
            let (a50, _, amax) = col(1);
            let (p50, _, pmax) = col(2);
            println!(
                "[gotas] {nome} · {quem}: estágio final p50 {f50:.3} p99 {f99:.3} máx {fmax:.3} ms · advect p50 {a50:.2} máx {amax:.2} ms · passo p50 {p50:.1} máx {pmax:.1} ms"
            );
        }
        println!("[gotas] {nome} · gotas vivas no máximo: {gotas_max}");
    }
    println!(
        "[gotas] load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
