//! O ACÚMULO das passagens e o CUSTO do Óleo Real (partido de `medida_oleo_real.rs`, 29/10).

use super::*;

// ── Acúmulo: o relevo do Óleo Real sobe com as passagens? ───────────────────────────────────────────
// O MESMO traço horizontal (tamanho 100, azul, pressão 1, passo 3 px, relógio fixo, espera a água) 1·2·4·8
// vezes, empilhado (`dy = 0`) ou desfasado 15 px por passagem (`dy = 15`). Mede na faixa central do traço
// (x 200..500, ±15 px do eixo médio): massa sett+susp por camada e total, a altura da FORMA (`CampoAltura::at`)
// e a fracção de células com tinta com `luz < 0,9` (sombra própria), ao Óleo Real de fábrica.

pub(super) const AW: u32 = 700;
pub(super) const AH: u32 = 420;
pub(super) const AY: f32 = 130.0;
pub(super) const AX0: f32 = 100.0;
pub(super) const AX1: f32 = 600.0;

pub(super) fn um_traco(t: &mut PainterTool, y: f32) -> u32 {
    let n = ((AX1 - AX0) / 3.0).ceil() as u32;
    t.on_canvas_pointer(ponto([AX0, y], PointerPhase::Down));
    for k in 1..=n {
        let x = AX0 + (AX1 - AX0) * k as f32 / n as f32;
        t.on_canvas_pointer(ponto([x, y], PointerPhase::Move));
        t.on_tick(8.0);
        if std::env::var("PH2D_OLEO_ACUMULO_TRACO").is_ok()
            && k % 6 == 0
            && (300.0..=420.0).contains(&x)
        {
            let s = t.paint.wetpaint.session.as_mut().expect("sessão");
            s.bring_home();
            let r = f32::from(s.ratio);
            let g = s.engine.active_grid();
            let i = ((350.0 / r) as usize + 1) + ((y / r) as usize + 1) * g.s;
            println!(
                "[traço] x={x:>5.0} célula(350,{y}) sett {:>7.0} susp {:>7.0} film {:>6.2}",
                g.sett[i], g.susp[i], g.film[i]
            );
        }
    }
    t.on_canvas_pointer(ponto([AX1, y], PointerPhase::Up));
    let mut ticks = 0;
    loop {
        t.on_tick(1000.0 / 60.0);
        ticks += 1;
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        if !s.engine.active_grid().has_fluid || ticks >= 60 * 60 {
            break;
        }
    }
    ticks
}

pub(super) fn imprime_acumulo(t: &mut PainterTool, rot: &str, passes: usize, yc: f32) {
    // As variantes da altura da SOMBRA no MESMO processo, sobre a MESMA tinta (W1b passo 2).
    for raio in [3usize, 1, 0] {
        let mut o = ph2d_wet_paint::oleo_real::OleoReal::default();
        o.af.raio_sombra = raio;
        imprime_acumulo_com(t, &format!("{rot} rs{raio}"), passes, yc, &o);
    }
}

