//! W0b: a espátula exprime-se com os knobs de hoje? As candidatas e a medida da faixa (partido de
//! `medida_oleo_real.rs`, 29/10).

use super::*;

// ── W0b: a espátula exprime-se com os knobs de hoje? ────────────────────────────────────────────────
// Um traço horizontal (tamanho 120, 450 px, azul, pressão 1, passo 3 px, relógio fixo, espera a água) por
// candidata, num tool novo. Mede a faixa: largura (m>10), rampa da borda 90→10 % do patamar, patamar p50,
// ruído célula-a-célula (m − média 3×3) no miolo 60 %, e orla (máx a ≤5 px da borda ÷ patamar).

pub(super) const FW: u32 = 700;
pub(super) const FH: u32 = 320;
pub(super) const FY: f32 = 160.0;
pub(super) const FX0: f32 = 100.0;
pub(super) const FX1: f32 = 550.0;

pub(super) type Ajuste = fn(&mut PainterTool);

pub(super) fn lisa_dura(t: &mut PainterTool) {
    t.paint.brush.jitter = 0.0;
    t.set_brush_hardness(1.0);
    t.set_brush_falloff(ph2d_painter_brush::Falloff::Constant as u8);
    t.set_brush_shape_kind(ph2d_painter_brush::TextureKind::None.to_u8());
}

pub(super) fn quadrada(t: &mut PainterTool) {
    lisa_dura(t);
    t.set_brush_dab_quina(1.0);
}

pub(super) fn quadrada_sem_dente(t: &mut PainterTool) {
    quadrada(t);
    t.paint.wetpaint.knobs.set(Knob::PaperGate, 0.0);
}

/// A lâmina: quadrada, lisa, dura, sem dente, achatada ATRAVÉS do traço (o fio da faca perpendicular à marcha).
pub(super) fn lamina(t: &mut PainterTool) {
    quadrada_sem_dente(t);
    t.set_brush_dab_flatten(0.8);
    t.set_brush_dab_angle(90.0);
}

/// A lâmina com a curva Custom de queda: miolo a 0,6, pico 1 a 0,85 do raio, 0 na borda (cantos Vector) — a orla
/// pela composição, sem lei nova.
pub(super) fn lamina_com_orla(t: &mut PainterTool) {
    lamina(t);
    t.set_brush_hardness(0.0);
    t.set_brush_falloff(ph2d_painter_brush::Falloff::Custom as u8);
    t.set_brush_falloff_point(0, 0.0, 0.6);
    t.set_brush_falloff_point(1, 1.0, 0.0);
    let pico = t.add_brush_falloff_point_at(0.85, 1.0).expect("ponto");
    for id in [0, 1, pico] {
        t.set_brush_falloff_point_handle(id, 1);
    }
}

