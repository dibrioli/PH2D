//! **Medição (ignorada; dono 10/10: «o preview não escorre como o brush real — tem que ser exato»)** —
//! quanto tempo a água de um pincel de fábrica leva até PARAR (o motor sem fluido) e quanto custa
//! simulá-lo, por tamanho de tela do preview a 1:1 (o pincel no tamanho real). Medição, nunca barra:
//! `cargo test -p ph2d-tool-painter --profile smoke --lib medida_o_preview -- --ignored --nocapture`.
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use std::time::Instant;

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_o_preview_ate_a_agua_parar() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    for nome in ["Wet Round", "Wet Wash"] {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .expect("o pincel de fábrica")
            .preset;
        for (w, h) in [
            (256u32, 256u32),
            (512, 256),
            (640, 320),
            (768, 384),
            (1024, 512),
        ] {
            for anel in [4u8, 8] {
                let mut t = PainterTool::default();
                t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
                t.set_paint_media(PaintMedia::WetPaint);
                t.set_wet_relogio_fixo(true);
                assert!(t.wet_preset_aplicar(p).is_empty());
                t.set_brush_size_px(30.0); // 60 px de diâmetro, a captura do dono
                t.paint.wetpaint.knobs.water = 1.0;
                t.paint.wetpaint.tilt_ring = anel;
                let (x0, x1) = (60.0, w as f32 - 60.0);
                let (cy, amp) = (h as f32 * 0.3, 45.0f32.min(h as f32 * 0.15));
                let n = ((x1 - x0) / 3.0) as u32;
                let em = |f: f32| {
                    [
                        x0 + (x1 - x0) * f,
                        cy + amp * (f * std::f32::consts::TAU).sin(),
                    ]
                };
                let t0 = Instant::now();
                t.on_canvas_pointer(ponto(em(0.0), PointerPhase::Down));
                for k in 1..=n {
                    t.on_canvas_pointer(ponto(em(k as f32 / n as f32), PointerPhase::Move));
                    t.on_tick(8.0);
                }
                t.on_canvas_pointer(ponto(em(1.0), PointerPhase::Up));
                let mut ticks = 0u32;
                let parou = loop {
                    t.on_tick(1000.0 / 60.0);
                    ticks += 1;
                    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
                    s.bring_home();
                    if !s.engine.active_grid().has_fluid {
                        break true;
                    }
                    if ticks >= 60 * 300 {
                        break false;
                    }
                };
                let custo = t0.elapsed().as_secs_f64();
                let px = t.canvas_rgba.as_ref();
                let fundo = (0..h as usize)
                    .rev()
                    .find(|&y| {
                        (0..w as usize).any(|x| {
                            let i = (y * w as usize + x) * 4;
                            px[i..i + 3]
                                .iter()
                                .max()
                                .zip(px[i..i + 3].iter().min())
                                .is_some_and(|(a, b)| a - b > 12)
                        })
                    })
                    .unwrap_or(0);
                println!(
                    "[preview] {nome} {w}×{h} anel {anel}: a água {} aos {:.1} s · custo {custo:.2} s · tinta até y {fundo} (o traço acaba em {:.0}; a tela em {h})",
                    if parou { "pára" } else { "NÃO parou" },
                    f64::from(ticks) / 60.0,
                    cy + amp + 30.0
                );
            }
        }
    }
    println!(
        "[preview] carga: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// As fotos do preview exato (o último quadro) e da tela 1:1 no máximo, com e sem papel escolhido:
/// `PH2D_SONDA_OUT/preview_<pincel>_<papel>.ppm`.
#[test]
#[ignore = "sonda — corre-se à mão"]
fn sonda_o_preview_e_a_tela_com_e_sem_papel() {
    use ph2d_painter_brush::TextureKind;
    let dir = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    for nome in ["Wet Round", "Wet Wash"] {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .expect("o pincel de fábrica")
            .preset;
        for papel in [None, Some(TextureKind::PaperCold)] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
            t.set_paint_media(PaintMedia::WetPaint);
            assert!(t.wet_preset_aplicar(p).is_empty());
            t.set_brush_size_px(30.0);
            t.paint.wetpaint.knobs.water = 1.0;
            t.paint.wetpaint.tilt_on = true;
            t.paint.wetpaint.tilt_ring = 8;
            t.set_brush_color_srgb8([200, 25, 25]);
            if let Some(k) = papel {
                t.set_brush_paper_kind(k.to_u8());
            }
            let mut ultimo = Vec::new();
            super::preview::pintar_preview(&t.wet_preview_cena(), &|| false, &mut |px, _, _| {
                ultimo = px.to_vec()
            });
            let (w, h) = (super::LARGURA_DO_PREVIEW, super::ALTURA_DO_PREVIEW);
            let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
            for q in ultimo.chunks(4) {
                f.extend_from_slice(&q[..3]);
            }
            let papel = papel.map_or("nenhum", |_| "cold");
            let _ = std::fs::write(
                format!("{dir}/preview_{}_{papel}.ppm", nome.replace(' ', "_")),
                f,
            );
        }
    }
}