pub(super) fn imprime_acumulo_com(
    t: &mut PainterTool,
    rot: &str,
    passes: usize,
    yc: f32,
    o: &ph2d_wet_paint::oleo_real::OleoReal,
) {
    let o = *o;
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let r = f32::from(s.ratio);
    let (gw, gh) = s.grid;
    let grades: Vec<(&ph2d_wet_paint::grid::Grid, bool)> = s
        .engine
        .layers
        .iter()
        .map(|l| (&l.grid, l.visible))
        .collect();
    let nl = grades.len();
    let campo =
        ph2d_wet_paint::oleo_real::CampoAltura::de(&grades, &o, usize::from(s.ratio), 1, 1, gw, gh);
    let st = grades[0].0.s;
    let (c0, c1) = ((200.0 / r) as usize + 1, ((500.0 / r) as usize + 1).min(gw));
    let (l0, l1) = (
        ((yc - 15.0) / r) as usize + 1,
        (((yc + 15.0) / r) as usize + 1).min(gh),
    );
    let mut por_camada: Vec<Vec<f64>> = vec![vec![]; nl];
    let (mut tot, mut alt, mut susp, mut horiz) = (vec![], vec![], vec![], vec![]);
    let (mut com_tinta, mut sombra) = (0usize, 0usize);
    for cy in l0..=l1 {
        for cx in c0..=c1 {
            let i = cx + cy * st;
            let mut m = 0.0;
            let mut su = 0.0;
            for (k, (g, vis)) in grades.iter().enumerate() {
                let v = f64::from(g.sett[i]) + f64::from(g.susp[i]);
                por_camada[k].push(v);
                if *vis {
                    m += v;
                    su += f64::from(g.susp[i]);
                }
            }
            tot.push(m);
            susp.push(su);
            alt.push(campo.at_sombra(cx as isize, cy as isize));
            let (x, y) = (cx as isize, cy as isize);
            // A subida máxima (células de altura por célula) nos 24 passos para a luz (topo-esquerda).
            let mut sobe = f64::NEG_INFINITY;
            for k in 1..=24isize {
                let d = (k as f64 * core::f64::consts::FRAC_1_SQRT_2).round() as isize;
                sobe = sobe.max((campo.at_sombra(x - d, y - d) - campo.at_sombra(x, y)) / k as f64);
            }
            horiz.push(sobe);
            if m > 0.0 {
                com_tinta += 1;
                if ph2d_wet_paint::oleo_real::termos(&o, &campo, cx, cy).luz < 0.9 {
                    sombra += 1;
                }
            }
        }
    }
    // A sombra própria em TODA a área do traço (não só a faixa central): onde cai?
    let (mut tinta_all, mut sombra_all, mut sombra_borda) = (0usize, 0usize, 0usize);
    let (b0, b1) = (
        ((yc - 80.0).max(0.0) / r) as usize + 1,
        (((yc + 130.0) / r) as usize + 1).min(gh),
    );
    for cy in b0..=b1 {
        for cx in c0..=c1 {
            let i = cx + cy * st;
            let m: f64 = grades
                .iter()
                .filter(|g| g.1)
                .map(|(g, _)| f64::from(g.sett[i]) + f64::from(g.susp[i]))
                .sum();
            if m > 0.0 {
                tinta_all += 1;
                if ph2d_wet_paint::oleo_real::termos(&o, &campo, cx, cy).luz < 0.9 {
                    sombra_all += 1;
                    // borda = algum vizinho a ≤ 3 células sem tinta
                    let perto_papel = (-3i64..=3).any(|d| {
                        let j = (cx as i64 + (cy as i64 + d) * st as i64) as usize;
                        grades
                            .iter()
                            .filter(|g| g.1)
                            .map(|(g, _)| f64::from(g.sett[j]) + f64::from(g.susp[j]))
                            .sum::<f64>()
                            <= 0.0
                    });
                    if perto_papel {
                        sombra_borda += 1;
                    }
                }
            }
        }
    }
    let camadas: Vec<String> = por_camada
        .iter_mut()
        .map(|v| format!("{:.0}/{:.0}/{:.0}", pct(v, 0.5), pct(v, 0.9), pct(v, 1.0)))
        .collect();
    println!(
        "{rot:<26} {passes:>2} r{r} | {:>7.0} {:>7.0} {:>7.0} | susp p50 {:>6.0} | {:>6.3} {:>6.3} {:>6.3} | horiz p50 {:>6.3} p99 {:>6.3} | sombra {:>5.1}% ({com_tinta}) | área {:>5.1}% borda {:>5.1}% de {tinta_all} | camadas {nl} [{}]",
        pct(&mut tot, 0.5),
        pct(&mut tot, 0.9),
        pct(&mut tot, 1.0),
        pct(&mut susp, 0.5),
        pct(&mut alt, 0.5),
        pct(&mut alt, 0.9),
        pct(&mut alt, 1.0),
        pct(&mut horiz, 0.5),
        pct(&mut horiz, 0.99),
        100.0 * sombra as f64 / com_tinta.max(1) as f64,
        100.0 * sombra_all as f64 / tinta_all.max(1) as f64,
        100.0 * sombra_borda as f64 / sombra_all.max(1) as f64,
        camadas.join(" · "),
    );
}

