//! **Sonda (ignorada): a bancada das GOTAS, a nossa metade** (doc 06). Os mesmos quatro traços que
//! `ferramentas/gotas.py` manda ao Rebelle — 150 px a 40 px de diâmetro, pressão 1, água 1,0 / 0,7 / 0,4 /
//! 0,2 — numa tela branca 1000×640, no relógio fixo, em três inclinações: a de FÁBRICA do nosso dial (desde
//! a W1b, a tela DIREITA), a meio e no máximo. A versão rápida e gateada é `gotas_pela_lei_do_rebelle`. Mede a gota mais longa abaixo de cada traço (a régua do doc 06: tinta =
//! `255 − min(rgb) > 25`) a 2 s e a 30 s, e grava `gotas_<variante>_<t>.ppm` em `PH2D_SONDA_OUT`.
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x cargo test -p ph2d-tool-painter --profile smoke --test it sonda_gotas -- --ignored --nocapture
//! ```

use super::gotas_pela_lei_do_rebelle::inclina;
use super::sonda_gestos::{H, W, tela};
use ph2d_brush_presets::Volumes;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};

const AGUAS: [f64; 4] = [1.0, 0.7, 0.4, 0.2];

/// Uma inclinação da bancada: o nome e como se põe no tool.
type Variante<'a> = (&'a str, &'a dyn Fn(&mut PainterTool));
const Y: f32 = 150.0;

fn ponto(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, Y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn gota_mais_longa(px: &[u8], coluna: usize) -> u32 {
    let x0 = 50 + coluna * 240;
    let base = (Y + 25.0) as usize;
    let mut fundo = 0usize;
    for y in base..H as usize {
        for x in x0..x0 + 170 {
            let i = (y * W as usize + x) * 4;
            if 255 - px[i..i + 3].iter().copied().min().unwrap_or(255) > 25 {
                fundo = y;
            }
        }
    }
    fundo.saturating_sub(base) as u32
}

fn grava(out: &str, nome: &str, px: &[u8]) {
    let mut f = format!("P6\n{W} {H}\n255\n").into_bytes();
    for p in px.as_chunks::<4>().0 {
        f.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{out}/{nome}.ppm"), f).unwrap();
}

#[test]
#[ignore = "sonda: a nossa metade da bancada das gotas; corre-se à mão"]
fn sonda_gotas() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    // Os dois knobs ESCONDIDOS do motor que modulam a gravidade em fios (`solver.rs`: `extFingering`,
    // `extRivulets` = fios pela largura da tela), ligados por um preset (não têm controlo no painel).
    let fios = |t: &mut PainterTool, fingering: f64, rivulets: f64| {
        let mut p = ph2d_brush_presets::Parametros::novo("WetPaint", "", "");
        p.engine.insert("extFingering".into(), fingering);
        p.engine.insert("extRivulets".into(), rivulets);
        let _ = t.wet_preset_aplicar(&ph2d_brush_presets::Preset {
            parametros: p,
            origem: ph2d_brush_presets::Origem::Fabrica,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![255; 4],
            },
            texturas: Default::default(),
        });
        t.handle_panel_event(PanelEvent::SetValue(ids::PAINTER_WETPAINT_TILT_RING, 8.0));
    };
    let variantes: [Variante<'_>; 6] = [
        ("fabrica", &|_| {}),
        ("tilt_meio", &|t| inclina(t, Some(4))),
        ("tilt_max", &|t| inclina(t, Some(8))),
        ("controlo", &|t| fios(t, 0.0, 15.0)),
        ("fios_15", &|t| fios(t, 1.0, 15.0)),
        ("fios_40", &|t| fios(t, 1.0, 40.0)),
    ];
    for (nome, inclina) in variantes {
        // Um tool por tempo: o `run_full` lê a tela UMA vez (a sessão molhada fecha-se nele).
        for (segundos, rotulo) in [(2u32, "2s"), (30, "30s")] {
            let mut t = tela();
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_wet_relogio_fixo(true);
            inclina(&mut t);
            t.set_brush_size_px(20.0);
            t.set_brush_color_srgb8([23, 101, 179]);
            for (i, &agua) in AGUAS.iter().enumerate() {
                t.wet_preset_aplicar_volumes(&Volumes {
                    water: Some(agua),
                    ..Volumes::default()
                });
                let x0 = 60.0 + i as f32 * 240.0;
                t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
                for k in 1..=50 {
                    t.on_canvas_pointer(ponto(x0 + 3.0 * k as f32, PointerPhase::Move));
                    t.on_tick(8.0);
                }
                t.on_canvas_pointer(ponto(x0 + 150.0, PointerPhase::Up));
            }
            for _ in 0..segundos * 60 {
                t.on_tick(1000.0 / 60.0);
            }
            let (px, _, _) = t.run_full();
            grava(&out, &format!("gotas_{nome}_{rotulo}"), &px);
            let tinta = px.as_chunks::<4>().0.iter().filter(|p| p[0] < 230).count();
            let g: Vec<String> = (0..4)
                .map(|c| format!("{:5}", gota_mais_longa(&px, c)))
                .collect();
            println!(
                "[gotas] {nome:<9} {rotulo:>4}  gota mais longa (px), água 1,0/0,7/0,4/0,2: {}   (px de tinta {tinta})",
                g.join(" ")
            );
        }
    }
}