/// A TELA 1:1: o S do preview exato pintado numa tela grande (1200×700), por pincel, diâmetro e papel,
/// no tilt máximo, até a água parar — quanto desce a tinta abaixo do traço (escuro > 30).
#[test]
#[ignore = "sonda — corre-se à mão"]
fn sonda_a_tela_escorre() {
    use ph2d_painter_brush::TextureKind;
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let (w, h) = (1200u32, 700u32);
    for nome in ["Wet Round", "Wet Wash"] {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .expect("o pincel de fábrica")
            .preset;
        for d in [60.0f32, 100.0] {
            for papel in [None, Some(TextureKind::PaperCold)] {
                let mut t = PainterTool::default();
                t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
                t.set_paint_media(PaintMedia::WetPaint);
                t.set_wet_relogio_fixo(true);
                assert!(t.wet_preset_aplicar(p).is_empty());
                t.set_brush_size_px(d * 0.5);
                t.paint.wetpaint.knobs.water = 1.0;
                t.paint.wetpaint.tilt_on = true;
                t.paint.wetpaint.tilt_ring = 8;
                t.set_brush_color_srgb8([200, 25, 25]);
                if let Some(k) = papel {
                    t.set_brush_paper_kind(k.to_u8());
                }
                let (x0, x1, cy, amp) = (100.0f32, 1100.0, 150.0, 60.0);
                let n = ((x1 - x0) / 3.0) as u32;
                let em = |f: f32| {
                    [
                        x0 + (x1 - x0) * f,
                        cy + amp * (f * std::f32::consts::TAU).sin(),
                    ]
                };
                t.on_canvas_pointer(ponto(em(0.0), PointerPhase::Down));
                for k in 1..=n {
                    t.on_canvas_pointer(ponto(em(k as f32 / n as f32), PointerPhase::Move));
                    t.on_tick(8.0);
                }
                t.on_canvas_pointer(ponto(em(1.0), PointerPhase::Up));
                for _ in 0..60 * 40 {
                    t.on_tick(1000.0 / 60.0);
                }
                let px = t.run_full().0;
                let fim = cy + amp + d * 0.5;
                let fundo = (0..h as usize)
                    .rev()
                    .find(|&y| {
                        (0..w as usize).any(|x| {
                            let i = (y * w as usize + x) * 4;
                            255 - px[i..i + 3].iter().min().copied().unwrap_or(255) > 30
                        })
                    })
                    .unwrap_or(0);
                println!(
                    "[tela] {nome} {d} px papel {:?}: tinta até y {fundo} (o traço acaba em {fim:.0}) ⇒ desce {:.0} px",
                    papel,
                    fundo as f32 - fim
                );
            }
        }
    }
}