#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_acumulo() {
    let b = biblioteca();
    println!(
        "{:<26} {:>2} | {:>7} {:>7} {:>7} | {:>13} | {:>6} {:>6} {:>6} | sombra própria (luz<0,9) na faixa · área · borda",
        "preset/variante", "n", "m p50", "m p90", "m max", "", "h p50", "h p90", "h max"
    );
    for preset in [
        "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
        "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
    ] {
        let p = &preset_rel(&b, preset);
        for dy in [0.0f32, 15.0] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (AW * AH * 4) as usize], AW, AH);
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_wet_relogio_fixo(true);
            let erros = t.wet_preset_aplicar(p);
            assert!(erros.is_empty(), "{erros:?}");
            t.set_brush_size_px(50.0);
            t.set_brush_color_srgb8(AZ);
            let nome = preset
                .rsplit('/')
                .next()
                .unwrap()
                .trim_end_matches(".ph2d-brush.png");
            let rot = format!("{nome} dy{dy}");
            for k in 0..8usize {
                let ticks = um_traco(&mut t, AY + dy * k as f32);
                let n = k + 1;
                if [1, 2, 4, 8].contains(&n) {
                    let yc = AY + dy * (k as f32) * 0.5;
                    imprime_acumulo(&mut t, &format!("{rot} (água {ticks})"), n, yc);
                }
            }
        }
    }
}

