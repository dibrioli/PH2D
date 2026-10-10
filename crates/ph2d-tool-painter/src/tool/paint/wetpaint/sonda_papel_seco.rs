//! **Sonda (ignorada): o pincel SECO pega nos PICOS do papel?** (dono 09/10: *«pincel molhado a tinta
//! acumula mais nos vales do papel (isso está OK). Mas o pincel seco a tinta não deve ficar mais dura nos
//! picos do papel? Acho que isso não está funcionando»*). Os quatro pincéis de fábrica com a ÁGUA DELES,
//! tela direita, um traço de 400 px a 60 px, 30 s; o papel do motor e o do painel armado. Por pixel do
//! miolo com tinta: o escuro contra a altura do papel na célula (`Grid::paper`, alto = pico) — a
//! correlação e o escuro médio no quinto mais alto ÷ no quinto mais baixo (> 1 = pega nos picos).
//!
//! `cargo test -p ph2d-tool-painter --profile smoke --lib sonda_papel_seco -- --ignored --nocapture`
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: usize = 600;
const H: usize = 300;

fn cp(x: f32) -> CanvasPointer {
    CanvasPointer {
        pos: [x, 150.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase: PointerPhase::Move,
    }
}

/// **Sonda (W1e, F2): o que o checkbox «Paper» do Wet Paint acrescenta ao Relief do painel.** O Wet Round
/// no Cold do painel, Relief 0 e 0,6 × o checkbox desligado e ligado, a tela COM a luz do relevo; grava
/// as quatro telas (`PH2D_SONDA_OUT`) e imprime os texels que cada interruptor move.
#[test]
#[ignore = "sonda: o checkbox Paper do Wet Paint contra o Relief; corre-se à mão"]
fn sonda_checkbox_paper() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset;
    let mut telas = Vec::new();
    for relief in [0.0f32, 0.6] {
        for caixa in [false, true] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
            t.set_paint_tool_mode("wetpaint");
            t.set_wet_relogio_fixo(true);
            assert!(t.wet_preset_aplicar(p).is_empty());
            t.wet_preset_aplicar_volumes(&p.parametros.volumes);
            t.set_brush_size_px(30.0);
            t.set_brush_color_srgb8([23, 101, 179]);
            t.paint.wetpaint.tilt_on = false;
            t.set_brush_paper_kind(1);
            t.set_substrate_depth(relief);
            t.paint.wetpaint.paper_visual = caixa;
            t.on_canvas_pointer(CanvasPointer {
                phase: PointerPhase::Down,
                ..cp(100.0)
            });
            for k in 1..=133 {
                t.on_canvas_pointer(cp(100.0 + 3.0 * k as f32));
                t.on_tick(8.0);
            }
            t.on_canvas_pointer(CanvasPointer {
                phase: PointerPhase::Up,
                ..cp(500.0)
            });
            for _ in 0..30 * 60 {
                t.on_tick(1000.0 / 60.0);
            }
            let mut px = t.run_full().0;
            t.apply_impasto_light(
                &mut px,
                crate::Region {
                    x: 0,
                    y: 0,
                    w: W as u32,
                    h: H as u32,
                },
            );
            if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
                let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
                for k in 0..W * H {
                    ppm.extend_from_slice(&px[k * 4..k * 4 + 3]);
                }
                std::fs::write(format!("{dir}/caixa_r{relief}_c{caixa}.ppm"), ppm).expect("grava");
            }
            telas.push(((relief, caixa), px));
        }
    }
    let dif = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count();
    let (r0c0, r0c1, r6c0, r6c1) = (&telas[0].1, &telas[1].1, &telas[2].1, &telas[3].1);
    println!(
        "[caixa] o checkbox move {} texels sem Relief e {} com Relief 0,6 · o Relief move {} sem o checkbox",
        dif(r0c0, r0c1),
        dif(r6c0, r6c1),
        dif(r0c0, r6c0)
    );
}

