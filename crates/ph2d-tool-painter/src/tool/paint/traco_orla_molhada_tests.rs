//! ⭐ **A orla molhada é o *Wet Edges* do Procreate MEDIDO** (bug #58, 5.ª volta — o dono 24/10: *«deve ficar como o
//! procreate, mas atualmente está desfocando as bordas»*). O par do Procreate (Intense Glaze, Flow Max, Wet 0 × Max)
//! mostra: a silhueta igual (a rampa um pouco mais dura), o miolo PLANO a ~0,92 da cobertura sem Wet, o grão do miolo
//! ÷3, as sobreposições sem escurecer, nenhuma orla. A 4.ª volta (o borrão gaussiano da borda) codificava o
//! contrário: os gates dela (`o_wet_suaviza_a_borda`, `o_wet_nao_clareia_o_miolo`) saíram com ela.
//!
//! Tudo na tela TRANSPARENTE: o alfa da tela é a cobertura composta.

use super::traco_orla_bordas_tests::{H, W};
use super::traco_orla_lei_tests::{caminho, cp, distancias, pinta, traco, z};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

/// O pincel Digital por omissão na tela transparente, 40 px, rosa, o tecto inerte (arma o plano), Wet `m`.
fn monta(m: f32, f: impl FnOnce(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(crate::PaintMedia::Digital);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([230, 90, 180]);
    t.set_brush_strength(0.6);
    t.set_traco_tecto(1.0);
    t.set_traco_molhada(m);
    f(&mut t);
    t
}

/// O alfa da tela em `(x, y)` sub-píxel (bilinear).
fn alfa(px: &[u8], x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let a = |x: usize, y: usize| f32::from(px[(y * W + x) * 4 + 3]) / 255.0;
    let top = a(x0, y0) * (1.0 - fx) + a(x0 + 1, y0) * fx;
    let bot = a(x0, y0 + 1) * (1.0 - fx) + a(x0 + 1, y0 + 1) * fx;
    top * (1.0 - fy) + bot * fy
}

/// Um recto a `graus` com a dureza 0,9 e Wet `m`: `(posição do contorno τ, subida 10–90 %, alfa no eixo)`, a média
/// de 5 cortes ao longo do traço e dos dois lados (px a partir do eixo).
fn borda(m: f32, graus: f32, tau: Option<f32>) -> (f32, f32, f32, f32) {
    let mut t = monta(m, |t| t.paint.brush.hardness = 0.9);
    let (s, c) = graus.to_radians().sin_cos();
    let (cx, cy) = (W as f32 / 2.0, H as f32 / 2.0);
    let pts = caminho(&[
        [cx - 120.0 * c, cy - 100.0 * s],
        [cx + 120.0 * c, cy + 100.0 * s],
    ]);
    pinta(&mut t, &pts, |_| 4);
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
    let px = t.canvas_rgba.to_vec();
    // O contorno `τ` é ½ do nível de uma passagem: o máximo do traço SECO (o recto não se sobrepõe).
    let tau = tau
        .unwrap_or_else(|| 0.5 * f32::from(px.chunks(4).map(|p| p[3]).max().unwrap_or(0)) / 255.0);
    let (mut contorno, mut rampa, mut eixo, mut n) = (0.0, 0.0, 0.0, 0.0);
    for ao_longo in [-40.0f32, -20.0, 0.0, 20.0, 40.0] {
        for lado in [-1.0f32, 1.0] {
            let (ox, oy) = (cx + ao_longo * c, cy + ao_longo * s);
            let (nx, ny) = (-s * lado, c * lado);
            let f = |d: f32| alfa(&px, ox + d * nx, oy + d * ny);
            let topo = f(0.0);
            let cruza = |nivel: f32| {
                (0..1600)
                    .map(|k| k as f32 * 0.05)
                    .find(|&d| f(d) < nivel)
                    .expect("a borda")
            };
            contorno += cruza(tau);
            rampa += cruza(0.1 * topo) - cruza(0.9 * topo);
            eixo += topo;
            n += 1.0;
        }
    }
    (contorno / n, rampa / n, eixo / n, tau)
}

/// ⭐ **(a) O Wet não mexe na silhueta**: a 0°, 17° e 45° o contorno `τ` fica no sítio (±0,3 px) e a subida 10–90 %
/// não alarga (≤ Wet 0 + 0,3 px); no Z nada se pinta onde o Wet 0 não pinta.
#[test]
fn o_wet_nao_mexe_na_silhueta() {
    for g in [0.0f32, 17.0, 45.0] {
        let (c0, r0, _, tau) = borda(0.0, g, None);
        let (c1, r1, _, _) = borda(1.0, g, Some(tau));
        eprintln!("{g}°: contorno {c0:.2} → {c1:.2} px · subida 10–90 % {r0:.2} → {r1:.2} px");
        assert!((c1 - c0).abs() <= 0.3, "{g}°: o contorno mexeu {c0} → {c1}");
        assert!(r1 <= r0 + 0.3, "{g}°: a borda alargou {r0} → {r1}");
    }
    let (mut seco, mut molhado) = (monta(0.0, |_| {}), monta(1.0, |_| {}));
    let (a, b) = (traco(&mut seco, &z()), traco(&mut molhado, &z()));
    let fora = (0..W * H)
        .filter(|&i| a.tela[i * 4 + 3] == 0 && b.tela[i * 4 + 3] != 0)
        .count();
    assert_eq!(fora, 0, "o Wet 1 pintou {fora} texels fora do traço seco");
}

/// O desvio-padrão do alfa da tela nos texels de `idx`.
fn desvio(tela: &[u8], idx: &[usize]) -> f32 {
    let a: Vec<f32> = idx.iter().map(|&i| f32::from(tela[i * 4 + 3])).collect();
    let m = a.iter().sum::<f32>() / a.len() as f32;
    (a.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / a.len() as f32).sqrt()
}

/// ⭐ **(b) O miolo fica plano**: com o grão (Noise) no Z o desvio-padrão do alfa do miolo cai ×2 a ×8 (o grão
/// fica, ~⅓); com o Flow
/// 0,3 (as sobreposições sobem) os texels da sobreposição ficam a ≤ 2/255 dos de uma passagem.
#[test]
fn o_wet_aplana_o_miolo_e_a_sobreposicao() {
    // O grão fino e raso (Noise, Size 0,05, Depth 0,35): no miolo de uma passagem ele modula a cobertura em
    // ~±10 % — o grão do Intense Glaze do Procreate (desvio 16–20 em ~180).
    let grao = |t: &mut PainterTool| {
        t.new_brush_texture();
        t.set_brush_texture_size_norm(0, 0.05);
        t.set_brush_texture_size_norm(1, 0.05);
        t.set_brush_grain_depth(0.35);
    };
    let (mut seco, mut molhado) = (monta(0.0, grao), monta(1.0, grao));
    let (a, b) = (traco(&mut seco, &z()), traco(&mut molhado, &z()));
    let d = distancias(&a.v, b.tau, 12);
    let fundo: Vec<usize> = (0..W * H).filter(|&i| d[i] > 8.0).collect();
    assert!(fundo.len() > 2_000, "o Z não tem miolo ({})", fundo.len());
    let (s0, s1) = (desvio(&a.tela, &fundo), desvio(&b.tela, &fundo));
    eprintln!("grão no miolo: desvio {s0:.2} → {s1:.2} /255");
    assert!(s0 > 4.0, "o pincel não tem grão no miolo ({s0})");
    assert!(
        s1 * 2.0 <= s0 && s1 * 8.0 >= s0,
        "o grão do miolo não caiu a ~⅓ (o Procreate: 16–20 → 4–9): {s0} → {s1}"
    );

    let flow = |t: &mut PainterTool| t.set_brush_flow(0.3);
    let (mut seco, mut molhado) = (monta(0.0, flow), monta(1.0, flow));
    let (a, b) = (traco(&mut seco, &z()), traco(&mut molhado, &z()));
    let nivel = 2.0 * b.tau;
    let d = distancias(&a.v, b.tau, 12);
    let (uma, sobre): (Vec<usize>, Vec<usize>) = (0..W * H)
        .filter(|&i| d[i] > 8.0)
        .filter(|&i| (a.v[i] - nivel).abs() < 0.02 || a.v[i] > 1.1 * nivel)
        .partition(|&i| a.v[i] <= nivel + 0.02);
    assert!(
        uma.len() > 500 && sobre.len() > 200,
        "a fixtura não tem sobreposição ({} uma passagem, {} sobre)",
        uma.len(),
        sobre.len()
    );
    let media = |idx: &[usize], tela: &[u8]| {
        idx.iter().map(|&i| f32::from(tela[i * 4 + 3])).sum::<f32>() / idx.len() as f32
    };
    let (m_uma, m_sobre) = (media(&uma, &b.tela), media(&sobre, &b.tela));
    let pior = sobre
        .iter()
        .map(|&i| (f32::from(b.tela[i * 4 + 3]) - m_uma).abs())
        .fold(0.0f32, f32::max);
    eprintln!(
        "Wet 1: uma passagem {m_uma:.1} · sobreposição {m_sobre:.1} (seco {:.1}) · pior {pior:.1} /255",
        media(&sobre, &a.tela)
    );
    assert!(
        pior <= 2.0,
        "a sobreposição escurece {pior}/255 sobre uma passagem"
    );
}

/// ⭐ **(c) O patamar**: no eixo de um recto o alfa com Wet 1 é `0,92 ± 0,03` do de Wet 0, e desce monótono com o Wet.
#[test]
fn o_patamar_e_o_do_procreate() {
    let eixo: Vec<f32> = [0.0f32, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|&m| borda(m, 0.0, None).2)
        .collect();
    eprintln!("alfa no eixo por Wet 0 · ¼ · ½ · ¾ · 1: {eixo:?}");
    let r = eixo[4] / eixo[0];
    assert!((r - 0.92).abs() <= 0.03, "o patamar é {r} do Wet 0");
    assert!(
        eixo.windows(2).all(|p| p[1] <= p[0] + 0.5 / 255.0),
        "o patamar não desce monótono: {eixo:?}"
    );
}

/// ⛔ Sonda + fotos (`PH2D_FOTO_DIR`): o par do Procreate refeito — rosa sobre cinza 35, o zigue-zague de ~24 px com
/// grão, Wet 0 e Wet 1 (`ours_wet0.png`, `ours_wet1.png`, ×3) — e o perfil do vermelho da borda para dentro.
#[test]
#[ignore = "sonda: escreve PNG em PH2D_FOTO_DIR"]
fn foto_do_par_do_procreate() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let nos = [
        [30.0, 225.0],
        [110.0, 15.0],
        [150.0, 205.0],
        [255.0, 25.0],
        [295.0, 225.0],
    ];
    for m in [0.0f32, 1.0] {
        let mut t = PainterTool::default();
        t.set_source([35, 35, 35, 255].repeat(W * H), W as u32, H as u32);
        t.set_paint_media(crate::PaintMedia::Digital);
        t.set_brush_size_px(12.0);
        t.set_brush_color_srgb8([230, 90, 180]);
        t.set_brush_strength(0.8);
        t.paint.brush.hardness = 0.9;
        t.new_brush_texture();
        t.set_brush_texture_size_norm(0, 0.05);
        t.set_brush_texture_size_norm(1, 0.05);
        t.set_brush_grain_depth(0.35);
        t.set_traco_tecto(1.0);
        t.set_traco_molhada(m);
        let pts = caminho(&nos);
        pinta(&mut t, &pts, |_| 4);
        let s = t.paint.plano_do_traco.as_ref().expect("o plano");
        let vmax = s.plano.chunks(4).map(|p| p[3]).max().unwrap_or(0);
        eprintln!(
            "Wet {m}: nível de uma passagem {:.3} · máximo do plano {:.3}",
            2.0 * s.fronteira(),
            f32::from(vmax) / 255.0
        );
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
        let px = t.canvas_rgba.to_vec();
        let red = |x: f32, y: f32| {
            let (x0, y0) = (x.floor() as usize, y.floor() as usize);
            let (fx, fy) = (x - x0 as f32, y - y0 as f32);
            let r = |x: usize, y: usize| f32::from(px[(y * W + x) * 4]);
            (r(x0, y0) * (1.0 - fx) + r(x0 + 1, y0) * fx) * (1.0 - fy)
                + (r(x0, y0 + 1) * (1.0 - fx) + r(x0 + 1, y0 + 1) * fx) * fy
        };
        // O perfil: na 1.ª perna, dos dois lados, a partir do 1.º píxel acima de 40 vindo de fora.
        let (a, b) = (nos[0], nos[1]);
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        let (ux, uy) = ((b[0] - a[0]) / len, (b[1] - a[1]) / len);
        let mut perfil = [0.0f32; 10];
        let mut n = 0.0;
        for s in [0.3f32, 0.4, 0.5, 0.6, 0.7] {
            for lado in [-1.0f32, 1.0] {
                let (cx, cy) = (a[0] + ux * len * s, a[1] + uy * len * s);
                let (nx, ny) = (-uy * lado, ux * lado);
                let borda = (0..400)
                    .map(|k| 20.0 - k as f32 * 0.05)
                    .find(|&d| red(cx + d * nx, cy + d * ny) > 40.0)
                    .expect("a borda");
                for (k, p) in perfil.iter_mut().enumerate() {
                    let d = borda - k as f32;
                    *p += red(cx + d * nx, cy + d * ny);
                }
                n += 1.0;
            }
        }
        let perfil: Vec<u8> = perfil.iter().map(|p| (p / n).round() as u8).collect();
        eprintln!("Wet {m}: vermelho da borda para dentro (d = 0..9) {perfil:?}");
        let z = 3;
        let mut c = super::tests::look_watercolor_arc::Canvas::new(W * z, H * z);
        for y in 0..H * z {
            for x in 0..W * z {
                let i = ((y / z) * W + x / z) * 4;
                c.set(x, y, [px[i], px[i + 1], px[i + 2]]);
            }
        }
        let p = std::path::Path::new(&dir).join(format!("ours_wet{}.png", m as u8));
        super::tests::look_watercolor_arc::write_png(&p, &c).expect("PNG");
    }
}
