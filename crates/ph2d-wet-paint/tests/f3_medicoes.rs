//! Medições ⚑ da E10 (F3 §4 (1)(3)(4)(6)) — sondas `#[ignore]`, doc `docs/Painter/Arquitetura/F3b_medicoes.md`.
//! `PH2D_F3_PRESETS=<f>` (linhas `nome|chave=v;…|size_px|water|opacity`, extraídas dos `.ph2d-brush.png`).
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --profile smoke --test f3_medicoes -- --ignored --nocapture --test-threads 1`

use ph2d_wet_paint::bristles_fios::Fios;
use ph2d_wet_paint::bristles_relevo::Relevo;
use ph2d_wet_paint::grid::{espessura, oleo_celula, tinta::M_UM};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::LeiDaTinta;
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};

const W: usize = 260;
const H: usize = 120;

struct Pre {
    nome: String,
    knobs: Vec<(String, f64)>,
    size_px: f64,
    water: f64,
    opacity: f64,
}

fn presets() -> Vec<Pre> {
    presets_de("PH2D_F3_PRESETS")
}

fn presets_de(var: &str) -> Vec<Pre> {
    let f = std::env::var(var).expect(var);
    std::fs::read_to_string(f)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('|').collect();
            Pre {
                nome: c[0].into(),
                knobs: c[1]
                    .split(';')
                    .filter_map(|kv| {
                        let (k, v) = kv.split_once('=')?;
                        Some((k.to_owned(), v.parse().ok()?))
                    })
                    .collect(),
                size_px: c[2].parse().unwrap(),
                water: c[3].parse().unwrap(),
                opacity: c[4].parse().unwrap(),
            }
        })
        .collect()
}

fn knob(key: &str) -> Option<Knob> {
    KNOB_DEFS.iter().find(|d| d.key == key).map(|d| d.knob)
}

fn motor(p: &Pre, extra: &[(Knob, f64)]) -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    let mut oleo = false;
    for (k, v) in &p.knobs {
        if k == "realOil" {
            oleo = *v > 0.0;
        }
        if let Some(kn) = knob(k) {
            e.set_knob(kn, *v);
        }
    }
    for &(k, v) in extra {
        e.set_knob(k, v);
    }
    e.sim.oleo = oleo;
    e.sliders.size = ((p.size_px * 0.5 - 2.0) / 33.0).clamp(0.0, 1.0);
    e.sliders.water = p.water;
    e.sliders.pigmento = p.opacity;
    e.sliders.pressure = 1.0;
    e
}

/// Três passadas no mesmo sítio (y = H/2), pausa de 120 passos entre elas e 300 no fim.
fn tres_passadas(e: &mut Engine) {
    let y = H as f64 * 0.5;
    for _ in 0..3 {
        // O número do traço avança como na porta do produto (`doors.rs:79`); o `pointer_down` legado não o faz.
        e.sim.traco_actual = e.sim.traco_actual.wrapping_add(1).max(1);
        e.pointer_down(30.0, y, None);
        for i in 1..=40 {
            e.pointer_frame(30.0 + 200.0 * f64::from(i) / 40.0, y);
        }
        e.pointer_up();
        while e.release_frame(230.0, y) {}
        for _ in 0..120 {
            e.step_simulation();
        }
    }
    for _ in 0..300 {
        e.step_simulation();
    }
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

struct Leitura {
    /// espessura (µm de pigmento, `em ÷ M_UM`) das células com massa
    um: Vec<f64>,
    /// altura do relevo Óleo Real, proxy linear: `em × ganho(Depth × Real Oil)` ÷ M_UM
    rel: Vec<f64>,
    /// `g.relevo` (o Emboss pousado) × `em` ÷ M_UM
    emb: Vec<f64>,
    sett_max: f64,
    massa_total: f64,
    /// células com massa > 1 a mais de r + 2 do eixo
    fora: usize,
    /// massa fora da faixa ÷ massa total
    fora_massa: f64,
    raio: f64,
}

fn le(e: &Engine) -> Leitura {
    let g = e.active_grid();
    let r = 2.0 + e.sliders.size * 33.0;
    let yc = H as f64 * 0.5;
    let mut l = Leitura {
        um: vec![],
        rel: vec![],
        emb: vec![],
        sett_max: 0.0,
        massa_total: 0.0,
        fora: 0,
        fora_massa: 0.0,
        raio: r,
    };
    for y in 1..g.h {
        for x in 1..g.w {
            let i = x + y * g.s;
            let m = f64::from(g.sett[i]) + f64::from(g.susp[i]);
            if m <= 1.0 {
                continue;
            }
            l.massa_total += m;
            l.sett_max = l.sett_max.max(f64::from(g.sett[i]));
            if (y as f64 - yc).abs() > r + 2.0 {
                l.fora += 1;
                l.fora_massa += m;
            }
            let t = espessura::em(g, i) / f64::from(M_UM);
            l.um.push(t);
            l.rel.push(t * oleo_celula::ganho(&g.oleo, i));
            let rv = g.relevo.get(i).copied().unwrap_or(1.0);
            l.emb.push(t * f64::from(rv));
        }
    }
    l.fora_massa /= l.massa_total.max(1e-9);
    l
}

#[test]
#[ignore = "sonda F3 (4)(6)(1)"]
fn f3_espessura_relevo_body() {
    println!(
        "M_UM={M_UM} H_REF={} (massa por dab de referência)",
        ph2d_wet_paint::painter::receita::H_REF
    );
    let ps = presets();
    println!(
        "preset|r|um_p50|um_p99|um_max|sett_max|relOleo_p50|relOleo_p99|relOleo_p50_D1|relOleo_p99_D1|emb_p50|emb_p99|emb_p50_E1|emb_p99_E1|lev1_fora|lev1_foraM|lev0_fora|lev0_foraM|lev0_um_p99|lev0_massa/lev1"
    );
    // Intercalado: por preset as três variantes seguidas, mesmo processo.
    for p in &ps {
        let mut a = motor(p, &[]);
        tres_passadas(&mut a);
        let mut b = motor(p, &[(Knob::OilDepth, 1.0), (Knob::Emboss, 1.0)]);
        tres_passadas(&mut b);
        let mut c = motor(p, &[(Knob::LevelingOwnStroke, 0.0)]);
        tres_passadas(&mut c);
        let (mut la, mut lb, mut lc) = (le(&a), le(&b), le(&c));
        println!(
            "{}|{:.1}|{:.3}|{:.3}|{:.3}|{:.0}|{:.4}|{:.4}|{:.4}|{:.4}|{:.4}|{:.4}|{:.4}|{:.4}|{}|{:.5}|{}|{:.5}|{:.3}|{:.4}",
            p.nome,
            la.raio,
            pct(&mut la.um, 0.5),
            pct(&mut la.um, 0.99),
            pct(&mut la.um, 1.0),
            la.sett_max,
            pct(&mut la.rel, 0.5),
            pct(&mut la.rel, 0.99),
            pct(&mut lb.rel, 0.5),
            pct(&mut lb.rel, 0.99),
            pct(&mut la.emb, 0.5),
            pct(&mut la.emb, 0.99),
            pct(&mut lb.emb, 0.5),
            pct(&mut lb.emb, 0.99),
            la.fora,
            la.fora_massa,
            lc.fora,
            lc.fora_massa,
            pct(&mut lc.um, 0.99),
            lc.massa_total / la.massa_total.max(1e-9),
        );
    }
}

/// O tecto da espessura: a mesma célula carregada N vezes (pigmentPerDab no máximo do registo, Water 0).
#[test]
#[ignore = "sonda F3 (4) tecto"]
fn f3_tecto_da_espessura() {
    let ppd_max = KNOB_DEFS
        .iter()
        .find(|d| d.knob == Knob::PigmentPerDab)
        .unwrap()
        .max;
    let p = Pre {
        nome: "carga".into(),
        knobs: vec![
            ("realOil".into(), 1.0),
            ("pigmentPerDab".into(), ppd_max),
            ("drag".into(), 0.0),
            ("smear".into(), 0.0),
            ("pickup".into(), 0.0),
        ],
        size_px: 40.0,
        water: 0.0,
        opacity: 1.0,
    };
    println!("pigmentPerDab max = {ppd_max}");
    println!("passadas|um_p50|um_p99|um_max|sett_max|finito");
    let mut e = motor(&p, &[]);
    for n in 1..=24 {
        tres_passadas(&mut e);
        let mut l = le(&e);
        let fin = l.um.iter().all(|v| v.is_finite());
        if n % 3 == 0 || n == 1 {
            println!(
                "{}|{:.2}|{:.2}|{:.2}|{:.0}|{}",
                n * 3,
                pct(&mut l.um, 0.5),
                pct(&mut l.um, 0.99),
                pct(&mut l.um, 1.0),
                l.sett_max,
                fin
            );
        }
    }
    // A precisão f32: o menor incremento de massa representável a cada espessura.
    for um in [1.0f32, 10.0, 100.0, 1000.0, 10_000.0] {
        let m = um * M_UM;
        let ulp = f32::from_bits(m.to_bits() + 1) - m;
        println!(
            "f32: {um} µm = massa {m:.0}, ulp {ulp} (= {:.2e} µm)",
            ulp / M_UM
        );
    }
}

/// Os leitores das cerdas: cada knob a min × max (o resto como um Flat Bristle) — quantas saídas dos fios mudam.
#[test]
#[ignore = "sonda F3 (3)"]
fn f3_cerdas_cada_knob_decide() {
    let base = Fios {
        pressao: 1.2,
        levanta: 0.4,
        ruido_px: 150.0,
        largura_px: 4.0,
        onda: 0.0,
        onda_px: 500.0,
        oscilacao_px: 0.0,
        span_px: 0.0,
        span_comprimento_px: 10.0,
        cor: [0.0, 0.3, 0.1],
        veio: [0.0, 0.0],
        veio_px: 300.0,
        veio_corte: 0.5,
    };
    type Set = fn(&mut Fios, f64);
    let ks: [(&str, Knob, Set); 17] = [
        ("bristlePressure", Knob::BristlePressure, |f, v| {
            f.pressao = v
        }),
        ("bristleLift", Knob::BristleLift, |f, v| f.levanta = v),
        ("bristleNoiseLength", Knob::BristleNoiseLength, |f, v| {
            f.ruido_px = v
        }),
        ("bristleLaneWidth", Knob::BristleLaneWidth, |f, v| {
            f.largura_px = v
        }),
        ("bristleWave", Knob::BristleWave, |f, v| f.onda = v),
        ("bristleWaveLength", Knob::BristleWaveLength, |f, v| {
            f.onda = 0.5;
            f.onda_px = v;
        }),
        ("bristleOscillation", Knob::BristleOscillation, |f, v| {
            f.oscilacao_px = v
        }),
        ("bristleSpan", Knob::BristleSpan, |f, v| f.span_px = v),
        ("bristleSpanLength", Knob::BristleSpanLength, |f, v| {
            f.span_px = 2.0;
            f.span_comprimento_px = v;
        }),
        ("bristleNoiseHue", Knob::BristleNoiseHue, |f, v| {
            f.cor[0] = v
        }),
        ("bristleNoiseSat", Knob::BristleNoiseSat, |f, v| {
            f.cor[1] = v
        }),
        ("bristleNoiseLight", Knob::BristleNoiseLight, |f, v| {
            f.cor[2] = v
        }),
        ("bristleStreak", Knob::BristleStreak, |f, v| f.veio[0] = v),
        ("bristleStreakLight", Knob::BristleStreakLight, |f, v| {
            f.veio[1] = v
        }),
        ("bristleStreakLength", Knob::BristleStreakLength, |f, v| {
            f.veio[0] = 0.5;
            f.veio_px = v;
        }),
        ("bristleStreakCut", Knob::BristleStreakCut, |f, v| {
            f.veio[0] = 0.5;
            f.veio_corte = v;
        }),
        ("bristleHeight", Knob::BristleHeight, |_, _| {}),
    ];
    let assinatura = |f: &Fios, rel: &Relevo| -> Vec<f64> {
        let mut s = vec![];
        for arco in [0.0, 37.0, 111.0, 400.0, 1234.0] {
            let d = f.resolvido(arco, [1.0, 0.0], 0.6);
            for k in -12..=12 {
                s.push(d.ganho_do_fio(k));
                s.extend(d.cor_do_fio([0.5, 0.3, 0.2], k).map(f64::from));
                s.push(d.veio_do_fio(k));
                s.push(rel.alvo_do_fio(k, d.ganho_do_fio(k), 0.6));
            }
            for yy in -20..=20 {
                for xx in [-9, 0, 9] {
                    let pt = d.no_ponto([f64::from(xx), f64::from(yy)]);
                    s.extend(pt.q);
                    s.push(pt.ganho);
                    s.push(f64::from(pt.fio));
                }
            }
        }
        s
    };
    println!("knob|min|max|saidas|diferentes|max_dif");
    for (nome, kn, set) in ks {
        let d = &KNOB_DEFS[kn as usize];
        assert_eq!(d.key, nome);
        let (mut lo, mut hi) = (base, base);
        set(&mut lo, d.min);
        set(&mut hi, d.max);
        let (rlo, rhi) = if kn == Knob::BristleHeight {
            (
                Relevo {
                    altura_px: d.min,
                    escava: 0.0,
                },
                Relevo {
                    altura_px: d.max,
                    escava: 0.0,
                },
            )
        } else {
            let r = Relevo {
                altura_px: 0.75,
                escava: 0.0,
            };
            (r, r)
        };
        let (a, b) = (assinatura(&lo, &rlo), assinatura(&hi, &rhi));
        let n = a
            .iter()
            .zip(&b)
            .filter(|(x, y)| x.to_bits() != y.to_bits())
            .count();
        let m = a
            .iter()
            .zip(&b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f64::max);
        println!("{nome}|{}|{}|{}|{n}|{m:.4}", d.min, d.max, a.len());
    }
}

/// Um traço de (x0,y0) a (x1,y1) com o número de traço NOVO, como a porta do produto
/// (`painter/doors.rs:79` `begin_direct_stroke`); o `pointer_down` legado não o avança.
fn traco(e: &mut Engine, a: [f64; 2], b: [f64; 2]) {
    e.sim.traco_actual = e.sim.traco_actual.wrapping_add(1).max(1);
    e.pointer_down(a[0], a[1], None);
    for i in 1..=40 {
        let t = f64::from(i) / 40.0;
        e.pointer_frame(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
    }
    e.pointer_up();
    while e.release_frame(b[0], b[1]) {}
}

fn passos(e: &mut Engine, n: u32) {
    for _ in 0..n {
        e.step_simulation();
    }
}

const XC: f64 = 130.0;
const YC: f64 = 60.0;

/// Massa por célula (sett + susp) e espessura µm.
fn campos(e: &Engine) -> (Vec<f64>, Vec<f64>) {
    let g = e.active_grid();
    let mut m = vec![0.0; W * H];
    let mut t = vec![0.0; W * H];
    for y in 1..g.h.min(H) {
        for x in 1..g.w.min(W) {
            let i = x + y * g.s;
            m[x + y * W] = f64::from(g.sett[i]) + f64::from(g.susp[i]);
            t[x + y * W] = espessura::em(g, i) / f64::from(M_UM);
        }
    }
    (m, t)
}

/// Cena do Body: traço 1 horizontal (y = YC), traço 2 vertical (x = XC) — sobre o 1.º molhado (`seco = false`, 10
/// passos) ou seco (`dry_canvas_now`). Devolve (só o 1.º, só o 2.º, os dois), lidos 300 passos depois.
fn cena_body(p: &Pre, lev: f64, seco: bool) -> [(Vec<f64>, Vec<f64>); 3] {
    let h1 = |e: &mut Engine| traco(e, [30.0, YC], [230.0, YC]);
    let v2 = |e: &mut Engine| traco(e, [XC, 8.0], [XC, 112.0]);
    let mut s1 = motor(p, &[(Knob::LevelingOwnStroke, lev)]);
    h1(&mut s1);
    passos(&mut s1, 10);
    if seco {
        s1.dry_canvas_now();
    }
    passos(&mut s1, 300);
    let mut s2 = motor(p, &[(Knob::LevelingOwnStroke, lev)]);
    v2(&mut s2);
    passos(&mut s2, 300);
    let mut ab = motor(p, &[(Knob::LevelingOwnStroke, lev)]);
    h1(&mut ab);
    passos(&mut ab, 10);
    if seco {
        ab.dry_canvas_now();
    }
    v2(&mut ab);
    passos(&mut ab, 300);
    [campos(&s1), campos(&s2), campos(&ab)]
}

#[test]
#[ignore = "sonda F3 (1) Body, 2 traços cruzados"]
fn f3_body_dois_tracos() {
    let ps: Vec<Pre> = presets()
        .into_iter()
        .filter(|p| {
            p.knobs
                .iter()
                .any(|(k, v)| k == "levelingOwnStroke" && *v == 1.0)
        })
        .collect();
    println!(
        "preset|estado|lev|r1|r2|cruz_p50|cruz_p99|borda3_p50|borda3_p99|escoa_1_para_2|fora1_massa"
    );
    for p in &ps {
        for seco in [false, true] {
            for lev in [1.0, 0.0] {
                let [s1, s2, ab] = cena_body(p, lev, seco);
                let e = motor(p, &[]);
                let r1 = 2.0 + e.sliders.size * 33.0;
                let r2 = r1;
                let (mut cruz, mut borda) = (vec![], vec![]);
                // Escoa: massa do conjunto − (só 1.º + só 2.º) na coluna do 2.º FORA da faixa do 1.º e na faixa do 1.º FORA
                // da coluna do 2.º (o que o 1.º perdeu para o 2.º e o que o 2.º arrastou do 1.º).
                let (mut ganho_fora1, mut m1_total) = (0.0, 0.0);
                for y in 1..H {
                    for x in 1..W {
                        let i = x + y * W;
                        let (dx, dy) = ((x as f64 - XC).abs(), (y as f64 - YC).abs());
                        m1_total += s1.0[i];
                        if dx <= r2 && dy <= r1 {
                            cruz.push(ab.1[i]);
                        }
                        if dx > r2 + 2.0 && dy > r1 - 3.0 && dy <= r1 {
                            borda.push(ab.1[i]);
                        }
                        if dx <= r2 + 2.0 && dy > r1 + 2.0 {
                            ganho_fora1 += ab.0[i] - s1.0[i] - s2.0[i];
                        }
                    }
                }
                let fora1: f64 = (1..H)
                    .flat_map(|y| (1..W).map(move |x| (x, y)))
                    .filter(|&(x, y)| {
                        (y as f64 - YC).abs() > r1 + 2.0 && (x as f64 - XC).abs() > r2 + 2.0
                    })
                    .map(|(x, y)| ab.0[x + y * W])
                    .sum();
                println!(
                    "{}|{}|{lev}|{r1:.1}|{r2:.1}|{:.3}|{:.3}|{:.3}|{:.3}|{:.4}|{:.4}",
                    p.nome,
                    if seco { "seco" } else { "molhado" },
                    pct(&mut cruz, 0.5),
                    pct(&mut cruz, 0.99),
                    pct(&mut borda, 0.5),
                    pct(&mut borda, 0.99),
                    ganho_fora1 / m1_total.max(1e-9),
                    fora1 / m1_total.max(1e-9),
                );
            }
        }
    }
}

/// (5) A água no papel × `paperGate`: o que entra (fracção da faixa com massa, água pousada) e a velocidade com que a
/// mancha molhada cresce, nos pincéis de tinta (`PH2D_F3_TINTA`), variantes intercaladas.
#[test]
#[ignore = "sonda F3 (5) Absorbency"]
fn f3_absorbency_paper_gate() {
    let ps = presets_de("PH2D_F3_TINTA");
    let gates = [0.0, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2];
    println!(
        "preset|gate|cobre_faixa|agua_t0|molhadas_t0|molhadas_t30|molhadas_t120|cresce_px2_por_passo|massa_total|sett_frac_t120"
    );
    for p in &ps {
        for &gt in &gates {
            let mut e = motor(p, &[(Knob::PaperGate, gt)]);
            traco(&mut e, [30.0, YC], [230.0, YC]);
            let r = 2.0 + e.sliders.size * 33.0;
            let le_ = |e: &Engine| {
                let g = e.active_grid();
                let (mut cob, mut nf, mut agua, mut wet, mut m, mut s) =
                    (0usize, 0usize, 0.0, 0usize, 0.0, 0.0);
                for y in 1..g.h {
                    for x in 1..g.w {
                        let i = x + y * g.s;
                        let mm = f64::from(g.sett[i]) + f64::from(g.susp[i]);
                        let dentro = (y as f64 - YC).abs() <= r && (40..=220).contains(&x);
                        if dentro {
                            nf += 1;
                            if mm > 1.0 {
                                cob += 1;
                            }
                        }
                        agua += f64::from(g.film[i]);
                        if g.film[i] > 0.01 {
                            wet += 1;
                        }
                        m += mm;
                        s += f64::from(g.sett[i]);
                    }
                }
                (cob as f64 / nf.max(1) as f64, agua, wet, m, s / m.max(1e-9))
            };
            let a = le_(&e);
            passos(&mut e, 30);
            let b = le_(&e);
            passos(&mut e, 90);
            let c = le_(&e);
            println!(
                "{}|{gt}|{:.3}|{:.1}|{}|{}|{}|{:.2}|{:.0}|{:.3}",
                p.nome,
                a.0,
                a.1,
                a.2,
                b.2,
                c.2,
                (b.2 as f64 - a.2 as f64) / 30.0,
                c.3,
                c.4
            );
        }
    }
}
