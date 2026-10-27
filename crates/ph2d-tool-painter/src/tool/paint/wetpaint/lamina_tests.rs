//! ⭐ **A lâmina da faca no produto** (Óleo Real W2 E4, `ph2d_wet_paint::lamina`): a lâmina chata atravessada que
//! SEGUE o traço (`dabRotation` 1) varre a mesma largura nos dois braços de um L, a FIXA (0) não; o rebordo
//! (`knifeRidge`) põe a crista nas bordas laterais dos dois braços e, a `0`, nem aloca o campo da altura.

use crate::{PaintMedia, PainterTool};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

const W: u32 = 220;
const H: u32 = 220;

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A lâmina H3 (W0b): quadrada, lisa, dura, sem dente, achatada 0,8 a 90° (o fio atravessado à marcha).
fn faca(segue: bool, rebordo: f64) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.paint.brush.jitter = 0.0;
    t.set_brush_hardness(1.0);
    t.set_brush_falloff(ph2d_painter_brush::Falloff::Constant as u8);
    t.set_brush_shape_kind(ph2d_painter_brush::TextureKind::None.to_u8());
    t.set_brush_dab_quina(1.0);
    t.set_brush_dab_flatten(0.8);
    t.set_brush_dab_angle(90.0);
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([20, 150, 230]);
    let k = &mut t.paint.wetpaint.knobs;
    k.set(Knob::PaperGate, 0.0);
    k.set(Knob::Leveling, 0.0);
    k.set(Knob::DabRotation, if segue { 1.0 } else { 0.0 });
    k.set(Knob::KnifeRidge, rebordo);
    t
}