/// O custo por célula da óptica na CPU (o caminho da pintura ao vivo até haver passe de placa): os
/// ambientes no MESMO processo, intercalados, o MÍNIMO de 7 rodadas (`docs/DevOps/MEDIR_VELOCIDADE.md`).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_custo() {
    use ph2d_wet_paint::oleo_real::{Ambiente, CampoAltura, OleoReal, termos_com};
    let mut t = pinta_folha();
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let (gw, gh) = s.grid;
    let grades: Vec<(&ph2d_wet_paint::grid::Grid, bool)> = s
        .engine
        .layers
        .iter()
        .map(|l| (&l.grid, l.visible))
        .collect();
    let campo = CampoAltura::de(
        &grades,
        &OleoReal::default(),
        usize::from(s.ratio),
        1,
        1,
        gw,
        gh,
    );
    // W2 E3-B: os quatro ambientes com o material NEUTRO e o Clássico e o Estúdio com um material CHEIO
    // (todos os lóbulos a meio), intercalados.
    let cheio = ph2d_wet_paint::material::MaterialCelula {
        metal: 0.5,
        verniz: 0.5,
        veludo: 0.5,
        subsuperficie: 0.5,
        ..ph2d_wet_paint::material::MaterialCelula::NEUTRO
    };
    let neutro = ph2d_wet_paint::material::MaterialCelula::NEUTRO;
    let variantes: Vec<(String, Ambiente, ph2d_wet_paint::material::MaterialCelula)> =
        Ambiente::TODOS
            .iter()
            .map(|&a| (format!("{a:?}"), a, neutro))
            .chain(
                [Ambiente::Classico, Ambiente::Estudio].map(|a| (format!("{a:?}+cheio"), a, cheio)),
            )
            .collect();
    let nv = variantes.len();
    let mut melhor = vec![f64::INFINITY; nv];
    let mut soma = 0.0;
    for rodada in 0..7 {
        for k in 0..nv {
            let i = (k + rodada) % nv;
            let (_, a, m) = &variantes[i];
            let o = OleoReal {
                ambiente: *a,
                ..OleoReal::default()
            };
            let t0 = std::time::Instant::now();
            for cy in (1..=gh).step_by(4) {
                for cx in (1..=gw).step_by(4) {
                    soma += termos_com(&o, &campo, cx, cy, m, [0.6, 0.1, 0.1]).vis;
                }
            }
            let ns = t0.elapsed().as_nanos() as f64 / ((gw / 4) * (gh / 4)) as f64;
            melhor[i] = melhor[i].min(ns);
        }
    }
    println!(
        "[custo] ns/célula (mínimo de 7): {:?} · {soma:.0}",
        variantes
            .iter()
            .map(|v| &v.0)
            .zip(melhor)
            .collect::<Vec<_>>()
    );
    // W3 M1: o CAMPO com a Subsurface (a irradiância da região alargada + as duas passagens), na tela inteira e
    // numa janela de pincelada (64×64 + o alcance), com o raio no padrão, a 16 e sem ela — intercalados.
    let (mx, my) = (gw / 2, gh / 2);
    let janelas = [
        ("tela", (1, 1, gw, gh)),
        ("pincelada", (mx - 32, my - 32, mx + 32, my + 32)),
    ];
    let raios = [("sem", 0.0, 4.0), ("r4", 1.0, 4.0), ("r16", 1.0, 16.0)];
    let mut melhor = [[f64::INFINITY; 3]; 2];
    for rodada in 0..5 {
        for (j, &(_, (x0, y0, x1, y1))) in janelas.iter().enumerate() {
            for k in 0..3 {
                let i = (k + rodada) % 3;
                let mut o = OleoReal::default();
                o.material.subsuperficie = raios[i].1;
                o.material.raio_subsuperficie = raios[i].2;
                let a = ph2d_wet_paint::oleo_real::alcance(&o);
                let (jx0, jy0) = (x0.saturating_sub(a).max(1), y0.saturating_sub(a).max(1));
                let (jx1, jy1) = ((x1 + a).min(gw), (y1 + a).min(gh));
                let t0 = std::time::Instant::now();
                let c = CampoAltura::de(&grades, &o, usize::from(s.ratio), jx0, jy0, jx1, jy1);
                soma += c.subsuperficie_em(mx, my)[0];
                let ns = t0.elapsed().as_nanos() as f64 / ((x1 + 1 - x0) * (y1 + 1 - y0)) as f64;
                melhor[j][i] = melhor[j][i].min(ns);
            }
        }
    }
    for (j, (nome, _)) in janelas.iter().enumerate() {
        println!(
            "[custo] campo {nome}, ns por célula PEDIDA (mínimo de 5): {:?} · {soma:.0}",
            raios.iter().map(|r| r.0).zip(melhor[j]).collect::<Vec<_>>()
        );
    }
    // K4: o RECOMPOR de uma pincelada (o campo + o sombreado e a sombra no papel de cada célula pedida) sem
    // Subsurface, com ela e a luz do céu GUARDADA, e com ela contada de novo (`esquece_luz_do_ceu`) — intercalados.
    let modos = ["sem", "r4 guardada", "r4 contada"];
    let mut melhor = [f64::INFINITY; 3];
    let (x0, y0, x1, y1) = (mx - 32, my - 32, mx + 32, my + 32);
    for rodada in 0..5 {
        for k in 0..3 {
            let i = (k + rodada) % 3;
            let mut o = OleoReal::default();
            if i > 0 {
                o.material.subsuperficie = 1.0;
            }
            let t0 = std::time::Instant::now();
            let mut c = CampoAltura::de(&grades, &o, usize::from(s.ratio), x0, y0, x1, y1);
            if i == 2 {
                c.esquece_luz_do_ceu();
            }
            for cy in y0..=y1 {
                for cx in x0..=x1 {
                    let m = c.material_em(cx, cy);
                    soma += termos_com(&o, &c, cx, cy, &m, [0.6, 0.1, 0.1]).vis;
                    soma += ph2d_wet_paint::oleo_real::sombra_no_papel(&o, &c, cx, cy);
                }
            }
            let ns = t0.elapsed().as_nanos() as f64 / ((x1 + 1 - x0) * (y1 + 1 - y0)) as f64;
            melhor[i] = melhor[i].min(ns);
        }
    }
    println!(
        "[custo] recompor da pincelada 65², ns por célula (mínimo de 5): {:?} · {soma:.0}",
        modos.iter().zip(melhor).collect::<Vec<_>>()
    );
    // K10: o mesmo recompor com o rebote das paredes no papel desligado e o de fábrica, Clássico e Estúdio.
    let modos = [
        "Clássico sem rebote",
        "Clássico",
        "Estúdio sem rebote",
        "Estúdio",
    ];
    let mut melhor = [f64::INFINITY; 4];
    for rodada in 0..7 {
        for k in 0..4 {
            let i = (k + rodada) % 4;
            let mut o = OleoReal {
                ambiente: if i < 2 {
                    Ambiente::Classico
                } else {
                    Ambiente::Estudio
                },
                ..OleoReal::default()
            };
            if i % 2 == 0 {
                o.af.rebote_papel = 0.0;
            }
            let t0 = std::time::Instant::now();
            let c = CampoAltura::de(&grades, &o, usize::from(s.ratio), x0, y0, x1, y1);
            for cy in y0..=y1 {
                for cx in x0..=x1 {
                    let m = c.material_em(cx, cy);
                    soma += termos_com(&o, &c, cx, cy, &m, [0.6, 0.1, 0.1]).vis;
                    soma += ph2d_wet_paint::oleo_real::sombra_no_papel(&o, &c, cx, cy);
                }
            }
            let ns = t0.elapsed().as_nanos() as f64 / ((x1 + 1 - x0) * (y1 + 1 - y0)) as f64;
            melhor[i] = melhor[i].min(ns);
        }
    }
    println!(
        "[custo] K10 recompor 65² com/sem rebote, ns por célula (mínimo de 7): {:?} · {soma:.0}",
        modos.iter().zip(melhor).collect::<Vec<_>>()
    );
    println!(
        "[custo] loadavg {}",
        std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
    );
}