pub(super) fn candidatas() -> Vec<(&'static str, &'static str, Ajuste)> {
    pub(super) const DAB: &str = "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png";
    pub(super) const FIL: &str = "WetPainter/Oil/Flat/Filbert.ph2d-brush.png";
    pub(super) const MRK: &str = "WetPainter/Marker/Marker/Marker Bullet.ph2d-brush.png";
    vec![
        ("A Dabber fabrica", DAB, |_| {}),
        ("B +jitter0", DAB, |t| t.paint.brush.jitter = 0.0),
        ("C +dura1 Constant", DAB, |t| {
            t.paint.brush.jitter = 0.0;
            t.set_brush_hardness(1.0);
            t.set_brush_falloff(ph2d_painter_brush::Falloff::Constant as u8);
        }),
        ("D C+sem cerdas", DAB, lisa_dura),
        ("E D+quina1", DAB, quadrada),
        ("F E+gate0", DAB, quadrada_sem_dente),
        ("G F+flat.8 ang0", DAB, |t| {
            quadrada_sem_dente(t);
            t.set_brush_dab_flatten(0.8);
            t.set_brush_dab_angle(0.0);
        }),
        ("H F+flat.8 ang90", DAB, |t| {
            quadrada_sem_dente(t);
            t.set_brush_dab_flatten(0.8);
            t.set_brush_dab_angle(90.0);
        }),
        ("I F+lev0 cap0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Capillary, 0.0);
        }),
        ("J F+lev2", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 2.0);
            t.paint.wetpaint.knobs.set(Knob::LevelingOwnStroke, 1.0);
        }),
        ("K F+agua0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.water = 0.0;
        }),
        ("L F+pig500", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 500.0);
        }),
        ("M F+drag0 pick0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Drag, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Pickup, 0.0);
        }),
        ("H2 H+pig500", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 500.0);
        }),
        ("H3 H+lev0", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("H4 H+edgeDark50", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 50.0);
        }),
        ("H5 H+edgeDark200", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 200.0);
        }),
        ("H6 H+heavyEdge1", DAB, |t| {
            lamina(t);
            t.set_wet_borda_que_pesa(1.0);
        }),
        ("H7 H+flat.95", DAB, |t| {
            lamina(t);
            t.set_brush_dab_flatten(0.95);
        }),
        ("R H sem quina", DAB, |t| {
            lamina(t);
            t.set_brush_dab_quina(0.0);
        }),
        ("S R+lev0", DAB, |t| {
            lamina(t);
            t.set_brush_dab_quina(0.0);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("T D+gate0 lev0", DAB, |t| {
            lisa_dura(t);
            t.paint.wetpaint.knobs.set(Knob::PaperGate, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("U lamina curva-orla", DAB, lamina_com_orla),
        ("V U+lev0", DAB, |t| {
            lamina_com_orla(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("N Filbert fabrica", FIL, |_| {}),
        ("O Filbert+F", FIL, quadrada_sem_dente),
        ("P Marker fabrica", MRK, |_| {}),
        ("Q Marker+F", MRK, quadrada_sem_dente),
    ]
}

pub(super) fn mediana(mut v: Vec<f64>) -> f64 {
    pct(&mut v, 0.5)
}

/// Onde, andando de `de` para fora (passo `dir` = ±1 em linhas), a coluna `m` cruza `alvo`
/// pela primeira vez — em linhas fracionárias (interpolação linear), `None` se nunca.
pub(super) fn cruza(m: &[f64], de: usize, dir: i64, alvo: f64) -> Option<f64> {
    let mut i = de as i64;
    loop {
        let j = i + dir;
        if j < 0 || j as usize >= m.len() {
            return None;
        }
        let (a, b) = (m[i as usize], m[j as usize]);
        if a >= alvo && b < alvo {
            let f = (a - alvo) / (a - b);
            return Some(i as f64 + dir as f64 * f);
        }
        i = j;
    }
}

#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_faca_candidatas() {
    let b = biblioteca();
    println!(
        "{:<20} {:>5} {:>7} {:>7} {:>7} {:>8} {:>8} {:>7} {:>6} {:>6}  perfil topo (m/P, passos de 1 célula da borda 10% p/ dentro)",
        "candidata",
        "razão",
        "larg",
        "rampT",
        "rampB",
        "P p50",
        "ruído",
        "ruído%",
        "orlaT",
        "orlaB"
    );
    for (nome, preset, ajuste) in candidatas() {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (FW * FH * 4) as usize], FW, FH);
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_wet_relogio_fixo(true);
        let p = &preset_rel(&b, preset);
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(60.0);
        t.set_brush_color_srgb8(AZ);
        ajuste(&mut t);
        let n = ((FX1 - FX0) / 3.0).ceil() as u32;
        t.on_canvas_pointer(ponto([FX0, FY], PointerPhase::Down));
        for k in 1..=n {
            let x = FX0 + (FX1 - FX0) * k as f32 / n as f32;
            t.on_canvas_pointer(ponto([x, FY], PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(ponto([FX1, FY], PointerPhase::Up));
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
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        let r = f64::from(s.ratio);
        let (gw, gh) = s.grid;
        let g = s.engine.active_grid();
        let st = g.s;
        let m = |cx: usize, cy: usize| {
            f64::from(g.sett[cx + cy * st]) + f64::from(g.susp[cx + cy * st])
        };
        // Colunas do meio do traço (px 200..450), em células.
        let c0 = (200.0 / r) as usize + 1;
        let c1 = ((450.0 / r) as usize + 1).min(gw);
        let yc = (f64::from(FY) / r) as usize + 1;
        let mut larg = vec![];
        let mut miolo = vec![];
        let mut cols: Vec<Vec<f64>> = vec![];
        for cx in c0..c1 {
            let col: Vec<f64> = (1..=gh).map(|cy| m(cx, cy)).collect();
            larg.push(col.iter().filter(|&&v| v > 10.0).count() as f64 * r);
            cols.push(col);
        }
        // O miolo 60 % da faixa (pela largura mediana), centrado no eixo do traço.
        let meia = mediana(larg.clone()) / r * 0.5;
        let (lo, hi) = (
            (yc as f64 - 0.6 * meia).ceil() as usize,
            (yc as f64 + 0.6 * meia).floor() as usize,
        );
        let mut ruido = vec![];
        for cx in c0..c1 {
            for cy in lo.max(2)..=hi.min(gh - 1) {
                miolo.push(m(cx, cy));
                let mut soma = 0.0;
                for dy in [-1i64, 0, 1] {
                    for dx in [-1i64, 0, 1] {
                        soma += m((cx as i64 + dx) as usize, (cy as i64 + dy) as usize);
                    }
                }
                ruido.push(m(cx, cy) - soma / 9.0);
            }
        }
        let plat = mediana(miolo).max(1e-9);
        // A ondulação: o desvio das médias por COLUNA (ao longo do traço) e por LINHA (através), ÷ patamar.
        let dp = |v: &[f64]| {
            let n = v.len().max(1) as f64;
            let mu = v.iter().sum::<f64>() / n;
            (v.iter().map(|x| (x - mu) * (x - mu)).sum::<f64>() / n).sqrt()
        };
        let por_col: Vec<f64> = (c0..c1)
            .map(|cx| {
                (lo.max(2)..=hi.min(gh - 1))
                    .map(|cy| m(cx, cy))
                    .sum::<f64>()
                    / (hi + 1 - lo) as f64
            })
            .collect();
        let por_lin: Vec<f64> = (lo.max(2)..=hi.min(gh - 1))
            .map(|cy| (c0..c1).map(|cx| m(cx, cy)).sum::<f64>() / (c1 - c0) as f64)
            .collect();
        let (ondula_x, ondula_y) = (100.0 * dp(&por_col) / plat, 100.0 * dp(&por_lin) / plat);
        let nr = ruido.len().max(1) as f64;
        let mu = ruido.iter().sum::<f64>() / nr;
        let sd = (ruido.iter().map(|v| (v - mu) * (v - mu)).sum::<f64>() / nr).sqrt();
        // Rampa e orla por coluna (índice da coluna = cy − 1).
        let (mut rt, mut rb, mut ot, mut ob) = (vec![], vec![], vec![], vec![]);
        let i0 = yc - 1;
        let orla_cel = (5.0 / r).ceil() as i64;
        let mut perfil_topo: Vec<Vec<f64>> = vec![];
        for col in &cols {
            for (dir, rampa, orla) in [(-1i64, &mut rt, &mut ot), (1i64, &mut rb, &mut ob)] {
                // A borda: andar do eixo para fora até cair abaixo de 10 %; o 90 % procura-se a partir daí PARA DENTRO
                // (a última vez que estava acima de 90 % antes da borda), para a orla não confundir.
                let Some(e10) = cruza(col, i0, dir, 0.1 * plat) else {
                    continue;
                };
                let k10 = e10.round() as i64;
                let mut e90 = None;
                let mut i = k10;
                while (i - i0 as i64) * dir > 0 {
                    let v = col[i as usize];
                    if v >= 0.9 * plat {
                        let prox = col[(i + dir) as usize];
                        e90 = Some(
                            i as f64
                                + dir as f64
                                    * ((v - 0.9 * plat) / (v - prox).max(1e-9)).clamp(0.0, 1.0),
                        );
                        break;
                    }
                    i -= dir;
                }
                if let Some(e90) = e90 {
                    rampa.push((e10 - e90).abs() * r);
                }
                let mx = (0..=orla_cel)
                    .map(|d| col[(k10 - dir * d).clamp(0, col.len() as i64 - 1) as usize])
                    .fold(0.0f64, f64::max);
                orla.push(mx / plat);
                if dir == -1 {
                    perfil_topo.push(
                        (0..12)
                            .map(|d| col[(k10 + d).clamp(0, col.len() as i64 - 1) as usize] / plat)
                            .collect(),
                    );
                }
            }
        }
        let perfil: Vec<String> = (0..12)
            .map(|d| format!("{:.2}", mediana(perfil_topo.iter().map(|p| p[d]).collect())))
            .collect();
        println!(
            "{:<20} {:>5} {:>7.1} {:>7.1} {:>7.1} {:>8.1} {:>8.1} {:>7.1} {:>6.2} {:>6.2}  {}  (ondulaX {:.1}% ondulaY {:.1}%, n rampa {}/{}, água {} ticks)",
            nome,
            r,
            mediana(larg),
            mediana(rt.clone()),
            mediana(rb.clone()),
            plat,
            sd,
            100.0 * sd / plat,
            mediana(ot),
            mediana(ob),
            perfil.join(" "),
            ondula_x,
            ondula_y,
            rt.len(),
            cols.len(),
            ticks
        );
    }
}