/// O L: horizontal de (30, 50) a (170, 50), depois vertical até (170, 190), num só traço.
fn traco_em_l(t: &mut PainterTool) {
    t.on_canvas_pointer(ponto([30.0, 50.0], PointerPhase::Down));
    let mut p = [30.0f32, 50.0];
    while p[0] < 170.0 {
        p[0] += 2.0;
        t.on_canvas_pointer(ponto(p, PointerPhase::Move));
        t.on_tick(8.0);
    }
    while p[1] < 190.0 {
        p[1] += 2.0;
        t.on_canvas_pointer(ponto(p, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(p, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
}

fn pintado(t: &PainterTool, x: u32, y: u32) -> bool {
    let i = ((x + y * W) * 4) as usize;
    let p = &t.canvas_rgba[i..i + 3];
    p.iter().map(|&v| 255 - i32::from(v)).sum::<i32>() > 40
}

/// As larguras ATRAVÉS de cada braço: a coluna `x = 100` do horizontal e a linha `y = 120` do vertical.
fn larguras(t: &PainterTool) -> (u32, u32) {
    let h = (0..H).filter(|&y| pintado(t, 100, y)).count() as u32;
    let v = (0..W).filter(|&x| pintado(t, x, 120)).count() as u32;
    (h, v)
}

#[test]
fn a_lamina_que_segue_o_traco_varre_a_mesma_largura_nos_dois_bracos_do_l() {
    let mut segue = faca(true, 0.0);
    traco_em_l(&mut segue);
    let (h, v) = larguras(&segue);
    assert!(
        h >= 36,
        "o braço horizontal da seguidora cobre a lâmina inteira: {h}"
    );
    assert!(h.abs_diff(v) <= 2, "seguidora: {h} × {v}");
    // A fixa (a 90° na tela) é larga no horizontal e fina no vertical: a rotação é do traço, não do acaso.
    let mut fixa = faca(false, 0.0);
    traco_em_l(&mut fixa);
    let (fh, fv) = larguras(&fixa);
    assert!(fh >= 36 && fv * 2 < fh, "fixa: {fh} × {fv}");
}

#[test]
fn o_rebordo_poe_a_crista_nas_bordas_dos_dois_bracos_e_a_zero_nao_existe() {
    let mut sem = faca(true, 0.0);
    traco_em_l(&mut sem);
    let s = sem.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    assert!(
        s.engine.active_grid().altura.is_empty(),
        "rebordo 0 alocou a altura"
    );

    let mut com = faca(true, 3.0);
    traco_em_l(&mut com);
    let s = com.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let ratio = f64::from(s.ratio);
    let g = s.engine.active_grid();
    let alt = |x: f64, y: f64| -> f32 {
        let (cx, cy) = (
            (x / ratio).floor() as usize + 1,
            (y / ratio).floor() as usize + 1,
        );
        ph2d_wet_paint::grid::altura::em(g, cx + cy * g.s)
    };
    // A crista: o máximo da altura num corte através de cada braço fica perto da borda, e o miolo é liso.
    let corte_h: Vec<f32> = (0..100).map(|y| alt(100.0, f64::from(y))).collect();
    let corte_v: Vec<f32> = (120..220).map(|x| alt(f64::from(x), 120.0)).collect();
    let mut cristas = Vec::new();
    for (nome, corte, centro) in [
        ("horizontal", &corte_h, 50usize),
        ("vertical", &corte_v, 50),
    ] {
        let max = corte.iter().copied().fold(0.0f32, f32::max);
        let k = corte.iter().position(|&v| v == max).expect("máximo");
        assert!(max > 0.5, "{nome}: crista {max}");
        assert!(
            k.abs_diff(centro) >= 15,
            "{nome}: a crista em {k}, longe da borda (centro {centro})"
        );
        assert!(
            corte[centro].abs() < 0.05 * max,
            "{nome}: miolo {}",
            corte[centro]
        );
        cristas.push((max, k.abs_diff(centro)));
    }
    // O mesmo rebordo nos dois braços: a altura e a distância ao eixo.
    let ([a, b], [da, db]) = ([cristas[0].0, cristas[1].0], [cristas[0].1, cristas[1].1]);
    assert!(
        (a - b).abs() < 0.05 * a && da.abs_diff(db) <= 1,
        "braços: {cristas:?}"
    );
}

/// ⭐ **A faca SÓ com `knifeLevel` nivela** (W3 K5): a lâmina corria só com rebordo, então uma faca com o nível e sem
/// rebordo pintava como sem nível. O desvio da espessura no miolo do braço horizontal cai com o nível; e o nível `0`
/// escrito é a faca de sempre, ao byte (tela e espessura).
#[test]
fn a_faca_so_com_o_nivel_nivela() {
    let corre = |nivel: Option<f64>| {
        let mut t = faca(true, 0.0);
        if let Some(n) = nivel {
            t.paint.wetpaint.knobs.set(Knob::KnifeLevel, n);
        }
        // Um toque longe abre a sessão; depois, SULCOS de tinta em pé (1000/4000 por linha) sob o braço horizontal.
        t.on_canvas_pointer(ponto([200.0, 200.0], PointerPhase::Down));
        t.on_canvas_pointer(ponto([200.0, 200.0], PointerPhase::Up));
        {
            let s = t.paint.wetpaint.session.as_mut().expect("sessão");
            s.bring_home();
            let r = usize::from(s.ratio);
            let g = s.engine.active_grid_mut();
            for y in 36 / r..64 / r {
                for x in 1..=g.w {
                    g.sett[x + y * g.s] = if y % 2 == 0 { 1000.0 } else { 4000.0 };
                }
            }
        }
        traco_em_l(&mut t);
        let tela = t.canvas_rgba.clone();
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        let ratio = f64::from(s.ratio);
        let g = s.engine.active_grid();
        let esp: Vec<f64> = (70..150)
            .flat_map(|x| (44..57).map(move |y| (x, y)))
            .map(|(x, y): (i32, i32)| {
                let (cx, cy) = (
                    (f64::from(x) / ratio).floor() as usize + 1,
                    (f64::from(y) / ratio).floor() as usize + 1,
                );
                ph2d_wet_paint::grid::espessura::em(g, cx + cy * g.s)
            })
            .collect();
        (tela, esp)
    };
    let desvio = |v: &[f64]| {
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt() / m.max(1e-9)
    };
    let (tela0, esp0) = corre(None);
    assert_eq!(corre(Some(0.0)), (tela0, esp0.clone()), "nível 0 = a faca de sempre, ao byte");
    let (_, esp1) = corre(Some(1.0));
    let (d0, d1) = (desvio(&esp0), desvio(&esp1));
    assert!(d1 < 0.5 * d0, "o desvio da espessura no miolo: sem nível {d0:.4}, com nível 1 {d1:.4}");
}

/// O Óleo Real de fábrica com os `campo=valor,…` de `af` (os campos que o rebordo mede).
fn oleo_de(af: &str) -> ph2d_wet_paint::oleo_real::OleoReal {
    let mut o = ph2d_wet_paint::oleo_real::OleoReal::default();
    for par in af.split(',').filter(|p| !p.is_empty()) {
        let (k, v) = par
            .split_once('=')
            .unwrap_or_else(|| panic!("{par}: campo=valor"));
        let v: f64 = v.parse().unwrap_or_else(|_| panic!("{par}: número"));
        match k {
            "forca_ambiente" => o.forca_ambiente = v,
            "raio_detalhe" => o.af.raio_detalhe = v as usize,
            "fios" => o.af.fios = v,
            "altitude" => o.altitude = v.to_radians(),
            "azimute" => o.azimute = v.to_radians(),
            outro => panic!("{outro}: campo desconhecido"),
        }
    }
    o
}

/// O contraste do rebordo no braço horizontal do L de cima (tela `y = 70`, `x` 120..300), como a régua da E4 o lê
/// na folha F42: por coluna, a luminância (sRGB) mínima no terço de cima da tinta e a máxima no de baixo, ÷ a
/// mediana do miolo; devolve as medianas das colunas.
fn contraste_do_rebordo(rgb: &[u8], px: &[u8], gw: usize, ratio: usize) -> (f64, f64) {
    let lum = |k: usize| {
        0.2126 * f64::from(rgb[3 * k])
            + 0.7152 * f64::from(rgb[3 * k + 1])
            + 0.0722 * f64::from(rgb[3 * k + 2])
    };
    let mediana = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let (mut topos, mut baixos) = (Vec::new(), Vec::new());
    for x in 120 / ratio..300 / ratio {
        let linhas: Vec<usize> = (10 / ratio..140 / ratio)
            .filter(|&y| px[4 * (x + y * gw) + 3] > 128)
            .collect();
        if linhas.len() < 9 {
            continue;
        }
        let n = linhas.len();
        let l: Vec<f64> = linhas.iter().map(|&y| lum(x + y * gw)).collect();
        let miolo = mediana(l[n / 4..3 * n / 4].to_vec());
        topos.push(l[..n / 3].iter().copied().fold(f64::INFINITY, f64::min) / miolo);
        baixos.push(l[2 * n / 3..].iter().copied().fold(0.0, f64::max) / miolo);
    }
    (mediana(topos), mediana(baixos))
}

/// **A prova visual da E4** (`docs/Painter/49_realshader/out/E4_lamina.jpg`): o L e uma curva com a lâmina que segue
/// o traço, em cima COM rebordo e em baixo sem, no Óleo Real de fábrica, em `$PH2D_E4_DIR/lamina.ppm`. Imprime a
/// largura varrida. Corre-se à mão:
/// `PH2D_E4_DIR=<pasta> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_e4_lamina -- --ignored --nocapture`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_e4_lamina_folha() {
    let dir = std::path::PathBuf::from(std::env::var("PH2D_E4_DIR").expect("PH2D_E4_DIR=<pasta>"));
    let raio: f32 = std::env::var("PH2D_E4_RAIO").map_or(28.0, |v| v.parse().expect("raio"));
    let rebordo: f64 =
        std::env::var("PH2D_E4_REBORDO").map_or(3.0, |v| v.parse().expect("rebordo"));
    let (w, h) = (900u32, 760u32);
    let mut t = faca(true, rebordo);
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_brush_size_px(raio);
    // Sem água: a tinta da faca não escorre (a lei da água do óleo é dos valores das facas, depois).
    t.paint.wetpaint.knobs.water = 0.0;
    let arrasta = |t: &mut PainterTool, pts: &[[f32; 2]]| {
        t.on_canvas_pointer(ponto(pts[0], PointerPhase::Down));
        for par in pts.windows(2) {
            let n = ((par[1][0] - par[0][0]).hypot(par[1][1] - par[0][1]) / 2.0).ceil() as u32;
            for k in 1..=n {
                let f = k as f32 / n as f32;
                let p = [
                    par[0][0] + (par[1][0] - par[0][0]) * f,
                    par[0][1] + (par[1][1] - par[0][1]) * f,
                ];
                t.on_canvas_pointer(ponto(p, PointerPhase::Move));
                t.on_tick(8.0);
            }
        }
        t.on_canvas_pointer(ponto(*pts.last().expect("ponto"), PointerPhase::Up));
        for _ in 0..60 {
            t.on_tick(1000.0 / 60.0);
        }
    };
    let curva = |cx: f32, cy: f32| -> Vec<[f32; 2]> {
        (0..=40)
            .map(|k| {
                let a = std::f32::consts::PI * (1.0 + k as f32 / 40.0);
                [cx + 130.0 * a.cos(), cy + 130.0 * a.sin()]
            })
            .collect()
    };
    for (y0, r) in [(0.0f32, rebordo), (380.0, 0.0)] {
        t.paint.wetpaint.knobs.set(Knob::KnifeRidge, r);
        arrasta(
            &mut t,
            &[[60.0, y0 + 70.0], [330.0, y0 + 70.0], [330.0, y0 + 340.0]],
        );
        let c = curva(640.0, y0 + 230.0);
        arrasta(&mut t, &c);
    }
    let largura = (0..h).filter(|&y| {
        let i = ((200 + y * w) * 4) as usize;
        t.canvas_rgba[i..i + 3]
            .iter()
            .map(|&v| 255 - i32::from(v))
            .sum::<i32>()
            > 40
    });
    println!(
        "[e4] raio {raio} rebordo {rebordo}: largura varrida no horizontal {} px",
        largura.count() / 2
    );
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let (gw, gh) = s.grid;
    let params = s.engine.sim.gather_params(&s.engine.tuning);
    let camadas: Vec<ph2d_wet_paint::render::RenderLayer<'_>> = s
        .engine
        .layers
        .iter()
        .map(|l| ph2d_wet_paint::render::RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let grades: Vec<(&ph2d_wet_paint::grid::Grid, bool)> =
        camadas.iter().map(|c| (c.grid, c.visible)).collect();
    let visual = ph2d_wet_paint::render::PigmentVisual {
        paper: false,
        relevo: true,
    };
    std::fs::create_dir_all(&dir).expect("pasta");
    // `PH2D_OLEO_AF="nome:campo=v,…;nome2:…"` — as variantes do render sobre a MESMA tinta (a vazia = a de fábrica).
    let variantes = std::env::var("PH2D_OLEO_AF").unwrap_or_default();
    let ratio = usize::from(s.ratio);
    for v in std::iter::once("").chain(variantes.split(';').filter(|v| !v.is_empty())) {
        let (nome, af) = v.split_once(':').unwrap_or((v, ""));
        let o = oleo_de(af);
        let campo = ph2d_wet_paint::oleo_real::CampoAltura::de(&grades, &o, ratio, 1, 1, gw, gh);
        let mut px = vec![0u8; gw * gh * 4];
        for cy in 1..=gh {
            let linha = &mut px[(cy - 1) * gw * 4..cy * gw * 4];
            ph2d_wet_paint::render::render_pigment_row_oleo(
                Some(&params),
                &camadas,
                visual,
                &o,
                &campo,
                cy,
                1,
                gw,
                linha,
            );
        }
        let mut rgb = Vec::with_capacity(gw * gh * 3);
        for (k, p) in px.as_chunks::<4>().0.iter().enumerate() {
            let a = u32::from(p[3]);
            let sombra =
                ph2d_wet_paint::oleo_real::sombra_no_papel(&o, &campo, k % gw + 1, k / gw + 1);
            let papel = 255 - u32::from(ph2d_wet_paint::oleo_real::sombra_em_u8(sombra));
            for &c in &p[..3] {
                rgb.push(((u32::from(c) * a + papel * (255 - a)) / 255) as u8);
            }
        }
        let (topo, baixo) = contraste_do_rebordo(&rgb, &px, gw, ratio);
        println!(
            "[e4] {:<12} topo {topo:.2} baixo {baixo:.2}  (Rebelle 0,40 / 1,39)",
            if nome.is_empty() { "fabrica" } else { nome }
        );
        let mut ppm = format!("P6\n{gw} {gh}\n255\n").into_bytes();
        ppm.extend_from_slice(&rgb);
        let f = if nome.is_empty() {
            "lamina.ppm".to_owned()
        } else {
            format!("lamina_{nome}.ppm")
        };
        std::fs::write(dir.join(f), ppm).expect("grava");
    }
    // A altura do rebordo (cinza: 0 = 128, ±4 px de relevo = ±127), para ver onde a crista ficou.
    let g = s.engine.active_grid();
    let mut pgm = format!("P5\n{gw} {gh}\n255\n").into_bytes();
    for cy in 1..=gh {
        for cx in 1..=gw {
            let v = ph2d_wet_paint::grid::altura::em(g, cx + cy * g.s) * f32::from(s.ratio);
            pgm.push((128.0 + v * 32.0).clamp(0.0, 255.0) as u8);
        }
    }
    std::fs::write(dir.join("altura.pgm"), pgm).expect("grava");
}
