//! O gate-lei das PONTAS RECTAS (J6, [`super::tampa`]): o recto de 40 px com a caixa ligada acaba cortado a direito
//! nas duas pontas (a CURVA da régua `leis_oraculo.pontas`, no recto: Rebelle Square Hard 40 px ≈ 0), e no meio pousa
//! a MESMA tinta que o redondo (a compensação da metade).

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const W: usize = 300;
const H: usize = 120;
const Y: f32 = 60.0;
const X0: f32 = 60.0;
const X1: f32 = 240.0;

fn recto(rectas: bool) -> Vec<f64> {
    recto_alem(rectas, 0.0)
}

fn recto_alem(rectas: bool, alem: f32) -> Vec<f64> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 60, 200]);
    if rectas {
        t.toggle_brush_pontas_rectas();
        t.set_brush_pontas_alem(alem);
    }
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, Y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(X0, PointerPhase::Down));
    let n = ((X1 - X0) / 3.0) as usize;
    for k in 1..=n {
        t.on_canvas_pointer(p(X0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(X0 + 3.0 * n as f32, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    t.canvas_rgba
        .chunks(4)
        .map(|px| f64::from(255 - px[0].min(px[1]).min(px[2])) / 255.0)
        .collect()
}

/// A CURVA das duas pontas (`leis_oraculo.pontas`): (o além no eixo − o além nas bordas) ÷ a meia-largura; e a tinta
/// do miolo (a soma das colunas do meio).
fn mede(c: &[f64]) -> ([f64; 2], f64) {
    let (xa, xb) = (X0 as usize + 50, X1 as usize - 50);
    let lin: Vec<f64> = (0..H)
        .map(|y| (xa..xb).map(|x| c[y * W + x]).sum::<f64>() / (xb - xa) as f64)
        .collect();
    let topo = lin.iter().cloned().fold(0.0, f64::max);
    let rows: Vec<usize> = (0..H).filter(|&y| lin[y] > 0.25 * topo).collect();
    let (r0, r1) = (rows[0] as f64, *rows.last().unwrap() as f64);
    let (meio, hw) = (0.5 * (r0 + r1), 0.5 * (r1 - r0 + 1.0));
    let mut nivel: Vec<f64> = (xa..xb)
        .map(|x| (0..H).map(|y| c[y * W + x]).fold(0.0, f64::max))
        .collect();
    nivel.sort_by(f64::total_cmp);
    let nivel = nivel[nivel.len() * 9 / 10];
    let mut curva = [0.0; 2];
    for (lado, k) in curva.iter_mut().enumerate() {
        let (mut eixo, mut bordas) = (Vec::new(), Vec::new());
        for &y in &rows {
            let v = (y as f64 - meio) / hw;
            let tem: Vec<usize> = (0..W).filter(|&x| c[y * W + x] >= 0.25 * nivel).collect();
            let (Some(&a), Some(&b)) = (tem.first(), tem.last()) else {
                continue;
            };
            let e = if lado == 0 {
                f64::from(X0) - a as f64
            } else {
                b as f64 - f64::from(X1)
            };
            if v.abs() < 0.25 {
                eixo.push(e);
            } else if (0.6..=0.8).contains(&v.abs()) {
                bordas.push(e);
            }
        }
        let med = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        *k = (med(eixo) - med(bordas)) / hw;
    }
    let tinta = (0..H)
        .flat_map(|y| (xa..xb).map(move |x| (y, x)))
        .map(|(y, x)| c[y * W + x])
        .sum();
    (curva, tinta)
}

/// ⭐ O GATE: com as pontas rectas o início e o fim do recto saem a direito (curva ≤ 0,08; o redondo ≥ 0,2), CHEIOS até
/// perto do corte (a rampa ≤ 0,35 da do redondo), o miolo pousa a tinta do redondo (± 3 %), e o `pontas_alem` leva o
/// corte além das pontas.
#[test]
fn as_pontas_rectas_cortam_o_traco_a_direito_e_pousam_a_mesma_tinta() {
    let (redondo, tinta_r) = mede(&recto(false));
    let (rectas, tinta_q) = mede(&recto(true));
    eprintln!(
        "curva ini·fim: redondo {redondo:.3?} · rectas {rectas:.3?}; tinta {tinta_r:.1} · {tinta_q:.1}"
    );
    let (q, r) = (pontas(&recto(true)), pontas(&recto(false)));
    let alem = pontas(&recto_alem(true, 0.5));
    eprintln!(
        "pontas [início, fim, rampa ini, rampa fim]: rectas {q:?} · redondo {r:?} · além 0,5 {alem:?}"
    );
    assert!(
        redondo.iter().all(|&k| k >= 0.2),
        "controlo: o redondo acaba em meia-lua ({redondo:?})"
    );
    assert!(
        rectas.iter().all(|&k| k.abs() <= 0.08),
        "as pontas rectas têm de cortar a direito: curva {rectas:?}"
    );
    let razao = tinta_q / tinta_r;
    assert!(
        (0.97..=1.03).contains(&razao),
        "o miolo pousa a tinta do redondo: {razao:.3}"
    );
    // As pontas CHEIAS até ao corte: a rampa de 10 → 90 % do miolo, ao longo do eixo.
    assert!(
        r[2] >= 40.0 && r[3] >= 40.0,
        "controlo: o redondo esmaece num raio ({r:?})"
    );
    // O Rebelle (a mesma régua no recto de 40 da bateria, `docs/Painter/Pinceis/J6_pontas_rectas.md`): 11–35 px.
    assert!(
        q[2] <= 0.35 * r[2] && q[3] <= 0.35 * r[3],
        "as pontas rectas são CHEIAS até perto do corte (rampa ≤ 0,35 da do redondo): {q:?} × {r:?}"
    );
    // `pontas_alem` 0,5 = o corte meia meia-largura (20 px: o tamanho 40 é o raio) além de cada ponta, e cheio.
    let (di, df) = (q[0] - alem[0], alem[1] - q[1]);
    assert!(
        (17.0..=23.0).contains(&di)
            && (17.0..=23.0).contains(&df)
            && alem[2] <= 0.4 * r[2]
            && alem[3] <= 0.4 * r[3],
        "o além 0,5 passa 20 px cada ponta, cheio: início {di} · fim {df} · {alem:?}"
    );
}

/// `[início, fim, rampa do início, rampa do fim]` (px) pela tinta do miolo (± 8 px do eixo), coluna a coluna: a ponta é
/// onde passa metade da do meio; a rampa, a distância de 10 % a 90 % dela.
fn pontas(c: &[f64]) -> [f64; 4] {
    let col = |x: usize| {
        (Y as usize - 8..=Y as usize + 8)
            .map(|y| c[y * W + x])
            .sum::<f64>()
            / 17.0
    };
    let meio =
        (X0 as usize + 60..X1 as usize - 60).map(col).sum::<f64>() / (X1 - X0 - 120.0) as f64;
    let primeira = |k: f64| (0..W).find(|&x| col(x) >= k * meio).unwrap_or(W) as f64;
    let ultima = |k: f64| (0..W).rev().find(|&x| col(x) >= k * meio).unwrap_or(0) as f64;
    [
        primeira(0.5),
        ultima(0.5),
        primeira(0.9) - primeira(0.1),
        ultima(0.1) - ultima(0.9),
    ]
}

/// O recto LARGO (400 px, além 1) das pontas rectas que anda 180 px e solta; com `parada` o último dab da mão chega
/// PARADO (caminho `0` desde o anterior — a mão que pára antes de levantar). Devolve os dabs do FECHO, o tempo do
/// soltar e a tela.
fn fecho_largo(parada: bool) -> (usize, std::time::Duration, Vec<u8>) {
    const LW: usize = 900;
    const LH: usize = 500;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; LW * LH * 4], LW as u32, LH as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(400.0);
    t.set_brush_color_srgb8([30, 60, 200]);
    t.toggle_brush_pontas_rectas();
    t.set_brush_pontas_alem(1.0);
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, 250.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut x = 250.0;
    t.on_canvas_pointer(p(x, PointerPhase::Down));
    for _ in 0..60 {
        x += 3.0;
        t.on_canvas_pointer(p(x, PointerPhase::Move));
        t.on_tick(8.0);
    }
    let brush = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .and_then(|s| s.pincel_do_fecho)
        .expect("o traço vivo");
    let alem = t.alem_no_motor(&brush);
    let sess = t.paint.wetpaint.session.as_mut().expect("o traço vivo");
    if parada {
        // O último dab da mão chega PARADO: o mesmo dab, com caminho `0` desde o anterior (o lote da mão).
        for l in &mut sess.lanes {
            if let Some(mut u) = l.ultimo {
                u.arc_len = l.arco;
                let _ = super::tampa::Tampa::do_dab(&u, l, super::tampa::Lote::Mao, alem);
            }
        }
    }
    let n: usize = sess
        .lanes
        .iter()
        .filter_map(|l| {
            let u = l.ultimo?;
            Some(super::tampa::quantos_do_fecho(
                l,
                &brush,
                u.radius_px.max(u.stroke_radius_px),
                alem,
            ))
        })
        .sum();
    let t0 = std::time::Instant::now();
    t.on_canvas_pointer(p(x, PointerPhase::Up));
    let dt = t0.elapsed();
    t.on_tick(1000.0 / 60.0);
    (n, dt, t.canvas_rgba.to_vec())
}

/// ⭐ O FECHO não depende de a mão PARAR antes de soltar: o passo dos dabs do fecho é o do Spacing do pincel (o dos
/// dabs do corpo), não o caminho do último dab da mão — com ele, um último dab parado pedia `⌈2,05·raio ÷ 0,5⌉`
/// dabs O(área) síncronos no soltar (raio 200: 820, medido 2,1 s × 0,33 s em `--profile smoke`). O traço da mão
/// que solta a andar é o mesmo dos dois lados, e o fecho pousa dabs (controlo).
#[test]
fn o_fecho_das_pontas_rectas_nao_depende_de_a_mao_parar() {
    let (anda, t_anda, tela_anda) = fecho_largo(false);
    if let Ok(f) = std::env::var("PH2D_TELA_FECHO") {
        std::fs::write(f, &tela_anda).unwrap();
    }
    let (para, t_para, tela_para) = fecho_largo(true);
    eprintln!(
        "fecho: mão a andar {anda} dabs em {t_anda:?} · último dab parado {para} dabs em {t_para:?}"
    );
    assert!(anda > 0, "controlo: o fecho pousa dabs");
    assert_eq!(
        anda, para,
        "o fecho não pode depender de a mão parar antes de soltar ({t_anda:?} × {t_para:?})"
    );
    assert!(
        tela_anda == tela_para,
        "o traço da mão parada é o da mão que solta a andar"
    );
}