#[test]
#[ignore = "sonda: o pincel seco nos picos do papel; corre-se à mão"]
fn sonda_papel_seco() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    for nome in ["Wet Round", "Wet Wash", "Wet Dry Brush", "Wet Drip"] {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .expect("o pincel de fábrica")
            .preset;
        for armado in [false, true] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
            t.set_paint_tool_mode("wetpaint");
            t.set_wet_relogio_fixo(true);
            assert!(t.wet_preset_aplicar(p).is_empty());
            t.wet_preset_aplicar_volumes(&p.parametros.volumes);
            t.set_brush_size_px(30.0);
            t.set_brush_color_srgb8([23, 101, 179]);
            t.paint.wetpaint.tilt_on = false;
            if armado {
                t.set_brush_paper_kind(1);
            }
            t.on_canvas_pointer(CanvasPointer {
                phase: PointerPhase::Down,
                ..cp(100.0)
            });
            for k in 1..=133 {
                t.on_canvas_pointer(cp(100.0 + 3.0 * k as f32));
                t.on_tick(8.0);
            }
            t.on_canvas_pointer(CanvasPointer {
                phase: PointerPhase::Up,
                ..cp(500.0)
            });
            let (papel, gw) = {
                let s = t.paint.wetpaint.session.as_mut().expect("sessão viva");
                s.bring_home();
                let g = s.engine.active_grid();
                (g.paper.clone(), g.w)
            };
            // O plano tem o anel de 1 célula: (w+2)·(h+2), célula (x+1, y+1) = pixel (x, y).
            assert_eq!(gw, W, "razão 1");
            assert_eq!(papel.len(), (W + 2) * (H + 2), "o anel de 1 célula");
            let gw = W + 2;
            for _ in 0..30 * 60 {
                t.on_tick(1000.0 / 60.0);
            }
            let px = t.run_full().0;
            if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
                let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
                for k in 0..W * H {
                    ppm.extend_from_slice(&px[k * 4..k * 4 + 3]);
                }
                let painel = if armado { "painel" } else { "fabrica" };
                std::fs::write(format!("{dir}/papel_{}_{painel}.ppm", nome.replace(' ', "_")), ppm)
                    .expect("grava a tela");
            }
            let mut pares: Vec<(f64, f64)> = Vec::new();
            for y in 135..165 {
                for x in 140..460 {
                    let i = (y * W + x) * 4;
                    let e = f64::from(255 - px[i..i + 3].iter().copied().min().unwrap_or(255));
                    if e > 25.0 {
                        pares.push((e, f64::from(papel[(y + 1) * gw + x + 1])));
                    }
                }
            }
            let n = pares.len() as f64;
            let (me, mh) = pares
                .iter()
                .fold((0.0, 0.0), |(a, b), p| (a + p.0 / n, b + p.1 / n));
            let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
            for (e, h) in &pares {
                sxy += (e - me) * (h - mh);
                sxx += (e - me) * (e - me);
                syy += (h - mh) * (h - mh);
            }
            let r = sxy / (sxx * syy).sqrt().max(1e-12);
            let mut por_h = pares.clone();
            por_h.sort_by(|a, b| a.1.total_cmp(&b.1));
            let q = por_h.len() / 5;
            let media =
                |s: &[(f64, f64)]| s.iter().map(|p| p.0).sum::<f64>() / s.len().max(1) as f64;
            let (vales, picos) = (media(&por_h[..q]), media(&por_h[por_h.len() - q..]));
            println!(
                "[papel] {nome:<14} papel {:<7} água {:?} | px com tinta {:5.1} % | r(escuro, altura) {:+.3} | picos {:5.1} ÷ vales {:5.1} = {:.2}",
                if armado { "painel" } else { "fábrica" },
                p.parametros.volumes.water,
                100.0 * n / (30.0 * 320.0),
                r,
                picos,
                vales,
                picos / vales.max(1e-9)
            );
        }
    }
}
